import { afterEach, expect, test } from "bun:test";
import { authAPI } from "@/api/auth/api";
import { userAPI } from "@/api/system/user/api";
import { useAuthStore } from "@/store/useAuthStore";

afterEach(() => {
    useAuthStore.getState().clearAuth();
});

test("feature adapters call generated operations with Bearer and unwrap data", async () => {
    useAuthStore.getState().updateToken("contract-token");
    const originalFetch = globalThis.fetch;
    const calls: Array<[string, RequestInit | undefined]> = [];
    globalThis.fetch = (async (url, init) => {
        calls.push([String(url), init]);
        expect(new Headers(init?.headers).get("authorization")).toBe("Bearer contract-token");
        const data = String(url).includes("/auth/me") ? { id: 1, username: "owner", isSystem: true, permissions: [] } : 7;
        return new Response(JSON.stringify({ code: 0, message: "Success", data }), { status: 200 });
    }) as typeof fetch;
    try {
        await expect(authAPI.me()).resolves.toMatchObject({ id: 1, username: "owner" });
        await expect(userAPI.create({ username: "new", email: "new@example.com", password: "secret", roleIds: [3] })).resolves.toBe(7);
        expect(calls[0]?.[0]).toBe("/api/auth/me");
        expect(calls[1]?.[0]).toBe("/api/system/users");
        expect(calls[1]?.[1]?.method).toBe("POST");
    } finally {
        globalThis.fetch = originalFetch;
    }
});

test("generated feature adapter preserves non-success API envelope errors", async () => {
    const originalFetch = globalThis.fetch;
    globalThis.fetch = (async () => new Response(JSON.stringify({ code: 10201, message: "Username already exists." }), { status: 200 })) as unknown as typeof fetch;
    try {
        await expect(userAPI.create({ username: "new", email: "new@example.com", password: "secret", roleIds: [3] })).rejects.toThrow("用户名已存在。");
    } finally {
        globalThis.fetch = originalFetch;
    }
});
