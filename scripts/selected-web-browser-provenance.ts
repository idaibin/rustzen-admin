import { dirname, relative, resolve } from "node:path";

const roots = [
    "scripts/verify-selected-web-bootstrap-browser.py",
    "scripts/selected-web-bootstrap-browser-cases.mjs",
    "scripts/verify-selected-web-browser-admission.ts",
    "scripts/selected-web-bootstrap-browser-receipt.ts",
    "scripts/verify-selected-web-runtime-attestation.ts",
    "scripts/selected-web-browser-provenance.ts",
    "scripts/distribution-verify-published-source-build-certificate.ts",
    "scripts/admin-browser-source-identity.sh",
] as const;
const pythonLocal = ["scripts/selected-web-bootstrap-browser-fixture.py", "scripts/selected_web_bootstrap_cdp.py"] as const;
const importPattern = /(?:import\s*(?:[^"']*?from\s*)?|export\s*[^"']*?from\s*|import\s*\()["'](\.[^"']+)["']/g;

export async function discoverVerifierSources(root: string, read = async (path: string) => Bun.file(path).bytes(), entryRoots: readonly string[] = [...roots, ...pythonLocal]) {
    const base = resolve(root), found = new Set<string>(), pending: string[] = [...entryRoots];
    while (pending.length) {
        const path = pending.pop()!, absolute = resolve(base, path);
        if (relative(base, absolute).startsWith("..")) throw Error("verifier source escaped root");
        if (found.has(path)) continue;
        const text = new TextDecoder().decode(await read(absolute)); found.add(path);
        for (const match of text.matchAll(importPattern)) {
            const local = resolve(dirname(absolute), match[1]!);
            const relativePath = relative(base, local);
            if (!relativePath.startsWith("..") && /\.(ts|js|mjs)$/.test(relativePath)) pending.push(relativePath);
        }
    }
    return [...found].sort();
}
export async function verifierProvenance(root: string, read = async (path: string) => Bun.file(path).bytes(), entryRoots?: readonly string[]) {
    return Promise.all((await discoverVerifierSources(root, read, entryRoots)).map(async path => ({
        path,
        sha256: new Bun.CryptoHasher("sha256").update(await read(resolve(root, path))).digest("hex"),
    })));
}
