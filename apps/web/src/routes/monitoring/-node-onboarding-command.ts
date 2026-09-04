export const nodeIdPattern = /^[A-Za-z0-9._-]{1,128}$/;

export function normalizeControllerUrl(value: string): string {
    const url = new URL(value.trim());
    if (
        !["http:", "https:"].includes(url.protocol) ||
        url.username ||
        url.password ||
        url.search ||
        url.hash
    ) {
        throw new Error("Use an HTTP(S) console address without credentials, query or fragment.");
    }
    return url.toString().replace(/\/+$/, "");
}

const shellQuote = (value: string) => `'${value.replaceAll("'", "'\\''")}'`;

export function nodeOnboardingCommand(nodeId: string, controllerUrl: string): string {
    if (!nodeIdPattern.test(nodeId)) throw new Error("Invalid node ID");
    const url = normalizeControllerUrl(controllerUrl);
    return [
        "# Bash / Linux / macOS",
        'read -r -s -p "Agent token: " RUSTZEN_MONITOR_AGENT_TOKEN',
        'printf "\\n"',
        "export RUSTZEN_MONITOR_AGENT_TOKEN",
        ': "${RUSTZEN_MONITOR_AGENT_TOKEN:?Agent token is required}"',
        `export RUSTZEN_MONITOR_NODE_ID=${shellQuote(nodeId)}`,
        `export RUSTZEN_MONITOR_CONTROLLER_URL=${shellQuote(url)}`,
        "rz-monitor-agent",
    ].join("\n");
}
