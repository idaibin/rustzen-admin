import { expect, test } from "bun:test";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { nativeEvidenceSummary } from "./monitor-load-admission.ts";

const names = ["published-certificate.json", "facts.json", "login-evidence.json", "verify.json", "dry-run.json", "apply.json", "install-status.json", "activate.json", "publication-marker.json", "activation-marker.json"];

async function fixture(ingress?: string) {
    const root = await mkdtemp(join(tmpdir(), "rz-sidecars-"));
    const evidence = join(root, "monitor-native-runtime-evidence.json"), release = join(root, "release-result.json");
    await Promise.all([evidence, release, ...names.map(name => join(root, name))].map(path => writeFile(path, "{}")));
    if (ingress !== undefined) await writeFile(join(root, "notification-ingress.check"), ingress);
    return { root, evidence, release };
}

test("pure Monitor validates absent ingress but preserves the 12-sidecar contract", async () => {
    for (const ingress of [undefined, "absent\n"]) {
        const value = await fixture(ingress);
        try { expect(Object.keys(await nativeEvidenceSummary(value.evidence, value.release, "monitor"))).toHaveLength(12); }
        finally { await rm(value.root, { recursive: true, force: true }); }
    }
});

test("monitor-notify requires its unauthorized ingress sidecar", async () => {
    for (const [ingress, passes] of [["unauthorized\n", true], ["absent\n", false], [undefined, false]] as const) {
        const value = await fixture(ingress);
        try {
            if (passes) expect(Object.keys(await nativeEvidenceSummary(value.evidence, value.release, "monitor-notify"))).toHaveLength(13);
            else await expect(nativeEvidenceSummary(value.evidence, value.release, "monitor-notify")).rejects.toThrow();
        } finally { await rm(value.root, { recursive: true, force: true }); }
    }
});
