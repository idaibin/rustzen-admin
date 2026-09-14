// P8f Analytics business browser driver. Drives one headless Chromium over CDP
// against the retained P8e deployment of the signed Analytics release.
import { canonicalJson, sha256 } from "../distribution/release-manifest-core.ts";
import { parseAnalyticsBusinessBrowserReceipt, analyticsBusinessJourneyIds } from "./analytics-business-browser-receipt.ts";
import { readFile, writeFile } from "node:fs/promises";
import { basename } from "node:path";

const args = new Map<string, string>();
for (let index = 2; index < Bun.argv.length; index += 2) args.set(Bun.argv[index]!, Bun.argv[index + 1]!);
const required = ["--context", "--cdp", "--signals", "--output"];
if (required.some((flag) => !args.get(flag)) || args.size !== required.length) throw Error("usage: --context FILE --cdp PORT --signals DIR --output DIR");
const output = args.get("--output")!;
const signals = args.get("--signals")!;
const context = JSON.parse(await readFile(args.get("--context")!, "utf8")) as {
    adminUrl: string; containerName: string; containerId: string; containerImage: string;
    releaseResult: string; certificate: string; passwordFile: string;
    nativeEvidence: string; harnessSourceIdentity: string; head: string; containerFacts: string;
    runtimeEvidence: string; runtimeEvidenceSha256: string;
};
const release = JSON.parse(await readFile(context.releaseResult, "utf8"));
const certificateBytes = await readFile(context.certificate);
const nativeEvidenceBytes = await readFile(context.nativeEvidence);
const facts = JSON.parse(await readFile(context.containerFacts, "utf8"));
const password = (await readFile(context.passwordFile, "utf8")).trim();
const base = new URL(context.adminUrl).origin;
const buildId: string = release.buildId;

// --- CDP wiring -------------------------------------------------------------
const version = await (await fetch(`http://127.0.0.1:${args.get("--cdp")}/json/version`)).json();
const page = await (await fetch(`http://127.0.0.1:${args.get("--cdp")}/json/new?${encodeURIComponent(`${base}/login`)}`, { method: "PUT" })).json();
let next = 0;
const pending = new Map<number, (value: any) => void>();
const requests: any[] = [];
const responses = new Map<string, any>();
const consoleErrors: string[] = [];
const socket = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((resolve, reject) => { socket.onopen = resolve; socket.onerror = reject; });
socket.onmessage = (event) => {
    const value = JSON.parse(String(event.data));
    if (value.id) pending.get(value.id)?.(value);
    if (value.method === "Network.requestWillBeSent") requests.push({ requestId: value.params.requestId, method: value.params.request.method, url: value.params.request.url });
    if (value.method === "Network.responseReceived") responses.set(value.params.requestId, { status: value.params.response.status });
    if (value.method === "Network.loadingFailed") responses.set(value.params.requestId, { status: "failed", error: value.params.errorText });
    if (value.method === "Runtime.consoleAPICalled" && value.params.type === "error") consoleErrors.push(String(value.params.args?.[0]?.value ?? "console error"));
    if (value.method === "Runtime.exceptionThrown") consoleErrors.push(String(value.params.exceptionDetails?.text ?? "exception"));
};
const call = (method: string, params: any = {}) => new Promise<any>((resolve, reject) => {
    const id = ++next;
    pending.set(id, (value) => value.error ? reject(Error(`CDP ${method} failed`)) : resolve(value.result));
    socket.send(JSON.stringify({ id, method, params }));
});
const evaluate = async (expression: string): Promise<any> => {
    const result = await call("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
    if (result.exceptionDetails) throw Error(`CDP evaluate failed: ${JSON.stringify(result.exceptionDetails.exception?.description ?? result.exceptionDetails.text)}`);
    return result.result.value;
};
await call("Network.enable");
await call("Page.enable");
await call("Emulation.setDeviceMetricsOverride", { width: 1440, height: 900, deviceScaleFactor: 1, mobile: false });

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
const wait = async (expression: string, label: string, timeoutMs = 20000) => {
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
        if (Boolean(await evaluate(expression).catch(() => false))) return;
        await sleep(100);
    }
    throw Error(`wait timed out: ${label}`);
};
const navigate = async (path: string) => { await call("Page.navigate", { url: `${base}${path}` }); };
const location = () => evaluate("location.pathname + location.search");
const setPreferences = async (theme: "dark" | "light", locale: "en-US" | "zh-CN", width: number, height: number) => {
    await evaluate(`localStorage.setItem('rustzen-admin-theme', ${JSON.stringify(theme)}); localStorage.setItem('rustzen-admin-locale', ${JSON.stringify(locale)}); true`);
    await call("Emulation.setDeviceMetricsOverride", { width, height, deviceScaleFactor: 1, mobile: width === 390 });
};
const reactInput = (selector: string, value: string) => `(() => { const element = document.querySelector(${JSON.stringify(selector)}); const descriptor = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value'); descriptor.set.call(element, ${JSON.stringify(value)}); element.dispatchEvent(new Event('input', { bubbles: true })); element.dispatchEvent(new Event('change', { bubbles: true })); })()`;
const markRequests = () => requests.length;
const httpSince = (mark: number, filter: (url: string, method: string) => boolean) =>
    requests.slice(mark).filter((entry) => filter(entry.url, entry.method)).map((entry) => {
        const response = responses.get(entry.requestId);
        const parsed = new URL(entry.url);
        return { method: entry.method, path: parsed.pathname + parsed.search, status: response?.status ?? "failed" };
    });
const pageFetch = (path: string, init: Record<string, unknown> = {}) => evaluate(`(async () => { const token = (() => { try { return JSON.parse(localStorage.getItem('auth-store') || '').state?.token } catch { return undefined } })(); const response = await fetch(${JSON.stringify(path)}, { ...${JSON.stringify(init)}, headers: { ...(token ? { authorization: 'Bearer ' + token } : {}), ...(${JSON.stringify(init.headers ?? {})}) } }); let body = null; try { body = await response.json() } catch {} return { status: response.status, body }; })()`);
const screenshots: any[] = [];
const shot = async (name: string, width: number, height: number) => {
    const png = await call("Page.captureScreenshot", { format: "png" });
    const bytes = Uint8Array.from(atob(png.data), (character) => character.charCodeAt(0));
    if (bytes.length < 24 || new TextDecoder().decode(bytes.slice(1, 4)) !== "PNG") throw Error("CDP screenshot is not PNG");
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    if (view.getUint32(16) !== width || view.getUint32(20) !== height) throw Error(`screenshot viewport differs: ${name}`);
    await writeFile(`${output}/${name}.png`, bytes);
    const reread = new Uint8Array(await readFile(`${output}/${name}.png`));
    if (sha256(reread) !== sha256(bytes)) throw Error("screenshot changed after write");
    screenshots.push({ file: `${name}.png`, sha256: sha256(bytes), bytes: bytes.length, dimensions: { width, height } });
    return name;
};

// --- journey recorder --------------------------------------------------------
const journeys: any[] = [];
const journey = async (id: (typeof analyticsBusinessJourneyIds)[number], summary: string, expected: string, run: () => Promise<{ actual: string; url: string; http: any[]; screenshot?: string }>) => {
    const result = await run();
    journeys.push({ id, summary, expected, actual: result.actual, url: result.url, http: result.http, ...(result.screenshot ? { screenshot: result.screenshot } : {}), passed: true });
};

// --- journeys ----------------------------------------------------------------
await navigate("/login");
await wait(`!!document.querySelector('#login_username')`, "initial login form");
await setPreferences("dark", "en-US", 1440, 900);
await journey("deployment-identity", "published endpoint health binds the signed release", "admin and insights /health selectedBinding equal the signed build and composition", async () => {
    const admin = await pageFetch("/health");
    const binding = admin.body?.selectedBinding;
    if (admin.status !== 200 || binding?.buildId !== buildId || binding?.compositionId !== facts.compositionId) throw Error(`admin health binding differs: ${JSON.stringify({ status: admin.status, binding, expectedBuild: buildId, expectedComposition: facts.compositionId })}`);
    if (facts.insightsHealth?.selectedBinding?.buildId !== buildId || facts.insightsHealth?.selectedBinding?.compositionId !== facts.compositionId) throw Error("insights health binding differs");
    if (facts.units.join(",") !== "rz-admin.service,rz-insights.service,rz.target" || facts.currentTarget !== `releases/${buildId}/payload`) throw Error("container facts differ");
    return { actual: `admin+insights selectedBinding buildId=${buildId.slice(0, 12)}… compositionId=${facts.compositionId.slice(0, 12)}…; payload units active at ${facts.currentTarget}`, url: "/health", http: [{ method: "GET", path: "/health", status: 200 }] };
});

await journey("unauthenticated-redirect", "protected Analytics route redirects anonymous users", "navigating /analytics/overview without a session lands on /login with no successful insights API read", async () => {
    const mark = markRequests();
    await navigate("/analytics/overview");
    await wait(`location.pathname === '/login'`, "login redirect");
    await wait(`!!document.querySelector('#login_username')`, "login form");
    const leaked = httpSince(mark, (url) => new URL(url).pathname.startsWith("/api/insights")).filter((entry) => typeof entry.status === "number" && entry.status < 400);
    if (leaked.length) throw Error(`unauthenticated insights read succeeded: ${JSON.stringify(leaked)}`);
    return { actual: "redirected to /login; zero successful /api/insights responses", url: await location(), http: [] };
});

await journey("unauthenticated-api-denied", "anonymous API probes are rejected without leaks", "overview and navigation APIs answer 401 with code/message envelopes only", async () => {
    const overview = await pageFetch("/api/insights/overview");
    const navigation = await pageFetch("/api/system/modules/navigation");
    for (const probe of [overview, navigation]) {
        if (probe.status !== 401) throw Error(`expected 401, received ${probe.status}`);
        const serialized = JSON.stringify(probe.body ?? {}).toLowerCase();
        for (const forbidden of ["sqlite", "panicked", "backtrace", "/var/lib"])
            if (serialized.includes(forbidden)) throw Error(`401 body leaks ${forbidden}`);
        if (/eyj[a-za-z0-9_-]{20,}/.test(serialized) || /"authorization"/.test(serialized)) throw Error("401 body carries credential material");
    }
    return { actual: "both probes returned 401 envelopes without internal detail", url: "/login", http: [{ method: "GET", path: "/api/insights/overview", status: 401 }, { method: "GET", path: "/api/system/modules/navigation", status: 401 }] };
});

await journey("wrong-credentials-rejected", "wrong owner password is rejected in the UI", "form submit keeps the login page and the API answers 401 without secrets", async () => {
    const mark = markRequests();
    await evaluate(reactInput("#login_username", "owner"));
    await evaluate(reactInput("#login_password", "wrong-password-000111"));
    await evaluate(`document.querySelector('button[type=submit]').click()`);
    await wait(`location.pathname === '/login' && !!document.querySelector('#login_username')`, "stays on login");
    const probe = await pageFetch("/api/auth/login", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ username: "owner", password: "wrong-password-000111" }) });
    if (probe.status !== 401) throw Error(`expected 401, received ${probe.status}`);
    const serialized = JSON.stringify(probe.body ?? {});
    if (/eyj[a-za-z0-9_-]{20,}/i.test(serialized)) throw Error("401 body carries a token value");
    const screenshot = await shot("login-error", 1440, 900);
    return { actual: "form stays on /login; POST /api/auth/login answered 401 with no credentials material", url: await location(), http: httpSince(mark, (url, method) => new URL(url).pathname === "/api/auth/login" && method === "POST"), screenshot };
});

await journey("default-passwords-rejected", "default passwords never authenticate", "owner/admin/viewer with the shipped default password all answer 401", async () => {
    const statuses: number[] = [];
    for (const account of ["owner", "admin", "viewer"]) {
        const probe = await pageFetch("/api/auth/login", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ username: account, password: "rustzen@123" }) });
        statuses.push(probe.status);
    }
    if (statuses.some((status) => status !== 401)) throw Error(`default password accepted: ${JSON.stringify(statuses)}`);
    return { actual: "all three default-password probes returned 401", url: "/login", http: statuses.map((status) => ({ method: "POST", path: "/api/auth/login", status })) };
});

await journey("owner-login", "owner signs in with the P8e credential", "shell content renders and the session APIs answer 200", async () => {
    const mark = markRequests();
    await evaluate(reactInput("#login_username", "owner"));
    await evaluate(reactInput("#login_password", password));
    await evaluate(`document.querySelector('button[type=submit]').click()`);
    await wait(`!!document.querySelector('.shell-content')`, "shell content");
    const failures = httpSince(mark, (url, method) => url.includes("/api/") && method !== "OPTIONS").filter((entry) => typeof entry.status === "number" && entry.status >= 400);
    if (failures.length) throw Error(`post-login API failures: ${JSON.stringify(failures)}`);
    return { actual: "owner session established; no post-login API failure", url: await location(), http: httpSince(mark, (url) => new URL(url).pathname === "/api/auth/login") };
});

await journey("menu-navigation-isolation", "navigation shows only the Analytics composition surface", "menu lists Analytics overview/details and system entries only; clicking navigates to /analytics/overview", async () => {
    await navigate("/");
    await wait(`!!document.querySelector('.shell-navigation')`, "navigation");
    const text = String(await evaluate(`document.querySelector('.shell-navigation').innerText`));
    for (const required of ["Analytics overview", "Analytics details", "Users"])
        if (!text.includes(required)) throw Error(`menu lacks ${required}`);
    for (const forbidden of ["Monitoring", "Nodes", "Incidents", "Reports", "Templates", "Scheduled"])
        if (text.includes(forbidden)) throw Error(`menu leaks ${forbidden}`);
    await evaluate(`[...document.querySelectorAll('.shell-navigation li')].find((item) => item.innerText.includes('Analytics overview'))?.click()`);
    await wait(`location.pathname === '/analytics/overview'`, "overview route");
    return { actual: "menu contains exactly the Analytics+system entries; click navigated to the overview route", url: await location(), http: [] };
});

await journey("overview-empty", "fresh deployment overview renders zero activity", "four metric cards and the daily table render with no error state", async () => {
    const mark = markRequests();
    await navigate("/analytics/overview");
    await wait(`document.body.innerText.includes('Daily activity')`, "overview content");
    await wait(`!document.querySelector('[role=alert]')`, "no alert");
    const metrics = await evaluate(`[...document.querySelectorAll('.ant-statistic-content-value')].map((node) => Number(node.innerText.replace(/[^0-9]/g, '')))`);
    if (!Array.isArray(metrics) || metrics.length < 4 || metrics.some((value: number) => Number.isNaN(value))) throw Error(`overview metrics unreadable: ${JSON.stringify(metrics)}`);
    if (metrics.some((value: number) => value !== 0)) throw Error(`fresh overview is not empty: ${JSON.stringify(metrics)}`);
    const screenshot = await shot("overview-empty", 1440, 900);
    return { actual: `four zero metrics (${metrics.join(",")}) with the daily table and no error state`, url: await location(), http: httpSince(mark, (url) => new URL(url).pathname === "/api/insights/overview"), screenshot };
});

await journey("details-empty-mobile", "mobile zh details empty state", "390x844 light zh-CN details route shows the empty record state", async () => {
    const mark = markRequests();
    await setPreferences("light", "zh-CN", 390, 844);
    await navigate("/analytics/details");
    await wait(`document.body.innerText.includes('暂无访问记录')`, "zh empty record state");
    const screenshot = await shot("details-empty-mobile", 390, 844);
    await setPreferences("dark", "en-US", 1440, 900);
    return { actual: "zh-CN details empty state rendered on the mobile viewport", url: await location(), http: httpSince(mark, (url) => new URL(url).pathname === "/api/insights/events"), screenshot };
});

await journey("overview-loading", "loading state under emulated latency", "2.5s emulated latency exposes the loading title before data renders", async () => {
    const mark = markRequests();
    await call("Network.setCacheDisabled", { cacheDisabled: true });
    await call("Network.emulateNetworkConditions", { offline: false, latency: 2500, downloadThroughput: -1, uploadThroughput: -1 });
    await navigate("/analytics/overview");
    await wait(`document.body.innerText.includes('Loading analytics overview')`, "loading title");
    const screenshot = await shot("overview-loading", 1440, 900);
    await call("Network.emulateNetworkConditions", { offline: false, latency: 0, downloadThroughput: -1, uploadThroughput: -1 });
    await wait(`document.body.innerText.includes('Daily activity')`, "loaded overview");
    return { actual: "loading title observed under emulated latency, then the populated shell rendered", url: await location(), http: httpSince(mark, (url) => new URL(url).pathname === "/api/insights/overview"), screenshot };
});

const projectKey = crypto.randomUUID();
await journey("collection-policy-and-seed", "owner enables collection and seeds real tracker events", "policy update answers 200/configured and 24 track posts answer 2xx", async () => {
    const update = await pageFetch("/api/insights/collection-policy", { method: "PUT", headers: { "content-type": "application/json" }, body: JSON.stringify({ collectionEnabled: true, projectKey, allowedOrigins: [base] }) });
    if (update.status !== 200 || update.body?.data?.collectionEnabled !== true || update.body?.data?.projectConfigured !== true) throw Error(`policy update differs: ${JSON.stringify(update)}`);
    const seeded = await evaluate(`(async () => { const visitorId = crypto.randomUUID(); const sessionId = crypto.randomUUID(); const now = Date.now(); const events = []; for (let index = 1; index <= 22; index += 1) events.push({ eventName: 'page_view', visitorId, sessionId, platform: 'web', pagePath: '/seed/page-' + String(index).padStart(2, '0'), occurredAt: new Date(now - index * 1000).toISOString() }); for (let index = 1; index <= 2; index += 1) events.push({ eventName: 'api_request', visitorId, sessionId, platform: 'web', apiPath: '/api/seed/call-' + index, apiMethod: 'GET', statusCode: 200, durationMs: 12 + index, occurredAt: new Date(now - index * 500).toISOString() }); const results = []; for (const body of events) results.push(await (await fetch('/api/insights/track', { method: 'POST', headers: { 'content-type': 'application/json', 'x-rustzen-project-key': ${JSON.stringify(projectKey)} }, body: JSON.stringify(body) })).status); return results; })()`);
    if (!Array.isArray(seeded) || seeded.length !== 24 || seeded.some((status: number) => status < 200 || status >= 300)) throw Error(`track seeding differs: ${JSON.stringify(seeded)}`);
    return { actual: `policy enabled (projectKey sha256 ${sha256(projectKey).slice(0, 16)}…); 24 tracker events accepted`, url: "/analytics/overview", http: [{ method: "PUT", path: "/api/insights/collection-policy", status: 200 }, { method: "POST", path: "/api/insights/track", status: 200 }] };
});

await journey("overview-populated", "seeded activity surfaces in the overview", "page views >= 22 and the daily table renders after real ingestion", async () => {
    const mark = markRequests();
    await navigate("/analytics/overview");
    await wait(`[...document.querySelectorAll('.ant-statistic-content-value')].some((node) => Number(node.innerText.replace(/[^0-9]/g, '')) >= 22)`, "non-zero metrics");
    const metrics = await evaluate(`[...document.querySelectorAll('.ant-statistic-content-value')].map((node) => Number(node.innerText.replace(/[^0-9]/g, '')))`);
    if (!metrics.some((value: number) => value >= 22)) throw Error(`overview did not surface seeded activity: ${JSON.stringify(metrics)}`);
    const screenshot = await shot("overview-populated", 1440, 900);
    return { actual: `metrics [${metrics.join(",")}] reflect the seeded activity`, url: await location(), http: httpSince(mark, (url) => new URL(url).pathname === "/api/insights/overview"), screenshot };
});

await journey("details-filter-pagination", "details table with filter reset and pagination", "20 rows on page one, page two request, path filter auto-query resets to page one with a filtered row", async () => {
    const mark = markRequests();
    await navigate("/analytics/details");
    await wait(`document.querySelectorAll('tbody tr').length >= 20`, "page one rows");
    const pageOne = Number(await evaluate(`document.querySelectorAll('tbody tr').length`));
    const pageTwoMark = markRequests();
    await evaluate(`[...document.querySelectorAll('.ant-pagination-next')].find((button) => !button.className.includes('disabled'))?.click()`);
    await sleep(800);
    const pageTwoHttp = httpSince(pageTwoMark, (url) => new URL(url).pathname === "/api/insights/events");
    if (!pageTwoHttp.some((entry) => entry.path.includes("current=2"))) throw Error(`pagination request missing: ${JSON.stringify(pageTwoHttp)}`);
    const filterMark = markRequests();
    await evaluate(reactInput("input[aria-label='Search page or API path']", "/seed/page-01"));
    await sleep(900);
    const filterHttp = httpSince(filterMark, (url) => new URL(url).pathname === "/api/insights/events");
    if (!filterHttp.some((entry) => entry.path.includes("current=1") && /[?&]path=/.test(entry.path))) throw Error(`filtered page-one request missing: ${JSON.stringify(filterHttp)}`);
    const rows = await evaluate(`[...document.querySelectorAll('tbody tr')].map((row) => row.innerText).filter((text) => text.includes('/seed/page-01'))`);
    if (rows.length !== 1) throw Error(`filtered rows differ: ${rows.length} rows`);
    const screenshot = await shot("details-populated", 1440, 900);
    void mark;
    return { actual: `page one rendered ${pageOne} rows; page-two and filtered page-one requests observed; one /seed/page-01 row`, url: await location(), http: [...pageTwoHttp.slice(0, 2), ...filterHttp.slice(0, 2)], screenshot };
});

await journey("error-state-retry", "a real Insights outage renders the error state and recovers", "stopping rz-insights shows the reload error state; restart plus retry restores data", async () => {
    const mark = markRequests();
    await navigate("/analytics/overview");
    await wait(`document.body.innerText.includes('Daily activity')`, "overview before failure");
    await writeFile(`${signals}/stop-insights-requested`, String(Date.now()));
    const stopped = Date.now() + 30000;
    while (Date.now() < stopped) { if (await Bun.file(`${signals}/insights-stopped`).exists()) break; await sleep(250); }
    if (!await Bun.file(`${signals}/insights-stopped`).exists()) throw Error("insights stop signal was not answered");
    await call("Page.reload");
    await wait(`document.body.innerText.includes('Unable to read analytics data')`, "error state", 30000);
    const screenshot = await shot("error-state", 1440, 900);
    await writeFile(`${signals}/restart-requested`, String(Date.now()));
    const back = Date.now() + 120000;
    while (Date.now() < back) { if (await Bun.file(`${signals}/restart-done`).exists()) break; await sleep(500); }
    if (!await Bun.file(`${signals}/restart-done`).exists()) throw Error("restart signal was not answered");
    await evaluate(`[...document.querySelectorAll('button')].find((button) => button.innerText.includes('Reload') || button.innerText.includes('重新加载'))?.click()`);
    await wait(`[...document.querySelectorAll('.ant-statistic-content-value')].some((node) => Number(node.innerText.replace(/[^0-9]/g, '')) >= 22)`, "recovered metrics", 30000);
    return { actual: "stopped rz-insights rendered the reload error state; after restart the retry restored the seeded metrics", url: await location(), http: httpSince(mark, (url) => new URL(url).pathname === "/api/insights/overview"), screenshot };
});

await journey("service-restart-recovery", "page survives a service restart", "after insights+admin restart the deployment recovers and the reloaded page shows data", async () => {
    const mark = markRequests();
    await writeFile(`${signals}/restart-2-requested`, String(Date.now()));
    const deadline = Date.now() + 120000;
    let restarted = false;
    while (Date.now() < deadline) {
        if (await Bun.file(`${signals}/restart-2-done`).exists()) {
            restarted = true;
            break;
        }
        await sleep(500);
    }
    if (!restarted) throw Error("restart signal was not answered");
    await wait(`fetch('/health', { cache: 'no-store' }).then((response) => response.status === 200).catch(() => false)`, "health after restart", 60000);
    await navigate("/analytics/overview");
    await wait(`[...document.querySelectorAll('.ant-statistic-content-value')].some((node) => Number(node.innerText.replace(/[^0-9]/g, '')) >= 22)`, "post-restart metrics");
    const body = String(await evaluate(`document.body.innerText`));
    if (body.includes("Unable to read analytics data") || body.includes("No activity yet")) throw Error("post-restart page rendered an error or empty state");
    return { actual: "services restarted; reloaded overview shows the seeded activity with no error/empty state", url: await location(), http: httpSince(mark, (url) => new URL(url).pathname === "/health" || new URL(url).pathname === "/api/insights/overview") };
});

await journey("monitor-reports-absent", "Monitor and Reports are unreachable in this composition", "their APIs answer 404 and their routes render no module surface", async () => {
    const monitor = await pageFetch("/api/monitor/nodes");
    const reports = await pageFetch("/api/reports/templates");
    if (monitor.status !== 404 || reports.status !== 404) throw Error(`unselected module API differs: ${monitor.status}/${reports.status}`);
    await navigate("/monitoring/nodes");
    await sleep(2000);
    const monitorText = String(await evaluate(`document.body.innerText`));
    if (/节点列表|node list|告警|incident|日报|summar|监控概览/i.test(monitorText)) throw Error(`monitor surface rendered: ${monitorText.slice(0, 120)}`);
    const monitorRoute = await location();
    await navigate("/reports/templates");
    await sleep(2000);
    const reportsText = String(await evaluate(`document.body.innerText`));
    if (/模板列表|template list|运行记录|run history|报表/i.test(reportsText)) throw Error(`reports surface rendered: ${reportsText.slice(0, 120)}`);
    return { actual: `monitor/reports APIs 404; routes render no module surface (monitor page head: ${JSON.stringify(monitorText.slice(0, 60))})`, url: `${monitorRoute} -> ${await location()}`, http: [{ method: "GET", path: "/api/monitor/nodes", status: 404 }, { method: "GET", path: "/api/reports/templates", status: 404 }] };
});

await journey("logout-session-invalidation", "logout revokes the session server-side", "old bearer token answers 401 after logout and re-login succeeds", async () => {
    const tokenDigest = await evaluate(`(async () => { const token = JSON.parse(localStorage.getItem('auth-store')).state?.token ?? ''; const bytes = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(token)); return [...new Uint8Array(bytes)].map((byte) => byte.toString(16).padStart(2, '0')).join('').slice(0, 16); })()`);
    const mark = markRequests();
    const revoked = await evaluate(`(async () => { const token = JSON.parse(localStorage.getItem('auth-store')).state?.token; await fetch('/api/auth/logout', { headers: { authorization: 'Bearer ' + token } }); const probe = await fetch('/api/auth/me', { headers: { authorization: 'Bearer ' + token } }); return probe.status; })()`);
    if (revoked !== 401) throw Error(`revoked token probe returned ${revoked}`);
    await wait(`location.pathname === '/login'`, "login after logout", 10000).catch(async () => {
        await evaluate(`localStorage.removeItem('auth-store'); location.href = '/login'`);
        await wait(`location.pathname === '/login'`, "login after client-side signout");
    });
    await wait(`!!document.querySelector('#login_username')`, "login form after logout");
    await evaluate(reactInput("#login_username", "owner"));
    await evaluate(reactInput("#login_password", password));
    await evaluate(`document.querySelector('button[type=submit]').click()`);
    await wait(`!!document.querySelector('.shell-content')`, "re-login shell");
    return { actual: `logout revoked the session (token sha256 ${tokenDigest}… now 401) and the owner re-logged in`, url: await location(), http: httpSince(mark, (url) => ["/api/auth/logout", "/api/auth/me", "/api/auth/login"].includes(new URL(url).pathname)) };
});

// --- receipt -----------------------------------------------------------------
const failureSummary = requests
    .filter((entry) => responses.get(entry.requestId)?.status === "failed")
    .map((entry) => ({ path: new URL(entry.url).pathname, error: String(responses.get(entry.requestId)?.error ?? "failed") }));
const receipt = parseAnalyticsBusinessBrowserReceipt({
    schemaVersion: 1,
    kind: "analytics-business-browser-receipt",
    environment: {
        chromiumVersion: String(version.Browser ?? "chromium"),
        viewport: { width: 1440, height: 900 },
        surface: "headless-chromium-cdp",
        adminOrigin: base,
        container: { name: context.containerName, id: context.containerId, image: context.containerImage, units: facts.units },
    },
    identity: {
        head: context.head,
        harnessSourceIdentity: context.harnessSourceIdentity,
        buildId,
        compositionId: facts.compositionId,
        archiveSha256: release.archiveSha256,
        manifestSha256: release.manifestSha256,
        envelopeSha256: release.envelopeSha256,
        certificateSha256: sha256(certificateBytes),
        nativeEvidence: { path: basename(context.nativeEvidence), sha256: sha256(nativeEvidenceBytes) },
    },
    journeys,
    network: { requestsTotal: requests.length, failureSummary },
    console: { errorCount: consoleErrors.length, errors: consoleErrors.slice(0, 10) },
    screenshots,
    flags: { browser: true, runtime: { evidencePath: context.runtimeEvidence, sha256: context.runtimeEvidenceSha256 }, load: false, releaseReady: false },
});
const serializedReceipt = canonicalJson(receipt);
if (serializedReceipt.includes(password) || serializedReceipt.includes(projectKey)) throw Error("receipt carries runtime secret material");
await writeFile(`${output}/receipt.json`, new TextEncoder().encode(serializedReceipt));
socket.close();
console.log(canonicalJson({ journeys: journeys.length, screenshots: screenshots.length, requests: requests.length }));
