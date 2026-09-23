import { describe, expect, test } from "bun:test";

import { createSelectedWebBootstrap } from "./selected-web-bootstrap.ts";

const digest = "a".repeat(64);

type Harness = ReturnType<typeof harness>;
type ScriptEntry = Record<string, unknown> & { onerror?: () => void };
type BootstrapElement = { innerHTML?: string; onclick?: null | (() => void) };

function harness(input: {
    responseDigest?: string;
    reject?: boolean;
    body?: string;
    contentType?: string;
    contentLength?: string | null;
} = {}) {
    const { responseDigest = digest, reject = false, body, contentType = "application/json", contentLength = null } = input;
    const storage = new Map<string, string>();
    const appended: ScriptEntry[] = [];
    const replacements: string[] = [];
    const root: BootstrapElement = { innerHTML: "" };
    const retry = { onclick: null as null | (() => void) };
    const requests: Array<{ path: string; options: Record<string, unknown> }> = [];
    const document = {
        getElementById: (id: string): BootstrapElement => (id === "root" ? root : retry),
        querySelector: () => ({ getAttribute: () => digest }),
        createElement: () => ({}) as ScriptEntry,
        head: { append: (entry: ScriptEntry) => appended.push(entry) },
    };
    const fetch = async (path: string, options: Record<string, unknown>) => {
        requests.push({ path, options });
        if (reject) throw new Error("offline");
        const responseBody = body ?? JSON.stringify({ bindingVersion: 1, webDigest: responseDigest });
        return {
            ok: true,
            headers: {
                get: (name: string) =>
                    name === "content-type" ? contentType : name === "content-length" ? contentLength : null,
            },
            text: async () => responseBody,
        };
    };
    const sessionStorage = {
        getItem: (key: string) => storage.get(key) ?? null,
        setItem: (key: string, value: string) => storage.set(key, value),
        removeItem: (key: string) => storage.delete(key),
    };
    const location = {
        href: "https://monitor.test/monitoring/nodes?q=web#node-1",
        replace: (value: string) => replacements.push(value),
        reload: () => replacements.push("reload"),
    };
    return { appended, document, fetch, location, replacements, requests, root, sessionStorage };
}

async function run(target: Harness) {
    const generated = createSelectedWebBootstrap({
        entryPath: "assets/index-safe.js",
        entryBytes: new TextEncoder().encode("entry"),
    });
    const source = generated.html.slice("<script>".length, -"</script>".length);
    new Function("document", "fetch", "sessionStorage", "location", source)(
        target.document,
        target.fetch,
        target.sessionStorage,
        target.location,
    );
    await Bun.sleep(5);
    return generated;
}

describe("selected Web bootstrap", () => {
    test("loads the integrity-bound entry only after an exact anonymous binding", async () => {
        const target = harness();
        const generated = await run(target);
        expect(target.requests).toEqual([
            {
                path: "/__web-binding",
                options: {
                    cache: "no-store",
                    credentials: "omit",
                    redirect: "error",
                    headers: { accept: "application/json" },
                },
            },
        ]);
        expect(target.appended).toHaveLength(1);
        expect(target.appended[0]).toMatchObject({
            type: "module",
            src: "/assets/index-safe.js",
            integrity: generated.integrity,
            crossOrigin: "anonymous",
        });
        expect(target.replacements).toEqual([]);
    });

    test("preserves a deep link, reloads once, then fails closed on mismatch", async () => {
        const target = harness({ responseDigest: "b".repeat(64) });
        await run(target);
        expect(target.appended).toHaveLength(0);
        expect(target.replacements).toHaveLength(1);
        const next = new URL(target.replacements[0]);
        expect(next.pathname).toBe("/monitoring/nodes");
        expect(next.searchParams.get("q")).toBe("web");
        expect(next.hash).toBe("#node-1");
        await run(target);
        expect(target.replacements).toHaveLength(1);
        expect(target.root.innerHTML).toContain('role="alert"');
    });

    test("keeps business code unloaded when the binding request fails", async () => {
        const target = harness({ reject: true });
        await run(target);
        expect(target.appended).toHaveLength(0);
        expect(target.requests).toHaveLength(1);
    });

    test("fails closed for malformed bindings without appending the entry", async () => {
        for (const input of [
            { body: "{}" },
            { body: JSON.stringify({ bindingVersion: 1, webDigest: digest, extra: true }) },
            { body: JSON.stringify({ bindingVersion: 2, webDigest: digest }) },
            { body: JSON.stringify({ bindingVersion: 1, webDigest: digest }), contentType: "text/html" },
            { body: JSON.stringify({ bindingVersion: 1, webDigest: digest }), contentLength: "257" },
        ]) {
            const target = harness(input);
            await run(target);
            expect(target.appended).toHaveLength(0);
            expect(target.requests).toHaveLength(1);
        }
    });

    test("fails closed when the integrity-bound entry cannot load and retry restores the deep link", async () => {
        const target = harness();
        await run(target);
        expect(target.appended).toHaveLength(1);
        target.appended[0].onerror?.();
        expect(target.replacements).toHaveLength(1);
        await run(target);
        target.appended[1].onerror?.();
        expect(target.root.innerHTML).toContain('role="alert"');
        target.document.getElementById("rustzen-web-retry").onclick?.();
        const retry = new URL(target.replacements[1]);
        expect(retry.pathname).toBe("/monitoring/nodes");
        expect(retry.searchParams.get("q")).toBe("web");
        expect(retry.searchParams.has("__rz_web_reload")).toBe(false);
        expect(retry.hash).toBe("#node-1");
    });

    test("rejects an unsafe or empty entry", () => {
        expect(() =>
            createSelectedWebBootstrap({
                entryPath: "../entry.js",
                entryBytes: new Uint8Array([1]),
            }),
        ).toThrow();
        expect(() =>
            createSelectedWebBootstrap({
                entryPath: "assets/index-safe.js",
                entryBytes: new Uint8Array(),
            }),
        ).toThrow();
    });
});
