import { mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, test } from "bun:test";
import {
    setArtifactAfterOpenHookForTest,
    setArtifactReadHookForTest,
} from "./release-manifest-artifacts.ts";
import { canonicalJson } from "./release-manifest-core.ts";
import {
    completeSelectedProtocol,
    parseSelectedProtocol,
    produceSelectedProtocol,
    readSelectedProtocol,
    reviewedProtocolOutput,
} from "./selected-protocol.ts";

const server = { preset: "monitor", target: "x86_64-unknown-linux-musl" };
const agent = { preset: "node-agent", target: "x86_64-unknown-linux-musl" };

test("selected protocol requires matching reviewed Controller and Agent outputs", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-protocol-"));
    try {
        const out = join(root, "out");
        const result = await produceSelectedProtocol(
            server,
            out,
            reviewedProtocolOutput(),
            reviewedProtocolOutput(),
        );
        expect(result.protocol.digest).toMatch(/^[a-f0-9]{64}$/);
        const output = reviewedProtocolOutput();
        const [descriptor] = output.split("\n");
        const mutatedDescriptor = descriptor.replace(
            '"version":1',
            '"version":2',
        );
        const mutated = `${mutatedDescriptor}\n${new Bun.CryptoHasher("sha256").update(mutatedDescriptor).digest("hex")}\n`;
        await expect(
            produceSelectedProtocol(server, out, output, mutated),
        ).rejects.toThrow("outputs differ");
        await expect(
            produceSelectedProtocol(server, out, mutated, mutated),
        ).rejects.toThrow("reviewed descriptor");
        const agentResult = await produceSelectedProtocol(
            agent,
            join(root, "agent"),
            reviewedProtocolOutput(),
            reviewedProtocolOutput(),
        );
        expect(agentResult.protocol.artifactClass).toBe("node-agent");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("selected protocol rejects descriptor, digest, class, composition and extra mutations", () => {
    const mutations: Array<(v: any) => void> = [
        (v) => (v.digest = "0".repeat(64)),
        (v) => (v.artifactClass = "node-agent"),
        (v) => (v.compositionId = "0".repeat(64)),
        (v) =>
            (v.descriptor = v.descriptor.replace('"version":1', '"version":2')),
        (v) => (v.extra = true),
    ];
    const valid = completeSelectedProtocol(server);
    for (const mutate of mutations) {
        const copy = structuredClone(valid);
        mutate(copy);
        expect(() => parseSelectedProtocol(copy, server)).toThrow();
    }
});

test("selected protocol accepts only the exact command wire format", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-protocol-wire-"));
    try {
        const output = reviewedProtocolOutput();
        for (const invalid of [
            output.slice(0, -1),
            `${output}\n`,
            output.replace("\n", " \n"),
        ])
            await expect(
                produceSelectedProtocol(
                    server,
                    join(root, crypto.randomUUID()),
                    invalid,
                    output,
                ),
            ).rejects.toThrow();
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("selected protocol is canonical and stable one-file input", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-protocol-"));
    const out = join(root, "out");
    try {
        const produced = await produceSelectedProtocol(
            server,
            out,
            reviewedProtocolOutput(),
            reviewedProtocolOutput(),
        );
        await writeFile(
            join(out, "protocol.json"),
            `${canonicalJson(produced.protocol)}\n`,
        );
        await expect(readSelectedProtocol(out, server)).rejects.toThrow(
            "canonical",
        );
        await writeFile(join(out, "extra.json"), "{}");
        await expect(readSelectedProtocol(out, server)).rejects.toThrow(
            "exactly protocol.json",
        );
        await rm(join(out, "extra.json"));
        await expect(readSelectedProtocol(out, agent)).rejects.toThrow(
            "reviewed descriptor",
        );
        await rm(out, { recursive: true, force: true });
        await symlink(root, out);
        await expect(
            produceSelectedProtocol(
                server,
                out,
                reviewedProtocolOutput(),
                reviewedProtocolOutput(),
            ),
        ).rejects.toThrow("symlink");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("selected protocol rejects TOCTOU replacement", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-protocol-"));
    const out = join(root, "out");
    try {
        await produceSelectedProtocol(
            server,
            out,
            reviewedProtocolOutput(),
            reviewedProtocolOutput(),
        );
        setArtifactReadHookForTest(async (path) => {
            if (path.endsWith("protocol.json")) await writeFile(path, "{}");
        });
        await expect(readSelectedProtocol(out, server)).rejects.toThrow(
            "changed",
        );
        setArtifactReadHookForTest();
        await produceSelectedProtocol(
            server,
            out,
            reviewedProtocolOutput(),
            reviewedProtocolOutput(),
        );
        setArtifactAfterOpenHookForTest(async (path) => {
            if (path.endsWith("protocol.json")) await writeFile(path, "{}");
        });
        await expect(readSelectedProtocol(out, server)).rejects.toThrow(
            "changed",
        );
    } finally {
        setArtifactReadHookForTest();
        setArtifactAfterOpenHookForTest();
        await rm(root, { recursive: true, force: true });
    }
});
