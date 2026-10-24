import { expect, test } from "bun:test";

import { getDialogSessionTransition } from "./-dialog-session";

test("same-record refresh keeps the session key; reopen and record changes initialize a new session", () => {
    let sessionKey: string | null = null;

    const startEdit = (recordId: number, open = true) => {
        const transition = getDialogSessionTransition(open, "edit", recordId, sessionKey);
        sessionKey = transition.sessionKey;
        return transition.shouldInitialize;
    };

    expect(startEdit(7)).toBe(true);
    expect(startEdit(7)).toBe(false);
    expect(startEdit(7, false)).toBe(false);
    expect(startEdit(7)).toBe(true);
    expect(startEdit(8)).toBe(true);
});

test("create sessions reset after close and remain separate from edit sessions", () => {
    let sessionKey: string | null = null;

    const transition = (open: boolean, mode: "create" | "edit", recordId?: number) => {
        const result = getDialogSessionTransition(open, mode, recordId, sessionKey);
        sessionKey = result.sessionKey;
        return result;
    };

    expect(transition(true, "create")).toEqual({
        sessionKey: "create:new",
        shouldInitialize: true,
    });
    expect(transition(true, "create")).toEqual({
        sessionKey: "create:new",
        shouldInitialize: false,
    });
    expect(transition(false, "create")).toEqual({ sessionKey: null, shouldInitialize: false });
    expect(transition(true, "create")).toEqual({
        sessionKey: "create:new",
        shouldInitialize: true,
    });
    expect(transition(true, "edit", 4)).toEqual({ sessionKey: "edit:4", shouldInitialize: true });
});
