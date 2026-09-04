import { describe, expect, test } from "bun:test";
import { resolve } from "node:path";

const validateDistribution = (value: string | undefined) => {
    const distribution = value ?? "full";
    if (distribution !== "full" && distribution !== "monitor")
        throw new Error("DISTRIBUTION must be full or monitor");
    return distribution;
};

function assertDockerfileGuard(dockerfile: string) {
    if (!dockerfile.includes('case "${DISTRIBUTION}" in full|monitor)'))
        throw new Error("Dockerfile must restrict DISTRIBUTION to full|monitor");
    if (!dockerfile.includes('cmp -s /tmp/rz-selected-web.source /tmp/rz-selected-web.embedded'))
        throw new Error("Dockerfile must compare selected Web source and embedded file hashes");
}

describe("Docker distribution input", () => {
    test("defaults only an omitted argument to full and accepts monitor", () => {
        expect(validateDistribution(undefined)).toBe("full");
        expect(validateDistribution("monitor")).toBe("monitor");
    });
    test("Dockerfile applies the same allowlist before Web or binary build", async () => {
        const dockerfile = await Bun.file(resolve(import.meta.dir, "../Dockerfile")).text();
        assertDockerfileGuard(dockerfile);
        expect(dockerfile.indexOf('DISTRIBUTION must be full or monitor')).toBeLessThan(dockerfile.indexOf('COPY --from=bun-runtime'));
        expect(dockerfile).toContain('rm -rf "apps/admin/selected-web/${composition}"');
        expect(dockerfile).toContain('cp -R "target/distributions/${composition}/web/dist/." "apps/admin/selected-web/${composition}/dist"');
    });
    test("fails if the Docker allowlist or selected-asset hash check is deleted", async () => {
        const dockerfile = await Bun.file(resolve(import.meta.dir, "../Dockerfile")).text();
        expect(() => assertDockerfileGuard(dockerfile.replace('case "${DISTRIBUTION}" in full|monitor)', ""))).toThrow("restrict");
        expect(() => assertDockerfileGuard(dockerfile.replace('cmp -s /tmp/rz-selected-web.source /tmp/rz-selected-web.embedded', ""))).toThrow("hashes");
    });
    test("rejects empty, misspelled and custom compositions before build", () => {
        for (const value of ["", "monitr", "custom"])
            expect(() => validateDistribution(value)).toThrow("DISTRIBUTION must be full or monitor");
    });
});
