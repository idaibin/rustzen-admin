import { lstat, mkdir, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { readSingleArtifactFile } from "./release-manifest-artifacts.ts";
import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import {
    completeSelectedConfigForTest,
    parseSelectedConfig,
    parseSelectedConfigBytes,
    type SelectedConfigContract,
} from "./selected-config-validator.ts";

export type ConfigRunner = (binary: "admin" | "monitor" | "agent") => unknown;
export {
    completeSelectedConfigForTest,
    parseSelectedConfig,
} from "./selected-config-validator.ts";

export async function produceSelectedConfig(
    selectionInput: unknown,
    outputRoot: string,
    run: ConfigRunner,
) {
    await rejectSymlink(outputRoot);
    const expected = completeSelectedConfigForTest(selectionInput);
    const owners =
        expected.artifactClass === "server"
            ? { access: run("admin"), monitor: run("monitor") }
            : { "monitor-agent": run("agent") };
    const contract = parseSelectedConfig(
        { ...expected, owners },
        selectionInput,
    );
    await rm(outputRoot, { recursive: true, force: true });
    await mkdir(outputRoot, { recursive: true });
    await writeFile(join(outputRoot, "config.json"), canonicalJson(contract), {
        mode: 0o644,
    });
    return readSelectedConfig(outputRoot, selectionInput);
}

export async function readSelectedConfig(
    root: string,
    selectionInput: unknown,
): Promise<{ contract: SelectedConfigContract; sha256: string }> {
    const file = await readSingleArtifactFile(root, "config.json");
    return {
        contract: parseSelectedConfigBytes(file.bytes, selectionInput),
        sha256: sha256(file.bytes),
    };
}

async function rejectSymlink(path: string) {
    try {
        if ((await lstat(path)).isSymbolicLink())
            throw new Error("config contract output root must not be symlink");
    } catch (error: unknown) {
        if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
}
