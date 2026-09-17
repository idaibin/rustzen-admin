import { expect, test } from "bun:test";

const profileSource = await Bun.file("src/routes/profile.tsx").text();
const statusSource = await Bun.file("src/routes/system/status.tsx").text();
const moduleLogRouteSource = await Bun.file("src/routes/system/module-log.tsx").text();
const moduleLogsSource = await Bun.file("src/routes/system/-module-log-diagnostics.tsx").text();

function count(source, value) {
    return source.split(value).length - 1;
}

test("Profile composes one page heading and one nested account-card heading", () => {
    expect(count(profileSource, "<PageHeader")).toBe(1);
    expect(count(profileSource, "<PageCard")).toBe(1);
    expect(profileSource).toContain("headingLevel={2}");
});

test("Module log diagnostics owns its standalone page while status keeps telemetry only", () => {
    expect(moduleLogRouteSource).toContain('createFileRoute("/system/module-log")');
    expect(moduleLogRouteSource).toContain("ModuleLogDiagnostics");
    expect(statusSource).not.toContain("ModuleLogDiagnostics");
    expect(moduleLogsSource).not.toContain("headingLevel={2}");
    expect(moduleLogsSource).toContain('title={t("模块日志诊断", "Module log diagnostics")}');
});

test("module log explanation is content below the diagnostics heading", () => {
    const cardStart = moduleLogsSource.indexOf("<PageCard");
    const cardContent = moduleLogsSource.indexOf(">\n                <Typography.Text", cardStart);
    const explanation = moduleLogsSource.indexOf("查看四个本地服务的受限日志尾部", cardStart);

    expect(moduleLogsSource.slice(cardStart, cardContent)).not.toContain("description=");
    expect(explanation).toBeGreaterThan(cardContent);
});
