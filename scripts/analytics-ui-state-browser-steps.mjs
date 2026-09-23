const login = [
    { action: "goto", url: "/login" },
    { action: "waitFor", selector: "#login_username" },
    { action: "fill", selector: "#login_username", value: "owner" },
    { action: "fill", selector: "#login_password", value: "rustzen@123" },
    { action: "click", selector: "button[type=submit]" },
    { action: "waitFor", selector: ".shell-content" },
];

const desktop = (steps) => [
    { action: "setUiPreferences", theme: "dark", locale: "en-US" },
    { action: "setViewport", width: 1440, height: 900 },
    ...login,
    ...steps,
];

const mobile = (steps) => [
    { action: "setUiPreferences", theme: "light", locale: "zh-CN" },
    { action: "setViewport", width: 390, height: 844 },
    ...login,
    ...steps,
];

const steps = {
    overviewLoading: desktop([
        { action: "goto", url: "/analytics/overview" },
        { action: "waitFor", selector: ".shell-content" },
        { action: "assertText", selector: ".shell-content", text: "Loading analytics overview" },
        { action: "waitFor", selector: "[aria-label*='Daily activity trend']" },
        { action: "assertNoHorizontalOverflow" },
        { action: "screenshotViewport", name: "analytics-overview-desktop-dark-en" },
    ]),
    detailsEmpty: mobile([
        { action: "goto", url: "/analytics/details" },
        { action: "waitFor", selector: ".shell-content" },
        { action: "waitFor", selector: "[aria-label*='分析明细']" },
        { action: "assertText", selector: ".shell-content", text: "暂无访问记录" },
        { action: "assertNoHorizontalOverflow" },
        { action: "screenshotViewport", name: "analytics-details-mobile-light-zh" },
    ]),
    overview403: desktop([
        { action: "goto", url: "/analytics/overview" },
        { action: "waitFor", selector: ".shell-content" },
        { action: "waitFor", selector: "[role=alert]" },
        { action: "assertText", selector: ".shell-content", text: "You do not have permission" },
    ]),
    details403: desktop([
        { action: "goto", url: "/analytics/details" },
        { action: "waitFor", selector: ".shell-content" },
        { action: "waitFor", selector: "[role=alert]" },
        { action: "assertText", selector: ".shell-content", text: "You do not have permission to view activity" },
    ]),
    overview500: desktop([
        { action: "goto", url: "/analytics/overview" },
        { action: "waitFor", selector: ".shell-content" },
        { action: "waitFor", selector: "[role=alert]" },
        { action: "assertText", selector: ".shell-content", text: "Failed to load analytics overview" },
    ]),
    details500: desktop([
        { action: "goto", url: "/analytics/details" },
        { action: "waitFor", selector: ".shell-content" },
        { action: "waitFor", selector: "[role=alert]" },
        { action: "assertText", selector: ".shell-content", text: "Failed to load activity" },
    ]),
    detailsFilterResetsPage: desktop([
        { action: "goto", url: "/analytics/details" },
        { action: "waitFor", selector: ".shell-content" },
        { action: "waitFor", selector: ".ant-pagination-item-2" },
        { action: "click", selector: ".ant-pagination-item-2" },
        { action: "pause", durationMs: 350 },
        { action: "fill", selector: "[aria-label='Search page or API path']", value: "/fixture-filter" },
        { action: "pause", durationMs: 500 },
        { action: "assertNoHorizontalOverflow" },
    ]),
    detailsBackgroundRefresh: desktop([
        { action: "goto", url: "/analytics/details" },
        { action: "waitFor", selector: ".shell-content" },
        { action: "assertText", selector: ".shell-content", text: "fixture-visitor" },
        { action: "pause", durationMs: 30_000 },
        { action: "waitFor", selector: "[role=alert]" },
        { action: "pause", durationMs: 300 },
        { action: "assertText", selector: "[role=alert]", text: "Background refresh failed" },
        { action: "assertText", selector: ".shell-content", text: "fixture-visitor" },
    ]),
    overviewBackground403: desktop([
        { action: "goto", url: "/analytics/overview" },
        { action: "waitFor", selector: "[aria-label*='Daily activity trend']" },
        { action: "pause", durationMs: 30_000 },
        { action: "waitFor", selector: "[role=alert]" },
        { action: "assertText", selector: ".shell-content", text: "You do not have permission to view analytics" },
        { action: "assertAbsent", selector: "[aria-label*='Daily activity trend']" },
    ]),
    detailsBackground403: desktop([
        { action: "goto", url: "/analytics/details" },
        { action: "waitFor", selector: ".ant-table-row" },
        { action: "pause", durationMs: 30_000 },
        { action: "waitFor", selector: "[role=alert]" },
        { action: "assertText", selector: ".shell-content", text: "You do not have permission to view activity" },
        { action: "assertAbsent", selector: ".ant-table-row" },
    ]),
};

process.stdout.write(`${JSON.stringify(steps)}\n`);
