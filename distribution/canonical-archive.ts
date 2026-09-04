import { createHash } from "node:crypto";
import { readArtifactFileTree } from "./release-manifest-artifacts.ts";
import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import {
    canonicalManifestBytes,
    parseReleaseManifest,
} from "./release-manifest-validator.ts";
import type { ReleaseManifest } from "./release-manifest-types.ts";
import type { StagingResult } from "./native-staging.ts";

const BLOCK = 512;
const text = new TextEncoder();
const utf8 = new TextDecoder("utf-8", { fatal: true });
export type CanonicalArchiveInput = {
    selection: unknown;
    staging: StagingResult;
    manifest: ReleaseManifest;
};
export type ReadCanonicalArchive = {
    root: string;
    manifest: ReleaseManifest;
    manifestSha256: string;
};

export async function createCanonicalArchive(
    input: CanonicalArchiveInput,
): Promise<Uint8Array> {
    const { root, manifestBytes, files } = await checkedInput(input);
    const members = [
        ...files.map((file) => ({
            path: `${root}/payload/${file.entry.path}`,
            bytes: file.bytes,
            mode: file.entry.mode,
        })),
        {
            path: `${root}/release-manifest.json`,
            bytes: manifestBytes,
            mode: "0644" as const,
        },
    ].sort((left, right) => compareText(left.path, right.path));
    const parts: Uint8Array[] = [];
    for (const member of members) {
        parts.push(
            header(
                splitUstarPath(member.path),
                member.bytes.length,
                member.mode,
            ),
        );
        parts.push(member.bytes, new Uint8Array(padding(member.bytes.length)));
    }
    return concat([...parts, new Uint8Array(BLOCK * 2)]);
}

export function readCanonicalArchive(
    bytes: Uint8Array,
    selection: unknown,
): ReadCanonicalArchive {
    const source = new Uint8Array(bytes);
    if (source.length < BLOCK * 2 || source.length % BLOCK)
        throw new Error("archive block length is invalid");
    const members: Array<{
        path: string;
        bytes: Uint8Array;
        mode: "0644" | "0755";
    }> = [];
    let offset = 0;
    while (offset < source.length) {
        const block = source.slice(offset, offset + BLOCK);
        if (zero(block)) {
            if (
                !zero(source.slice(offset + BLOCK, offset + BLOCK * 2)) ||
                offset + BLOCK * 2 !== source.length
            )
                throw new Error(
                    "archive must end with exactly two zero blocks",
                );
            break;
        }
        const parsed = parseHeader(block);
        offset += BLOCK;
        if (parsed.size > source.length - offset)
            throw new Error("archive member size exceeds input");
        const content = source.slice(offset, offset + parsed.size);
        offset += parsed.size;
        const pad = padding(parsed.size);
        if (!zero(source.slice(offset, offset + pad)))
            throw new Error("archive padding must be zero");
        offset += pad;
        members.push({ path: parsed.path, bytes: content, mode: parsed.mode });
    }
    if (!members.length || offset !== source.length - BLOCK * 2)
        throw new Error("archive has no canonical members");
    const manifestMember = members.at(-1);
    if (
        !manifestMember ||
        !manifestMember.path.endsWith("/release-manifest.json")
    )
        throw new Error("archive manifest member is missing or unordered");
    let value: unknown;
    try {
        value = JSON.parse(utf8.decode(manifestMember.bytes));
    } catch {
        throw new Error("archive manifest JSON is invalid");
    }
    const manifest = parseReleaseManifest(value, selection);
    if (utf8.decode(manifestMember.bytes) !== canonicalJson(manifest))
        throw new Error("archive manifest bytes are not canonical");
    const root = archiveRoot(manifest);
    const expected = [
        ...manifest.files.map((file) => ({
            path: `${root}/payload/${file.path}`,
            mode: file.mode,
        })),
        { path: `${root}/release-manifest.json`, mode: "0644" as const },
    ].sort((left, right) => compareText(left.path, right.path));
    if (
        canonicalJson(
            members.map((member) => ({
                path: member.path,
                mode: member.mode,
            })),
        ) !== canonicalJson(expected)
    )
        throw new Error(
            "archive member inventory or order differs from manifest",
        );
    for (const file of manifest.files) {
        const member = members.find(
            (item) => item.path === `${root}/payload/${file.path}`,
        );
        if (
            !member ||
            member.mode !== file.mode ||
            member.bytes.length !== file.size ||
            sha256(member.bytes) !== file.sha256
        )
            throw new Error("archive payload differs from manifest");
    }
    return { root, manifest, manifestSha256: sha256(manifestMember.bytes) };
}

export const archiveSha256 = (bytes: Uint8Array): string =>
    createHash("sha256").update(bytes).digest("hex");

async function checkedInput(input: CanonicalArchiveInput) {
    const manifest = parseReleaseManifest(input.manifest, input.selection);
    const staging = input.staging;
    if (
        canonicalJson(Object.keys(staging).sort()) !==
        canonicalJson([
            "artifactClass",
            "buildId",
            "files",
            "root",
            "sha256",
            "target",
        ])
    )
        throw new Error("archive staging reference fields are invalid");
    if (
        staging.buildId !== manifest.buildId ||
        staging.target !== manifest.target ||
        staging.artifactClass !== manifest.artifactClass
    )
        throw new Error("archive staging tuple differs from manifest");
    if (
        !staging.root.endsWith(
            `/${staging.buildId}/${staging.target}/${staging.artifactClass}/payload`,
        )
    )
        throw new Error("archive staging root differs from tuple");
    const files = await readArtifactFileTree(staging.root);
    if (
        canonicalJson(files.map((file) => file.entry)) !==
            canonicalJson(staging.files) ||
        canonicalJson(staging.files) !== canonicalJson(manifest.files) ||
        sha256(canonicalJson(staging.files)) !== staging.sha256
    )
        throw new Error("archive staging inventory differs from manifest");
    return {
        root: archiveRoot(manifest),
        manifestBytes: canonicalManifestBytes(manifest, input.selection),
        files,
    };
}
function archiveRoot(manifest: ReleaseManifest): string {
    return `rz-${manifest.artifactClass}-${manifest.buildId}`;
}
function header(
    path: { name: Uint8Array; prefix: Uint8Array },
    size: number,
    mode: "0644" | "0755",
): Uint8Array {
    if (!Number.isSafeInteger(size) || size < 0)
        throw new Error("archive size is invalid");
    const result = new Uint8Array(BLOCK);
    result.set(path.name, 0);
    put(result, 100, 8, `000${mode}\0`);
    put(result, 108, 8, "0000000\0");
    put(result, 116, 8, "0000000\0");
    put(result, 124, 12, octal(size, 11));
    put(result, 136, 12, "00000000000\0");
    result.fill(32, 148, 156);
    result[156] = 0x30;
    put(result, 257, 6, "ustar\0");
    put(result, 263, 2, "00");
    result.set(path.prefix, 345);
    put(result, 148, 8, `${octal(sum(result), 6).slice(0, -1)}\0 `);
    return result;
}
function parseHeader(block: Uint8Array) {
    if (
        block.length !== BLOCK ||
        !field(block, 257, "ustar\0") ||
        !field(block, 263, "00") ||
        !zero(block.slice(157, 257)) ||
        !zero(block.slice(265, 345)) ||
        !zero(block.slice(500, 512)) ||
        block[156] !== 0x30
    )
        throw new Error("archive header is not canonical ustar");
    const mode = fieldText(block, 100, 8);
    if (mode !== "0000644\0" && mode !== "0000755\0")
        throw new Error("archive mode is invalid");
    if (
        !field(block, 108, "0000000\0") ||
        !field(block, 116, "0000000\0") ||
        !field(block, 136, "00000000000\0")
    )
        throw new Error("archive ownership or mtime is invalid");
    const expected = `${octal(sumWithSpaces(block), 6).slice(0, -1)}\0 `;
    if (fieldText(block, 148, 8) !== expected)
        throw new Error("archive checksum is invalid");
    const sizeField = fieldText(block, 124, 12);
    if (!/^[0-7]{11}\0$/.test(sizeField))
        throw new Error("archive size field is invalid");
    const size = Number.parseInt(sizeField.slice(0, -1), 8);
    if (!Number.isSafeInteger(size))
        throw new Error("archive size exceeds safe range");
    const name = cString(block.slice(0, 100));
    const prefix = cString(block.slice(345, 500));
    const path = prefix ? `${prefix}/${name}` : name;
    const canonical = splitUstarPath(path);
    if (
        canonical.name.length !== text.encode(name).length ||
        canonical.prefix.length !== text.encode(prefix).length
    )
        throw new Error("archive path is not canonical");
    return { path, size, mode: mode.slice(3, 7) as "0644" | "0755" };
}
export function splitUstarPath(path: string) {
    scalar(path);
    if (
        !path ||
        path.includes("\0") ||
        path.startsWith("/") ||
        path.split("/").some((part) => !part || part === "." || part === "..")
    )
        throw new Error("archive member path is invalid");
    const full = text.encode(path);
    if (full.length <= 100) return { name: full, prefix: new Uint8Array() };
    for (
        let index = path.lastIndexOf("/");
        index > 0;
        index = path.lastIndexOf("/", index - 1)
    ) {
        const prefix = text.encode(path.slice(0, index));
        const name = text.encode(path.slice(index + 1));
        if (prefix.length <= 155 && name.length <= 100) return { name, prefix };
    }
    throw new Error("archive path exceeds ustar name/prefix limits");
}
function cString(value: Uint8Array): string {
    const end = value.indexOf(0);
    const bytes = end < 0 ? value : value.slice(0, end);
    if (end >= 0 && !zero(value.slice(end)))
        throw new Error("archive text field is not zero padded");
    const result = utf8.decode(bytes);
    scalar(result);
    return result;
}
function put(block: Uint8Array, at: number, size: number, value: string) {
    const bytes = text.encode(value);
    if (bytes.length !== size) throw new Error("archive field overflow");
    block.set(bytes, at);
}
function field(block: Uint8Array, at: number, value: string) {
    return fieldText(block, at, value.length) === value;
}
function fieldText(block: Uint8Array, at: number, size: number) {
    return new TextDecoder("ascii", { fatal: true }).decode(
        block.slice(at, at + size),
    );
}
function octal(value: number, width: number) {
    const encoded = value.toString(8);
    if (encoded.length > width) throw new Error("archive octal field overflow");
    return `${encoded.padStart(width, "0")}\0`;
}
function sum(block: Uint8Array) {
    return block.reduce((total, byte) => total + byte, 0);
}
function sumWithSpaces(block: Uint8Array) {
    const copy = block.slice();
    copy.fill(32, 148, 156);
    return sum(copy);
}
function padding(size: number) {
    return (BLOCK - (size % BLOCK)) % BLOCK;
}
function zero(value: Uint8Array) {
    return value.every((byte) => byte === 0);
}
function concat(parts: Uint8Array[]) {
    const result = new Uint8Array(
        parts.reduce((total, part) => total + part.length, 0),
    );
    let offset = 0;
    for (const part of parts) {
        result.set(part, offset);
        offset += part.length;
    }
    return result;
}
function compareText(left: string, right: string) {
    return left < right ? -1 : left > right ? 1 : 0;
}
function scalar(value: string) {
    for (let index = 0; index < value.length; index++) {
        const unit = value.charCodeAt(index);
        if (unit >= 0xd800 && unit <= 0xdbff) {
            const low = value.charCodeAt(++index);
            if (low < 0xdc00 || low > 0xdfff)
                throw new Error("archive text has isolated surrogate");
        } else if (unit >= 0xdc00 && unit <= 0xdfff)
            throw new Error("archive text has isolated surrogate");
    }
}
