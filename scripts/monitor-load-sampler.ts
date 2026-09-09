export type Reading = { at: number; rss: number; hwm: number; pidsCurrent: number; pidsPeak: number; memoryCurrent: number; memoryPeak: number; events: Record<string, number> };
export type Owner = { pid: number; dev: string; ino: string; sha256: string };
export type Owners = { admin: Owner; monitor: Owner };
export type Phase<T> = { name: string; workDurationMs: number; snapshots: [Reading, Reading]; services: { before: Owners; after: Owners }; result: T };
export type Clock = { now(): number };
export async function phase<T>(name: string, clock: Clock, usage: () => Promise<Omit<Reading, "at">>, services: () => Promise<Owners>, work: () => Promise<T>): Promise<Phase<T>> {
    const before = { at: clock.now(), ...(await usage()) }, beforeServices = await services();
    const started = clock.now(), result = await work(), workDurationMs = clock.now() - started;
    const after = { at: clock.now(), ...(await usage()) }, afterServices = await services();
    if (after.at <= before.at || !Number.isFinite(workDurationMs) || workDurationMs < 0) throw Error("phase snapshot order differs");
    return { name, workDurationMs, snapshots: [before, after], services: { before: beforeServices, after: afterServices }, result };
}
export function sameOwner(a: Owner, b: Owner) { return a.pid === b.pid && a.dev === b.dev && a.ino === b.ino && a.sha256 === b.sha256; }
export function stablePhase(phase: Phase<unknown>) { if (!sameOwner(phase.services.before.admin, phase.services.after.admin) || !sameOwner(phase.services.before.monitor, phase.services.after.monitor)) throw Error("phase service identity differs"); }
export function faultPhase(phase: Phase<unknown>) { if (!sameOwner(phase.services.before.admin, phase.services.after.admin) || sameOwner(phase.services.before.monitor, phase.services.after.monitor)) throw Error("fault service identity differs"); }
export function maxima(values: readonly Reading[]) { if (!values.length) throw Error("missing resource snapshots"); const peak = (key: keyof Reading) => Math.max(...values.map(x => Number(x[key]))); return { rss: peak("rss"), hwm: peak("hwm"), pidsPeak: peak("pidsPeak"), memoryCurrent: peak("memoryCurrent"), memoryPeak: peak("memoryPeak") }; }
export function quiet(reading: Reading) { return { rss: reading.rss, pidsCurrent: reading.pidsCurrent, at: reading.at }; }
export function compareQuiet(first: { rss: number; pidsCurrent: number }, second: { rss: number; pidsCurrent: number }) { if (second.rss > first.rss + 16 * 1024 * 1024 || second.pidsCurrent > first.pidsCurrent + 2) throw Error("quiet baseline drift"); }
