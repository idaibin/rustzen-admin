const login = [
    { action: "goto", url: "/login" },
    { action: "waitFor", selector: "#login_username" },
    { action: "fill", selector: "#login_username", value: "owner" },
    { action: "fill", selector: "#login_password", value: "rustzen@123" },
    { action: "click", selector: "button[type=submit]" },
    { action: "waitFor", selector: ".shell-content" },
];

const [first, second] = process.argv.slice(2);
if (!first || !second || first === second) {
    throw new Error("expected two distinct Monitor node IDs");
}

const bootSelector = (nodeId) => `[data-testid="monitor-node-boot-id-${nodeId}"]`;
const viewSelector = (nodeId) => `[data-testid="monitor-node-view"][data-node-id="${nodeId}"]`;
const detailBootSelector = (nodeId) => `[data-testid="monitor-node-details-boot-id-${nodeId}"]`;
const historySelector = (nodeId) => `[data-testid="monitor-node-history-5m-${nodeId}"]`;

const detail = (nodeId, bootId) => [
    { action: "waitFor", selector: viewSelector(nodeId) },
    { action: "assertText", selector: bootSelector(nodeId), text: bootId },
    { action: "click", selector: viewSelector(nodeId) },
    { action: "waitFor", selector: "[data-testid=monitor-node-details]" },
    { action: "waitFor", selector: detailBootSelector(nodeId) },
    { action: "assertText", selector: detailBootSelector(nodeId), text: bootId },
    { action: "waitFor", selector: historySelector(nodeId) },
    { action: "assertText", selector: historySelector(nodeId), text: "5-minute history" },
    { action: "waitFor", selector: `${historySelector(nodeId)} .recharts-wrapper` },
    { action: "screenshotViewport", name: `monitor-agent-${nodeId}-detail-dark-en` },
    { action: "pressKey", key: "Escape" },
];

process.stdout.write(
    `${JSON.stringify([
        { action: "setUiPreferences", theme: "dark", locale: "en-US" },
        { action: "setViewport", width: 1440, height: 900 },
        ...login,
        { action: "goto", url: "/monitoring/nodes" },
        { action: "waitFor", selector: "[data-testid=monitor-nodes-table]" },
        ...detail(first, process.env.RUSTZEN_VERIFY_BOOT_A),
        ...detail(second, process.env.RUSTZEN_VERIFY_BOOT_B),
        { action: "assertNoHorizontalOverflow" },
    ])}\n`,
);
