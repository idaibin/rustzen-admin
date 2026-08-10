import { expect, test } from "bun:test";

import { checkIncidentFilter, checkIncidentHref } from "./-check-context";

test("check context links to incidents with the check source filter", () => {
    expect(checkIncidentFilter("check/1")).toEqual({ sourceType: "check", sourceId: "check/1" });
    expect(checkIncidentHref("check/1")).toBe(
        "/monitoring/incidents?sourceType=check&sourceId=check%2F1",
    );
});
