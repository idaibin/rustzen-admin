import { readdir, lstat } from "node:fs/promises";
import { join, relative } from "node:path";

import { canonicalJson, filesDigest, sha256, validHash } from "./release-manifest-core.ts";
import type { FileEntry } from "./release-manifest-types.ts";

export const WEB_BINDING_SLOT = "__RUSTZEN_WEB_DIGEST__";
const stamp = (value: string) =>
    `<meta name="rustzen-web-binding" content="${value}" />`;
const stampPattern = /<meta name="rustzen-web-binding" content="(__RUSTZEN_WEB_DIGEST__|[a-f0-9]{64})" \/>/g;
const markerPattern = /rustzen-web-binding/gi;

export type WebBinding = {
    bindingVersion: 1;
    compositionId: string;
    selectedApiDigest: string;
    webDigest: string;
};
export type WebFile = FileEntry & { bytes: Uint8Array };

export function parseWebBinding(value: unknown): WebBinding {
    if (!value || typeof value !== "object" || Array.isArray(value))
        throw new Error("selected Web binding must be an object");
    const record = value as Record<string, unknown>;
    const keys = ["bindingVersion", "compositionId", "selectedApiDigest", "webDigest"];
    if (Object.keys(record).length !== keys.length || keys.some((key) => !(key in record)))
        throw new Error("selected Web binding has an invalid schema");
    if (record.bindingVersion !== 1 || typeof record.compositionId !== "string")
        throw new Error("selected Web binding has invalid fields");
    return {
        bindingVersion: 1,
        compositionId: validHash(record.compositionId),
        selectedApiDigest: validHash(string(record.selectedApiDigest, "selectedApiDigest")),
        webDigest: validHash(string(record.webDigest, "webDigest")),
    };
}

export function canonicalBindingBytes(binding: WebBinding): Uint8Array {
    return new TextEncoder().encode(canonicalJson(parseWebBinding(binding)));
}

export function normalizeStampedIndex(bytes: Uint8Array): Uint8Array {
    const text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    if ([...text.matchAll(markerPattern)].length !== 1)
        throw new Error("selected Web index must contain one unambiguous binding marker");
    const matches = [...text.matchAll(stampPattern)];
    if (matches.length !== 1)
        throw new Error("selected Web index must contain exactly one binding stamp");
    return new TextEncoder().encode(text.replace(stampPattern, stamp(WEB_BINDING_SLOT)));
}

export function stampIndex(bytes: Uint8Array, webDigest: string): Uint8Array {
    validHash(webDigest);
    const normalized = new TextDecoder("utf-8", { fatal: true }).decode(normalizeStampedIndex(bytes));
    return new TextEncoder().encode(normalized.replace(stamp(WEB_BINDING_SLOT), stamp(webDigest)));
}

export function webDigest(files: WebFile[]): string {
    const entries = files
        .map((file) => ({
            path: checkedPath(file.path),
            sha256: sha256(file.path === "index.html" ? normalizeStampedIndex(file.bytes) : file.bytes),
        }))
        .sort(comparePath);
    if (!entries.length || new Set(entries.map((entry) => entry.path)).size !== entries.length)
        throw new Error("selected Web digest requires unique files");
    return filesDigest(entries.map((entry) => ({
        path: entry.path,
        type: "file",
        mode: "0644",
        size: 0,
        sha256: entry.sha256,
    })));
}

export function createWebBinding(input: {
    compositionId: string;
    selectedApiBytes: Uint8Array;
    files: WebFile[];
}): WebBinding {
    return {
        bindingVersion: 1,
        compositionId: validHash(input.compositionId),
        selectedApiDigest: sha256(input.selectedApiBytes),
        webDigest: webDigest(input.files),
    };
}

export function verifyWebBinding(input: {
    binding: unknown;
    compositionId: string;
    selectedApiBytes: Uint8Array;
    files: WebFile[];
}): WebBinding {
    const binding = parseWebBinding(input.binding);
    if (binding.compositionId !== input.compositionId)
        throw new Error("selected Web binding composition mismatch");
    if (binding.selectedApiDigest !== sha256(input.selectedApiBytes))
        throw new Error("selected Web binding API digest mismatch");
    if (binding.webDigest !== webDigest(input.files))
        throw new Error("selected Web binding digest mismatch");
    assertStampedDigest(binding, input.files);
    return binding;
}

export function verifyWebDigestBinding(input: {
    binding: unknown;
    compositionId: string;
    files: WebFile[];
}): WebBinding {
    const binding = parseWebBinding(input.binding);
    if (binding.compositionId !== input.compositionId)
        throw new Error("selected Web binding composition mismatch");
    if (binding.webDigest !== webDigest(input.files))
        throw new Error("selected Web binding digest mismatch");
    assertStampedDigest(binding, input.files);
    return binding;
}

export async function readWebFiles(root: string): Promise<WebFile[]> {
    const output: WebFile[] = [];
    await visit(root, root, output);
    return output.sort(comparePath);
}

async function visit(directory: string, root: string, output: WebFile[]) {
    for (const entry of await readdir(directory, { withFileTypes: true })) {
        const path = join(directory, entry.name);
        const state = await lstat(path);
        if (entry.isSymbolicLink() || state.isSymbolicLink())
            throw new Error(`selected Web output contains symlink: ${path}`);
        if (entry.isDirectory()) await visit(path, root, output);
        else if (entry.isFile()) {
            const bytes = await Bun.file(path).bytes();
            const relativePath = checkedPath(relative(root, path).replaceAll("\\", "/"));
            output.push({ path: relativePath, type: "file", mode: "0644", size: bytes.length, sha256: sha256(bytes), bytes });
        } else throw new Error(`selected Web output contains unsupported entry: ${path}`);
    }
}

function checkedPath(path: string): string {
    if (!path || path.startsWith("/") || path.includes("\\") || path.split("/").some((part) => !part || part === "." || part === ".."))
        throw new Error("selected Web binding path is unsafe");
    return path;
}
function assertStampedDigest(binding: WebBinding, files: WebFile[]) {
    const index = files.find((file) => file.path === "index.html");
    if (!index) throw new Error("selected Web output has no index.html");
    const text = new TextDecoder("utf-8", { fatal: true }).decode(index.bytes);
    if ([...text.matchAll(markerPattern)].length !== 1)
        throw new Error("selected Web index must contain one unambiguous binding marker");
    const matches = [...text.matchAll(stampPattern)];
    if (matches.length !== 1 || matches[0]?.[1] !== binding.webDigest)
        throw new Error("selected Web binding stamp mismatch");
}
function string(value: unknown, label: string): string {
    if (typeof value !== "string") throw new Error(`selected Web binding ${label} must be a string`);
    return value;
}
function comparePath(left: { path: string }, right: { path: string }) {
    return left.path < right.path ? -1 : left.path > right.path ? 1 : 0;
}
