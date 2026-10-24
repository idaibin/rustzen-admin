import { afterEach, beforeEach, expect, test } from "bun:test";

import { renderToStaticMarkup } from "react-dom/server";

import { getLocale, setLocale } from "@/lib/i18n";

import { ModuleHealthPanel } from "./-module-health-panel";

let previousLocale = getLocale();

beforeEach(() => {
    previousLocale = getLocale();
});

afterEach(() => setLocale(previousLocale));

const commonProps = {
    dataUpdatedAt: 0,
    error: null,
    isPending: false,
    isFetching: false,
    onRetry: () => {},
};

test("renders only connected modules and keeps a connected offline module visible", () => {
    setLocale("en-US");
    const html = renderToStaticMarkup(
        <ModuleHealthPanel
            {...commonProps}
            data={[{ module: "insights", available: false, releaseVersion: null }]}
        />,
    );

    expect(html).toContain("Insights");
    expect(html).toContain("Unavailable");
    expect(html).toContain("md:grid-cols-3 xl:grid-cols-1");
    expect(html).not.toContain("Monitor");
    expect(html).not.toContain("Automation");
});

test("hides the panel only after a successful empty response", () => {
    const successfulEmpty = renderToStaticMarkup(<ModuleHealthPanel {...commonProps} data={[]} />);
    const loading = renderToStaticMarkup(
        <ModuleHealthPanel {...commonProps} data={undefined} isPending />,
    );
    const failed = renderToStaticMarkup(
        <ModuleHealthPanel {...commonProps} data={undefined} error={new Error("request failed")} />,
    );

    expect(successfulEmpty).toBe("");
    expect(loading).toContain("Loading module status");
    expect(failed).toContain("Failed to load module status");
    expect(failed).toContain("Reload");
});
