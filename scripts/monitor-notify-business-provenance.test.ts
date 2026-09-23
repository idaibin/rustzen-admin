import { expect, test } from "bun:test";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { stable } from "./monitor-load-admission.ts";
import { businessFixed, businessRoots, captureStableProvenance, requiredBusinessProvenance } from "./monitor-notify-business-provenance.ts";
import { discoverVerifierSources } from "./selected-web-browser-provenance.ts";

test("P8f-B provenance contract matches the current recursive closure", async () => {
    const discovered = await discoverVerifierSources(process.cwd(), undefined, businessRoots);
    expect([...discovered, ...businessFixed, "bootstrap.json", "context.json"].sort()).toEqual([...requiredBusinessProvenance]);
    expect(requiredBusinessProvenance).toContain("scripts/monitor-notify-business-cdp.ts");
});

test("P8f-B rejects a changed root, indirect dependency, or shell after capture", async () => {
    for (const changed of ["root.ts", "dep.ts", "gate.sh"]) {
        const root = await mkdtemp(join(tmpdir(), "rz-provenance-"));
        try {
            await writeFile(join(root, "root.ts"), 'import "./dep.ts";\n'); await writeFile(join(root, "dep.ts"), "export {};\n"); await writeFile(join(root, "gate.sh"), "true\n");
            await writeFile(join(root, "context"), "context"); await writeFile(join(root, "bootstrap"), "bootstrap");
            const capture = await captureStableProvenance(root, [{ label: "context.json", snapshot: await stable(join(root, "context")) }, { label: "bootstrap.json", snapshot: await stable(join(root, "bootstrap")) }], ["root.ts"], ["gate.sh"], ["bootstrap.json", "context.json", "dep.ts", "gate.sh", "root.ts"]);
            await writeFile(join(root, changed), "changed\n");
            await expect(capture.verifyUnchanged()).rejects.toThrow(`P8f-B provenance changed: ${changed}`);
        } finally { await rm(root, { recursive: true, force: true }); }
    }
});
