import { expect, test } from "bun:test";

import { getRouteCapabilityCodes } from "./useAuthStore";

test("reports templates can be reached by either flow or schedule viewers", () => {
    expect(getRouteCapabilityCodes("/reports/templates")).toEqual([
        "reports:flow:view",
        "reports:schedule:view",
    ]);
});

test("other routes keep one explicit capability", () => {
    expect(getRouteCapabilityCodes("/monitoring/summaries")).toEqual(["monitor:node:view"]);
});

test("module log diagnostics route requires its owner-only view capability", () => {
    expect(getRouteCapabilityCodes("/system/module-log")).toEqual(["system:module:log:view"]);
});

test("incident routes require the incident viewer capability", () => {
    expect(getRouteCapabilityCodes("/monitoring/incidents")).toEqual(["monitor:incident:view"]);
});
