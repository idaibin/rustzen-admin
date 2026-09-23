import { rm } from "node:fs/promises";
import {
    archiveSha256,
    createCanonicalArchive,
    readCanonicalArchive,
} from "../distribution/canonical-archive.ts";
import {
    manifestInputs,
    monitorSelection,
    serverManifestFixture,
    stagedPayloadFixture,
} from "../distribution/release-manifest-fixtures.ts";
import { produceReleaseManifest } from "../distribution/release-manifest.ts";

for (const kind of ["server", "node-agent"] as const) {
    const fixture =
        kind === "server"
            ? await serverManifestFixture()
            : await stagedPayloadFixture("agent");
    const selection =
        kind === "server"
            ? monitorSelection
            : { preset: "node-agent", target: monitorSelection.target };
    const manifest =
        kind === "server"
            ? fixture.manifest
            : await produceReleaseManifest({
                  ...manifestInputs,
                  selection,
                  staging: fixture.staging,
              });
    try {
        const bytes = await createCanonicalArchive({
            selection,
            staging: fixture.staging,
            manifest,
        });
        readCanonicalArchive(bytes, selection);
        process.stdout.write(`${kind} ${archiveSha256(bytes)}\n`);
    } finally {
        await rm(fixture.root, { recursive: true, force: true });
    }
}
