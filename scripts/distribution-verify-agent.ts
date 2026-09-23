// Verify the normal/build dependency closure of the host Agent target.
// Native packages and full server distributions have separate acceptance gates.
import { resolve } from "node:path";

const forbidden = /^(?:axum(?:-|$)|sqlx(?:-|$)|rustzen-(?:auth|ipc|storage)$)/;
const args = [
    "cargo",
    "tree",
    "--locked",
    "-p",
    "rustzen-monitor",
    "--no-default-features",
    "--features",
    "agent",
    "--edges",
    "normal,build",
    "--prefix",
    "none",
    "--format",
    "{p}",
];
try {
    const result = Bun.spawnSync(args, { cwd: resolve(import.meta.dir, "..") });
    if (result.exitCode !== 0)
        throw new Error(`cargo tree failed: ${new TextDecoder().decode(result.stderr)}`);
    const dependencies = [
        ...new Set(
            new TextDecoder()
                .decode(result.stdout)
                .trim()
                .split("\n")
                .map((line) => line.split(" ")[0]),
        ),
    ].sort();
    if (!dependencies.includes("rustzen-monitor")) throw new Error("missing Agent dependency root");
    const violations = dependencies.filter((name) => forbidden.test(name));
    if (violations.length)
        throw new Error(`Agent includes server dependencies: ${violations.join(", ")}`);
    console.log(
        JSON.stringify(
            {
                ok: true,
                binary: "rz-monitor-agent",
                scope: "host normal/build dependencies",
                dependencies,
            },
            null,
            2,
        ),
    );
} catch (error) {
    console.error(
        JSON.stringify({
            ok: false,
            error: error instanceof Error ? error.message : String(error),
        }),
    );
    process.exitCode = 1;
}
