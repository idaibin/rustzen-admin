import type { ArtifactFile } from "./release-manifest-artifacts.ts";
import { parseWebBinding, verifyWebDigestBinding } from "./selected-web-binding.ts";

export function releaseWebDigest(input: {
    compositionId: string;
    files: ArtifactFile[];
    descriptor: ArtifactFile;
}): string {
    const web = input.files
        .filter((file) => file.entry.path.startsWith("web/"))
        .map((file) => ({ ...file.entry, path: file.entry.path.slice(4) }));
    const binding = parseWebBinding(parseJson(input.descriptor.bytes));
    verifyWebDigestBinding({
        binding,
        compositionId: input.compositionId,
        files: input.files
            .filter((file) => file.entry.path.startsWith("web/"))
            .map((file) => ({ ...file.entry, path: file.entry.path.slice(4), bytes: file.bytes })),
    });
    return binding.webDigest;
}

function parseJson(bytes: Uint8Array): unknown {
    try {
        return JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes));
    } catch {
        throw new Error("selected Web binding descriptor is invalid JSON");
    }
}
