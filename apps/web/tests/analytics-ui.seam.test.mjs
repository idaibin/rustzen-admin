import { describe, expect, test } from "bun:test";

const details = await Bun.file(
    new URL("../src/routes/analytics/details.tsx", import.meta.url),
).text();
const overview = await Bun.file(
    new URL("../src/routes/analytics/overview.tsx", import.meta.url),
).text();
const api = await Bun.file(new URL("../src/api/insights/api.ts", import.meta.url)).text();

describe("Analytics UI contract", () => {
    test("routes subscribe to locale updates without a policy-status display request", () => {
        expect(details).toContain("useLocale();");
        expect(overview).toContain("useLocale();");
        expect(details).not.toContain("CollectionPolicyStatus");
        expect(overview).not.toContain("CollectionPolicyStatus");
    });

    test("details filters wrap to full width before the small breakpoint", () => {
        expect(details).toContain("flex-wrap items-center gap-3 sm:w-auto sm:flex-nowrap");
        expect(details).toContain('className="w-full sm:w-40"');
        expect(details).toContain('className="w-full sm:w-60"');
    });

    test("403 takes precedence over cached Analytics data while 500 retains it", () => {
        expect(details).toContain("dataUpdatedAt");
        expect(details).toContain("BackgroundRefreshNotice");
        expect(details).toContain("refetchInterval: 30_000");
        expect(details).toContain("error instanceof ApiRequestError && error.status === 403");
        expect(details).toContain("if (permissionDenied) {");
        expect(details).toContain("if (!data && error) {");
        expect(details.indexOf("if (permissionDenied) {")).toBeLessThan(
            details.indexOf("if (!data && error) {"),
        );
        expect(details).toContain("You do not have permission to view activity");
        expect(details).toContain("useFilteredPage(JSON.stringify([eventKind, path]))");
        expect(details).toContain("setCurrent(1);");
        expect(overview).toContain("error instanceof ApiRequestError && error.status === 403");
        expect(overview).toContain("if (permissionDenied) {");
        expect(overview).toContain("if (!overview) {");
        expect(overview.indexOf("if (permissionDenied) {")).toBeLessThan(
            overview.indexOf("if (!overview) {"),
        );
        expect(overview).toContain("You do not have permission to view analytics");
    });

    test("analytics reads suppress global handling so each route owns its error state", () => {
        expect(api.match(/silent: true/g)?.length).toBe(2);
        expect(api).toContain("Route-local DataState needs the typed HTTP status");
    });
});
