const login = [
    { action: "goto", url: "/login" },
    { action: "waitFor", selector: "#login_username" },
    { action: "fill", selector: "#login_username", value: "owner" },
    { action: "fill", selector: "#login_password", value: "rustzen@123" },
    { action: "click", selector: "button[type=submit]" },
    { action: "waitFor", selector: ".shell-content" },
];

const manager = (steps) => [
    { action: "setUiPreferences", theme: "dark", locale: "en-US" },
    { action: "setViewport", width: 1440, height: 900 },
    ...login,
    ...steps,
];

const viewer = (steps) => [
    { action: "setUiPreferences", theme: "light", locale: "zh-CN" },
    { action: "setViewport", width: 390, height: 844 },
    { action: "goto", url: "/login" },
    { action: "waitFor", selector: "#login_username" },
    { action: "fill", selector: "#login_username", value: "schedule_viewer" },
    { action: "fill", selector: "#login_password", value: "schedule-viewer-password" },
    { action: "click", selector: "button[type=submit]" },
    { action: "waitFor", selector: ".shell-content" },
    ...steps,
];

const englishScheduleMenu = [
    { action: "assertText", selector: "[data-testid='navigation-reports-schedules'][data-label='Scheduled reports']", text: "Scheduled reports" },
    { action: "assertAbsent", selector: "[data-testid='navigation-reports-schedules'][data-label='定时报表']" },
];

const openCreate = [
    { action: "goto", url: "/reports/templates" },
    { action: "waitFor", selector: "[data-testid=schedule-create]" },
    { action: "assertText", selector: "[data-testid=schedule-panel]", text: "Installation timezone:" },
    { action: "click", selector: "[data-testid=schedule-create]" },
    { action: "waitFor", selector: "[data-testid=schedule-dialog]" },
];

const steps = {
    cancelCreate: manager([
        ...openCreate,
        { action: "click", selector: "xpath=//button[normalize-space()='Cancel']" },
        { action: "pause", durationMs: 300 },
        { action: "assertFocus", selector: "[data-testid=schedule-create]" },
        { action: "assertNoHorizontalOverflow" },
        { action: "screenshotViewport", name: "schedule-form-desktop-dark-en" },
    ]),
    malformedInput: manager([
        ...openCreate,
        { action: "fill", selector: "textarea", value: "[]" },
        { action: "click", selector: "[data-testid=schedule-save]" },
        { action: "waitFor", selector: ".ant-message-error" },
        { action: "assertText", selector: ".ant-message-error", text: "valid JSON object" },
    ]),
    missingTime: manager([
        ...openCreate,
        { action: "fill", selector: "[data-testid=schedule-due-time]", value: "" },
        { action: "click", selector: "[data-testid=schedule-save]" },
        { action: "waitFor", selector: ".ant-message-error" },
        { action: "assertText", selector: ".ant-message-error", text: "Complete the schedule fields" },
    ]),
    secretRejected: manager([
        ...openCreate,
        { action: "fill", selector: "textarea", value: '{"token":"sr-ui-002-secret-marker"}' },
        { action: "click", selector: "[data-testid=schedule-save]" },
        { action: "waitFor", selector: "[data-testid=schedule-save-error]" },
        { action: "assertText", selector: "[data-testid=schedule-save-error]", text: "may contain a secret" },
        { action: "assertValue", selector: "textarea", value: '{"token":"sr-ui-002-secret-marker"}' },
    ]),
    createDaily: manager([
        ...openCreate,
        ...englishScheduleMenu,
        { action: "assertText", selector: "[data-testid=schedule-cadence]", text: "Daily" },
        { action: "fill", selector: "[data-testid=schedule-due-time]", value: "10:15" },
        { action: "fill", selector: "[data-testid=schedule-description]", value: "sr-ui-002 daily" },
        { action: "click", selector: "[data-testid=schedule-save]" },
        { action: "waitFor", selector: "[data-testid=schedule-edit]" },
        { action: "assertText", selector: "[data-testid=schedule-panel]", text: "Daily" },
        { action: "pause", durationMs: 300 },
        { action: "assertFocus", selector: "[data-testid=schedule-create]" },
    ]),
    editWeekly: manager([
        { action: "goto", url: "/reports/templates" },
        { action: "waitFor", selector: "[data-testid=schedule-edit]" },
        ...englishScheduleMenu,
        { action: "click", selector: "[data-testid=schedule-edit]" },
        { action: "waitFor", selector: "[data-testid=schedule-dialog]" },
        { action: "click", selector: "[data-testid=schedule-cadence]" },
        { action: "waitFor", selector: "[data-testid=schedule-cadence-weekly]" },
        { action: "click", selector: "[data-testid=schedule-cadence-weekly]" },
        { action: "waitFor", selector: "[data-testid=schedule-weekday]" },
        { action: "click", selector: "[data-testid=schedule-weekday]" },
        { action: "waitFor", selector: "[data-testid=schedule-weekday-0]" },
        { action: "click", selector: "[data-testid=schedule-weekday-0]" },
        { action: "fill", selector: "[data-testid=schedule-due-time]", value: "10:16" },
        { action: "click", selector: "[data-testid=schedule-save]" },
        { action: "waitFor", selector: "[data-testid=schedule-edit]" },
        { action: "assertText", selector: "[data-testid=schedule-panel]", text: "Weekly Monday" },
        { action: "pause", durationMs: 300 },
        { action: "assertFocus", selector: "[data-testid=schedule-edit]" },
        { action: "assertNoHorizontalOverflow" },
    ]),
    viewer: viewer([
        { action: "goto", url: "/reports/templates" },
        { action: "waitFor", selector: "[data-testid=schedule-panel]" },
        { action: "assertText", selector: "[data-testid=schedule-panel]", text: "定时报表计划" },
        { action: "assertText", selector: "[data-testid=schedule-panel]", text: "安装时区：" },
        { action: "waitFor", selector: "[data-testid^=schedule-timezone-]" },
        { action: "assertText", selector: "[data-testid^=schedule-timezone-]", text: "10:16 · UTC" },
        { action: "assertElementLayout", selector: "[data-testid^=schedule-timezone-]", visibleCount: 1, withinViewportRight: true },
        { action: "assertAbsent", selector: "[data-testid=schedule-actions-column]" },
        { action: "assertAbsent", selector: "[data-testid=schedule-create]" },
        { action: "assertAbsent", selector: "[data-testid=schedule-dialog]" },
        { action: "assertAbsent", selector: "[data-testid=schedule-edit]" },
        { action: "assertAbsent", selector: "[data-testid=schedule-toggle]" },
        { action: "assertAbsent", selector: "[data-testid=schedule-delete]" },
        { action: "assertNoHorizontalOverflow" },
        { action: "screenshotViewport", name: "schedule-form-mobile-light-zh" },
    ]),
};

process.stdout.write(`${JSON.stringify(steps)}\n`);
