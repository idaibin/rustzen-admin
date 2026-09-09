import { createHash } from "node:crypto";
import { lstatSync, readFileSync, realpathSync } from "node:fs";
import { dirname, isAbsolute, join, resolve } from "node:path";

const [candidate, sourceSha, staged] = process.argv.slice(2);
if (!candidate || !sourceSha || !staged) throw new Error("expected candidate, source SHA, and staged binaries");
const candidateReal = realpathSync(candidate);
const cache = new Map();
const read = (name) => {
    if (cache.has(name)) return cache.get(name);
    if (isAbsolute(name) || resolve(candidateReal, name) !== join(candidateReal, name)) throw new Error(`unsafe evidence path: ${name}`);
    const path = join(candidateReal, name); const stat = lstatSync(path);
    if (!stat.isFile() || stat.isSymbolicLink() || dirname(realpathSync(path)) !== candidateReal) throw new Error(`unsafe evidence file: ${name}`);
    const bytes = readFileSync(path); cache.set(name, bytes); return bytes;
};
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const file = (name) => { const bytes = read(name); return { file: name, sha256: sha(bytes), bytes: bytes.length }; };
const selector = (name, node) => `[data-testid="monitor-node-${name}-${node}"]`;
const canonical = (nodes) => {
    const login = [{ action: "goto", url: "/login" }, { action: "waitFor", selector: "#login_username" }, { action: "fill", selector: "#login_username", value: "owner" }, { action: "fill", selector: "#login_password", value: "rustzen@123" }, { action: "click", selector: "button[type=submit]" }, { action: "waitFor", selector: ".shell-content" }];
    const detail = ({ nodeId, bootId }) => [{ action: "waitFor", selector: `[data-testid="monitor-node-view"][data-node-id="${nodeId}"]` }, { action: "assertText", selector: selector("boot-id", nodeId), text: bootId }, { action: "click", selector: `[data-testid="monitor-node-view"][data-node-id="${nodeId}"]` }, { action: "waitFor", selector: "[data-testid=monitor-node-details]" }, { action: "waitFor", selector: selector("details-boot-id", nodeId) }, { action: "assertText", selector: selector("details-boot-id", nodeId), text: bootId }, { action: "waitFor", selector: selector("history-5m", nodeId) }, { action: "assertText", selector: selector("history-5m", nodeId), text: "5-minute history" }, { action: "waitFor", selector: `${selector("history-5m", nodeId)} .recharts-wrapper` }, { action: "screenshotViewport", name: `monitor-agent-${nodeId}-detail-dark-en` }, { action: "pressKey", key: "Escape" }];
    return [{ action: "setUiPreferences", theme: "dark", locale: "en-US" }, { action: "setViewport", width: 1440, height: 900 }, ...login, { action: "goto", url: "/monitoring/nodes" }, { action: "waitFor", selector: "[data-testid=monitor-nodes-table]" }, ...nodes.flatMap(detail), { action: "assertNoHorizontalOverflow" }];
};
const png = (bytes) => {
    if (bytes.length < 33 || !bytes.subarray(0, 8).equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10])) || bytes.readUInt32BE(8) !== 13 || bytes.subarray(12, 16).toString() !== "IHDR") throw new Error("invalid PNG signature or IHDR");
    return `${bytes.readUInt32BE(16)} x ${bytes.readUInt32BE(20)}`;
};
const manifest = JSON.parse(read("manifest.json"));
if (manifest.schemaVersion !== 1 || manifest.status !== "passed" || manifest.sourceTreeSha256 !== sourceSha) throw new Error("manifest identity is invalid");
const nodes = manifest.nodeBootIds;
if (!Array.isArray(nodes) || JSON.stringify(nodes.map(({ nodeId }) => nodeId)) !== '["linux-agent-a","linux-agent-b"]' || new Set(nodes.map(({ bootId }) => bootId)).size !== 2) throw new Error("node boot mapping is invalid");
for (const [key, binary] of Object.entries({ admin: "rz-admin", monitor: "rz-monitor", reports: "rz-reports", agent: "rz-monitor-agent" })) if (manifest.binaries[key] !== sha(readFileSync(join(staged, binary)))) throw new Error(`binary hash mismatch: ${key}`);
const expected = canonical(nodes);
for (const [descriptor, name] of [[manifest.browser.steps, "browser-steps.json"], [manifest.browser.flow, "browser-flow.json"]]) if (!descriptor || descriptor.file !== name || JSON.stringify(file(name)) !== JSON.stringify({ file: name, sha256: descriptor.sha256, bytes: descriptor.bytes })) throw new Error("steps or flow binding is invalid");
if (JSON.stringify(JSON.parse(read(manifest.browser.steps.file))) !== JSON.stringify(expected)) throw new Error("canonical browser steps are invalid");
const flows = JSON.parse(read(manifest.browser.flow.file)).data;
const flow = Array.isArray(flows) ? flows.filter(({ id }) => id === manifest.browser.flow.id) : [];
if (flow.length !== 1 || JSON.stringify(flow[0].steps) !== JSON.stringify(expected)) throw new Error("persisted flow definition is invalid");
if (!Array.isArray(manifest.browser.receipts) || manifest.browser.receipts.length !== 1) throw new Error("receipt count is invalid");
const receipt = manifest.browser.receipts[0]; const receiptBytes = read("browser-steps-receipt.json");
if (!receipt || receipt.file !== "browser-steps-receipt.json" || JSON.stringify(file(receipt.file)) !== JSON.stringify({ file: receipt.file, sha256: receipt.sha256, bytes: receipt.bytes }) || receipt.runId !== manifest.browser.runId) throw new Error("receipt binding is invalid");
const entries = JSON.parse(receiptBytes).data;
if (!Array.isArray(entries) || entries.length !== expected.length || entries.some((entry, index) => entry.runId !== receipt.runId || entry.stepIndex !== index || entry.status !== "succeeded" || entry.action !== expected[index].action)) throw new Error("receipt steps are invalid");
const names = ["nodes-linux-agent-a.png", "nodes-linux-agent-b.png"];
if (!Array.isArray(manifest.browser.artifacts) || manifest.browser.artifacts.length !== 2 || JSON.stringify(manifest.browser.artifacts.map(({ file: name }) => name)) !== JSON.stringify(names)) throw new Error("detail PNG names are invalid");
for (const artifact of manifest.browser.artifacts) { const bytes = read(artifact.file); if (JSON.stringify(file(artifact.file)) !== JSON.stringify({ file: artifact.file, sha256: artifact.sha256, bytes: artifact.bytes }) || png(bytes) !== "1440 x 900" || artifact.dimensions !== "1440 x 900") throw new Error(`artifact binding is invalid: ${artifact.file}`); }
process.stdout.write("dual-Agent Nodes evidence verified\n");
