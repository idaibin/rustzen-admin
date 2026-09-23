import { chmod, mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { releaseFixture, monitorSelection } from "./release-manifest-fixtures.ts";
import { resolveSelection } from "./resolver.ts";
import { selectedServerInventory } from "./selected-server-inventory.ts";

export async function nativeStagingRoots(
    kind: "server" | "agent",
    selection?: { preset: string; target: string },
) {
    const fixture = await releaseFixture(kind, selection);
    const binaryRoot = join(fixture.root, "binary");
    await mkdir(join(binaryRoot, "bin"), { recursive: true });
    const names =
        kind === "server"
            ? selectedServerInventory(
                  resolveSelection(selection ?? monitorSelection),
              ).binaries.map((path) => path.slice("bin/".length))
            : ["rz-monitor-agent"];
    for (const name of names) {
        await writeFile(join(binaryRoot, "bin", name), name);
        await chmod(join(binaryRoot, "bin", name), 0o755);
    }
    return {
        fixture,
        binaryRoot,
        selection:
            kind === "server"
                ? selection ?? monitorSelection
                : { preset: "node-agent", target: monitorSelection.target },
    };
}
