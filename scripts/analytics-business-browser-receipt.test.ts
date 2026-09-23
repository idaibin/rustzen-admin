import { expect, test } from "bun:test";
import { canonicalJson, sha256 } from "../distribution/release-manifest-core.ts";
import { analyticsBusinessJourneyIds, parseAnalyticsBusinessBrowserReceipt } from "./analytics-business-browser-receipt.ts";

const hash = "a".repeat(64);
const screenshot = (name: string, width = 1440, height = 900) => ({ file: `${name}.png`, sha256: hash, bytes: 4096, dimensions: { width, height } });

const receipt: any = {
    schemaVersion: 1,
    kind: "analytics-business-browser-receipt",
    environment: {
        chromiumVersion: "Chrome/140.0.0.0",
        viewport: { width: 1440, height: 900 },
        surface: "headless-chromium-cdp",
        adminOrigin: "http://127.0.0.1:34567",
        container: { name: "rz-p8e-native-runtime-1", id: hash, image: hash, units: ["rz-admin.service", "rz-insights.service", "rz-full.service"] },
    },
    identity: {
        head: "0123456789abcdef0123456789abcdef01234567",
        harnessSourceIdentity: `git:0123456789abcdef0123456789abcdef01234567 tree:${hash} state:dirty`,
        buildId: hash,
        compositionId: hash,
        archiveSha256: hash,
        manifestSha256: hash,
        envelopeSha256: hash,
        certificateSha256: hash,
        nativeEvidence: { path: "analytics-native-runtime-evidence.json", sha256: hash },
    },
    journeys: analyticsBusinessJourneyIds.map((id) => ({
        id,
        summary: `summary ${id}`,
        expected: `expected ${id}`,
        actual: `actual ${id}`,
        url: "/analytics/overview",
        http: [{ method: "GET", path: "/api/insights/overview", status: 200 }],
        passed: true,
    })),
    network: { requestsTotal: 42, failureSummary: [{ path: "/api/insights/overview", error: "Failed" }] },
    console: { errorCount: 1, errors: ["console error"] },
    screenshots: [
        screenshot("login-error"), screenshot("overview-empty"), screenshot("details-empty-mobile", 390, 844),
        screenshot("overview-loading"), screenshot("overview-populated"), screenshot("details-populated"), screenshot("error-state"),
    ],
    flags: { browser: true, runtime: { evidencePath: "target/rz/p8e-analytics-runtime/analytics-native-runtime-evidence.json", sha256: hash }, load: false, releaseReady: false },
};

test("analytics business receipt binds the full journey inventory", () => {
    expect(parseAnalyticsBusinessBrowserReceipt(receipt)).toEqual(receipt);
    for (const mutate of [
        (value: any) => value.journeys.pop(),
        (value: any) => { value.journeys[3].passed = false; },
        (value: any) => { value.journeys[3].http = []; },
        (value: any) => { value.flags.browser = false; },
        (value: any) => { value.flags.load = true; },
        (value: any) => { value.flags.releaseReady = true; },
        (value: any) => { delete value.identity.nativeEvidence; },
        (value: any) => { value.identity.harnessSourceIdentity = "dirty"; },
        (value: any) => { value.environment.adminOrigin = "http://0.0.0.0:1"; },
        (value: any) => { value.screenshots[2].dimensions = { width: 800, height: 600 }; },
        (value: any) => value.screenshots.pop(),
        (value: any) => { value.journeys[10].actual = "Bearer eyJabc"; },
        (value: any) => { value.extra = true; },
        (value: any) => { value.network.requestsTotal = 0; },
    ]) {
        const mutated = structuredClone(receipt);
        mutate(mutated);
        expect(() => parseAnalyticsBusinessBrowserReceipt(mutated)).toThrow();
    }
    expect(canonicalJson({ journeyCount: analyticsBusinessJourneyIds.length })).toBe('{"journeyCount":17}');
    expect(sha256("fixed")).toMatch(/^[a-f0-9]{64}$/);
});
