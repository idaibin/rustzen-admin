import { describe, expect, test } from "bun:test";

const driver = new URL("./selected-web-bootstrap-browser-cases.mjs", import.meta.url).pathname;

describe("selected-Web browser acceptance matrix", () => {
    test("covers the bounded bootstrap closure", () => {
        const result = Bun.spawnSync([process.execPath, driver], { stdout: "pipe", stderr: "pipe" });
        expect(result.exitCode).toBe(0);
        const matrix = JSON.parse(new TextDecoder().decode(result.stdout));
        expect(matrix.bindingRequest).toEqual({ credentials: "omit", cache: "no-store" });
        expect(Object.keys(matrix.cases)).toEqual([
            "success",
            "bindingMismatch",
            "bindingNetworkFailure",
            "sriEntryFailure",
        ]);
        for (const scenario of Object.values(matrix.cases)) {
            expect(scenario.path).toBe("/monitoring/nodes?q=web#node-1");
            expect(scenario.expected.reloads).toBeLessThanOrEqual(1);
        }
        expect(matrix.cases.success.expected.installationDigest).toBe("stamp");
        for (const name of ["bindingMismatch", "bindingNetworkFailure", "sriEntryFailure"])
            expect(matrix.cases[name].expected.businessRequests).toBe(0);
    });
});
