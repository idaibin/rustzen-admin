import { describe, expect, test } from "bun:test";

const moduleSource = await Bun.file(
    new URL("../src/routes/system/module.tsx", import.meta.url),
).text();
const provider = await Bun.file(
    new URL("../src/components/theme-provider.tsx", import.meta.url),
).text();
const css = await Bun.file(new URL("../src/styles/theme.css", import.meta.url)).text();
const layoutCss = await Bun.file(new URL("../src/style.css", import.meta.url)).text();

function palette(selector) {
    const block = css.slice(css.indexOf(`${selector} {`)).split("}")[0];
    return Object.fromEntries(
        [...block.matchAll(/--([\w-]+):\s*(#[\da-f]{6});/gi)].map(([, name, value]) => [
            name,
            value,
        ]),
    );
}
function rgb(hex) {
    return [1, 3, 5].map((offset) => Number.parseInt(hex.slice(offset, offset + 2), 16) / 255);
}
function luminance(channels) {
    const linear = channels.map((v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
    return linear[0] * 0.2126 + linear[1] * 0.7152 + linear[2] * 0.0722;
}
function contrast(left, right) {
    const a = luminance(left),
        b = luminance(right);
    return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
}

describe("module status theme ownership", () => {
    const health = moduleSource.slice(moduleSource.indexOf("function ModuleHealthTag("));
    test("health branches preserve precedence and semantic variants", () => {
        expect(health).toMatch(/if \(!module.enabled\)[\s\S]*?color="default"/);
        expect(health).toMatch(/if \(module.available\)[\s\S]*?color="success"/);
        expect(health).toMatch(/if \(module.compatible\)[\s\S]*?color="warning"/);
        expect(health).toMatch(/if \(module.releaseVersion\)[\s\S]*?color="error"/);
        expect(health.match(/color="error"/g)).toHaveLength(2);
        expect(health.indexOf("!module.enabled")).toBeLessThan(health.indexOf("module.available"));
        expect(health.indexOf("module.available")).toBeLessThan(
            health.indexOf("module.compatible"),
        );
        expect(health.indexOf("module.compatible")).toBeLessThan(
            health.indexOf("module.releaseVersion"),
        );
        expect(moduleSource).not.toMatch(/color="(?:green|orange|red|blue)"/);
    });
    test("enabled state uses success and diagnostic tags remain focusable", () => {
        expect(moduleSource).toContain('module.enabled ? "success" : "default"');
        expect(health.match(/tabIndex=\{0\}/g)).toHaveLength(3);
        expect(health.match(/trigger=\{\["hover", "focus"\]\}/g)).toHaveLength(3);
        for (const label of ["Disabled", "Available", "Unavailable", "Incompatible", "Not ready"])
            expect(health).toContain(`"${label}"`);
    });
});

describe("shared theme contrast guard", () => {
    test("keyboard scroll-region focus uses an opaque inset semantic ring", () => {
        expect(layoutCss).toMatch(
            /\.data-table-shell \[role="region"\]\[tabindex="0"\]:focus-visible\s*\{\s*outline: 2px solid var\(--ring\);\s*outline-offset: -2px;/,
        );
    });
    for (const [name, tokens] of [
        ["light", palette(":root")],
        ["dark", palette(".dark")],
    ]) {
        test(`${name} semantic text is readable on the owned pale surfaces`, () => {
            for (const [role, antRole] of [
                ["success", "Success"],
                ["warning", "Warning"],
                ["danger", "Error"],
                ["info", "Info"],
            ]) {
                const foreground = rgb(tokens[`status-${role}`]);
                const surface = foreground.map(
                    (channel, index) => channel * 0.05 + rgb(tokens.card)[index] * 0.95,
                );
                expect(css).toContain(
                    `--status-${role}-surface: color-mix(in srgb, var(--status-${role}) 5%, var(--card));`,
                );
                expect(provider).toContain(`color${antRole}Text: color("status-${role}")`);
                expect(provider).toContain(`color${antRole}Bg: "var(--status-${role}-surface)"`);
                expect(contrast(foreground, surface), `${name} ${role}`).toBeGreaterThanOrEqual(
                    4.5,
                );
            }
        });
        test(`${name} link, destructive, and focus states retain contrast`, () => {
            for (const state of ["link", "link-hover", "link-active"])
                expect(
                    contrast(rgb(tokens[state]), rgb(tokens.card)),
                    `${name} ${state}`,
                ).toBeGreaterThanOrEqual(4.5);
            for (const state of ["status-danger", "status-danger-hover", "status-danger-active"])
                expect(
                    contrast(rgb(tokens[state]), rgb(tokens["primary-foreground"])),
                    `${name} ${state}`,
                ).toBeGreaterThanOrEqual(4.5);
            expect(
                contrast(rgb(tokens.primary), rgb(tokens.card)),
                `${name} focus boundary`,
            ).toBeGreaterThanOrEqual(3);
            expect(provider).toContain('colorPrimaryBorder: color("primary")');
            expect(provider).toContain('defaultActiveColor: "var(--link-active)"');
            expect(provider).toContain('itemSelectedColor: "var(--sidebar-accent-foreground)"');
            expect(
                contrast(rgb(tokens[name === "light" ? "link-hover" : "link"]), rgb(tokens.accent)),
                `${name} selected navigation text`,
            ).toBeGreaterThanOrEqual(4.5);
            expect(
                contrast(rgb(tokens["link-active"]), rgb(tokens.popover)),
                `${name} outlined active text on overlay`,
            ).toBeGreaterThanOrEqual(4.5);
            expect(provider).toContain('colorLinkHover: color("link-hover")');
            expect(provider).toContain('colorErrorHover: color("status-danger-hover")');
        });
        test(`${name} primary default, hover and active text is readable`, () => {
            for (const state of ["primary", "primary-hover", "primary-active"])
                expect(
                    contrast(rgb(tokens[state]), rgb(tokens["primary-foreground"])),
                    `${name} ${state}`,
                ).toBeGreaterThanOrEqual(4.5);
        });
    }
});
