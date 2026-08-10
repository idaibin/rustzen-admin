import { expect, test } from "bun:test";

import { formatDateTime } from "./format-date-time";

const value = "2026-08-10T12:34:56Z";

test("shows a complete localized date and time in the browser timezone", () => {
    const expected = new Intl.DateTimeFormat(undefined, {
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
    }).format(new Date(value));

    expect(formatDateTime(value)).toBe(expected);
});

test("shows a complete localized date and seconds in the requested timezone", () => {
    const expected = new Intl.DateTimeFormat(undefined, {
        timeZone: "Asia/Shanghai",
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
    }).format(new Date(value));

    expect(formatDateTime(value, "Asia/Shanghai")).toBe(expected);
    expect(formatDateTime(value, "Asia/Shanghai")).not.toBe(
        new Intl.DateTimeFormat(undefined, { timeZone: "Asia/Shanghai" }).format(new Date(value)),
    );
});

test("returns a safe placeholder for invalid date or timezone input", () => {
    expect(formatDateTime("not-a-date", "Asia/Shanghai")).toBe("-");
    expect(formatDateTime(value, "Not/A-Timezone")).toBe("-");
});
