type Runtime = { containerId: string; imageId: string; hostPort: number; containerPort: number; pid: number; dev: string; ino: string; sha256: string };
const hash = /^[a-f0-9]{64}$/;
export function inspected(raw: unknown, host: string, hostPort: number) {
    const value = Array.isArray(raw) ? raw[0] : raw as any;
    if (!value?.Id || !value?.Image || value?.State?.Running !== true || value?.Platform !== "linux") throw new Error("runtime container is not a running linux container");
    if (host !== "127.0.0.1") throw new Error("admin-url host is not loopback IPv4");
    const matches = Object.entries(value?.NetworkSettings?.Ports || {}).flatMap(([key, mapped]: [string, any]) =>
        key.endsWith("/tcp") ? (mapped || []).filter((row: any) => Number(row.HostPort) === hostPort && ["127.0.0.1", "0.0.0.0"].includes(row.HostIp)).map(() => Number(key.split("/")[0])) : []);
    if (matches.length !== 1 || !Number.isInteger(matches[0])) throw new Error("admin-url host port does not map uniquely into runtime container");
    return { containerId: value.Id, imageId: value.Image, hostPort, containerPort: matches[0] };
}
export function image(raw: unknown) { const value = Array.isArray(raw) ? raw[0] : raw as any; if (value?.Os !== "linux" || value?.Architecture !== "amd64") throw new Error("runtime image is not linux/amd64"); }
export function process(raw: string, runtime: { containerId: string; imageId: string; hostPort: number; containerPort: number }, expected: string): Runtime {
    const rows = raw.trim().split("\n").filter(Boolean).map(line => line.split("\t"));
    if (rows.length !== 1 || rows[0].length !== 4 || !/^\d+$/.test(rows[0][0]) || !/^\d+$/.test(rows[0][1]) || !/^\d+$/.test(rows[0][2]) || !hash.test(rows[0][3]) || rows[0][3] !== expected) throw new Error("runtime listener does not uniquely own certified admin executable");
    return { ...runtime, pid: Number(rows[0][0]), dev: rows[0][1], ino: rows[0][2], sha256: rows[0][3] };
}
export function same(left: Runtime, right: Runtime) { return JSON.stringify(left) === JSON.stringify(right); }
const args = new Map<string, string>();
if (import.meta.main) {
    const required = ["--runtime-container", "--admin-url", "--expected-sha256"];
    for (let i = 2; i < Bun.argv.length; i += 2) { const key = Bun.argv[i], value = Bun.argv[i + 1]; if (!key || !value || !required.includes(key) || args.has(key)) throw new Error("invalid runtime attestation arguments"); args.set(key, value); }
    if (args.size !== required.length || !hash.test(args.get("--expected-sha256")!)) throw new Error("invalid runtime attestation arguments");
    const url = new URL(args.get("--admin-url")!); if (!url.port) throw new Error("admin-url has no port");
    const inspect = Bun.spawnSync(["docker", "inspect", args.get("--runtime-container")!], { stdout: "pipe", stderr: "pipe" });
    if (inspect.exitCode !== 0) throw new Error("cannot inspect runtime container");
    const runtime = inspected(JSON.parse(new TextDecoder().decode(inspect.stdout)), url.hostname, Number(url.port));
    const imageInspect = Bun.spawnSync(["docker", "image", "inspect", runtime.imageId], { stdout: "pipe", stderr: "pipe" });
    if (imageInspect.exitCode !== 0) throw new Error("cannot inspect runtime image"); image(JSON.parse(new TextDecoder().decode(imageInspect.stdout)));
    const probe = String.raw`p=$1; h=$(printf '%04X' "$p"); i=$(awk -v h="$h" '$4=="0A" && toupper($2) ~ ":"h"$" {print $10}' /proc/net/tcp /proc/net/tcp6 | sort -u); test $(printf '%s\n' "$i" | sed '/^$/d' | wc -l) -eq 1; n=$(printf '%s' "$i"); for f in /proc/[0-9]*/fd/*; do test "$(readlink "$f" 2>/dev/null)" = "socket:[$n]" && echo "$f" | sed "s@^/proc/@@" | cut -d/ -f1; done | sort -u | while read pid; do e=/proc/$pid/exe; set -- $(stat -Lc '%d %i' "$e") $(sha256sum "$e" | awk '{print $1}'); printf '%s\t%s\t%s\t%s\n' "$pid" "$1" "$2" "$3"; done`;
    const run = Bun.spawnSync(["docker", "exec", runtime.containerId, "sh", "-c", probe, "sh", String(runtime.containerPort)], { stdout: "pipe", stderr: "pipe" });
    if (run.exitCode !== 0) throw new Error("cannot attest runtime listener");
    console.log(JSON.stringify(process(new TextDecoder().decode(run.stdout), runtime, args.get("--expected-sha256")!)));
}
