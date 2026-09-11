import { canonicalJson, sha256, validHash } from "./release-manifest-core.ts";
import { parseMonitorNativeRuntimeEvidence, type MonitorNativeRuntimeEvidence } from "./monitor-native-runtime-evidence.ts";
type Json = Record<string, any>;
const FACT_KEYS = ["keyId", "buildId", "compositionId", "certificateSha256", "manifestSha256", "archiveSha256", "envelopeSha256", "publicationSha256", "activationSha256", "adminPid", "adminDev", "adminIno", "adminSha256", "monitorPid", "monitorDev", "monitorIno", "monitorSha256", "insightsPid", "insightsDev", "insightsIno", "insightsSha256"];

export async function revalidateMonitorNativeRuntime(input: {
    evidencePath: string; admissionPath: string; releaseResultPath: string; factsPath: string; loginEvidencePath: string;
    verifyPath: string; dryRunPath: string; applyPath: string; statusPath: string; activatePath: string; publicationMarkerPath: string; activationMarkerPath: string; notificationIngressPath?: string;
}): Promise<MonitorNativeRuntimeEvidence> {
    if (Object.entries(input).some(([key, path]) => key !== "notificationIngressPath" && !path)) fail("runtime revalidator inputs are invalid");
    const evidenceText = await stableText(input.evidencePath);
    const [admission, release, facts, logins, verify, dryRun, apply, status, activate] = await Promise.all([
        input.admissionPath, input.releaseResultPath, input.factsPath, input.loginEvidencePath, input.verifyPath, input.dryRunPath, input.applyPath, input.statusPath, input.activatePath,
    ].map(readJson));
    const publicationBytes = await stableBytes(input.publicationMarkerPath);
    const activationBytes = await stableBytes(input.activationMarkerPath);
    const publication = parseJson(publicationBytes, "publication marker");
    const activationMarker = parseJson(activationBytes, "activation marker");
    const result = parseMonitorNativeRuntimeEvidence(JSON.parse(evidenceText));
    if (evidenceText !== canonicalJson(result)) fail("runtime evidence is not canonical");
    bindAdmission(admission, release, result); bindFacts(facts, result); bindLogins(logins); await bindNotificationIngress(input.notificationIngressPath, result);
    if (facts.publicationSha256 !== sha256(publicationBytes) || facts.activationSha256 !== sha256(activationBytes) || facts.publicationSha256 !== result.markers.publicationSha256 || facts.activationSha256 !== result.markers.activationSha256) fail("marker bytes differ");
    assertCommand(verify, "verify", { build_id: result.selection.buildId, artifact_class: "server", target: result.selection.target, files: "positive" });
    assertCommand(dryRun, "apply", { dry_run: true, release: "release" }); assertRelease(dryRun.data.release, result);
    assertCommand(apply, "apply", { dry_run: false, release: "release" }); assertRelease(apply.data.release, result);
    assertCommand(status, "install-status", { present: true, markerPresent: true, runnable: false, destination: "/opt/rz", current: `releases/${result.selection.buildId}/payload` });
    assertCommand(activate, result.selection.preset === "analytics" ? "activate-analytics-server" : "activate-monitor-server", { unit: "rz.target", config: result.selection.preset === "analytics" ? ["/opt/rz/config/rz-admin.env", "/opt/rz/config/rz-insights.env"] : ["/opt/rz/config/rz-admin.env", "/opt/rz/config/rz-monitor.env"] });
    bindMarkers(publication, activationMarker, result);
    if (canonicalJson(admission.binaryDigests) !== canonicalJson(result.release.binaryDigests)) fail("certificate binary inventory differs");
    return result;
}
function bindAdmission(admission: Json, release: Json, result: MonitorNativeRuntimeEvidence) {
    const legacyMonitor = result.selection.preset === "monitor" && result.checks.notificationIngress === undefined;
    only(admission, ["archiveSha256", "binaryDigests", "buildId", "certificateSha256", "envelopeSha256", "manifestSha256", "selection"]);
    if (legacyMonitor && admission.selection.preset === undefined) {
        only(admission.selection, ["artifactClass", "compositionId", "target"]);
    } else {
        only(admission.selection, ["artifactClass", "compositionId", "preset", "target"]);
    }
    only(release, ["archiveSha256", "buildId", "envelopeSha256", "manifestSha256", "root"]);
    const preset = legacyMonitor ? admission.selection.preset ?? "monitor" : admission.selection.preset;
    if (preset !== result.selection.preset || admission.selection.target !== "x86_64-unknown-linux-musl" || admission.selection.artifactClass !== "server" || admission.buildId !== result.selection.buildId || admission.buildId !== release.buildId || admission.certificateSha256 !== result.release.certificateSha256 || admission.selection.compositionId !== result.selection.compositionId) fail("evidence selection differs");
    for (const key of ["archiveSha256", "manifestSha256", "envelopeSha256"]) if (admission[key] !== release[key] || admission[key] !== result.release[key]) fail("release admission differs");
}
function bindFacts(facts: Json, result: MonitorNativeRuntimeEvidence) {
    const analytics = result.selection.preset === "analytics";
    only(facts, analytics ? FACT_KEYS.filter((key) => !key.startsWith("monitor")) : FACT_KEYS.filter((key) => !key.startsWith("insights"))); const [admin, secondary] = result.services;
    const secondaryPrefix = analytics ? "insights" : "monitor";
    if (facts.keyId !== result.release.keyId || facts.buildId !== result.selection.buildId || facts.compositionId !== result.selection.compositionId || facts.certificateSha256 !== result.release.certificateSha256 || facts.manifestSha256 !== result.release.manifestSha256 || facts.archiveSha256 !== result.release.archiveSha256 || facts.envelopeSha256 !== result.release.envelopeSha256 || facts.adminPid !== admin.mainPid || facts.adminDev !== admin.executable.dev || facts.adminIno !== admin.executable.ino || facts.adminSha256 !== admin.executable.sha256 || facts[`${secondaryPrefix}Pid`] !== secondary.mainPid || facts[`${secondaryPrefix}Dev`] !== secondary.executable.dev || facts[`${secondaryPrefix}Ino`] !== secondary.executable.ino || facts[`${secondaryPrefix}Sha256`] !== secondary.executable.sha256) fail("runtime facts differ");
}
function bindLogins(logins: Json) {
    only(logins, ["owner", "defaults"]); only(logins.owner, ["status", "body"]); if (logins.owner.status !== "200" || typeof logins.owner.body?.data?.token !== "string" || !logins.owner.body.data.token) fail("owner login differs");
    if (!Array.isArray(logins.defaults) || logins.defaults.length !== 3) fail("default login inventory differs");
    for (const [index, account] of ["owner", "admin", "viewer"].entries()) { const item = logins.defaults[index]; only(item, ["account", "status", "body"]); if (item.account !== account || item.status !== "401" || item.body?.code !== 10101) fail("default login differs"); }
}
function bindMarkers(publication: Json, activation: Json, result: MonitorNativeRuntimeEvidence) {
    only(publication, ["journal", "state", "version"]); only(publication.journal, ["archiveSha256", "artifactClass", "buildId", "compositionId", "envelopeSha256", "keyId", "manifestSha256", "phase", "target", "trustedKeySha256", "version", "workRootNonce"]);
    const journal = publication.journal; if (publication.version !== 1 || publication.state !== "payload-published" || journal.version !== 1 || journal.phase !== "payload-publishing" || journal.buildId !== result.selection.buildId || journal.compositionId !== result.selection.compositionId || journal.target !== result.selection.target || journal.artifactClass !== "server" || journal.keyId !== result.release.keyId || journal.archiveSha256 !== result.release.archiveSha256 || journal.manifestSha256 !== result.release.manifestSha256 || journal.envelopeSha256 !== result.release.envelopeSha256 || !validNonce(journal.workRootNonce) || !valid(journal.trustedKeySha256)) fail("publication marker differs");
    only(activation, ["adminConfigSha256", "buildId", "monitorConfigSha256", "schemaFingerprint", "dataContractId", "state", "unitSha256", "version"]);
    const owners = result.selection.preset === "monitor-notify" ? ["admin", "admin-notifications", "monitor", "monitor-notifications"] : result.selection.preset === "analytics" ? ["admin", "insights"] : ["admin", "monitor"];
    const units = result.selection.preset === "analytics" ? ["rz-admin.service", "rz-insights.service", "rz.target"] : ["rz-admin.service", "rz-monitor.service", "rz.target"];
    only(activation.schemaFingerprint, owners); only(activation.dataContractId, owners); only(activation.unitSha256, units);
    if (activation.version !== 2 || activation.state !== "ready" || activation.buildId !== result.selection.buildId || ![activation.adminConfigSha256, activation.monitorConfigSha256, ...Object.values(activation.schemaFingerprint), ...Object.values(activation.dataContractId), ...Object.values(activation.unitSha256)].every(valid)) fail("activation marker differs");
}
async function stableBytes(path: string) { const first = new Uint8Array(await Bun.file(path).arrayBuffer()); const second = new Uint8Array(await Bun.file(path).arrayBuffer()); if (sha256(first) !== sha256(second)) fail("runtime evidence input changed while read"); return first; }
async function stableText(path: string) { return new TextDecoder().decode(await stableBytes(path)); }
async function readJson(path: string): Promise<Json> { return JSON.parse(await stableText(path)); }
function parseJson(bytes: Uint8Array, label: string): Json { try { const value = JSON.parse(new TextDecoder().decode(bytes)); if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(); return value; } catch { fail(`${label} is invalid`); } }
function assertRelease(value: Json, evidence: MonitorNativeRuntimeEvidence) { only(value, ["build_id", "artifact_class", "target", "files"]); if (value.build_id !== evidence.selection.buildId || value.artifact_class !== "server" || value.target !== evidence.selection.target || !Number.isSafeInteger(value.files) || value.files < 1) fail("apply release differs"); }
function assertCommand(value: Json, command: string, data: Json) { only(value, ["schema_version", "ok", "command", "data"]); if (value.schema_version !== 1 || value.ok !== true || value.command !== command || !value.data) fail("CLI envelope differs"); only(value.data, Object.keys(data)); for (const [key, expected] of Object.entries(data)) if (expected === "positive" ? !Number.isSafeInteger(value.data[key]) || value.data[key] < 1 : expected === "release" ? !value.data[key] : canonicalJson(value.data[key]) !== canonicalJson(expected)) fail("CLI data differs"); }
function only(value: Json, keys: string[]) { if (!value || typeof value !== "object" || Array.isArray(value) || Object.keys(value).length !== keys.length || Object.keys(value).some((key) => !keys.includes(key))) fail("JSON schema differs"); }
function valid(value: unknown) { try { return typeof value === "string" && validHash(value) === value; } catch { return false; } }
function validNonce(value: unknown) { return typeof value === "string" && /^[a-f0-9]{32,128}$/.test(value); }
function fail(message: string): never { throw new Error(message); }

async function bindNotificationIngress(path: string | undefined, result: MonitorNativeRuntimeEvidence) {
    if (!path) { if (result.selection.preset === "monitor-notify") fail("notification ingress evidence is required"); return; }
    const value = (await stableText(path)).trim();
    const expected = result.selection.preset === "monitor-notify" ? "unauthorized" : "absent";
    if (value !== expected || result.checks.notificationIngress !== expected) fail("notification ingress evidence differs");
}
