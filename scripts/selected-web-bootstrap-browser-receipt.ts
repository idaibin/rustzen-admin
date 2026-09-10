import { canonicalJson } from "../distribution/release-manifest-core.ts";

const cases = ["success", "bindingMismatch", "bindingNetworkFailure", "sriEntryFailure"];
export function parseSelectedWebBootstrapBrowserReceipt(value: unknown) {
    if (!value || typeof value !== "object" || Array.isArray(value)) throw Error("P8f receipt is invalid");
    const record = value as Record<string, unknown>, keys = ["adminHealth", "cases", "chromiumVersion", "digests", "integritySensitivityPassed", "release", "runtime", "schemaVersion", "sensitivity", "sourceIdentity", "status", "verifier"];
    if (canonicalJson(Object.keys(record).sort()) !== canonicalJson(keys) || record.schemaVersion !== 1 || record.status !== "passed" || record.integritySensitivityPassed !== true || typeof record.chromiumVersion !== "string" || !record.chromiumVersion || !Array.isArray(record.cases) || canonicalJson(record.cases.map(item => (item as Record<string, unknown>)?.case)) !== canonicalJson(cases)) throw Error("P8f receipt schema differs");
    for (const item of record.cases as Record<string, unknown>[]) if (!caseReceipt(item, record.digests as Record<string, unknown>)) throw Error(`P8f receipt case differs: ${item.case}`);
    if (!object(record.sensitivity) || record.sensitivity.marker !== true || !Array.isArray(record.sensitivity.requests) || record.sensitivity.requests.some((request:any) => String(request.path).startsWith("/api/")) || (()=>{const r=record.sensitivity.requests as any[],b=r.findIndex(x=>x.path==="/__web-binding"),j=r.findIndex(x=>String(x.path).startsWith("/assets/")&&String(x.path).endsWith(".js")); return b<0||r[b].method!=="GET"||r[b].cookie!==false||r[b].authorization!==false||(j>=0&&b>j)})()) throw Error("P8f sensitivity differs");
    if (!object(record.digests) || !keysOf(record.digests, ["adminBinary", "binding", "buildId", "compositionId", "html", "installation", "verified"]) || !object(record.digests.adminBinary) || !keysOf(record.digests.adminBinary, ["after", "before"]) || !Object.values(record.digests).flatMap(value => object(value) ? Object.values(value) : [value]).every(value => /^[a-f0-9]{64}$/.test(String(value))) || record.digests.binding !== record.digests.html || record.digests.html !== record.digests.installation || record.digests.installation !== record.digests.verified || !object(record.release) || !keysOf(record.release, ["archiveSha256", "binaryDigests", "buildId", "certificateSha256", "envelopeSha256", "manifestSha256", "selection"]) || !Array.isArray(record.release.binaryDigests) || !validReceiptSelection(record.release.selection) || record.release.buildId !== record.digests.buildId || record.release.selection.compositionId !== record.digests.compositionId || !validReceiptSourceIdentity(record.sourceIdentity) || !health(record.adminHealth, record.digests) || !object(record.sensitivity) || !keysOf(record.sensitivity, ["health", "marker", "requests"]) || !health(record.sensitivity.health, record.digests, "before", "after") || !object(record.verifier) || !keysOf(record.verifier, ["sources"]) || !Array.isArray(record.verifier.sources) || record.verifier.sources.some(source => !object(source) || !keysOf(source, ["path", "sha256"]) || typeof source.path !== "string" || !/^[a-f0-9]{64}$/.test(String(source.sha256))) || !object(record.runtime) || !keysOf(record.runtime, ["after", "before"]) || !runtime(record.runtime.before) || !runtime(record.runtime.after) || canonicalJson(record.runtime.before) !== canonicalJson(record.runtime.after)) throw Error("P8f receipt runtime differs");
    return record;
}
function object(value: unknown): value is Record<string, any> { return Boolean(value) && typeof value === "object" && !Array.isArray(value); }
function keysOf(value: Record<string, unknown>, keys: string[]) { return canonicalJson(Object.keys(value).sort()) === canonicalJson(keys); }
function runtime(value: unknown) { return object(value) && keysOf(value, ["containerId", "containerPort", "dev", "hostPort", "imageId", "ino", "pid", "sha256"]) && typeof value.pid === "number" && /^[a-f0-9]{64}$/.test(String(value.sha256)); }
function caseReceipt(item: Record<string, unknown>, digests: Record<string, unknown>) {
    if (!keysOf(item, ["assertions", "browser", "case", "health", "requests"]) || !object(item.assertions) || !keysOf(item.assertions, ["bindingCredentialOmitted", "bindingFirstActive", "entryExecuted", "failureApiAbsent"]) || !health(item.health, digests, "before", "after") || !object(item.browser) || !keysOf(item.browser, ["alert", "entryExecuted", "manualRetry", "reloadCount", "stamp", "url", "value"]) || !Array.isArray(item.requests) || item.requests.some(request => !object(request) || !keysOf(request, ["authorization", "cookie", "method", "path"]))) return false;
    const assertions = item.assertions, browser = item.browser, requests = item.requests as Record<string, unknown>[], binding = requests.filter(request => request.path === "/__web-binding"), assets = requests.filter(request => typeof request.path === "string" && request.path.startsWith("/assets/") && request.path.endsWith(".js")), url = new URL(String(browser.url));
    if (!["bindingCredentialOmitted", "bindingFirstActive", "failureApiAbsent"].every(key => assertions[key] === true) || binding.length < 1 || binding.some(request => request.method !== "GET" || request.authorization !== false || request.cookie !== false) || requests.some(request => request.path === "/monitoring/nodes?q=web" && request.authorization !== false) || requests.findIndex(request => request.path === "/__web-binding") > requests.findIndex(request => typeof request.path === "string" && request.path.startsWith("/assets/") && request.path.endsWith(".js")) && assets.length) return false;
    const success = item.case === "success";
    const executesEntry = success;
    const api = requests.filter(request => typeof request.path === "string" && request.path.startsWith("/api/"));
    if (!success && (api.length || browser.value !== null || browser.stamp !== digests.html || browser.alert !== true || browser.reloadCount !== 1 || !object(browser.manualRetry) || browser.manualRetry.canonicalDocument !== "/monitoring/nodes?q=web" || browser.manualRetry.postRetryAutomaticReloadCount !== 1)) return false;
    const stamp = browser.stamp;
    return url.hostname === "127.0.0.1" && assertions.entryExecuted === executesEntry && browser.entryExecuted === executesEntry && (success ? url.pathname === "/login" && assets.length > 0 && browser.alert === false && browser.reloadCount === 0 && browser.manualRetry === null && api.length === 2 && api[0]!.method === "POST" && api[0]!.path === "/api/auth/login" && api[0]!.authorization === false && api[0]!.cookie === true && api[1]!.method === "GET" && api[1]!.path === "/api/installation" && api[1]!.authorization === true && api[1]!.cookie === true && stamp === digests.html && object(browser.value) && keysOf(browser.value, ["buildId", "compositionId", "login", "webDigest"]) && browser.value.login === true && browser.value.buildId === digests.buildId && browser.value.compositionId === digests.compositionId && browser.value.webDigest === digests.verified : url.pathname === "/monitoring/nodes" && url.searchParams.get("q") === "web" && Boolean(url.searchParams.get("__rz_web_reload")) && url.hash === "#node-1");
}
const monitorComposition = "8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b";
const notifyComposition = "0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d";
const sourceIdentityPattern = /^git:[0-9a-f]{40} tree:[0-9a-f]{64} state:(clean|dirty)$/;
export function validReceiptSelection(value: unknown) {
    if (!object(value) || value.artifactClass !== "server" || value.target !== "x86_64-unknown-linux-musl") return false;
    if (keysOf(value, ["artifactClass", "compositionId", "target"])) return value.compositionId === monitorComposition;
    if (!keysOf(value, ["artifactClass", "compositionId", "preset", "target"])) return false;
    return (value.preset === "monitor" && value.compositionId === monitorComposition) || (value.preset === "monitor-notify" && value.compositionId === notifyComposition);
}
export function validReceiptSourceIdentity(value: unknown) {
    if (!object(value)) return false;
    if (keysOf(value, ["current", "expected"])) return typeof value.expected === "string" && value.current === value.expected && sourceIdentityPattern.test(value.expected);
    return keysOf(value, ["productSourceIdentity", "verifierSourceIdentity"]) && typeof value.productSourceIdentity === "string" && sourceIdentityPattern.test(value.productSourceIdentity) && typeof value.verifierSourceIdentity === "string" && sourceIdentityPattern.test(value.verifierSourceIdentity);
}
function health(value: unknown, digests: Record<string, any>, first="initial", last="final") { if(!object(value)||!keysOf(value,[last,first])||canonicalJson(value[first])!==canonicalJson(value[last])) return false; const item=value[first]; return object(item)&&item.status==="ok"&&object(item.selectedBinding)&&item.selectedBinding.buildId===digests.buildId&&item.selectedBinding.compositionId===digests.compositionId; }

if (import.meta.main) {
    const path = Bun.argv[2];
    if (!path || Bun.argv.length !== 3) throw Error("usage: <P8f receipt>");
    const bytes = await Bun.file(path).bytes(), value = parseSelectedWebBootstrapBrowserReceipt(JSON.parse(new TextDecoder().decode(bytes)));
    if (new TextDecoder().decode(bytes) !== canonicalJson(value)) throw Error("P8f receipt is not canonical");
}
