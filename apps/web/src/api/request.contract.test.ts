import { afterEach, expect, test } from "bun:test";

import type { DeploymentUpload } from "@/api/generated/admin-contract";

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

const [{ authAPI }, { userAPI }, generatedContract, authStore] = await Promise.all([
    import("@/api/auth/api"),
    import("@/api/system/user/api"),
    import("@/api/generated/admin-contract"),
    import("@/store/useAuthStore"),
]);
const [{ reportsAPI }, { ApiRequestError, apiRequest }] = await Promise.all([
    import("@/api/reports/api"),
    import("@/api/request"),
]);
const { insightsAPI } = await import("@/api/insights/api");
const { DeployComponent, exportManageLogs, getRoleOptions, updateAccountAvatar, uploadDeployment } =
    generatedContract;
const { useAuthStore } = authStore;

afterEach(() => {
    useAuthStore.getState().clearAuth();
});

test("feature adapters call generated operations with Bearer and unwrap data", async () => {
    useAuthStore.getState().updateToken("contract-token");
    const originalFetch = globalThis.fetch;
    const calls: Array<[string, RequestInit | undefined]> = [];
    globalThis.fetch = (async (url, init) => {
        const requestUrl =
            typeof url === "string" ? url : url instanceof URL ? url.toString() : url.url;
        calls.push([requestUrl, init]);
        expect(new Headers(init?.headers).get("authorization")).toBe("Bearer contract-token");
        const data = requestUrl.includes("/auth/me")
            ? { id: 1, username: "owner", isSystem: true, permissions: [] }
            : 7;
        return new Response(JSON.stringify({ code: 0, message: "Success", data }), { status: 200 });
    }) as typeof fetch;
    try {
        await expect(authAPI.me()).resolves.toMatchObject({ id: 1, username: "owner" });
        await expect(
            userAPI.create({
                username: "new",
                email: "new@example.com",
                password: "secret",
                roleIds: [3],
            }),
        ).resolves.toBe(7);
        expect(calls[0]?.[0]).toBe("/api/auth/me");
        expect(calls[1]?.[0]).toBe("/api/system/users");
        expect(calls[1]?.[1]?.method).toBe("POST");
    } finally {
        globalThis.fetch = originalFetch;
    }
});

test("generated feature adapter preserves non-success API envelope errors", async () => {
    const originalFetch = globalThis.fetch;
    globalThis.fetch = (async () =>
        new Response(JSON.stringify({ code: 10201, message: "Username already exists." }), {
            status: 200,
        })) as unknown as typeof fetch;
    try {
        await expect(
            userAPI.create({
                username: "new",
                email: "new@example.com",
                password: "secret",
                roleIds: [3],
            }),
        ).rejects.toThrow("用户名已存在。");
    } finally {
        globalThis.fetch = originalFetch;
    }
});

test("schedule saves surface a typed 400 JSON error without a global toast", async () => {
    const originalFetch = globalThis.fetch;
    const originalError = console.error;
    const errors: unknown[][] = [];
    console.error = (...args) => errors.push(args);
    globalThis.fetch = (async () =>
        new Response(JSON.stringify({ code: 40001, message: "Schedule target is disabled." }), {
            status: 400,
        })) as unknown as typeof fetch;
    try {
        await expect(
            reportsAPI.createSchedule({
                flowId: "flow-1",
                cadence: "daily",
                dueTime: "09:00",
                input: {},
            }),
        ).rejects.toMatchObject({
            name: "ApiRequestError",
            code: 40001,
            status: 400,
            message: "Schedule target is disabled.",
        } satisfies { name: string; code: number; status: number; message: string });
        expect(errors).toEqual([]);
    } finally {
        globalThis.fetch = originalFetch;
        console.error = originalError;
    }
});

test("schedule saves surface a typed non-zero envelope error without a global toast", async () => {
    const originalFetch = globalThis.fetch;
    const originalError = console.error;
    const errors: unknown[][] = [];
    console.error = (...args) => errors.push(args);
    globalThis.fetch = (async () =>
        new Response(JSON.stringify({ code: 40002, message: "Schedule already exists." }), {
            status: 200,
        })) as unknown as typeof fetch;
    try {
        await expect(
            reportsAPI.updateSchedule("schedule-1", {
                flowId: "flow-1",
                cadence: "daily",
                dueTime: "09:00",
                input: {},
            }),
        ).rejects.toMatchObject({
            name: "ApiRequestError",
            code: 40002,
            status: 200,
            message: "Schedule already exists.",
        } satisfies { name: string; code: number; status: number; message: string });
        expect(errors).toEqual([]);
    } finally {
        globalThis.fetch = originalFetch;
        console.error = originalError;
    }
});

test("schedule saves normalize network failures without a global toast", async () => {
    const originalFetch = globalThis.fetch;
    const originalError = console.error;
    const errors: unknown[][] = [];
    console.error = (...args) => errors.push(args);
    globalThis.fetch = (async () => {
        throw new TypeError("network unavailable");
    }) as unknown as typeof fetch;
    try {
        await expect(
            reportsAPI.createSchedule({
                flowId: "flow-1",
                cadence: "daily",
                dueTime: "09:00",
                input: {},
            }),
        ).rejects.toBeInstanceOf(ApiRequestError);
        expect(errors).toEqual([]);
    } finally {
        globalThis.fetch = originalFetch;
        console.error = originalError;
    }
});

test("analytics reads preserve typed HTTP errors without global toasts", async () => {
    const originalFetch = globalThis.fetch;
    const originalError = console.error;
    const errors: unknown[][] = [];
    console.error = (...args) => errors.push(args);
    globalThis.fetch = (async () =>
        new Response(JSON.stringify({ code: 40002, message: "fixture forbidden", data: null }), {
            status: 403,
            headers: { "content-type": "application/json" },
        })) as unknown as typeof fetch;
    try {
        await expect(insightsAPI.overview({})).rejects.toMatchObject({
            name: "ApiRequestError",
            code: 40002,
            status: 403,
        });
        await expect(insightsAPI.events({ current: 1, pageSize: 20 })).rejects.toMatchObject({
            name: "ApiRequestError",
            code: 40002,
            status: 403,
        });
        expect(errors).toEqual([]);
    } finally {
        globalThis.fetch = originalFetch;
        console.error = originalError;
    }
});

test("default requests preserve network errors for route-local classifiers", async () => {
    const originalFetch = globalThis.fetch;
    globalThis.fetch = (async () => {
        throw new TypeError("network unavailable");
    }) as unknown as typeof fetch;
    try {
        await expect(
            apiRequest({ url: "/api/monitor/nodes", method: "PUT" }),
        ).rejects.toBeInstanceOf(TypeError);
    } finally {
        globalThis.fetch = originalFetch;
    }
});

test("generated CSV operation reads text instead of JSON", async () => {
    const originalFetch = globalThis.fetch;
    globalThis.fetch = (async () =>
        new Response("id,action\\n1,login\\n", {
            status: 200,
            headers: { "content-type": "text/csv; charset=utf-8" },
        })) as unknown as typeof fetch;
    try {
        const csv: string = await exportManageLogs();
        expect(csv).toBe("id,action\\n1,login\\n");
    } finally {
        globalThis.fetch = originalFetch;
    }
});

test("generated multipart operations preserve filenames and omit manual content types", async () => {
    const originalFetch = globalThis.fetch;
    const calls: RequestInit[] = [];
    useAuthStore.getState().updateToken("multipart-token");
    globalThis.fetch = (async (_url: RequestInfo | URL, init?: RequestInit) => {
        calls.push(init ?? {});
        return new Response(
            JSON.stringify({ code: 0, message: "Success", data: "/avatars/new.png" }),
            {
                status: 200,
                headers: { "content-type": "application/json" },
            },
        );
    }) as unknown as typeof fetch;
    try {
        const typedUpload: DeploymentUpload = {
            component: DeployComponent.release,
            version: "0.5.0",
            file: new File(["release"], "release.tar", { type: "application/octet-stream" }),
        };
        // @ts-expect-error DeploymentUpload only accepts the DeployComponent enum.
        const invalidUpload: DeploymentUpload = { ...typedUpload, component: "arbitrary" };
        void invalidUpload;
        await expect(
            updateAccountAvatar(
                { file: new File(["png"], "avatar.png", { type: "image/png" }) },
                { headers: { "Content-Type": "application/json", "X-Custom": "kept" } },
            ),
        ).resolves.toMatchObject({ data: "/avatars/new.png" });
        await uploadDeployment(typedUpload, { headers: { "X-Upload": "kept" } });
        expect(calls).toHaveLength(2);
        for (const call of calls) {
            expect(new Headers(call.headers).get("content-type")).toBeNull();
            expect(new Headers(call.headers).get("authorization")).toBe("Bearer multipart-token");
            expect(call.body).toBeInstanceOf(FormData);
        }
        expect(new Headers(calls[0]?.headers).get("x-custom")).toBe("kept");
        expect(new Headers(calls[1]?.headers).get("x-upload")).toBe("kept");
        const avatarRequest = calls[0];
        const deploymentRequest = calls[1];
        expect(avatarRequest).toBeDefined();
        expect(deploymentRequest).toBeDefined();
        expect((avatarRequest.body as FormData).get("file")).toMatchObject({
            name: "avatar.png",
            type: "image/png",
        });
        expect((deploymentRequest.body as FormData).get("file")).toMatchObject({
            name: "release.tar",
            type: "application/octet-stream",
        });
    } finally {
        globalThis.fetch = originalFetch;
    }
});

test("generated role options serializes q and limit query parameters", async () => {
    const originalFetch = globalThis.fetch;
    let receivedUrl = "";
    globalThis.fetch = (async (url: RequestInfo | URL) => {
        receivedUrl = typeof url === "string" ? url : url instanceof URL ? url.toString() : url.url;
        return new Response(JSON.stringify({ code: 0, message: "Success", data: [] }), {
            status: 200,
            headers: { "content-type": "application/json" },
        });
    }) as unknown as typeof fetch;
    try {
        await getRoleOptions({ q: "admin user", limit: 5 });
        expect(receivedUrl).toBe("/api/system/roles/options?q=admin+user&limit=5");
    } finally {
        globalThis.fetch = originalFetch;
    }
});
