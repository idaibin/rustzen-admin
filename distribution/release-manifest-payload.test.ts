import { chmod, rm, symlink, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { expect, test } from "bun:test";
import {
    manifestInputs,
    monitorSelection,
    stagedPayloadFixture,
} from "./release-manifest-fixtures.ts";
import { produceReleaseManifest } from "./release-manifest.ts";

test("payload manifest rejects missing, extra, cross-class and mutated bytes", async () => {
    const fixture = await stagedPayloadFixture("server");
    const manifest = () =>
        produceReleaseManifest({
            ...manifestInputs,
            selection: monitorSelection,
            staging: fixture.staging,
        });
    try {
        for (const path of [
            "bin/rz-admin",
            "bin/rz-admin",
            "contracts/config/config.json",
            "contracts/native/native-layout.json",
            "contracts/protocol/protocol.json",
            "contracts/api/api.json",
            "contracts/schema/schema.json",
            "systemd/rz-monitor.service",
            "web/index.html",
        ]) {
            const moved = `${path}.held`;
            await Bun.$`mv ${join(fixture.payloadRoot, path)} ${join(fixture.payloadRoot, moved)}`;
            await expect(manifest()).rejects.toThrow();
            await Bun.$`mv ${join(fixture.payloadRoot, moved)} ${join(fixture.payloadRoot, path)}`;
        }
        for (const mutated of [
            { ...fixture.staging, buildId: "0".repeat(64) },
            { ...fixture.staging, target: "wrong" },
            { ...fixture.staging, artifactClass: "node-agent" as const },
            { ...fixture.staging, sha256: "0".repeat(64) },
            { ...fixture.staging, files: [] },
            { ...fixture.staging, unknown: true },
        ]) {
            await expect(
                produceReleaseManifest({
                    ...manifestInputs,
                    selection: monitorSelection,
                    staging: mutated as any,
                }),
            ).rejects.toThrow();
        }
        for (const path of [
            "config/rz.env",
            "contracts/extra.json",
            "bin/rz-monitor-agent",
        ]) {
            const full = join(fixture.payloadRoot, path);
            await Bun.$`mkdir -p ${join(full, "..")}`;
            await writeFile(full, "external");
            await chmod(full, 0o644);
            await expect(manifest()).rejects.toThrow("inventory");
            await rm(full);
        }
        for (const path of [
            "contracts/config/config.json",
            "contracts/native/native-layout.json",
            "contracts/protocol/protocol.json",
            "contracts/api/api.json",
            "contracts/schema/schema.json",
            "systemd/rz-monitor.service",
        ]) {
            const full = join(fixture.payloadRoot, path);
            const original = new Uint8Array(await Bun.file(full).arrayBuffer());
            await writeFile(full, "mutated");
            await expect(manifest()).rejects.toThrow();
            await writeFile(full, original);
        }
    } finally {
        await rm(fixture.root, { recursive: true, force: true });
    }
});
