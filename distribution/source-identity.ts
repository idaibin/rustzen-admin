import { validHash } from "./release-manifest-core.ts";

export type SourceIdentity = {
    gitSha: string;
    treeSha256: string;
    state: "clean" | "dirty";
    text: string;
};

export function parseSourceIdentity(value: string): SourceIdentity {
    const match = /^git:([0-9a-f]{40}) tree:([0-9a-f]{64}) state:(clean|dirty)$/.exec(value);
    if (!match) throw new Error("source identity is invalid");
    return {
        gitSha: match[1],
        treeSha256: validHash(match[2]),
        state: match[3] as "clean" | "dirty",
        text: value,
    };
}
