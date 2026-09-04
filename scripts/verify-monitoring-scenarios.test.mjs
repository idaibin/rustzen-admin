import { expect, test } from "bun:test";

import { assertMonitoringNavigation, monitoringNavigationPaths } from "./verify-monitoring-scenarios.mjs";

test("Monitor navigation contains exactly the four current surfaces", () => {
    expect(() => assertMonitoringNavigation([...monitoringNavigationPaths].reverse())).not.toThrow();
});

for (const retiredPath of ["/monitoring/settings", "/monitoring/checks"]) {
    test(`Monitor navigation explicitly rejects retired ${retiredPath} page`, () => {
        expect(() => assertMonitoringNavigation([...monitoringNavigationPaths, retiredPath]))
            .toThrow(`retired Monitor page must not be navigable: ${retiredPath}`);
    });
}
