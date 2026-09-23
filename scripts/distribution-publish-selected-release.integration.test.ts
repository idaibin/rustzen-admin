import { generateKeyPairSync } from "node:crypto";
import { cp, mkdir, rm, stat, writeFile } from "node:fs/promises";
import { basename, join, resolve } from "node:path";
import { expect, test } from "bun:test";
import {
    monitorSelection,
    serverManifestFixture,
} from "../distribution/release-manifest-fixtures.ts";
import {
    createExport,
    sourceIdentity,
} from "../distribution/container-export-test-fixture.ts";
import { verifyReleaseSnapshot } from "../distribution/release-publisher.ts";

test("selected release CLI reads PEM paths without emitting key material", async () => {
    const repository = resolve(import.meta.dir, "..");
    const fixture = await serverManifestFixture();
    const root = join(repository, "target", "release-cli-test");
    const keys = generateKeyPairSync("ed25519");
    const privateKey = keys.privateKey
        .export({ type: "pkcs8", format: "pem" })
        .toString();
    try {
        await mkdir(root, { recursive: true });
        await writeFile(
            join(root, "staging.json"),
            JSON.stringify(fixture.staging),
        );
        await writeFile(
            join(root, "manifest.json"),
            JSON.stringify(fixture.manifest),
        );
        await writeFile(join(root, "private.pem"), privateKey, { mode: 0o600 });
        await writeFile(
            join(root, "public.pem"),
            keys.publicKey.export({ type: "spki", format: "pem" }),
        );
        const result = Bun.spawnSync(
            [
                process.execPath,
                "scripts/distribution-publish-selected-release.ts",
                "--selection",
                "distribution/fixtures/monitor.json",
                "--staging",
                "target/release-cli-test/staging.json",
                "--manifest",
                "target/release-cli-test/manifest.json",
                "--private-key",
                join(root, "private.pem"),
                "--public-key",
                join(root, "public.pem"),
                "--key-id",
                "cli-test",
            ],
            { cwd: repository },
        );
        expect(result.exitCode).toBe(0);
        const output = new TextDecoder().decode(result.stdout);
        expect(JSON.parse(output).root).toEndWith("/release");
        expect(output).not.toContain(privateKey);
    } finally {
        await rm(root, { recursive: true, force: true });
        await rm(fixture.root, { recursive: true, force: true });
    }
});

test("captured Monitor release CLI strictly signs a verified export without printing PEMs", async () => {
    const repository = resolve(import.meta.dir, "..");
    const root = join(repository, "target", "container-release-cli-test");
    const keys = generateKeyPairSync("ed25519");
    const privateKey = keys.privateKey
        .export({ type: "pkcs8", format: "pem" })
        .toString();
    const exportRoot = await createExport();
    try {
        await mkdir(root, { recursive: true });
        await cp(exportRoot, join(root, "export"), { recursive: true });
        await writeFile(join(root, "private.pem"), privateKey, { mode: 0o600 });
        await writeFile(
            join(root, "public.pem"),
            keys.publicKey.export({ type: "spki", format: "pem" }),
            { mode: 0o644 },
        );
        const command = [
            process.execPath,
            "scripts/distribution-publish-monitor-container-release.ts",
            "--selection",
            "distribution/fixtures/monitor.json",
            "--export-root",
            "target/container-release-cli-test/export",
            "--expected-source-identity",
            sourceIdentity,
            "--output-base",
            "target/container-release-cli-test/output",
            "--private-key",
            join(root, "private.pem"),
            "--public-key",
            join(root, "public.pem"),
            "--key-id",
            "captured-cli",
        ];
        const invalidPublic = command.map((value, index) =>
            command[index - 1] === "--public-key"
                ? join(root, "private.pem")
                : value,
        );
        invalidPublic[invalidPublic.indexOf("--output-base") + 1] =
            "target/container-release-cli-test/invalid-output";
        const rejectedPublic = Bun.spawnSync(invalidPublic, {
            cwd: repository,
            stdout: "pipe",
            stderr: "pipe",
        });
        expect(rejectedPublic.exitCode).not.toBe(0);
        expect(
            await Bun.file(
                join(root, "invalid-output", ".native-staging"),
            ).exists(),
        ).toBeFalse();
        const result = Bun.spawnSync(command, {
            cwd: repository,
            stdout: "pipe",
            stderr: "pipe",
        });
        if (result.exitCode !== 0)
            throw new Error(new TextDecoder().decode(result.stderr));
        const output = new TextDecoder().decode(result.stdout);
        const published = JSON.parse(output);
        expect(published.root).toEndWith("/release");
        expect(output).not.toContain(privateKey);
        const found = Bun.spawnSync(
            ["find", published.root, "-maxdepth", "1", "-type", "f"],
            { stdout: "pipe", stderr: "pipe" },
        );
        if (found.exitCode !== 0)
            throw new Error(new TextDecoder().decode(found.stderr));
        const names = new TextDecoder()
            .decode(found.stdout)
            .trim()
            .split("\n")
            .map((path) => basename(path))
            .sort();
        expect(names).toEqual([
            "archive.tar",
            "release-manifest.json",
            "signature-envelope.json",
        ]);
        for (const name of names)
            expect((await stat(join(published.root, name))).mode & 0o777).toBe(
                0o644,
            );
        const trusted = {
            keyId: "captured-cli",
            publicKey: await Bun.file(join(root, "public.pem")).text(),
        };
        const verified = verifyReleaseSnapshot(
            published.root,
            { preset: "monitor", target: "x86_64-unknown-linux-musl" },
            trusted,
        );
        await expect(verified).resolves.toBeDefined();
        await writeFile(join(published.root, "archive.tar"), "tampered");
        await expect(
            verifyReleaseSnapshot(
                published.root,
                { preset: "monitor", target: "x86_64-unknown-linux-musl" },
                trusted,
            ),
        ).rejects.toThrow();
        const duplicate = Bun.spawnSync(command, {
            cwd: repository,
            stdout: "pipe",
            stderr: "pipe",
        });
        expect(duplicate.exitCode).not.toBe(0);
        for (const args of [
            [],
            ["--unknown", "x"],
            [
                "--selection",
                "distribution/fixtures/monitor.json",
                "--selection",
                "distribution/fixtures/monitor.json",
            ],
        ]) {
            const invalid = Bun.spawnSync(
                [
                    process.execPath,
                    "scripts/distribution-publish-monitor-container-release.ts",
                    ...args,
                ],
                { cwd: repository, stdout: "pipe", stderr: "pipe" },
            );
            expect(invalid.exitCode).not.toBe(0);
        }
    } finally {
        await rm(root, { recursive: true, force: true });
        await rm(exportRoot, { recursive: true, force: true });
    }
});

test("captured monitor-notify release CLI is selection-bound", async () => {
    const repository = resolve(import.meta.dir, "..");
    const root = join(
        repository,
        "target",
        "notify-container-release-cli-test",
    );
    const notify = {
        schemaVersion: 1,
        preset: "monitor-notify",
        target: "x86_64-unknown-linux-musl",
    };
    const keys = generateKeyPairSync("ed25519");
    const privateKey = keys.privateKey
        .export({ type: "pkcs8", format: "pem" })
        .toString();
    const exportRoot = await createExport(notify);
    try {
        await mkdir(root, { recursive: true });
        await cp(exportRoot, join(root, "export"), { recursive: true });
        await writeFile(join(root, "private.pem"), privateKey, { mode: 0o600 });
        await writeFile(
            join(root, "public.pem"),
            keys.publicKey.export({ type: "spki", format: "pem" }),
            { mode: 0o644 },
        );
        const command = [
            process.execPath,
            "scripts/distribution-publish-monitor-container-release.ts",
            "--selection",
            "distribution/fixtures/monitor-notify.json",
            "--export-root",
            "target/notify-container-release-cli-test/export",
            "--expected-source-identity",
            sourceIdentity,
            "--output-base",
            "target/notify-container-release-cli-test/output",
            "--private-key",
            join(root, "private.pem"),
            "--public-key",
            join(root, "public.pem"),
            "--key-id",
            "notify-cli",
        ];
        const result = Bun.spawnSync(command, {
            cwd: repository,
            stdout: "pipe",
            stderr: "pipe",
        });
        if (result.exitCode !== 0)
            throw new Error(new TextDecoder().decode(result.stderr));
        const out = JSON.parse(new TextDecoder().decode(result.stdout));
        const trusted = {
            keyId: "notify-cli",
            publicKey: await Bun.file(join(root, "public.pem")).text(),
        };
        await expect(
            verifyReleaseSnapshot(out.root, notify, trusted),
        ).resolves.toBeDefined();
        const cross = [...command];
        cross[cross.indexOf("--selection") + 1] =
            "distribution/fixtures/monitor.json";
        cross[cross.indexOf("--output-base") + 1] =
            "target/notify-container-release-cli-test/cross";
        expect(Bun.spawnSync(cross, { cwd: repository }).exitCode).not.toBe(0);
    } finally {
        await rm(root, { recursive: true, force: true });
        await rm(exportRoot, { recursive: true, force: true });
    }
});
