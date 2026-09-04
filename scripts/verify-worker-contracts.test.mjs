import { expect, test } from "bun:test";

const serviceVerifier = await Bun.file(new URL("./verify-services.sh", import.meta.url)).text();
const workerVerifier = await Bun.file(
    new URL("./verify-worker-contracts.mjs", import.meta.url),
).text();

test("the disposable service verifier explicitly matches the schedule fixture timezone", () => {
    expect(serviceVerifier).toContain("export RUSTZEN_TIMEZONE=UTC");
    expect(workerVerifier).toContain('scheduleSettings.timezone !== "UTC"');
    expect(workerVerifier).toContain("date.getUTCHours()");
    expect(workerVerifier).toContain("date.getUTCDay()");
});

test("the worker contract rejects unsafe pathname fields from Insights details", () => {
    expect(workerVerifier).toContain('for (const field of ["pagePath", "apiPath", "referrer"])');
    expect(workerVerifier).toContain("/[?#]/.test(event[field])");
    expect(workerVerifier).toContain("Insights details exposed an unsafe ${field}");
});
