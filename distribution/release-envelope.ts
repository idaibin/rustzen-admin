import { sign, verify } from "node:crypto";
import { canonicalJson, sha256, validHash } from "./release-manifest-core.ts";
import type { ReleaseManifest } from "./release-manifest-types.ts";

const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8", { fatal: true });
const payloadKeys = [
    "domain",
    "envelopeVersion",
    "algorithm",
    "keyId",
    "releaseClass",
    "releaseVersion",
    "target",
    "artifactClass",
    "compositionId",
    "buildId",
    "archiveSha256",
    "manifestSha256",
    "agentProtocolContractId",
];
const keyIdPattern = /^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$/;
export type EnvelopePayload = {
    domain: "rustzen-selected-release-v1";
    envelopeVersion: 1;
    algorithm: "Ed25519";
    keyId: string;
    releaseClass: "production" | "test";
    releaseVersion: string;
    target: string;
    artifactClass: "server" | "node-agent";
    compositionId: string;
    buildId: string;
    archiveSha256: string;
    manifestSha256: string;
    agentProtocolContractId: string;
};
export type TrustedReleaseKey = { keyId: string; publicKey: string };

export function releaseEnvelopePayload(
    manifest: ReleaseManifest,
    keyId: string,
    archive: Uint8Array,
    manifestBytes: Uint8Array,
): EnvelopePayload {
    return {
        domain: "rustzen-selected-release-v1",
        envelopeVersion: 1,
        algorithm: "Ed25519",
        keyId: validKeyId(keyId),
        releaseClass: manifest.releaseClass,
        releaseVersion: manifest.releaseVersion,
        target: manifest.target,
        artifactClass: manifest.artifactClass,
        compositionId: manifest.compositionId,
        buildId: manifest.buildId,
        archiveSha256: sha256(archive),
        manifestSha256: sha256(manifestBytes),
        agentProtocolContractId: requiredProtocol(manifest),
    };
}
export function signReleaseEnvelope(
    payload: EnvelopePayload,
    privateKey: string,
): Uint8Array {
    const checked = parseEnvelopePayload(payload);
    const signature = sign(
        null,
        encoder.encode(canonicalJson(checked)),
        privateKey,
    );
    return encoder.encode(
        canonicalJson({
            payload: checked,
            signature: signature.toString("base64"),
        }),
    );
}
export function verifyReleaseEnvelope(
    bytes: Uint8Array,
    trusted: TrustedReleaseKey,
): EnvelopePayload {
    let value: unknown;
    const source = decoder.decode(bytes);
    try {
        value = JSON.parse(source);
    } catch {
        throw new Error("release envelope JSON is invalid");
    }
    const record = object(value, "release envelope");
    only(record, ["payload", "signature"]);
    const payload = parseEnvelopePayload(record.payload);
    const signature = strictBase64(record.signature);
    if (source !== canonicalJson({ payload, signature: record.signature }))
        throw new Error("release envelope bytes are not canonical");
    if (
        trusted.keyId !== payload.keyId ||
        !validKeyId(trusted.keyId)
    )
        throw new Error("release envelope key ID is not trusted");
    if (
        !verify(
            null,
            encoder.encode(canonicalJson(payload)),
            trusted.publicKey,
            signature,
        )
    )
        throw new Error("release envelope signature is invalid");
    return payload;
}
export function parseEnvelopePayload(value: unknown): EnvelopePayload {
    const record = object(value, "release envelope payload");
    only(record, payloadKeys);
    if (
        record.domain !== "rustzen-selected-release-v1" ||
        record.envelopeVersion !== 1 ||
        record.algorithm !== "Ed25519"
    )
        throw new Error("release envelope domain or algorithm is invalid");
    const releaseClass = string(record.releaseClass, "releaseClass");
    const artifactClass = string(record.artifactClass, "artifactClass");
    if (releaseClass !== "production" && releaseClass !== "test")
        throw new Error("release envelope class is invalid");
    if (artifactClass !== "server" && artifactClass !== "node-agent")
        throw new Error("release envelope artifact class is invalid");
    return {
        domain: "rustzen-selected-release-v1",
        envelopeVersion: 1,
        algorithm: "Ed25519",
        keyId: validKeyId(string(record.keyId, "keyId")),
        releaseClass,
        releaseVersion: nonempty(record.releaseVersion, "releaseVersion"),
        target: nonempty(record.target, "target"),
        artifactClass,
        compositionId: validHash(string(record.compositionId, "compositionId")),
        buildId: validHash(string(record.buildId, "buildId")),
        archiveSha256: validHash(string(record.archiveSha256, "archiveSha256")),
        manifestSha256: validHash(
            string(record.manifestSha256, "manifestSha256"),
        ),
        agentProtocolContractId: validHash(
            string(record.agentProtocolContractId, "agentProtocolContractId"),
        ),
    };
}
function requiredProtocol(manifest: ReleaseManifest): string {
    if (!manifest.agentProtocolContractId)
        throw new Error("release manifest lacks Agent protocol identity");
    return manifest.agentProtocolContractId;
}
function strictBase64(value: unknown): Buffer {
    if (
        typeof value !== "string" ||
        !/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(
            value,
        )
    )
        throw new Error("release envelope signature is not strict base64");
    const result = Buffer.from(value, "base64");
    if (result.length !== 64 || result.toString("base64") !== value)
        throw new Error("release envelope signature is not Ed25519 bytes");
    return result;
}
function object(value: unknown, label: string): Record<string, unknown> {
    if (!value || typeof value !== "object" || Array.isArray(value))
        throw new Error(`${label} must be an object`);
    return value as Record<string, unknown>;
}
function only(value: Record<string, unknown>, keys: string[]) {
    if (
        canonicalJson(Object.keys(value).sort()) !==
        canonicalJson(keys.slice().sort())
    )
        throw new Error("release envelope fields are invalid");
}
function string(value: unknown, label: string): string {
    if (typeof value !== "string") throw new Error(`${label} must be a string`);
    return value;
}
function nonempty(value: unknown, label: string): string {
    const result = string(value, label);
    if (!result) throw new Error(`${label} must be nonempty`);
    return result;
}
export function validKeyId(value: string): string {
    if (!keyIdPattern.test(value)) throw new Error("keyId is invalid");
    return value;
}
