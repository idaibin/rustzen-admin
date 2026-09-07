import { lstat, mkdir, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import { readSingleArtifactFile } from "./release-manifest-artifacts.ts";
import {
    completeSelectedApiContractForTest,
    parseSelectedApiBytes,
    parseSelectedApiContract,
    type SelectedApiContract,
} from "./selected-contract-validator.ts";

export type { SelectedApiContract } from "./selected-contract-validator.ts";
export {
    completeSelectedApiContractForTest,
    parseSelectedApiContract,
} from "./selected-contract-validator.ts";

export type ContractRunner = (owner: "admin" | "monitor" | "notifications") => unknown;

export async function produceSelectedContract(
    selectionInput: unknown,
    outputRoot: string,
    run: ContractRunner,
) {
    await rejectSymlink(outputRoot);
    const expected = completeSelectedApiContractForTest(selectionInput);
    const owners: Record<string, unknown> = {
        admin: run("admin"),
        monitor: run("monitor"),
    };
    if (expected.preset === "monitor-notify") owners.notifications = run("notifications");
    const contract = parseSelectedApiContract(
        {
            compositionId: expected.compositionId,
            preset: expected.preset,
            owners,
        },
        selectionInput,
    );
    const bytes = new TextEncoder().encode(canonicalJson(contract));
    await rm(outputRoot, { recursive: true, force: true });
    await mkdir(outputRoot, { recursive: true });
    await writeFile(join(outputRoot, "api.json"), bytes, { mode: 0o644 });
    const verified = await readSelectedApiContract(outputRoot, selectionInput);
    return {
        compositionId: contract.compositionId,
        sha256: verified.sha256,
        path: join(outputRoot, "api.json"),
    };
}

export async function readSelectedApiContract(
    outputRoot: string,
    selectionInput: unknown,
): Promise<{ contract: SelectedApiContract; sha256: string }> {
    const file = await readSingleArtifactFile(outputRoot, "api.json");
    return {
        contract: parseSelectedApiBytes(file.bytes, selectionInput),
        sha256: sha256(file.bytes),
    };
}

async function rejectSymlink(path: string) {
    try {
        const stat = await lstat(path);
        if (stat.isSymbolicLink())
            throw new Error(
                "selected contract output root must not be symlink",
            );
    } catch (error: unknown) {
        if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
}
