import { expect, test } from "bun:test";

import { QueryClient, QueryObserver } from "@tanstack/react-query";

import { ApiRequestError } from "@/api/request";

import { isMonitorPermissionDenied } from "./-save-state";

const routes = ["overview", "nodes", "incidents", "summaries"];

test("all Monitoring routes render permission state before retained read data", async () => {
    for (const route of routes) {
        const source = await Bun.file(`src/routes/monitoring/${route}.tsx`).text();
        expect(source).toContain("isMonitorPermissionDenied(error)");
        expect(source).toContain('kind="permission"');
    }
});

test("all Monitoring routes retain cached data only for normal background failures", async () => {
    for (const route of routes) {
        const source = await Bun.file(`src/routes/monitoring/${route}.tsx`).text();
        expect(source).toContain("hasMonitorBackgroundRefreshFailure");
        expect(source).toContain("refetchInterval: 30_000");
        expect(source).toContain("retry: false");
    }
});

const waitFor = async (predicate: () => boolean) => {
    for (let attempt = 0; attempt < 40; attempt += 1) {
        if (predicate()) return;
        await Bun.sleep(5);
    }
    throw new Error("QueryObserver result did not settle");
};

test("initial and cached background 403 reads each call once and resolve to permission", async () => {
    for (const cached of [false, true]) {
        const client = new QueryClient();
        const queryKey = ["monitor", "observer-403", cached];
        if (cached) client.setQueryData(queryKey, { protected: "retained" });
        let calls = 0;
        const observer = new QueryObserver<{ protected: string }, ApiRequestError>(client, {
            queryKey,
            queryFn: async () => {
                calls += 1;
                throw new ApiRequestError("forbidden", { status: 403 });
            },
            retry: false,
        });
        const unsubscribe = observer.subscribe(() => undefined);
        try {
            await waitFor(() => observer.getCurrentResult().error !== null);
            const result = observer.getCurrentResult();
            expect(calls).toBe(1);
            expect(isMonitorPermissionDenied(result.error)).toBe(true);
            expect(result.data).toEqual(cached ? { protected: "retained" } : undefined);
        } finally {
            unsubscribe();
            client.clear();
        }
    }
});

test("Incident filters remain named controls for immediate page-one queries", async () => {
    const source = await Bun.file("src/routes/monitoring/incidents.tsx").text();

    expect(source).toContain('aria-label={t("告警状态", "Incident status")}');
    expect(source).toContain('aria-label={t("告警类型", "Incident kind")}');
    expect(source).toContain("setCurrent(1);");
});

test("Incidents retains three readable mobile columns and truncates long event content", async () => {
    const source = await Bun.file("src/routes/monitoring/incidents.tsx").text();

    expect(source).toContain("width: 160");
    expect(source).toContain('className: "monitoring-incident-primary-column"');
    expect(source).toContain('responsive: ["sm"]');
    expect(source.match(/monitoring-incident-detail-column/g)).toHaveLength(2);
    expect(source).toContain('className="truncate font-medium"');
    expect(source).toContain('className="truncate text-xs text-muted-foreground"');
});
