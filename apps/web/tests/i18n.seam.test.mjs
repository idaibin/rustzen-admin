import { afterAll, beforeEach, describe, expect, test, vi } from "bun:test";

import { getLocale, setLocale, subscribeLocale } from "../src/lib/i18n.ts";

const originalDescriptors = Object.fromEntries(
    ["document", "localStorage", "window"].map((name) => [
        name,
        Object.getOwnPropertyDescriptor(globalThis, name),
    ]),
);

afterAll(() => {
    for (const [name, descriptor] of Object.entries(originalDescriptors)) {
        if (descriptor) Object.defineProperty(globalThis, name, descriptor);
        else delete globalThis[name];
    }
});

describe("i18n locale seam", () => {
    let store;

    beforeEach(() => {
        store = {};
        Object.defineProperty(globalThis, "localStorage", {
            configurable: true,
            value: {
                getItem: (key) => store[key],
                setItem: (key, value) => {
                    store[key] = value;
                },
            },
        });
        Object.defineProperty(globalThis, "window", {
            configurable: true,
            value: {
                addEventListener: () => undefined,
                removeEventListener: () => undefined,
                location: { reload: vi.fn() },
            },
        });
        Object.defineProperty(globalThis, "document", {
            configurable: true,
            value: {
                addEventListener: () => undefined,
                removeEventListener: () => undefined,
                documentElement: { lang: "" },
            },
        });
        setLocale("zh-CN");
    });

    test("persists locale and updates document language without a full reload", () => {
        setLocale("en-US");

        expect(getLocale()).toBe("en-US");
        expect(localStorage.getItem("rustzen-admin-locale")).toBe("en-US");
        expect(document.documentElement.lang).toBe("en-US");
        expect(window.location.reload.mock.calls).toHaveLength(0);
    });

    test("notifies active subscribers once and honors cleanup", () => {
        const listener = vi.fn();
        const unsubscribe = subscribeLocale(listener);

        setLocale("en-US");
        expect(listener).toHaveBeenCalledTimes(1);

        unsubscribe();
        setLocale("zh-CN");
        expect(listener).toHaveBeenCalledTimes(1);
    });
});
