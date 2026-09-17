import { afterEach, expect, test } from "bun:test";

import { renderToStaticMarkup } from "react-dom/server";

import { setLocale } from "@/lib/i18n";

import { NodeOnboarding, onboardingStepCopy } from "./-node-onboarding";

afterEach(() => setLocale("zh-CN"));

test("onboarding exposes ordered signed-install prerequisites without an executable action", () => {
    expect(onboardingStepCopy).toHaveLength(6);
    const text = onboardingStepCopy.flat().join("\n");
    for (const required of [
        "archive",
        "release manifest",
        "signature envelope",
        "trusted public key",
        "key ID",
        "root-only",
        "rz apply",
        "prepare-monitor-agent-access",
        "pin-monitor-controller",
        "activate-monitor-agent",
    ]) {
        expect(text).toContain(required);
    }
    expect(text).not.toContain("rz-monitor-agent\n");

    for (const [locale, selectedIndex] of [
        ["zh-CN", 0],
        ["en-US", 1],
    ] as const) {
        setLocale(locale);
        const html = renderToStaticMarkup(<NodeOnboarding />);
        for (const copy of onboardingStepCopy) {
            expect(html).toContain(copy[selectedIndex]);
            expect(html).not.toContain(copy[1 - selectedIndex]);
        }
        expect(html).toContain("disabled");
        expect(html).not.toContain("Copy");
        expect(html).not.toContain('type="password"');
        expect(html).not.toContain("http://");
    }
});
