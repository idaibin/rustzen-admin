import { expect, test } from "bun:test";

const driver = new URL("./monitoring-ui-state-browser-steps.mjs", import.meta.url).pathname;

test("incidents expose a stable pagination seam for the mobile delivery journey", async () => {
    const incidents = await Bun.file(
        new URL("../apps/web/src/routes/monitoring/incidents.tsx", import.meta.url),
    ).text();
    expect(incidents).toContain('data-testid="incidents-pagination"');
});

test("mobile summaries emit the readable three-column geometry contract", async () => {
    const result = Bun.spawnSync([process.execPath, driver], {
        stdout: "pipe",
        stderr: "pipe",
    });
    expect(result.exitCode).toBe(0);
    const steps = JSON.parse(new TextDecoder().decode(result.stdout)).summariesPaging;
    const layouts = steps.filter((step) => step.action === "assertElementLayout");
    expect(layouts).toContainEqual({
        action: "assertElementLayout",
        selector: ".ant-table-thead th:not(.ant-table-cell-scrollbar)",
        visibleCount: 3,
    });
    expect(layouts).toContainEqual({
        action: "assertElementLayout",
        selector: ".ant-table-thead .monitoring-summary-detail-column",
        visibleCount: 0,
    });
    expect(layouts).toContainEqual({
        action: "assertElementLayout",
        selector: ".ant-table-thead th:first-child",
        visibleCount: 1,
        maxHeight: 64,
        withinViewportRight: true,
    });
    expect(layouts).toContainEqual({
        action: "assertElementLayout",
        selector: ".ant-table-tbody > tr.ant-table-row",
        visibleCount: 1,
        maxHeight: 72,
        withinViewportRight: true,
    });
    for (const selector of [
        ".ant-table-body > table",
        "[data-testid=summaries-pagination] .ant-pagination",
    ]) {
        expect(layouts).toContainEqual({
            action: "assertElementLayout",
            selector,
            visibleCount: 1,
            withinViewportRight: true,
        });
    }
    const summaries = await Bun.file(
        new URL("../apps/web/src/routes/monitoring/summaries.tsx", import.meta.url),
    ).text();
    expect(summaries).toContain('data-testid="summaries-pagination"');
    expect(summaries.match(/monitoring-summary-detail-column/g)).toHaveLength(6);
});
