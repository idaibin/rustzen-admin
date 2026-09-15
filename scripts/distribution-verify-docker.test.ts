import { describe, expect, test } from "bun:test";
import { resolve } from "node:path";

const validateDistribution = (value: string | undefined) => {
    const distribution = value ?? "full";
    if (!["full", "monitor", "monitor-notify", "analytics"].includes(distribution))
        throw new Error("DISTRIBUTION must be full, monitor, monitor-notify or analytics");
    return distribution;
};
const installs = (branch: string) =>
    [...branch.matchAll(/"(\/out\/[^"\s]+)"/g)].map((match) => match[1]).sort();
const exact = (actual: string[], expected: string[], label: string) => {
    if (JSON.stringify(actual) !== JSON.stringify([...expected].sort()))
        throw new Error(`${label} output is not exact`);
};
function branches(dockerfile: string) {
    // Split on the export RUN's own branch separators instead of fragile
    // cross-line regexes: each segment is one build branch body.
    const cargoRun = dockerfile.slice(dockerfile.indexOf("RUN --mount=type=cache,target=/root/.cargo/registry"));
    const analyticsAt = cargoRun.indexOf('elif [ "${RZ_DISTRIBUTION}" = "analytics" ]; then');
    const tripleAt = cargoRun.indexOf('elif [ "${TARGET_TRIPLE}"');
    const elseAt = cargoRun.indexOf("else \\\n");
    const fiAt = cargoRun.indexOf("\n    fi", elseAt);
    const monitorHead = 'if [ "${RZ_DISTRIBUTION}" = "monitor" ] || [ "${RZ_DISTRIBUTION}" = "monitor-notify" ]; then';
    const monitor = cargoRun.includes(monitorHead) && analyticsAt > 0 && cargoRun.indexOf(monitorHead) < analyticsAt
        ? [null, cargoRun.slice(cargoRun.indexOf(monitorHead) + monitorHead.length, analyticsAt)]
        : null;
    const analytics = analyticsAt > 0 && tripleAt > analyticsAt
        ? [null, cargoRun.slice(analyticsAt, tripleAt)]
        : null;
    const aarch64 = tripleAt > 0 && elseAt > tripleAt
        ? [null, cargoRun.slice(tripleAt, elseAt)]
        : null;
    const full = elseAt > 0 && fiAt > elseAt
        ? [null, cargoRun.slice(elseAt, fiAt)]
        : null;
    if (!monitor || !analytics || !aarch64 || !full)
        throw new Error("Dockerfile build branches are incomplete");
    return { monitor: monitor[1]!, analytics: analytics[1]!, aarch64: aarch64[1]!, full: full[1]! };
}
function assertDockerfileGuard(dockerfile: string) {
    if (!dockerfile.includes('case "${RZ_DISTRIBUTION}" in full|monitor|monitor-notify|analytics)'))
        throw new Error("Dockerfile must restrict RZ_DISTRIBUTION to exact supported distributions");
    if (!dockerfile.includes("cmp -s /tmp/rz-selected-web.source /tmp/rz-selected-web.embedded"))
        throw new Error("Dockerfile must compare selected Web source and embedded file hashes");
    for (const required of [
        "ARG SOURCE_IDENTITY=",
        'test -n "${SOURCE_IDENTITY}"',
        'test "${TARGET_TRIPLE}" = "x86_64-unknown-linux-musl"',
        "scripts/distribution-produce-contracts.ts",
        "scripts/distribution-produce-protocol.ts",
        "scripts/distribution-produce-native-layout.ts",
        "scripts/distribution-web-inventory-schema.ts",
        "scripts/distribution-web-inventory-policy.ts",
        "scripts/distribution-selected-api-policy.ts",
        "scripts/distribution-web-allowed-packages.ts",
        "scripts/distribution-notification-delivery-closure.ts",
        "COPY distribution distribution",
        "bun scripts/distribution-produce-container-export.ts",
        "--binary-root /tmp/rz-monitor-producers",
        "--output-root /out/release/contracts/protocol",
        "--output-root /out/release/contracts/native",
        "--output-root /out",
        "RUSTZEN_CONTAINER_BUILD_COMMANDS=",
        "monitor-notify) fixture=distribution/fixtures/monitor-notify.json",
        "admin_features=monitor-distribution,notifications",
        "monitor_features=notifications",
        "analytics) fixture=distribution/fixtures/analytics.json",
        "--features analytics-distribution --bin rz-admin",
        "--features selected-distribution --bin rz-insights",
        "RUSTZEN_CONTAINER_EVIDENCE=linux-amd64-buildkit",
    ])
        if (!dockerfile.includes(required))
            throw new Error(`Dockerfile is missing P8b Monitor export boundary: ${required}`);
    const b = branches(dockerfile);
    if (!b.aarch64.includes("rustzen-reports &&"))
        throw new Error("aarch64 build failure must stop install");
    exact(
        installs(b.monitor),
        [
            "/out/release/server/bin/rz-admin",
            "/out/release/server/bin/rz-monitor",
            "/out/witness/bin/rz-monitor-agent",
        ],
        "monitor",
    );
    exact(installs(b.analytics), ["/out/release/server/bin/rz-admin", "/out/release/server/bin/rz-insights"], "analytics");
    if (b.analytics.includes("rz-monitor") || b.analytics.includes("rz-reports") || b.analytics.includes("witness")) throw new Error("Analytics output is polluted");
    for (const kind of ["--kind api", "--kind config", "--kind schema"])
        if (!b.analytics.includes(`bun scripts/distribution-produce-contracts.ts --selection "\${fixture}" --binary-root /tmp/rz-analytics-producers ${kind}`))
            throw new Error(`Analytics contracts must be produced per kind: ${kind}`);
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
    test("defaults only an omitted argument to full and accepts selected analytics", () => {
        expect(validateDistribution(undefined)).toBe("full");
        expect(validateDistribution("monitor")).toBe("monitor");
        expect(validateDistribution("monitor-notify")).toBe("monitor-notify");
        expect(validateDistribution("analytics")).toBe("analytics");
    });
    test("Dockerfile has exact monitor/server-agent and full output inventories", async () => {
        const dockerfile = await Bun.file(resolve(import.meta.dir, "../Dockerfile")).text();
        assertDockerfileGuard(dockerfile);
        expect(dockerfile.indexOf("DISTRIBUTION must be full, monitor, monitor-notify or analytics")).toBeLessThan(
            dockerfile.indexOf("COPY --from=bun-runtime"),
        );
    });
    test("fresh Linux selected Web passes the same portable Admin build gate", async () => {
        const dockerfile = await Bun.file(resolve(import.meta.dir, "../Dockerfile")).text();
        const produced = dockerfile.indexOf("bun scripts/distribution-build-web.ts");
        const embedded = dockerfile.indexOf(
            'cp "target/distributions/${composition}/web/inventory.json"',
        );
        const compiled = dockerfile.indexOf(
            'cargo build --release --target "${TARGET_TRIPLE}" -p rustzen-admin --no-default-features --features "${admin_features}"',
        );
        expect(produced).toBeGreaterThan(0);
        expect(embedded).toBeGreaterThan(produced);
        expect(compiled).toBeGreaterThan(embedded);
        const gate = await Bun.file(
            resolve(import.meta.dir, "../apps/admin/build_support/selected_web.rs"),
        ).text();
        expect(gate).not.toContain("INVENTORY_SHA256");
        expect(gate).not.toContain("canonical inventory digest");
    });
    test("rejects output pollution, omission and non-propagating build failures", async () => {
        const dockerfile = await Bun.file(resolve(import.meta.dir, "../Dockerfile")).text();
        [
            (s: string) => s.replace('case "${RZ_DISTRIBUTION}" in full|monitor|monitor-notify|analytics)', ""),
            (s: string) => s.replaceAll("monitor-notify) fixture=distribution/fixtures/monitor-notify.json", "monitor-notify) fixture=${RZ_DISTRIBUTION}"),
            (s: string) => s.replaceAll("admin_features=monitor-distribution,notifications", "admin_features=${FEATURES}"),
            (s: string) => s.replace('test "${TARGET_TRIPLE}" = "x86_64-unknown-linux-musl"', "true"),
            (s: string) =>
                s.replace("cmp -s /tmp/rz-selected-web.source /tmp/rz-selected-web.embedded", ""),
            (s: string) =>
                s.replace(
                    "/out/witness/bin/rz-monitor-agent",
                    "/out/release/server/bin/rz-monitor-agent",
                ),
            (s: string) =>
                s.replace(
                    '/out/release/server/bin/rz-monitor" &&',
                    '/out/release/server/bin/rz-reports" &&',
                ),
            (s: string) => s.replace("/out/bin/rz-reports", "/out/bin/rz-extra"),
            (s: string) => s.replace("/out/bin/rz-insights", "/out/bin/rz-extra"),
            (s: string) => s.replace("/out/release/server/bin/rz-admin", "/out/bin/rz-admin"),
            (s: string) => s.replace("/out/release/server/bin/rz-monitor", "/out/bin/rz-monitor"),
            (s: string) =>
                s.replace("/out/witness/bin/rz-monitor-agent", "/out/bin/rz-monitor-agent"),
            (s: string) =>
                s.replace(
                    '/out/witness/bin/rz-monitor-agent"',
                    '/out/witness/bin/rz-monitor-agent" && install -m 0755 "/app/sentinel" "/out/bin/rz-reports"',
                ),
            (s: string) =>
                s.replace(
                    '/out/witness/bin/rz-monitor-agent"',
                    '/out/witness/bin/rz-monitor-agent" && install -m 0755 "/app/sentinel" "/out/other/bin/rz-extra"',
                ),
            (s: string) =>
                s.replace(
                    '/out/bin/rz-reports"',
                    '/out/bin/rz-reports" && install -m 0755 "/app/sentinel" "/out/server/bin/rz-extra"',
                ),
            (s: string) => s.replace("rustzen-reports &&", "rustzen-reports;"),
            (s: string) => s.replace('test -n "${SOURCE_IDENTITY}"', "true"),
            (s: string) =>
                s.replaceAll(
                    "bun scripts/distribution-produce-container-export.ts",
                    "bun scripts/missing-export.ts",
                ),
            (s: string) =>
                s.replaceAll("RUSTZEN_CONTAINER_BUILD_COMMANDS=", "RUSTZEN_CONTAINER_COMMANDS="),
            (s: string) => s.replace(" scripts/distribution-web-inventory-schema.ts", ""),
            ...[
                "scripts/distribution-web-inventory-policy.ts",
                "scripts/distribution-web-allowed-packages.ts",
                "scripts/distribution-notification-delivery-closure.ts",
            ].map((source) => (s: string) => s.replace(` ${source}`, "")),
            (s: string) => s.replace("COPY distribution distribution", "COPY distribution/resolver.ts distribution/resolver.ts"),
        ].forEach((mutation, index) => {
            try {
                assertDockerfileGuard(mutation(dockerfile));
            } catch {
                return;
            }
            throw new Error(`mutation ${index} must fail closed`);
        });
    });
    test("rejects empty, misspelled and custom compositions before build", () => {
        for (const value of ["", "monitr", "custom"])
            expect(() => validateDistribution(value)).toThrow(
                "DISTRIBUTION must be full, monitor, monitor-notify or analytics",
            );
    });
});
