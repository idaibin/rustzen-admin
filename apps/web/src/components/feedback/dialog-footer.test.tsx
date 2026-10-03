import { expect, test } from "bun:test";

import { renderToStaticMarkup } from "react-dom/server";

import { setLocale } from "@/lib/i18n";

import { DialogFooter } from "./dialog-footer";

test("pending dialog footer exposes busy submit and disables cancellation", () => {
    setLocale("en-US");
    const html = renderToStaticMarkup(
        <DialogFooter onCancel={() => {}} submitLabel="Save" submitting />,
    );
    expect(html).toContain('aria-busy="true"');
    expect(html).toMatch(/<button[^>]*disabled=""[^>]*>[\s\S]*?Cancel/);
    setLocale("zh-CN");
});

test("failed dialog footer keeps a named retry action and a readable alert", () => {
    const html = renderToStaticMarkup(
        <DialogFooter onCancel={() => {}} submitLabel="Retry save" error="Fixture failed" />,
    );
    expect(html).toContain('role="alert"');
    expect(html).toContain("Fixture failed");
    expect(html).toContain("Retry save");
    expect(html).not.toContain('disabled=""');
});
