import { afterAll, expect, test } from "bun:test";

const localStorageMock = {
    getItem: () => null,
    setItem: () => undefined,
    removeItem: () => undefined,
    clear: () => undefined,
};
const originalLocalStorage = Object.getOwnPropertyDescriptor(globalThis, "localStorage");
const originalWindow = Object.getOwnPropertyDescriptor(globalThis, "window");
Object.defineProperty(globalThis, "localStorage", {
    configurable: true,
    value: localStorageMock,
});
Object.defineProperty(globalThis, "window", {
    configurable: true,
    value: { localStorage: localStorageMock },
});

const { menuAPI } = await import("./api");

function restoreProperty(
    target: object,
    key: PropertyKey,
    descriptor: PropertyDescriptor | undefined,
) {
    if (descriptor) {
        Object.defineProperty(target, key, descriptor);
    } else {
        delete (target as Record<PropertyKey, unknown>)[key];
    }
}

afterAll(() => {
    restoreProperty(globalThis, "localStorage", originalLocalStorage);
    restoreProperty(globalThis, "window", originalWindow);
});

test("browser shim cleanup restores existing and absent property descriptors", () => {
    const target = {} as Record<PropertyKey, unknown>;
    const existing = { configurable: true, enumerable: true, value: "original", writable: false };
    Object.defineProperty(target, "existing", existing);
    const originalExisting = Object.getOwnPropertyDescriptor(target, "existing");
    Object.defineProperty(target, "existing", {
        configurable: true,
        enumerable: false,
        value: "replacement",
        writable: true,
    });
    Object.defineProperty(target, "absent", { configurable: true, value: "temporary" });

    restoreProperty(target, "existing", originalExisting);
    restoreProperty(target, "absent", undefined);

    expect(Object.getOwnPropertyDescriptor(target, "existing")).toEqual(originalExisting);
    expect(Object.hasOwn(target, "absent")).toBeFalse();
});

test("menu update targets the inventory endpoint with PUT", async () => {
    const originalFetch = globalThis.fetch;
    let requestUrl = "";
    let requestInit: RequestInit | undefined;
    globalThis.fetch = (async (url, init) => {
        requestUrl = typeof url === "string" ? url : url instanceof URL ? url.toString() : url.url;
        requestInit = init;
        return new Response(JSON.stringify({ code: 0, message: "Success", data: 42 }), {
            status: 200,
            headers: { "content-type": "application/json" },
        });
    }) as typeof fetch;

    try {
        await expect(
            menuAPI.update(42, { name: "Monitor", sortOrder: 1, status: 1 }),
        ).resolves.toBe(42);
        expect(requestUrl).toBe("/api/system/menus/inventory/42");
        expect(requestInit?.method).toBe("PUT");
    } finally {
        globalThis.fetch = originalFetch;
    }
});
