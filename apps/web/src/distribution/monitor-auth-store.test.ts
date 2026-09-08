import { expect, test } from "bun:test";

import { getRouteCapabilityCodes } from "./monitor-auth-store";

test("selected monitor root and generated landing page share the overview capability", () => {
    expect(getRouteCapabilityCodes("/")).toEqual(["monitor:overview:view"]);
    expect(getRouteCapabilityCodes("/monitoring/overview")).toEqual(["monitor:overview:view"]);
});
