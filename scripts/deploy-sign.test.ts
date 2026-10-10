import { expect, test } from "bun:test";
import crypto from "node:crypto";
import fs from "node:fs";
import { join, resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");
const signer = join(repositoryRoot, "scripts/deploy-sign.mjs");
const workDir = fs.mkdtempSync(join(fs.realpathSync(process.env.TMPDIR ?? "/tmp"), "rz-deploy-sign-"));
const keyFile = join(workDir, "test-sign-key.pem");
fs.writeFileSync(
    keyFile,
    crypto.generateKeyPairSync("ed25519").privateKey.export({ type: "pkcs8", format: "pem" }),
);
const realArtifact = join(repositoryRoot, "target/rz/rz-0.5.1-x86_64.tar");

const CORE_MEMBERS = {
    binaries: ["rz", "rz-admin"],
    units: ["rz-full.service", "rz-recovery.service", "rz-admin.service", "rz-update.service", "rz-update.path"],
    reportsConfig: false,
    identity: false,
};
const FULL_MEMBERS = {
    binaries: ["rz", "rz-admin", "rz-monitor", "rz-insights", "rz-reports"],
    units: [
        "rz-full.service",
        "rz-recovery.service",
        "rz-admin.service",
        "rz-monitor.service",
        "rz-insights.service",
        "rz-reports.service",
        "rz-update.service",
        "rz-update.path",
    ],
    reportsConfig: true,
    identity: true,
};

function elfBinary(name: string, version: string): Buffer {
    const marker =
        `RUSTZEN_RELEASE_MARKER\nartifact=rz-bundle-member\nbinary=${name}\nversion=${version}\n` +
        (name === "rz-admin"
            ? "frontend_sha256=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n"
            : "");
    const elf = Buffer.alloc(256);
    elf.writeUInt32LE(0x464c457f, 0);
    elf.writeUInt16LE(62, 18);
    elf.write(marker, 32, "utf8");
    return elf;
}

function ustarEntry(path: string, mode: number, data: Buffer): Buffer {
    const header = Buffer.alloc(512);
    header.write(path.slice(0, 99), 0, "utf8");
    header.write(data.length.toString(8).padStart(11, "0") + "\0", 124, "utf8");
    header.write(mode.toString(8).padStart(7, "0") + "\0", 100, "utf8");
    header.write("00000000000\0", 148, "utf8"); // checksum placeholder
    header.writeUInt8(0x30, 156); // regular file
    header.write("ustar\00000", 257, "utf8");
    let checksum = 0;
    for (const byte of header) checksum += byte;
    header.write(checksum.toString(8).padStart(6, "0") + "\0 ", 148, "utf8");
    return Buffer.concat([header, data, Buffer.alloc((512 - (data.length % 512)) % 512)]);
}

function buildArchive(
    version: string,
    arch: string,
    shape: typeof CORE_MEMBERS,
    mutate?: (members: Map<string, Buffer>) => void,
): string {
    const root = `rz-${version}-${arch}`;
    const members = new Map<string, Buffer>();
    for (const binary of shape.binaries) {
        members.set(`${root}/bin/${binary}`, elfBinary(binary, version));
    }
    for (const unit of shape.units) {
        members.set(`${root}/systemd/${unit}`, Buffer.from("[Unit]\n"));
    }
    members.set(`${root}/config/rz.env`, Buffer.from("RUSTZEN_ENV=production\n"));
    if (shape.reportsConfig) {
        members.set(`${root}/config/rz-reports.env`, Buffer.from("RUSTZEN_ENV=production\n"));
    }
    if (shape.identity) {
        members.set(`${root}/identity/controller.json`, Buffer.from("{}\n"));
    }
    members.set(`${root}/setup-layout.sh`, Buffer.from("#!/bin/sh\n"));
    mutate?.(members);
    const file = join(workDir, `bundle-${crypto.randomUUID()}.tar`);
    const padded = Buffer.concat([...members.values()]);
    void padded;
    const archive = Buffer.concat(
        [...members.entries()].map(([path, data]) => ustarEntry(path, path.endsWith(".sh") ? 0o755 : path.includes("/bin/") ? 0o755 : 0o644, data)),
    );
    fs.writeFileSync(file, archive);
    return file;
}

const run = (command: string, file: string, version: string, composition?: string, env?: Record<string, string>) =>
    Bun.spawnSync(
        [
            process.execPath,
            signer,
            command,
            "--file",
            file,
            "--version",
            version,
            "--arch",
            "x86_64",
            ...(composition ? ["--composition", composition] : []),
        ],
        {
            cwd: repositoryRoot,
            env: { ...process.env, RUSTZEN_DEPLOY_SIGN_KEY_FILE: keyFile, ...env },
            stdout: "pipe",
            stderr: "pipe",
        },
    );

const expectSignRejection = (file: string, version: string, composition?: string) => {
    const result = run("sign-bundle", file, version, composition);
    expect(result.exitCode).toBe(1);
    expect(result.stderr.toString()).toContain("member inventory does not match composition");
};

/// Append an extra member to an already signed bundle's tar content: the
/// verifier derives the composition from members before touching the
/// marker, so a foreign member must be rejected even in a signed artifact.
const spliceMember = (file: string, path: string, data: Buffer) => {
    const signed = fs.readFileSync(file);
    const markerOffset = signed.indexOf(Buffer.from("\nRUSTZEN_BUNDLE_SIGNED_MARKER_BEGIN\n"));
    expect(markerOffset).toBeGreaterThan(0);
    const spliced = Buffer.concat([
        signed.subarray(0, markerOffset),
        ustarEntry(path, 0o755, data),
        signed.subarray(markerOffset),
    ]);
    fs.writeFileSync(file, spliced);
};

test("signs and verifies an exact synthetic core bundle", () => {
    const file = buildArchive("1.2.3", "x86_64", CORE_MEMBERS);
    expect(run("sign-bundle", file, "1.2.3", "core").exitCode).toBe(0);
    expect(run("verify-bundle", file, "1.2.3", "core").exitCode).toBe(0);
});

test("signs and verifies an exact synthetic full bundle", () => {
    const file = buildArchive("1.2.3", "x86_64", FULL_MEMBERS);
    expect(run("sign-bundle", file, "1.2.3", "full").exitCode).toBe(0);
    expect(run("verify-bundle", file, "1.2.3", "full").exitCode).toBe(0);
});

test("rejects a core bundle carrying a foreign module binary (review counterexample)", () => {
    const file = buildArchive("1.2.3", "x86_64", CORE_MEMBERS, (members) => {
        members.set("rz-1.2.3-x86_64/bin/rz-insights", elfBinary("rz-insights", "1.2.3"));
    });
    expectSignRejection(file, "1.2.3", "core");
    // The signed path: a previously valid signed core bundle with the
    // foreign binary spliced in must be rejected by verify too.
    const valid = buildArchive("1.2.3", "x86_64", CORE_MEMBERS);
    expect(run("sign-bundle", valid, "1.2.3", "core").exitCode).toBe(0);
    spliceMember(valid, "rz-1.2.3-x86_64/bin/rz-insights", elfBinary("rz-insights", "1.2.3"));
    const verified = run("verify-bundle", valid, "1.2.3", "core");
    expect(verified.exitCode).toBe(1);
    expect(verified.stderr.toString()).toContain("member inventory does not match composition");
});

test("rejects a core bundle missing a required member", () => {
    const file = buildArchive("1.2.3", "x86_64", CORE_MEMBERS, (members) => {
        members.delete("rz-1.2.3-x86_64/bin/rz");
    });
    expectSignRejection(file, "1.2.3", "core");
});

test("rejects a core bundle with an unknown member", () => {
    const file = buildArchive("1.2.3", "x86_64", CORE_MEMBERS, (members) => {
        members.set("rz-1.2.3-x86_64/extra.txt", Buffer.from("unknown\n"));
    });
    expectSignRejection(file, "1.2.3", "core");
});

test("rejects a full bundle missing the Reports config", () => {
    const file = buildArchive("1.2.3", "x86_64", FULL_MEMBERS, (members) => {
        members.delete("rz-1.2.3-x86_64/config/rz-reports.env");
    });
    expectSignRejection(file, "1.2.3", "full");
});

const realArtifactPresent = fs.existsSync(realArtifact);
// Absence is reported as a skipped test carrying an explicit NOT-VERIFIED
// name — never as a pass.
test.skipIf(!realArtifactPresent)(
    "the real legacy full artifact verifies and is rejected as core (skipped = artifact unavailable = NOT VERIFIED)",
    () => {
        // The real artifact is signed by the real deployment key: run
        // without the test-key override so the default key resolution
        // (repository key) applies.
        const passThrough = (composition?: string) => {
            const child = Bun.spawnSync(
                [
                    process.execPath,
                    signer,
                    "verify-bundle",
                    "--file",
                    realArtifact,
                    "--version",
                    "0.5.1",
                    "--arch",
                    "x86_64",
                    ...(composition ? ["--composition", composition] : []),
                ],
                {
                    cwd: repositoryRoot,
                    env: { ...process.env },
                    stdout: "pipe",
                    stderr: "pipe",
                },
            );
            return child;
        };
        expect(passThrough().exitCode).toBe(0);
        const claimedCore = passThrough("core");
        expect(claimedCore.exitCode).toBe(1);
        expect(claimedCore.stderr.toString()).toContain("members derive full");
    },
);
