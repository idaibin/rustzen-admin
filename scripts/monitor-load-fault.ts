export type Boundary = {
    at: number;
    status?: number;
    code?: number;
    kind: "response" | "timeout" | "transport";
};
export function orderedClock(now: () => number, initial: number) {
    let last = initial;
    return () => (last = Math.max(last + 1, now()));
}
export function classify(
    value: { status?: number; code?: number; failure?: "timeout" | "transport" | "http" | "contract" },
    at: number,
): Boundary {
    if (!value.status) return { at, kind: value.failure === "timeout" ? "timeout" : "transport" };
    return { at, status: value.status, code: value.code, kind: "response" };
}
export function stableOutage(values: readonly Boundary[]) {
    if (values.length < 4 || values.at(-1)!.at - values[0]!.at < 750)
        throw Error("stable outage window is short");
    for (let value of values) {
        if (
            value.kind !== "response" ||
            value.status !== 503 ||
            value.code !== 40001
        )
            throw Error("forbidden fault boundary");
    }
    return values;
}
export async function controlledBoundary(
    start: () => Promise<{ status?: number; code?: number; failure?: "timeout" | "transport" | "http" | "contract" }>,
    now: () => number,
    barrier: () => Promise<boolean>,
    stop: () => Promise<number>,
    count = 4,
) {
    if (!Number.isInteger(count) || count < 1) throw Error("controlled inflight count differs");
    const pending = Array.from({ length: count }, () => {
        const startAt = now();
        return start().then(value => { const end = now(); return { ...classify(value, end), start: startAt, end }; });
    });
    if (!(await barrier())) throw Error("controlled inflight barrier differs");
    const stopAt = await stop(), rows = await Promise.all(pending);
    if (rows.length !== count || rows.some(row => !(row.start < stopAt && stopAt < row.end))) throw Error("controlled inflight did not cross monitor stop");
    return { stopAt, rows };
}
export function milestones(values: readonly { step: string; at: number }[]) {
    let expected = ["stop", "listenerGone", "listenerReady", "registryHealthy"];
    if (
        JSON.stringify(values.map((x) => x.step)) !==
            JSON.stringify(expected) ||
        values.some((x, i) => i && x.at <= values[i - 1]!.at)
    )
        throw Error("fault milestones differ");
}
