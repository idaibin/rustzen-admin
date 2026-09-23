import { describe, expect, test } from "bun:test";

import { ApiRequestError } from "@/api/request";

import { classifyWriteFailure, visibleMessages } from "./message-write-error";

describe("message write failures", () => {
    test("classifies authorization, disappearance and retryable failures", () => {
        expect(classifyWriteFailure(new ApiRequestError("denied", { status: 403 }))).toBe(
            "forbidden",
        );
        expect(classifyWriteFailure(new ApiRequestError("gone", { status: 404 }))).toBe("missing");
        expect(classifyWriteFailure(new Error("offline"))).toBe("retryable");
    });

    test("never exposes cached or disappeared rows after a write denial", () => {
        const cached = [{ id: "secret" }, { id: "visible" }];
        expect(visibleMessages(cached, true, new Set())).toEqual([]);
        expect(visibleMessages(cached, false, new Set(["secret"]))).toEqual([{ id: "visible" }]);
    });
});
