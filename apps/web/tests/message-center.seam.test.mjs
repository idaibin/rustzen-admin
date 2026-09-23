import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = (path) => readFileSync(new URL(`../src/${path}`, import.meta.url), "utf8");
const root = source("routes/__root.tsx");
const layout = source("components/layout/index.tsx");
const center = source("notifications/message-center.tsx");
const drawer = source("notifications/message-drawer.tsx");
const detail = source("notifications/message-detail.tsx");
const lifecycle = source("notifications/use-notification-realtime.ts");
const api = source("api/notifications/api.ts");
const listItem = source("notifications/message-list-item.tsx");
const main = source("main.tsx");
const incident = source("routes/monitoring/-incident-drawer.tsx");
const reports = source("routes/reports/runs.tsx");

test("the authenticated shell owns one optional message-center contribution", () => {
    expect(root.match(/<NotificationShell \/>/g)).toHaveLength(1);
    expect(root).toContain("headerActions={token ? <NotificationShell /> : null}");
    expect(layout).toContain("headerActions?: ReactNode");
    expect(layout).not.toContain("notifications/");
});

test("the bell and drawer retain the durable inbox state matrix", () => {
    expect(center).toContain("overflowCount={99}");
    expect(center).toContain("enabled: Boolean(token)");
    expect(drawer).toContain("enabled: open");
    expect(center).toContain('aria-label={t("打开消息中心"');
    const surface = `${drawer}\n${detail}`;
    for (const seam of [
        "useInfiniteQuery",
        "unreadOnly",
        "fetchNextPage",
        "markRead.mutate",
        "markAll.mutate",
        "Messages could not be loaded",
        "Background refresh failed",
        "Message details are unavailable",
        "Messages are retained for",
    ])
        expect(surface).toContain(seam);
    expect(surface).not.toContain("dangerouslySetInnerHTML");
});

test("one realtime lifecycle follows auth, visibility and BFCache changes", () => {
    expect(lifecycle).toContain("new NotificationRealtimeClient");
    expect(lifecycle).toContain('window.addEventListener("pagehide", pageHide)');
    expect(lifecycle).toContain('window.addEventListener("pageshow", pageShow)');
    expect(lifecycle).toContain('document.addEventListener("visibilitychange", visibility)');
    expect(lifecycle).toContain("60_000");
    expect(lifecycle).toContain("currentGeneration: () => useAuthStore.getState().authGeneration");
    expect(lifecycle).toContain("queryClient.cancelQueries");
    expect(api.match(/signal,/g)?.length).toBeGreaterThanOrEqual(3);
});

test("message rows and write failures remain explicit and accessible", () => {
    expect(listItem).toContain('<button\n            type="button"');
    expect(listItem).toContain("aria-label={t(`打开消息：${item.title}`");
    expect(drawer).toContain("MessageWriteError");
    expect(drawer).toContain('failure === "missing"');
    expect(drawer).toContain("selectedId && !forbidden");
});

test("auth changes and deep links cannot reuse another identity's cached detail", () => {
    expect(main).toContain("bindAuthenticatedQueryCache(queryClient, useAuthStore)");
    expect(incident).toContain('["monitor", "incident", generation, incidentId]');
    expect(incident).toContain("!error && !isFetching ? data : undefined");
    expect(reports).toContain('["reports", "run", generation, linkedRunId]');
    expect(reports).toContain("selectedGeneration.current === generation ? selected : undefined");
    expect(`${incident}\n${reports}`).toContain('refetchOnMount: "always"');
});
