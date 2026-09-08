const login = [
    { action: "goto", url: "/login" },
    { action: "waitFor", selector: "#login_username" },
    { action: "fill", selector: "#login_username", value: "owner" },
    { action: "fill", selector: "#login_password", value: "runtime-owner-password" },
    { action: "click", selector: "button[type=submit]" },
    { action: "waitFor", selector: ".shell-content" },
];
const desktop = (steps) => [
    { action: "setUiPreferences", theme: "light", locale: "en-US" },
    { action: "setViewport", width: 1440, height: 900 },
    ...login,
    ...steps,
];
const mobile = (steps) => [
    { action: "setUiPreferences", theme: "light", locale: "en-US" },
    { action: "setViewport", width: 390, height: 844 },
    ...login,
    ...steps,
];
const bell = "button[aria-label='Open message center']";
const drawer = ".ant-drawer";
const row = "button[aria-label^='Open message:']";
const singleReadTarget =
    "li.ant-list-item:has(button[aria-label='Open message: Runtime page browser-page-21']) .ant-list-item-action button";
const openDrawer = [
    { action: "waitFor", selector: bell },
    { action: "click", selector: bell },
    { action: "waitFor", selector: drawer },
];

const steps = {
    emptyDesktop: desktop([
        ...openDrawer,
        { action: "waitFor", selector: ".ant-empty-description" },
        { action: "assertText", selector: drawer, text: "No messages" },
        { action: "assertNoHorizontalOverflow" },
    ]),
    realtimeInvalidation: desktop([
        ...openDrawer,
        { action: "waitFor", selector: ".ant-empty-description" },
        { action: "assertText", selector: drawer, text: "No messages" },
        { action: "waitFor", selector: ".ant-badge-count" },
        { action: "assertText", selector: ".ant-badge-count", text: "1" },
        {
            action: "waitFor",
            selector: "button[aria-label='Open message: cpuHigh threshold exceeded']",
        },
    ]),
    loading: desktop([
        ...openDrawer,
        { action: "waitFor", selector: ".ant-skeleton" },
        { action: "waitFor", selector: row },
        { action: "assertNoHorizontalOverflow" },
    ]),
    populatedDetail: desktop([
        { action: "waitFor", selector: ".ant-badge-count" },
        { action: "assertText", selector: ".ant-badge-count", text: "1" },
        ...openDrawer,
        { action: "waitFor", selector: row },
        { action: "click", selector: row },
        { action: "waitFor", selector: "[aria-label='Message details']" },
        { action: "assertText", selector: "[aria-label='Message details']", text: "Incident opened" },
        { action: "assertNoHorizontalOverflow" },
        { action: "screenshotViewport", name: "message-center-desktop-en" },
    ]),
    unreadFilterPaging: desktop([
        ...openDrawer,
        { action: "waitFor", selector: row },
        { action: "click", selector: ".ant-segmented-item:nth-child(2)" },
        { action: "waitFor", selector: "button" },
        { action: "assertText", selector: drawer, text: "Load more" },
        { action: "click", selector: ".ant-drawer button.ant-btn-block" },
        { action: "assertNoHorizontalOverflow" },
    ]),
    singleRead: desktop([
        ...openDrawer,
        { action: "waitFor", selector: singleReadTarget },
        { action: "click", selector: singleReadTarget },
        { action: "pause", durationMs: 300 },
        { action: "assertAbsent", selector: singleReadTarget },
    ]),
    readAll: desktop([
        ...openDrawer,
        { action: "waitFor", selector: row },
        { action: "click", selector: "button[aria-label='Mark all read']" },
        { action: "pause", durationMs: 300 },
        { action: "assertAbsent", selector: ".ant-list-item-action button" },
    ]),
    forbiddenClears: desktop([
        ...openDrawer,
        { action: "waitFor", selector: row },
        { action: "pause", durationMs: 2500 },
        { action: "click", selector: ".ant-segmented-item:nth-child(2)" },
        { action: "waitFor", selector: ".ant-alert-warning" },
        {
            action: "assertText",
            selector: ".ant-alert-warning",
            text: "Message center unavailable",
        },
        { action: "assertAbsent", selector: row },
    ]),
    unauthorized: [
        { action: "setUiPreferences", theme: "light", locale: "en-US" },
        { action: "setViewport", width: 1440, height: 900 },
        { action: "goto", url: "/login" },
        { action: "waitFor", selector: "#login_username" },
        { action: "fill", selector: "#login_username", value: "owner" },
        { action: "fill", selector: "#login_password", value: "runtime-owner-password" },
        { action: "click", selector: "button[type=submit]" },
        { action: "pause", durationMs: 1000 },
        { action: "waitFor", selector: "#login_username" },
        { action: "assertAbsent", selector: bell },
    ],
    incidentDeepLink: desktop([
        ...openDrawer,
        { action: "waitFor", selector: ".ant-drawer button.ant-btn-block" },
        { action: "click", selector: ".ant-drawer button.ant-btn-block" },
        {
            action: "waitFor",
            selector: "button[aria-label='Open message: cpuHigh threshold exceeded']",
        },
        {
            action: "click",
            selector: "button[aria-label='Open message: cpuHigh threshold exceeded']",
        },
        { action: "waitFor", selector: "button[aria-label='View related record']" },
        { action: "click", selector: "button[aria-label='View related record']" },
        { action: "waitFor", selector: ".ant-drawer" },
        {
            action: "assertText",
            selector: ".ant-drawer",
            text: "cpuHigh threshold exceeded",
        },
    ]),
    mobile: mobile([
        ...openDrawer,
        { action: "waitFor", selector: row },
        { action: "assertNoHorizontalOverflow" },
        { action: "screenshotViewport", name: "message-center-mobile-en" },
    ]),
};

if (Object.keys(steps).length !== 11) throw new Error("message-center browser matrix drifted");
process.stdout.write(`${JSON.stringify(steps, null, 2)}\n`);
