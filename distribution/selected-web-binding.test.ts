import { describe, expect, test } from "bun:test";

import { sha256 } from "./release-manifest-core.ts";
import {
    WEB_BINDING_SLOT,
    createWebBinding,
    normalizeStampedIndex,
    stampIndex,
    verifyWebBinding,
    type WebFile,
} from "./selected-web-binding.ts";

const bytes = (value: string) => new TextEncoder().encode(value);
const file = (path: string, value: string): WebFile => ({
    path,
    type: "file",
    mode: "0644",
    size: bytes(value).length,
    sha256: sha256(bytes(value)),
    bytes: bytes(value),
});
const slot = `<meta name="rustzen-web-binding" content="${WEB_BINDING_SLOT}" />`;

describe("selected Web binding", () => {
    test("binds one canonical stamp to the selected API and emitted files", () => {
        const files = [file("index.html", `<head>${slot}</head>`), file("assets/app.js", "app")];
        const selectedApiBytes = bytes("selected-api");
        const compositionId = "1".repeat(64);
        const binding = createWebBinding({ compositionId, selectedApiBytes, files });
        const stamped = stampIndex(files[0].bytes, binding.webDigest);
        const finalFiles = [{ ...files[0], bytes: stamped, size: stamped.length, sha256: sha256(stamped) }, files[1]];
        expect(verifyWebBinding({ binding, compositionId, selectedApiBytes, files: finalFiles })).toEqual(binding);
    });

    test("rejects browser-equivalent duplicate or noncanonical marker tags", () => {
        for (const extra of [
            "<meta name='rustzen-web-binding' content='bad'>",
            '<meta content="bad" name="rustzen-web-binding">',
            '<meta name="rustzen-web-binding" content="bad">',
        ]) {
            expect(() => normalizeStampedIndex(bytes(`<head>${slot}${extra}</head>`))).toThrow(
                "unambiguous",
            );
        }
        expect(() => normalizeStampedIndex(bytes("<meta name='rustzen-web-binding' content='bad'>"))).toThrow();
    });
});
