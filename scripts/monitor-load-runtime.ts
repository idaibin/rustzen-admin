export type Docker = (args: string[], timeoutMs?: number) => Promise<string>;
export async function boundedProcess(command: string[], timeoutMs: number) {
    if (!command.length || !Number.isInteger(timeoutMs) || timeoutMs < 1)
        throw Error("invalid bounded process input");
    let r = Bun.spawn(command, {
        stdout: "pipe",
        stderr: "pipe",
    }), timedOut = false, timer = setTimeout(() => {
        timedOut = true;
        r.kill("SIGKILL");
    }, timeoutMs);
    let exitCode: number, stdout: string, stderr: string;
    try {
        [exitCode, stdout, stderr] = await Promise.all([
            r.exited,
            new Response(r.stdout).text(),
            new Response(r.stderr).text(),
        ]);
    } finally {
        clearTimeout(timer);
    }
    if (timedOut) throw Error(`bounded process timed out after ${timeoutMs}ms`);
    if (exitCode !== 0) throw Error(stderr);
    return stdout.trim();
}
export async function docker(args: string[], timeoutMs = 10_000) {
    return boundedProcess(["docker", ...args], timeoutMs);
}
export async function listener(
    d: Docker,
    container: string,
    port: number,
    expected: string,
) {
    let script = `p=$1;h=$(printf '%04X' "$p");i=$(awk -v h="$h" '$4=="0A"&&toupper($2)~":"h"$"{print $10}' /proc/net/tcp /proc/net/tcp6|sort -u);test $(printf '%s\\n' "$i"|sed '/^$/d'|wc -l) -eq 1;n=$(printf '%s' "$i");for f in /proc/[0-9]*/fd/*;do test "$(readlink "$f" 2>/dev/null)" = "socket:[$n]"&&echo "$f";done|cut -d/ -f3|sort -u|while read p;do e=$(readlink -f /proc/$p/exe);set -- $(stat -Lc '%d %i' "$e") $(sha256sum "$e"|cut -d' ' -f1);printf '%s %s %s %s\\n' "$p" "$1" "$2" "$3";done`;
    let rows = (
        await d(["exec", container, "sh", "-ceu", script, "sh", String(port)])
    )
        .trim()
        .split("\n")
        .filter(Boolean)
        .map((x) => x.split(" "));
    return parseListenerOwner(rows, expected);
}
export function parseListenerOwner(rows: string[][], expected: string): Owner {
    if (rows.length !== 1 || rows[0]!.length !== 4 || !/^\d+$/.test(rows[0]![0]!) || !/^[a-f0-9]{64}$/.test(rows[0]![3]!) || rows[0]![3] !== expected)
        throw Error("listener owner differs");
    let [pid, dev, ino, sha256] = rows[0]!;
    if (!dev || !ino || !sha256) throw Error("listener owner differs");
    return { pid: Number(pid), dev, ino, sha256 };
}
export async function listenerGone(d: Docker, container: string, port: number) {
    const script = `h=$(printf '%04X' "$1"); ! awk -v h="$h" '$4=="0A"&&toupper($2)~":"h"$"{found=1}END{exit !found}' /proc/net/tcp /proc/net/tcp6`;
    await d(["exec", container, "sh", "-ceu", script, "sh", String(port)]);
}

export type Owner = { pid: number; dev: string; ino: string; sha256: string };
export async function service(d: Docker, container: string, unit: string, bin: string): Promise<Owner> {
    let raw = await d(["exec", container, "sh", "-ceu", `pid=$(systemctl show -p MainPID --value ${unit}); test "$pid" -gt 1; exe=$(readlink -f /proc/$pid/exe); expected=$(readlink -f ${bin}); test -n "$expected" && test "$exe" = "$expected"; set -- $(stat -Lc '%d %i' "$exe") $(sha256sum "$exe"|cut -d' ' -f1); printf '%s %s %s %s' "$pid" "$1" "$2" "$3"`]), row = raw.split(" ");
    return parseServiceOwner(row);
}
export function parseServiceOwner(row: string[]): Owner {
    if (row.length !== 4 || !/^\d+$/.test(row[0]!) || !row[1] || !row[2] || !/^[a-f0-9]{64}$/.test(row[3]!)) throw Error("service owner differs");
    return { pid: Number(row[0]), dev: row[1]!, ino: row[2]!, sha256: row[3]! };
}
export function certifiedOwner(owner: Owner, digest: unknown) {
    if (owner.sha256 !== digest) throw Error("service digest differs from certified binary");
}
export function restartedOwner(previous: Owner, next: Owner, listenerOwner: Owner) {
    if (next.pid === previous.pid || next.dev !== previous.dev || next.ino !== previous.ino || next.sha256 !== previous.sha256 || listenerOwner.pid !== next.pid || listenerOwner.dev !== next.dev || listenerOwner.ino !== next.ino)
        throw Error("Monitor listener owner differs");
}
export async function usage(d: Docker, container: string) {
    let script = `for f in memory.max memory.current memory.peak pids.max pids.current pids.peak;do printf '%s=' "$f";cat /sys/fs/cgroup/$f;done;for e in max oom oom_kill;do printf 'memory.events.%s=' "$e";awk -v e=$e '$1==e{print $2}' /sys/fs/cgroup/memory.events;done;printf 'pids.events.max=';awk -v e=max '$1==e{print $2}' /sys/fs/cgroup/pids.events;rss=0;hwm=0;for p in $(systemctl show -p MainPID --value rz-admin.service rz-monitor.service);do test "$p" -gt 1 && test -r /proc/$p/status || continue;set -- $(awk '/VmRSS/{r=$2}/VmHWM/{h=$2} END{print r+0,h+0}' /proc/$p/status);rss=$((rss+$1));hwm=$((hwm+$2));done;printf 'rss=%s\\nhwm=%s\\n' "$((rss*1024))" "$((hwm*1024))"`, raw = await d(["exec", container, "sh", "-ceu", script], 1800), v = Object.fromEntries(raw.split("\n").filter(Boolean).map(x => x.split("=", 2))), n = (k: string) => { let x = Number(v[k]); if (!Number.isFinite(x)) throw Error(`invalid resource sample: ${k}`); return x; };
    if (n("memory.max") !== 512 * 1024 * 1024 || n("pids.max") !== 256 || n("pids.current") > 64 || n("pids.peak") > 64 || n("memory.peak") > 384 * 1024 * 1024) throw Error("container limits differ");
    return { rss: n("rss"), hwm: n("hwm"), pidsCurrent: n("pids.current"), pidsPeak: n("pids.peak"), memoryCurrent: n("memory.current"), memoryPeak: n("memory.peak"), events: { "memory.max": n("memory.max"), "memory.events.max": n("memory.events.max"), "memory.events.oom": n("memory.events.oom"), "memory.events.oom_kill": n("memory.events.oom_kill"), "pids.events.max": n("pids.events.max") } };
}
