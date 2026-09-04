import { describe, expect, test } from "bun:test";
import { resolve } from "node:path";

const validateDistribution = (value: string | undefined) => {
    const distribution = value ?? "full";
    if (distribution !== "full" && distribution !== "monitor")
        throw new Error("DISTRIBUTION must be full or monitor");
    return distribution;
};
const installs = (branch: string) =>
    [...branch.matchAll(/"(\/out\/[^"\s]+)"/g)].map((match) => match[1]).sort();
const exact = (actual: string[], expected: string[], label: string) => {
    if (JSON.stringify(actual) !== JSON.stringify([...expected].sort()))
        throw new Error(`${label} output is not exact`);
};
function branches(dockerfile: string) {
    const monitor = dockerfile.match(
        /if \[ "\$\{DISTRIBUTION\}" = "monitor" \]; then([\s\S]*?)elif \[ "\$\{TARGET_TRIPLE\}"/,
    );
    const aarch64 = dockerfile.match(
        /elif \[ "\$\{TARGET_TRIPLE\}" = "aarch64-unknown-linux-gnu" \]; then([\s\S]*?)else/,
    );
    const full = dockerfile.match(
        /else \\\n        RUSTFLAGS=([\s\S]*?)\n    fi/,
    );
    if (!monitor || !aarch64 || !full)
        throw new Error("Dockerfile build branches are incomplete");
    return { monitor: monitor[1], aarch64: aarch64[1], full: full[1] };
}
function assertDockerfileGuard(dockerfile: string) {
    if (!dockerfile.includes('case "${DISTRIBUTION}" in full|monitor)'))
        throw new Error(
            "Dockerfile must restrict DISTRIBUTION to full|monitor",
        );
    if (
        !dockerfile.includes(
            "cmp -s /tmp/rz-selected-web.source /tmp/rz-selected-web.embedded",
        )
    )
        throw new Error(
            "Dockerfile must compare selected Web source and embedded file hashes",
        );
    const b = branches(dockerfile);
    if (!b.aarch64.includes("rustzen-reports &&"))
        throw new Error("aarch64 build failure must stop install");
    exact(
        installs(b.monitor),
        [
            "/out/agent/bin/rz-monitor-agent",
            "/out/server/bin/rz-admin",
            "/out/server/bin/rz-monitor",
        ],
        "monitor",
    );
    const fullPaths = [
        "/out/bin/rz",
        "/out/bin/rz-admin",
        "/out/bin/rz-insights",
        "/out/bin/rz-monitor",
        "/out/bin/rz-reports",
    ];
    exact(installs(b.aarch64), fullPaths, "aarch64 full");
    exact(installs(b.full), fullPaths, "default full");
}

describe("Docker distribution input", () => {
    test("defaults only an omitted argument to full and accepts monitor", () => {
        expect(validateDistribution(undefined)).toBe("full");
        expect(validateDistribution("monitor")).toBe("monitor");
    });
    test("Dockerfile has exact monitor/server-agent and full output inventories", async () => {
        const dockerfile = await Bun.file(
            resolve(import.meta.dir, "../Dockerfile"),
        ).text();
        assertDockerfileGuard(dockerfile);
        expect(
            dockerfile.indexOf("DISTRIBUTION must be full or monitor"),
        ).toBeLessThan(dockerfile.indexOf("COPY --from=bun-runtime"));
    });
    test("rejects output pollution, omission and non-propagating build failures", async () => {
        const dockerfile = await Bun.file(
            resolve(import.meta.dir, "../Dockerfile"),
        ).text();
        for (const mutation of [
            (s: string) =>
                s.replace('case "${DISTRIBUTION}" in full|monitor)', ""),
            (s: string) =>
                s.replace(
                    "cmp -s /tmp/rz-selected-web.source /tmp/rz-selected-web.embedded",
                    "",
                ),
            (s: string) =>
                s.replace(
                    "/out/agent/bin/rz-monitor-agent",
                    "/out/server/bin/rz-monitor-agent",
                ),
            (s: string) =>
                s.replace(
                    '/out/server/bin/rz-monitor" &&',
                    '/out/server/bin/rz-reports" &&',
                ),
            (s: string) =>
                s.replace("/out/bin/rz-reports", "/out/bin/rz-extra"),
            (s: string) =>
                s.replace("/out/bin/rz-insights", "/out/bin/rz-extra"),
            (s: string) =>
                s.replace("/out/server/bin/rz-admin", "/out/bin/rz-admin"),
            (s: string) =>
                s.replace("/out/server/bin/rz-monitor", "/out/bin/rz-monitor"),
            (s: string) =>
                s.replace(
                    "/out/agent/bin/rz-monitor-agent",
                    "/out/bin/rz-monitor-agent",
                ),
            (s: string) =>
                s.replace("/out/bin/rz-admin", "/out/server/bin/rz-admin"),
            (s: string) =>
                s.replace("/out/bin/rz-monitor", "/out/agent/bin/rz-monitor"),
            (s: string) =>
                s.replace(
                    '/out/agent/bin/rz-monitor-agent"',
                    '/out/agent/bin/rz-monitor-agent" && install -m 0755 "/app/sentinel" "/out/bin/rz-reports"',
                ),
            (s: string) =>
                s.replace(
                    '/out/agent/bin/rz-monitor-agent"',
                    '/out/agent/bin/rz-monitor-agent" && install -m 0755 "/app/sentinel" "/out/other/bin/rz-extra"',
                ),
            (s: string) =>
                s.replace(
                    '/out/bin/rz-reports"',
                    '/out/bin/rz-reports" && install -m 0755 "/app/sentinel" "/out/server/bin/rz-extra"',
                ),
            (s: string) => s.replace("rustzen-reports &&", "rustzen-reports;"),
        ])
            expect(() => assertDockerfileGuard(mutation(dockerfile))).toThrow();
    });
    test("rejects empty, misspelled and custom compositions before build", () => {
        for (const value of ["", "monitr", "custom"])
            expect(() => validateDistribution(value)).toThrow(
                "DISTRIBUTION must be full or monitor",
            );
    });
});
