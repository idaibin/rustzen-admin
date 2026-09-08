const deepLink = "/monitoring/nodes?q=web#node-1";

const cases = {
    success: {
        path: deepLink,
        binding: { bindingVersion: 1, webDigest: "stamp" },
        expected: { entry: "loaded", installationDigest: "stamp", reloads: 0 },
    },
    bindingMismatch: {
        path: deepLink,
        binding: { bindingVersion: 1, webDigest: "other" },
        expected: { entry: "absent", businessRequests: 0, reloads: 1, alert: true },
    },
    bindingNetworkFailure: {
        path: deepLink,
        binding: "network-error",
        expected: { entry: "absent", businessRequests: 0, reloads: 1, alert: true },
    },
    sriEntryFailure: {
        path: deepLink,
        binding: { bindingVersion: 1, webDigest: "stamp" },
        entry: "integrity-error",
        expected: { entry: "attempted", businessRequests: 0, reloads: 1, alert: true },
    },
};

for (const [name, scenario] of Object.entries(cases)) {
    if (scenario.path !== deepLink || scenario.expected.reloads > 1)
        throw new Error(`invalid selected-Web browser scenario: ${name}`);
    if (scenario.expected.businessRequests === 0 && scenario.expected.entry === "loaded")
        throw new Error(`successful scenario must allow business requests: ${name}`);
}

process.stdout.write(`${JSON.stringify({ version: 1, bindingRequest: { credentials: "omit", cache: "no-store" }, cases }, null, 2)}\n`);
