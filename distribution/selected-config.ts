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

export type ConfigRunner = (
    binary: "admin" | "monitor" | "agent",
    owner: "access" | "monitor" | "monitor-agent" | "notifications",
) => unknown;
export {
    completeSelectedConfigForTest,
    parseSelectedConfig,
    parseSelectedConfigBytes,
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
            ? {
                  access: run("admin", "access"),
                  monitor: run("monitor", "monitor"),
                  ...(expected.preset === "monitor-notify"
                      ? { notifications: run("admin", "notifications") }
                      : {}),
              }
            : { "monitor-agent": run("agent", "monitor-agent") };
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
