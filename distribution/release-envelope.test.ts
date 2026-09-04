import { generateKeyPairSync } from "node:crypto";
import { rm } from "node:fs/promises";
import { expect, test } from "bun:test";
import { createCanonicalArchive } from "./canonical-archive.ts";
import {
    parseEnvelopePayload,
    releaseEnvelopePayload,
    signReleaseEnvelope,
    verifyReleaseEnvelope,
} from "./release-envelope.ts";
import { canonicalJson } from "./release-manifest-core.ts";
import {
    monitorSelection,
    serverManifestFixture,
} from "./release-manifest-fixtures.ts";
import { canonicalManifestBytes } from "./release-manifest-validator.ts";

test("envelope is canonical, signed and bound to every fixed payload field", async () => {
    const fixture = await serverManifestFixture();
    const keys = generateKeyPairSync("ed25519");
    const trusted = {
        keyId: "test-key",
        publicKey: keys.publicKey
            .export({ type: "spki", format: "pem" })
            .toString(),
    };
    try {
        const archive = await createCanonicalArchive({
            selection: monitorSelection,
            staging: fixture.staging,
            manifest: fixture.manifest,
        });
        const payload = releaseEnvelopePayload(
            fixture.manifest,
            trusted.keyId,
            archive,
            canonicalManifestBytes(fixture.manifest, monitorSelection),
        );
        const bytes = signReleaseEnvelope(
            payload,
            keys.privateKey.export({ type: "pkcs8", format: "pem" }).toString(),
        );
        expect(verifyReleaseEnvelope(bytes, trusted)).toEqual(payload);
        expect(() =>
            parseEnvelopePayload({ ...payload, unexpected: true }),
        ).toThrow();
        expect(() =>
            parseEnvelopePayload({ ...payload, algorithm: "RSA" }),
        ).toThrow();
        const malformed = JSON.parse(new TextDecoder().decode(bytes));
        malformed.signature = "AAAA";
        expect(() =>
            verifyReleaseEnvelope(
                new TextEncoder().encode(canonicalJson(malformed)),
                trusted,
            ),
        ).toThrow();
        expect(() =>
            verifyReleaseEnvelope(bytes, { ...trusted, keyId: "wrong" }),
        ).toThrow();
        expect(() =>
            verifyReleaseEnvelope(bytes, {
                ...trusted,
                publicKey: generateKeyPairSync("ed25519")
                    .publicKey.export({ type: "spki", format: "pem" })
                    .toString(),
            }),
        ).toThrow();
    } finally {
        await rm(fixture.root, { recursive: true, force: true });
    }
});
