import { describe, expect, test } from "bun:test";

const gate = await Bun.file(new URL("./verify-selected-web-bootstrap-browser.py", import.meta.url)).text();
const fixture = await Bun.file(new URL("./selected-web-bootstrap-browser-fixture.py", import.meta.url)).text();
const fixtureTest = new URL("./selected-web-bootstrap-browser-fixture.test.py", import.meta.url).pathname;
const admission = await Bun.file(new URL("./verify-selected-web-browser-admission.ts", import.meta.url)).text();

describe("selected-Web Chromium gate", () => {
    test("requires fresh evidence and executes each bootstrap failure through a loopback fixture", () => {
        expect(Bun.spawnSync(["python3", fixtureTest]).exitCode).toBe(0);
        for (const value of ["--runtime-container", "--admin-url", "--output", "--chromium", "--password-file", "--selection", "--export-root", "--release-result", "--certificate", "--public-key", "--expected-source-identity", "--admin-bin", "subprocess.Popen", "--headless=new"])
            expect(gate).toContain(value);
        for (const name of ["success", "bindingMismatch", "bindingNetworkFailure", "sriEntryFailure"])
            expect(gate).toContain(name);
        expect(gate).toContain("/api/installation");
        expect(gate).toContain('binding_request["authorization"]');
        expect(gate).toContain('binding_request["cookie"]');
        expect(fixture).toContain("rz_bootstrap_proof");
        expect(gate).toContain("entryExecuted");
        expect(gate).toContain("reloadCount");
        expect(gate).toContain("manualRetry");
        expect(gate).toContain("/api/auth/login");
        expect(gate).toContain("/health");
        expect(gate).toContain("stat.S_IMODE");
        expect(gate).toContain("os.open(path, flags)");
        expect(gate).toContain("os.fstat(descriptor)");
        expect(gate).toContain("os.O_NOFOLLOW");
        expect(gate).toContain("stat.S_ISREG");
        expect(gate).toContain("state.st_uid != os.getuid()");
        expect(gate).toContain('parsed.hostname != "127.0.0.1"');
        expect(gate).toContain("output.mkdir(parents=False)");
        expect(gate).toContain("--remote-allow-origins=*");
        expect(gate).toContain('parsed.path + (f"?{parsed.query}" if parsed.query else "")');
        expect(gate.indexOf("output.mkdir(parents=False)")).toBeGreaterThan(gate.indexOf("browser receipt is not canonical"));
        expect(fixture).toContain("sriIntegrityRemoved");
        expect(gate).toContain("__rz_sri_tamper_executed");
        expect(gate).toContain('"integritySensitivityPassed":integritySensitivityPassed');
        expect(gate).not.toContain("integrityRemovalRejected");
        expect(gate).toContain("sort_keys=True");
        expect(gate).not.toContain('add_argument("--password"');
        expect(fixture).toContain("bindingNetworkFailure");
        expect(fixture).toContain("sriEntryFailure");
        expect(fixture).toContain("requests.append");
    });
});
