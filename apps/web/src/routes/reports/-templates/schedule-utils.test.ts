import { expect, test } from "bun:test";

import { formatScheduleOccurrenceDue } from "./schedule-utils";

test("schedule occurrence due prefers resolved UTC time", () => {
    expect(
        formatScheduleOccurrenceDue(
            { dueAt: "2026-09-05T08:30:00Z", dueLocal: "2026-09-05T08:30" },
            "UTC",
        ),
    ).toContain("2026");
});

test("schedule occurrence due keeps DST-gap local time and installation timezone", () => {
    expect(
        formatScheduleOccurrenceDue(
            { dueAt: null, dueLocal: "2026-03-08T02:30" },
            "America/Los_Angeles",
        ),
    ).toBe("2026-03-08T02:30 · America/Los_Angeles");
});
