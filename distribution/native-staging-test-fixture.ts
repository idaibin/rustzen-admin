import { chmod, mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { releaseFixture, monitorSelection } from "./release-manifest-fixtures.ts";

export async function nativeStagingRoots(kind: "server" | "agent") {
    const fixture = await releaseFixture(kind);
    const binaryRoot = join(fixture.root, "binary");
    await mkdir(join(binaryRoot, "bin"), { recursive: true });
    const names =
        kind === "server" ? ["rz-admin", "rz-monitor"] : ["rz-monitor-agent"];
    for (const name of names) {
        await writeFile(join(binaryRoot, "bin", name), name);
        await chmod(join(binaryRoot, "bin", name), 0o755);
    }
    return {
        fixture,
        binaryRoot,
        selection:
            kind === "server"
                ? monitorSelection
                : { preset: "node-agent", target: monitorSelection.target },
    };
}
