import { expect, test } from "bun:test";
import { resolve } from "node:path";

function assertDockerfileGuard(dockerfile: string) {
    for (const forbidden of ["ARG DISTRIBUTION", "selected-distribution", "monitor-distribution"])
        if (dockerfile.includes(forbidden))
            throw new Error("Dockerfile retains selected distribution input: " + forbidden);

    for (const required of [
        "COPY apps/web/package.json apps/web/bun.lock apps/web/",
        "cd apps/web && bun install --frozen-lockfile --ignore-scripts",
        "COPY apps/web apps/web",
        "cd apps/web && bun run vp build",
        "-p rustzen-cli -p rustzen-admin -p rustzen-monitor -p rustzen-insights -p rustzen-reports",
        'install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz" /out/bin/rz',
        'install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-admin" /out/bin/rz-admin',
        'install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-monitor" /out/bin/rz-monitor',
        'install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-insights" /out/bin/rz-insights',
        'install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-reports" /out/bin/rz-reports',
        "FROM scratch AS export",
        "COPY --from=build /out /",
    ])
        if (!dockerfile.includes(required))
            throw new Error("Dockerfile is missing full-release boundary: " + required);
}

test("Dockerfile builds and exports only the complete five-binary release", async () => {
    const dockerfile = await Bun.file(resolve(import.meta.dir, "../Dockerfile")).text();
    assertDockerfileGuard(dockerfile);
    expect(dockerfile.indexOf("cd apps/web && bun run vp build")).toBeLessThan(
        dockerfile.indexOf("cargo build --release"),
    );
});

test("full-release Docker guard rejects omissions and selected-distribution inputs", async () => {
    const dockerfile = await Bun.file(resolve(import.meta.dir, "../Dockerfile")).text();
    for (const mutation of [
        (source: string) => source.replaceAll(" -p rustzen-reports", ""),
        (source: string) => source.replace("/out/bin/rz-reports", "/out/bin/rz-extra"),
        (source: string) => "ARG DISTRIBUTION=full\n" + source,
    ]) {
        expect(() => assertDockerfileGuard(mutation(dockerfile))).toThrow();
    }
});
