import { lstat, mkdir, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { readSingleArtifactFile } from "./release-manifest-artifacts.ts";
import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import {
    generatedNativeLayout,
    type NativeLayout,
} from "./native-layout-source.ts";

export {
    generatedNativeLayout,
    nativeUnitBytes,
    type NativeLayout,
} from "./native-layout-source.ts";

export async function produceNativeLayout(
    selection: unknown,
    outputRoot: string,
) {
    await rejectSymlink(outputRoot);
    const layout = generatedNativeLayout(selection);
    await rm(outputRoot, { recursive: true, force: true });
    await mkdir(outputRoot, { recursive: true });
    await writeFile(
        join(outputRoot, "native-layout.json"),
        canonicalJson(layout),
        { mode: 0o644 },
    );
    return readNativeLayout(outputRoot, selection);
}

export async function readNativeLayout(
    root: string,
    selection: unknown,
): Promise<{ layout: NativeLayout; sha256: string }> {
    const file = await readSingleArtifactFile(root, "native-layout.json");
    const text = new TextDecoder("utf-8", { fatal: true }).decode(file.bytes);
    let value: unknown;
    try {
        value = JSON.parse(text);
    } catch {
        throw new Error("native layout is not JSON");
    }
    const layout = parseNativeLayout(value, selection);
    if (text !== canonicalJson(layout))
        throw new Error("native layout is not canonical");
    return { layout, sha256: sha256(file.bytes) };
}

export function parseNativeLayout(
    value: unknown,
    selection: unknown,
): NativeLayout {
    const expected = generatedNativeLayout(selection);
    if (!value || typeof value !== "object" || Array.isArray(value))
        throw new Error("native layout must be an object");
    if (canonicalJson(value) !== canonicalJson(expected))
        throw new Error("native layout differs from selected generated source");
    return expected;
}

async function rejectSymlink(path: string) {
    try {
        if ((await lstat(path)).isSymbolicLink())
            throw new Error("native layout output root must not be symlink");
    } catch (error: unknown) {
        if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
}
