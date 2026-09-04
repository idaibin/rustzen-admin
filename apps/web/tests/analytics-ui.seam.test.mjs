import { describe, expect, test } from "bun:test";

const details = await Bun.file(
    new URL("../src/routes/analytics/details.tsx", import.meta.url),
).text();
const overview = await Bun.file(
    new URL("../src/routes/analytics/overview.tsx", import.meta.url),
).text();

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
});
