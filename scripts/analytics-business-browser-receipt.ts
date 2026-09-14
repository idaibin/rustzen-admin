import { canonicalJson, validHash } from "../distribution/release-manifest-core.ts";

export const analyticsBusinessJourneyIds = [
    "deployment-identity",
    "unauthenticated-redirect",
    "unauthenticated-api-denied",
    "wrong-credentials-rejected",
    "default-passwords-rejected",
    "owner-login",
    "menu-navigation-isolation",
    "overview-empty",
    "details-empty-mobile",
    "overview-loading",
    "collection-policy-and-seed",
    "overview-populated",
    "details-filter-pagination",
    "error-state-retry",
    "service-restart-recovery",
    "monitor-reports-absent",
    "logout-session-invalidation",
] as const;

type HttpRecord = { method: string; path: string; status: number | "failed" };
export type AnalyticsBusinessJourney = {
    id: (typeof analyticsBusinessJourneyIds)[number];
    summary: string;
    expected: string;
    actual: string;
    url: string;
    http: HttpRecord[];
    screenshot?: string;
    passed: boolean;
};
export type AnalyticsBusinessBrowserReceipt = {
    schemaVersion: 1;
    kind: "analytics-business-browser-receipt";
    environment: {
        chromiumVersion: string;
        viewport: { width: number; height: number };
        surface: "headless-chromium-cdp";
        adminOrigin: string;
        container: { name: string; id: string; image: string; units: string[] };
    };
    identity: {
        head: string;
        harnessSourceIdentity: string;
        buildId: string;
        compositionId: string;
        archiveSha256: string;
        manifestSha256: string;
        envelopeSha256: string;
        certificateSha256: string;
        nativeEvidence: { path: string; sha256: string };
    };
    journeys: AnalyticsBusinessJourney[];
    network: { requestsTotal: number; failureSummary: Array<{ path: string; error: string }> };
    console: { errorCount: number; errors: string[] };
    screenshots: Array<{ file: string; sha256: string; bytes: number; dimensions: { width: number; height: number } }>;
    flags: { browser: true; runtime: { evidencePath: string; sha256: string }; load: false; releaseReady: false };
};

const journeysWithNetworkEvidence = new Set<string>([
    "deployment-identity",
    "unauthenticated-api-denied",
    "wrong-credentials-rejected",
    "default-passwords-rejected",
    "collection-policy-and-seed",
    "overview-populated",
    "details-filter-pagination",
    "error-state-retry",
    "service-restart-recovery",
    "monitor-reports-absent",
    "logout-session-invalidation",
]);

export function parseAnalyticsBusinessBrowserReceipt(value: unknown): AnalyticsBusinessBrowserReceipt {
    const receipt = object(value, "analytics business receipt");
    only(receipt, ["schemaVersion", "kind", "environment", "identity", "journeys", "network", "console", "screenshots", "flags"]);
    if (receipt.schemaVersion !== 1 || receipt.kind !== "analytics-business-browser-receipt")
        throw new Error("analytics business receipt version or kind is invalid");
    const environment = object(receipt.environment, "analytics business environment");
    only(environment, ["chromiumVersion", "viewport", "surface", "adminOrigin", "container"]);
    if (environment.surface !== "headless-chromium-cdp")
        throw new Error("analytics business surface is invalid");
    if (!/^https?:\/\/127\.0\.0\.1:[0-9]+$/.test(text(environment.adminOrigin, "admin origin")))
        throw new Error("analytics business admin origin is invalid");
    const container = object(environment.container, "analytics business container");
    only(container, ["name", "id", "image", "units"]);
    const viewport = object(environment.viewport, "analytics business viewport");
    only(viewport, ["width", "height"]);
    if (viewport.width !== 1440 || viewport.height !== 900)
        throw new Error("analytics business default viewport differs");
    const identity = object(receipt.identity, "analytics business identity");
    only(identity, ["head", "harnessSourceIdentity", "buildId", "compositionId", "archiveSha256", "manifestSha256", "envelopeSha256", "certificateSha256", "nativeEvidence"]);
    if (!/^git:[0-9a-f]{40} tree:[0-9a-f]{64} state:(clean|dirty)$/.test(text(identity.harnessSourceIdentity, "harness source identity")))
        throw new Error("analytics business harness source identity is invalid");
    const nativeEvidence = object(identity.nativeEvidence, "analytics business native evidence");
    only(nativeEvidence, ["path", "sha256"]);
    validHash(text(nativeEvidence.sha256, "native evidence sha256"));
    const journeys = Array.isArray(receipt.journeys) ? receipt.journeys.map((item) => {
        const journey = object(item, "analytics business journey");
        only(journey, ["id", "summary", "expected", "actual", "url", "http", "passed", "screenshot"]);
        const id = text(journey.id, "journey id") as AnalyticsBusinessJourney["id"];
        if (!(analyticsBusinessJourneyIds as readonly string[]).includes(id))
            throw new Error("analytics business journey id is invalid");
        const http = (Array.isArray(journey.http) ? journey.http : []).map((entry) => {
            const record = object(entry, "analytics business http record");
            only(record, ["method", "path", "status"]);
            const status = record.status;
            if (typeof status !== "number" && status !== "failed")
                throw new Error("analytics business http status is invalid");
            return { method: text(record.method, "http method"), path: text(record.path, "http path"), status };
        });
        if (journeysWithNetworkEvidence.has(id) && http.length === 0)
            throw new Error(`analytics business journey lacks network evidence: ${id}`);
        return {
            id,
            summary: text(journey.summary, "journey summary"),
            expected: text(journey.expected, "journey expected"),
            actual: text(journey.actual, "journey actual"),
            url: text(journey.url, "journey url"),
            http,
            ...(journey.screenshot === undefined ? {} : { screenshot: text(journey.screenshot, "journey screenshot") }),
            passed: journey.passed === true,
        };
    }) : (() => { throw new Error("analytics business journeys must be an array"); })();
    if (canonicalJson(journeys.map((journey) => journey.id)) !== canonicalJson([...analyticsBusinessJourneyIds]))
        throw new Error("analytics business journey inventory differs");
    if (journeys.some((journey) => !journey.passed))
        throw new Error("analytics business journey did not pass");
    const network = object(receipt.network, "analytics business network");
    only(network, ["requestsTotal", "failureSummary"]);
    const failureSummary = (Array.isArray(network.failureSummary) ? network.failureSummary : []).map((entry) => {
        const record = object(entry, "analytics business failure");
        only(record, ["path", "error"]);
        return { path: text(record.path, "failure path"), error: text(record.error, "failure error") };
    });
    if (!Number.isSafeInteger(network.requestsTotal) || network.requestsTotal < 1)
        throw new Error("analytics business network summary is invalid");
    const consoleSummary = object(receipt.console, "analytics business console");
    only(consoleSummary, ["errorCount", "errors"]);
    if (!Number.isSafeInteger(consoleSummary.errorCount) || !Array.isArray(consoleSummary.errors)
        || consoleSummary.errors.some((entry) => typeof entry !== "string"))
        throw new Error("analytics business console summary is invalid");
    const screenshots = (Array.isArray(receipt.screenshots) ? receipt.screenshots : []).map((entry) => {
        const record = object(entry, "analytics business screenshot");
        only(record, ["file", "sha256", "bytes", "dimensions"]);
        const dimensions = object(record.dimensions, "analytics business screenshot dimensions");
        only(dimensions, ["width", "height"]);
        if ((dimensions.width !== 1440 || dimensions.height !== 900) && (dimensions.width !== 390 || dimensions.height !== 844))
            throw new Error("analytics business screenshot dimensions differ");
        if (!Number.isSafeInteger(record.bytes) || record.bytes < 1024 || !/^[a-z0-9-]+\.png$/.test(text(record.file, "screenshot file")))
            throw new Error("analytics business screenshot record is invalid");
        return { file: text(record.file, "screenshot file"), sha256: validHash(text(record.sha256, "screenshot sha256")), bytes: record.bytes, dimensions: { width: dimensions.width, height: dimensions.height } };
    });
    if (new Set(screenshots.map((shot) => shot.file)).size !== screenshots.length || screenshots.length !== 7)
        throw new Error("analytics business screenshot inventory is invalid");
    const flags = object(receipt.flags, "analytics business flags");
    only(flags, ["browser", "runtime", "load", "releaseReady"]);
    const runtime = object(flags.runtime, "analytics business runtime reference");
    only(runtime, ["evidencePath", "sha256"]);
    validHash(text(runtime.sha256, "runtime evidence sha256"));
    if (flags.browser !== true || flags.load !== false || flags.releaseReady !== false)
        throw new Error("analytics business flags differ");
    for (const journey of journeys)
        if (journey.screenshot && !screenshots.some((shot) => shot.file === `${journey.screenshot}.png`))
            throw new Error("analytics business journey references an unknown screenshot");
    const serialized = JSON.stringify(value);
    for (const forbidden of ["p8e-owner-password", "Bearer ", "eyJ", "x-rustzen-project-key\":\"", "rustzen@123"])
        if (serialized.includes(forbidden))
            throw new Error(`analytics business receipt leaks sensitive material: ${forbidden.trim()}`);
    return {
        schemaVersion: 1,
        kind: "analytics-business-browser-receipt",
        environment: {
            chromiumVersion: text(environment.chromiumVersion, "chromium version"),
            viewport: { width: viewport.width, height: viewport.height },
            surface: "headless-chromium-cdp",
            adminOrigin: text(environment.adminOrigin, "admin origin"),
            container: {
                name: text(container.name, "container name"),
                id: text(container.id, "container id"),
                image: text(container.image, "container image"),
                units: (Array.isArray(container.units) ? container.units : []).map((unit) => text(unit, "container unit")),
            },
        },
        identity: {
            head: text(identity.head, "head"),
            harnessSourceIdentity: text(identity.harnessSourceIdentity, "harness source identity"),
            buildId: validHash(text(identity.buildId, "build id")),
            compositionId: validHash(text(identity.compositionId, "composition id")),
            archiveSha256: validHash(text(identity.archiveSha256, "archive sha256")),
            manifestSha256: validHash(text(identity.manifestSha256, "manifest sha256")),
            envelopeSha256: validHash(text(identity.envelopeSha256, "envelope sha256")),
            certificateSha256: validHash(text(identity.certificateSha256, "certificate sha256")),
            nativeEvidence: { path: text(nativeEvidence.path, "native evidence path"), sha256: text(nativeEvidence.sha256, "native evidence sha256") },
        },
        journeys,
        network: { requestsTotal: network.requestsTotal, failureSummary },
        console: { errorCount: consoleSummary.errorCount, errors: consoleSummary.errors },
        screenshots,
        flags: { browser: true, runtime: { evidencePath: text(runtime.evidencePath, "runtime evidence path"), sha256: text(runtime.sha256, "runtime evidence sha256") }, load: false, releaseReady: false },
    };
}
function object(value: unknown, label: string): Record<string, any> { if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(`${label} must be an object`); return value as Record<string, any>; }
function only(value: Record<string, any>, allowed: string[]) { for (const key of Object.keys(value)) if (!allowed.includes(key)) throw new Error(`unknown ${key} field in analytics business receipt`); }
function text(value: unknown, label: string): string { if (typeof value !== "string" || !value) throw new Error(`${label} is invalid`); return value; }

if (Bun.argv.length === 3) {
    const bytes = await Bun.file(Bun.argv[2]!).bytes();
    const value = JSON.parse(new TextDecoder().decode(bytes));
    const parsed = parseAnalyticsBusinessBrowserReceipt(value);
    if (new TextDecoder().decode(bytes) !== canonicalJson(parsed)) throw new Error("analytics business receipt file is not canonical");
    console.log(canonicalJson({ verified: true, journeys: parsed.journeys.length, screenshots: parsed.screenshots.length }));
}
