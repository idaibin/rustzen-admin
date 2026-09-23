import { expect, test } from "bun:test";
import { parseSourceIdentity } from "./source-identity.ts";
test("source identity accepts only fixed git tree state tuples", () => {
    expect(parseSourceIdentity(`git:${"a".repeat(40)} tree:${"b".repeat(64)} state:clean`).state).toBe("clean");
    for (const value of ["git:abc tree:def state:clean", `git:${"a".repeat(40)} tree:${"b".repeat(64)} state:other`, "tree:x"])
        expect(() => parseSourceIdentity(value)).toThrow("identity");
});
