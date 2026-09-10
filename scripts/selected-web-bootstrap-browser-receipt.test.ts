import { expect, test } from "bun:test";
import { validReceiptSelection, validReceiptSourceIdentity } from "./selected-web-bootstrap-browser-receipt.ts";
import { discoverVerifierSources, verifierProvenance } from "./selected-web-browser-provenance.ts";

const monitor = "8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b";
const notify = "0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d";
const base = { artifactClass: "server", target: "x86_64-unknown-linux-musl" };

test("receipt selection admits only the reviewed legacy and explicit tuples", () => {
    expect(validReceiptSelection({ ...base, compositionId: monitor })).toBe(true);
    expect(validReceiptSelection({ ...base, preset: "monitor", compositionId: monitor })).toBe(true);
    expect(validReceiptSelection({ ...base, preset: "monitor-notify", compositionId: notify })).toBe(true);
    expect(validReceiptSelection({ ...base, preset: "monitor-notify", compositionId: monitor })).toBe(false);
    expect(validReceiptSelection({ ...base, preset: "monitor", compositionId: notify })).toBe(false);
    expect(validReceiptSelection({ ...base, compositionId: notify })).toBe(false);
});

test("receipt identities require canonical product and verifier identities", () => {
    const identity = `git:${"a".repeat(40)} tree:${"b".repeat(64)} state:dirty`;
    expect(validReceiptSourceIdentity({ productSourceIdentity: identity, verifierSourceIdentity: identity })).toBe(true);
    expect(validReceiptSourceIdentity({ productSourceIdentity: identity, verifierSourceIdentity: "git:bad" })).toBe(false);
    expect(validReceiptSourceIdentity({ expected: identity, current: identity })).toBe(true);
    expect(validReceiptSourceIdentity({ expected: identity, current: identity.replace("dirty", "clean") })).toBe(false);
});

test("admission provenance hashes every direct verifier source and import closure", async () => {
    const discovered = await discoverVerifierSources(process.cwd());
    for (const path of ["scripts/verify-selected-web-bootstrap-browser.py", "scripts/selected-web-bootstrap-browser-fixture.py", "scripts/selected_web_bootstrap_cdp.py", "scripts/selected-web-bootstrap-browser-receipt.ts", "scripts/verify-selected-web-runtime-attestation.ts", "distribution/release-manifest-core.ts", "distribution/atomic-rename.ts"]) expect(discovered).toContain(path);
    const first = await verifierProvenance(process.cwd());
    expect(first).toHaveLength((await discoverVerifierSources(process.cwd())).length);
    expect(first.every(item => /^[a-f0-9]{64}$/.test(item.sha256))).toBe(true);
    const changed = await verifierProvenance(process.cwd(), async path => new TextEncoder().encode(path.endsWith("runtime-attestation.ts") ? "changed" : "same"));
    const baseline = await verifierProvenance(process.cwd(), async () => new TextEncoder().encode("same"));
    expect(changed.find(item => item.path.endsWith("runtime-attestation.ts"))?.sha256).not.toBe(baseline.find(item => item.path.endsWith("runtime-attestation.ts"))?.sha256);
});
test("provenance discovery follows static, export and dynamic local imports", async () => {
    const files = new Map([
        ["root.ts", 'import "./child.ts"; export { x } from "./child.ts"; import("./dynamic.ts");'],
        ["child.ts", 'export { y } from "./grandchild.ts";'], ["dynamic.ts", 'import "./grandchild.ts";'], ["grandchild.ts", "one"],
    ]);
    const read = async (path: string) => new TextEncoder().encode(files.get(path.replace(process.cwd() + "/", ""))!);
    expect(await discoverVerifierSources(process.cwd(), read, ["root.ts"])).toEqual(["child.ts", "dynamic.ts", "grandchild.ts", "root.ts"]);
    const before = await verifierProvenance(process.cwd(), read, ["root.ts"]); files.set("grandchild.ts", "two");
    const after = await verifierProvenance(process.cwd(), read, ["root.ts"]);
    expect(after.find(x => x.path === "grandchild.ts")?.sha256).not.toBe(before.find(x => x.path === "grandchild.ts")?.sha256);
});
