import { afterEach, expect, test } from "bun:test";

const persistedAuthStore = new Map<string, string>();
const localStorageMock = {
    getItem: (key: string) => persistedAuthStore.get(key) ?? null,
    setItem: (key: string, value: string) => persistedAuthStore.set(key, value),
    removeItem: (key: string) => persistedAuthStore.delete(key),
    clear: () => persistedAuthStore.clear(),
};
Object.defineProperty(globalThis, "localStorage", {
    configurable: true,
    value: localStorageMock,
});
Object.defineProperty(globalThis, "window", {
    configurable: true,
    value: { localStorage: localStorageMock },
});

const [{ backupModuleLogs }, { moduleLogAPI }, { useAuthStore }] = await Promise.all([
    import("@/api/generated/admin-contract"),
    import("@/api/system/status/module-logs"),
    import("@/store/useAuthStore"),
]);

afterEach(() => {
    useAuthStore.getState().clearAuth();
});

test("module log adapter unwraps typed metadata, tail, preview, and confirmation responses", async () => {
    useAuthStore.getState().updateToken("module-log-token");
    const originalFetch = globalThis.fetch;
    const calls: Array<[string, RequestInit | undefined]> = [];
    globalThis.fetch = (async (url, init) => {
        const requestUrl =
            typeof url === "string" ? url : url instanceof URL ? url.toString() : url.url;
        calls.push([requestUrl, init]);
        if (requestUrl.startsWith("/api/system/status/module-logs/tail")) {
            return new Response(
                JSON.stringify({
                    code: 0,
                    message: "Success",
                    data: {
                        module: "admin",
                        date: "2026-08-10",
                        content: "tail",
                        nextCursor: "cursor-1",
                        truncated: true,
                        lineCount: 1,
                        byteCount: 4,
                    },
                }),
                { status: 200 },
            );
        }
        if (requestUrl.endsWith("cleanup/preview")) {
            return new Response(
                JSON.stringify({
                    code: 0,
                    message: "Success",
                    data: {
                        previewId: "preview-1",
                        token: "token-1",
                        expiresAt: "2026-08-10T01:00:00Z",
                        cutoffDate: "2026-08-01",
                        candidates: [],
                        failures: [],
                    },
                }),
                { status: 200 },
            );
        }
        if (requestUrl.endsWith("cleanup/confirm")) {
            return new Response(
                JSON.stringify({
                    code: 0,
                    message: "Success",
                    data: {
                        previewId: "preview-1",
                        removed: [],
                        retained: [],
                        failures: [],
                        partial: false,
                    },
                }),
                { status: 200 },
            );
        }
        return new Response(
            JSON.stringify({
                code: 0,
                message: "Success",
                data: [
                    {
                        module: "admin",
                        fileName: "admin.2026-08-10",
                        date: "2026-08-10",
                        sizeBytes: 4,
                        modifiedAt: "2026-08-10T00:00:00Z",
                        readable: true,
                        active: true,
                    },
                ],
            }),
            { status: 200 },
        );
    }) as typeof fetch;
    try {
        await expect(
            moduleLogAPI.list({ module: "admin", date: "2026-08-10" }),
        ).resolves.toHaveLength(1);
        await expect(
            moduleLogAPI.tail({ module: "admin", date: "2026-08-10" }),
        ).resolves.toMatchObject({ content: "tail", truncated: true, nextCursor: "cursor-1" });
        await expect(moduleLogAPI.previewCleanup()).resolves.toMatchObject({
            previewId: "preview-1",
        });
        await expect(moduleLogAPI.confirmCleanup("token-1")).resolves.toMatchObject({
            previewId: "preview-1",
        });
        expect(calls.map(([url]) => url)).toEqual([
            "/api/system/status/module-logs?module=admin&date=2026-08-10",
            "/api/system/status/module-logs/tail?module=admin&date=2026-08-10",
            "/api/system/status/module-logs/cleanup/preview",
            "/api/system/status/module-logs/cleanup/confirm",
        ]);
        expect(new Headers(calls[0]?.[1]?.headers).get("authorization")).toBe(
            "Bearer module-log-token",
        );
        expect(calls[3]?.[1]?.body).toBe(JSON.stringify({ token: "token-1" }));
    } finally {
        globalThis.fetch = originalFetch;
    }
});

test("module log backup validates archive metadata before downloading", async () => {
    const originalFetch = globalThis.fetch;
    const originalDocument = globalThis.document;
    const originalCreateObjectURL = URL.createObjectURL;
    const originalRevokeObjectURL = URL.revokeObjectURL;
    const downloads: string[] = [];
    let requestBody = "";
    globalThis.fetch = (async (_url, init) => {
        requestBody = String(init?.body ?? "");
        return new Response(new Blob(["archive"]), {
            status: 200,
            headers: {
                "content-disposition": "attachment; filename=rustzen-module-logs.tar",
                "content-type": "application/x-tar",
                "x-rustzen-archive-file-count": "2",
                "x-rustzen-archive-sha256":
                    "0eb3e36bfb24dcd9bb1d1bece1531216b59539a8fde17ee80224af0653c92aa3",
            },
        });
    }) as typeof fetch;
    Object.defineProperty(globalThis, "document", {
        configurable: true,
        value: {
            createElement: () => ({
                click: () => downloads.push("rustzen-module-logs.tar"),
                download: "",
                href: "",
            }),
            body: {
                appendChild: () => undefined,
                removeChild: () => undefined,
            },
        },
    });
    URL.createObjectURL = () => "blob:module-logs";
    URL.revokeObjectURL = () => undefined;
    try {
        await expect(
            moduleLogAPI.backup([
                { module: "admin", date: "2026-08-09" },
                { module: "reports", date: "2026-08-08" },
                { module: "admin", date: "2026-08-09" },
            ]),
        ).resolves.toEqual({
            filename: "rustzen-module-logs.tar",
            fileCount: 2,
            archiveSha256: "0eb3e36bfb24dcd9bb1d1bece1531216b59539a8fde17ee80224af0653c92aa3",
        });
        expect(JSON.parse(requestBody)).toEqual({
            files: [
                { module: "admin", date: "2026-08-09" },
                { module: "reports", date: "2026-08-08" },
            ],
        });
        expect(downloads).toEqual(["rustzen-module-logs.tar"]);
        await expect(
            moduleLogAPI.backup([{ module: "unknown", date: "2026-08-08" }]),
        ).rejects.toThrow("Unsupported module log module");
    } finally {
        globalThis.fetch = originalFetch;
        Object.defineProperty(globalThis, "document", {
            configurable: true,
            value: originalDocument,
        });
        URL.createObjectURL = originalCreateObjectURL;
        URL.revokeObjectURL = originalRevokeObjectURL;
    }
});

test("module log backup rejects a metadata count that differs from unique selected files", async () => {
    const originalFetch = globalThis.fetch;
    const originalDocument = globalThis.document;
    const originalCreateObjectURL = URL.createObjectURL;
    const originalRevokeObjectURL = URL.revokeObjectURL;
    const downloads: string[] = [];
    let objectUrlCalls = 0;
    globalThis.fetch = (async () =>
        new Response(new Blob(["archive"]), {
            status: 200,
            headers: {
                "content-disposition": "attachment; filename=rustzen-module-logs.tar",
                "x-rustzen-archive-file-count": "1",
                "x-rustzen-archive-sha256":
                    "0eb3e36bfb24dcd9bb1d1bece1531216b59539a8fde17ee80224af0653c92aa3",
            },
        })) as unknown as typeof fetch;
    Object.defineProperty(globalThis, "document", {
        configurable: true,
        value: {
            createElement: () => ({ click: () => downloads.push("downloaded"), download: "", href: "" }),
            body: { appendChild: () => undefined, removeChild: () => undefined },
        },
    });
    URL.createObjectURL = () => {
        objectUrlCalls += 1;
        return "blob:module-logs";
    };
    URL.revokeObjectURL = () => undefined;
    try {
        await expect(
            moduleLogAPI.backup([
                { module: "admin", date: "2026-08-09" },
                { module: "reports", date: "2026-08-08" },
            ]),
        ).rejects.toThrow("file count does not match");
        expect(downloads).toEqual([]);
        expect(objectUrlCalls).toBe(0);
    } finally {
        globalThis.fetch = originalFetch;
        Object.defineProperty(globalThis, "document", {
            configurable: true,
            value: originalDocument,
        });
        URL.createObjectURL = originalCreateObjectURL;
        URL.revokeObjectURL = originalRevokeObjectURL;
    }
});

test("module log backup fails closed when archive metadata is absent, malformed, or mismatched", async () => {
    const originalFetch = globalThis.fetch;
    const originalDocument = globalThis.document;
    const originalCreateObjectURL = URL.createObjectURL;
    const originalRevokeObjectURL = URL.revokeObjectURL;
    const downloads: string[] = [];
    Object.defineProperty(globalThis, "document", {
        configurable: true,
        value: {
            createElement: () => ({ click: () => downloads.push("downloaded"), download: "", href: "" }),
            body: { appendChild: () => undefined, removeChild: () => undefined },
        },
    });
    URL.createObjectURL = () => "blob:module-logs";
    URL.revokeObjectURL = () => undefined;
    try {
        const invalidHeaderSets: HeadersInit[] = [
            {},
            {
                "content-disposition": "attachment; filename=rustzen-module-logs.tar",
                "x-rustzen-archive-file-count": "one",
                "x-rustzen-archive-sha256": "not-a-sha256",
            },
            {
                "content-disposition": "attachment; filename=rustzen-module-logs.tar",
                "x-rustzen-archive-file-count": "1",
                "x-rustzen-archive-sha256":
                    "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            },
        ];
        for (const headers of invalidHeaderSets) {
            globalThis.fetch = (async () =>
                new Response(new Blob(["archive"]), { status: 200, headers })) as unknown as typeof fetch;
            await expect(
                moduleLogAPI.backup([{ module: "admin", date: "2026-08-09" }]),
            ).rejects.toThrow("Module log backup");
        }
        expect(downloads).toEqual([]);
    } finally {
        globalThis.fetch = originalFetch;
        Object.defineProperty(globalThis, "document", {
            configurable: true,
            value: originalDocument,
        });
        URL.createObjectURL = originalCreateObjectURL;
        URL.revokeObjectURL = originalRevokeObjectURL;
    }
});

test("generated binary operation returns a Blob without JSON parsing", async () => {
    const originalFetch = globalThis.fetch;
    globalThis.fetch = (async (_url, init) => {
        expect(init?.method).toBe("POST");
        expect(init?.body).toBe(
            JSON.stringify({ files: [{ module: "admin", date: "2026-08-10" }] }),
        );
        return new Response(new Blob(["archive"]), {
            status: 200,
            headers: { "content-type": "application/x-tar" },
        });
    }) as typeof fetch;
    try {
        const archive = await backupModuleLogs({
            files: [{ module: "admin", date: "2026-08-10" }],
        });
        expect(archive).toBeInstanceOf(Blob);
        expect(archive.size).toBe(7);
        expect(await archive.text()).toBe("archive");
    } finally {
        globalThis.fetch = originalFetch;
    }
});
