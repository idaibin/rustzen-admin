import { relative, resolve } from "node:path";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { stable, type Stable } from "./monitor-load-admission.ts";
import { discoverVerifierSources } from "./selected-web-browser-provenance.ts";

export const businessRoots = ["scripts/monitor-notify-business-browser-driver.ts", "scripts/monitor-notify-business-browser-receipt.ts", "scripts/monitor-notify-business-context.ts", "scripts/monitor-notify-business-admission.ts"] as const;
export const businessFixed = ["scripts/verify-monitor-notify-business-browser-linux.sh", "scripts/verify-monitor-notify-business-browser-workflow-linux.sh", "scripts/cleanup-retained-p8e-container.sh"] as const;
export const requiredBusinessProvenance = [
    "bootstrap.json", "context.json",
    "distribution/container-export-elf.ts", "distribution/container-export-path.ts", "distribution/container-export-plan.ts", "distribution/container-export-validator.ts",
    "distribution/monitor-native-runtime-evidence.ts", "distribution/monitor-native-runtime-revalidator.ts", "distribution/native-layout-source.ts", "distribution/native-layout.ts",
    "distribution/release-manifest-artifacts.ts", "distribution/release-manifest-core.ts", "distribution/release-manifest-types.ts", "distribution/resolver.ts", "distribution/schema-contract.ts",
    "distribution/selected-config-validator.ts", "distribution/selected-config.ts", "distribution/selected-contract-validator.ts", "distribution/selected-protocol.ts", "distribution/selected-web-binding.ts",
    "distribution/source-build-plan.ts", "distribution/workspace-version.ts",
    "scripts/cleanup-retained-p8e-container.sh", "scripts/distribution-web-allowed-packages.ts", "scripts/distribution-web-inventory-policy.ts", "scripts/distribution-web-inventory-schema.ts",
    "scripts/monitor-load-admission.ts", "scripts/monitor-load-signed.ts", "scripts/monitor-notify-business-admission.ts", "scripts/monitor-notify-business-browser-driver.ts",
    "scripts/monitor-notify-business-browser-receipt.ts", "scripts/monitor-notify-business-cdp.ts", "scripts/monitor-notify-business-context.ts", "scripts/monitor-notify-business-provenance.ts",
    "scripts/selected-web-bootstrap-browser-receipt.ts", "scripts/selected-web-browser-provenance.ts", "scripts/verify-monitor-notify-business-browser-linux.sh",
    "scripts/verify-monitor-notify-business-browser-workflow-linux.sh",
] as const;

type Input = { label: string; snapshot: Stable };
export async function captureStableProvenance(root: string, inputs: Input[], entryRoots: readonly string[], fixed: readonly string[], expected: readonly string[]) {
    const base = resolve(root), captured = new Map<string, { path: string; sha256: string }>();
    const capture = async (path: string) => {
        const absolute = resolve(base, path), label = relative(base, absolute);
        const found = captured.get(label);
        if (found) return Uint8Array.from((await stable(found.path)).bytes);
        const snapshot = await stable(absolute); captured.set(label, { path: snapshot.path, sha256: snapshot.sha256 }); return Uint8Array.from(snapshot.bytes);
    };
    await discoverVerifierSources(base, capture, entryRoots);
    for (const path of fixed) await capture(path);
    for (const input of inputs) captured.set(input.label, { path: input.snapshot.path, sha256: input.snapshot.sha256 });
    const labels = [...captured.keys()].sort();
    if (canonicalJson(labels) !== canonicalJson(expected)) throw Error("P8f-B provenance closure differs");
    const entries = labels.map(path => ({ path, sha256: captured.get(path)!.sha256 }));
    return { entries, async verifyUnchanged() { for (const label of labels) if ((await stable(captured.get(label)!.path)).sha256 !== captured.get(label)!.sha256) throw Error(`P8f-B provenance changed: ${label}`); } };
}

export function captureBusinessProvenance(root: string, context: Stable, bootstrap: Stable) {
    return captureStableProvenance(root, [{ label: "context.json", snapshot: context }, { label: "bootstrap.json", snapshot: bootstrap }], businessRoots, businessFixed, requiredBusinessProvenance);
}
