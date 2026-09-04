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

test("incident routes require the incident viewer capability", () => {
    expect(getRouteCapabilityCodes("/monitoring/incidents")).toEqual(["monitor:incident:view"]);
});
