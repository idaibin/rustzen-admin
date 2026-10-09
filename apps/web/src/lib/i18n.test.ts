import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";

const key = "rustzen-admin-locale";
const modulePath = new URL("./i18n.ts", import.meta.url).pathname;
const html = readFileSync(new URL("../../index.html", import.meta.url), "utf8");
const bootstrap = html.match(/<script>([\s\S]*?)<\/script>/)?.[1];

type Result = { locale: string; lang?: string; stored: string | null; copy: string };

// A fresh process exercises module initialization without sharing the locale singleton.
function initialize(stored: string | null, browser = true, select?: string): Result {
    const result = Bun.spawnSync([
        process.execPath,
        "--eval",
        `let stored = ${JSON.stringify(stored)};
        if (${browser}) {
            globalThis.window = {};
            globalThis.document = { documentElement: { lang: "" } };
            globalThis.navigator = { language: "zh-CN" };
            globalThis.localStorage = {
                getItem: () => stored,
                setItem: (_, value) => { stored = value; },
            };
        }
        const { getLocale, setLocale, t } = await import(${JSON.stringify(modulePath)});
        ${select ? `setLocale(${JSON.stringify(select)});` : ""}
        console.log(JSON.stringify({ locale: getLocale(), lang: globalThis.document?.documentElement.lang,
            stored, copy: t("保存", "Save") }));`,
    ]);
    expect(result.exitCode).toBe(0);
    return JSON.parse(result.stdout.toString());
}

for (const [stored, expected] of [
    [null, "en-US"],
    ["invalid", "en-US"],
    ["", "en-US"],
    ["en-US", "en-US"],
    ["zh-CN", "zh-CN"],
] as const) {
    test(`initial locale ${JSON.stringify(stored)} uses ${expected} without overwriting storage`, () => {
        const result = initialize(stored);
        expect(result.locale).toBe(expected);
        expect(result.lang).toBe(expected);
        expect(result.stored).toBe(stored);
        expect(result.copy).toBe(expected === "en-US" ? "Save" : "保存");

        const document = { documentElement: { lang: "", classList: { toggle() {} } } };
        const values = new Map<string, string>();
        if (stored !== null) values.set(key, stored);
        expect(bootstrap).toBeDefined();
        runInNewContext(bootstrap!, {
            document,
            localStorage: {
                getItem: (name: string) => values.get(name) ?? null,
                setItem: (name: string, value: string) => values.set(name, value),
            },
        });
        expect(document.documentElement.lang).toBe(expected);
        expect(values.get(key) ?? null).toBe(stored);
    });
}

test("no-window initialization uses English", () => {
    expect(initialize(null, false)).toEqual({ locale: "en-US", stored: null, copy: "Save" });
});

test("explicit language changes survive fresh initialization in both directions", () => {
    const chinese = initialize(null, true, "zh-CN");
    expect(chinese).toEqual({ locale: "zh-CN", lang: "zh-CN", stored: "zh-CN", copy: "保存" });
    expect(initialize(chinese.stored)).toEqual(chinese);
    const english = initialize(chinese.stored, true, "en-US");
    expect(english).toEqual({ locale: "en-US", lang: "en-US", stored: "en-US", copy: "Save" });
    expect(initialize(english.stored)).toEqual(english);
});
