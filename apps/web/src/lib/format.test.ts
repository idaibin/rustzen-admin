import { describe, expect, test } from "bun:test";

import { formatBytes, formatDuration, formatPercent } from "./format";

describe("formatBytes", () => {
    test("preserves byte unit boundaries", () => {
        expect([0, 1, 1023, 1024, 1536, 1024 ** 2, 1024 ** 4].map(formatBytes)).toEqual([
            "0 B",
            "1 B",
            "1023 B",
            "1 KB",
            "1.5 KB",
            "1 MB",
            "1 TB",
        ]);
    });

    test("caps at the largest supported unit", () => {
        expect(formatBytes(1024 ** 5)).toBe("1024 TB");
    });
});

describe("formatPercent", () => {
    test("keeps one decimal and strips trailing zeros", () => {
        expect(formatPercent(63.44)).toBe("63.4%");
        expect(formatPercent(0)).toBe("0%");
        expect(formatPercent(100)).toBe("100%");
    });
});

describe("formatDuration", () => {
    test("renders milliseconds and keeps null as a dash", () => {
        expect(formatDuration(0)).toBe("0 ms");
        expect(formatDuration(1250)).toBe("1250 ms");
        expect(formatDuration(null)).toBe("-");
        expect(formatDuration(undefined)).toBe("-");
    });
});
