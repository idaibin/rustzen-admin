import { describe, expect, test } from "bun:test";

import { SseParser, type SseEvent } from "./sse-parser";

const bytes = (value: string) => new TextEncoder().encode(value);

describe("SseParser", () => {
    test("decodes split UTF-8, CRLF, comments and multiline data", () => {
        const events: SseEvent[] = [];
        const parser = new SseParser((event) => events.push(event));
        const input = bytes(
            ": heartbeat\r\nevent: inbox.changed\r\nid: 42\r\ndata: 第一行\r\ndata: second\r\n\r\n",
        );
        const split = input.indexOf(0xe4) + 1;
        parser.push(input.slice(0, split));
        parser.push(input.slice(split));
        parser.finish();
        expect(events).toEqual([{ event: "inbox.changed", id: "42", data: "第一行\nsecond" }]);
    });

    test("rejects oversized lines, events, malformed UTF-8 and partial EOF", () => {
        expect(() => new SseParser(() => {}).push(bytes(`data: ${"x".repeat(8193)}`))).toThrow();
        const parser = new SseParser(() => {});
        parser.push(bytes(`data: ${"x".repeat(8000)}\ndata: ${"y".repeat(8000)}\n`));
        expect(() => parser.push(bytes(`data: ${"z".repeat(500)}\n`))).toThrow();
        expect(() => new SseParser(() => {}).push(Uint8Array.of(0xff))).toThrow();
        const partial = new SseParser(() => {});
        partial.push(bytes("data: partial"));
        expect(() => partial.finish()).toThrow();
    });

    test("limits the accumulated event, id and data fields to 16 KiB", () => {
        const accepted: SseEvent[] = [];
        const exact = new SseParser((event) => accepted.push(event));
        exact.push(
            bytes(
                `event: ${"e".repeat(8000)}\r\nid: ${"i".repeat(8000)}\r\ndata: ${"d".repeat(361)}\r\n\r\n`,
            ),
        );
        expect(accepted).toHaveLength(1);

        const oversized = new SseParser(() => {});
        expect(() =>
            oversized.push(
                bytes(
                    `event: ${"e".repeat(8000)}\r\nid: ${"i".repeat(8000)}\r\ndata: ${"d".repeat(362)}\r\n`,
                ),
            ),
        ).toThrow("SSE event exceeds 16 KiB");
    });
});
