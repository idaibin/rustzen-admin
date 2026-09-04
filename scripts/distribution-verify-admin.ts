// Verify the direct normal/build dependency boundary of the minimal Admin host.
// Selected Web assets and native packages have separate acceptance gates.
import { resolve } from "node:path";

const forbidden = new Set([
    "croner",
    "ed25519-dalek",
    "hex",
    "image",
    "libc",

    "sysinfo",
    "tar",
]);
const required = new Set([
    "rustzen-admin",
    "rustzen-auth",
    "rustzen-config",
    "rustzen-ipc",
    "rustzen-storage",
]);
const args = [
    "cargo",
    "tree",
    "--locked",
    "-p",
    "rustzen-admin",
    "--no-default-features",
    "--features",
    "monitor-distribution",
    "--edges",
    "normal,build",
    "--depth",
    "1",
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
    const missing = [...required].filter((name) => !dependencies.includes(name));
    if (missing.length) throw new Error(`minimal Admin is missing required dependencies: ${missing.join(", ")}`);
    const violations = dependencies.filter((name) => forbidden.has(name));
    if (violations.length)
        throw new Error(`minimal Admin directly includes optional owner dependencies: ${violations.join(", ")}`);
    const featureArgs = args.map((argument, index) =>
        args[index - 1] === "--edges" ? "features" : argument,
    );
    const featureResult = Bun.spawnSync(featureArgs, { cwd: resolve(import.meta.dir, "..") });
    if (featureResult.exitCode !== 0)
        throw new Error(`cargo feature tree failed: ${new TextDecoder().decode(featureResult.stderr)}`);
    const enabledFeatures = new TextDecoder().decode(featureResult.stdout);
    if (enabledFeatures.includes('tower-http feature "fs"'))
        throw new Error("minimal Admin enables tower-http/fs for filesystem-backed full Web asset serving");
    console.log(
        JSON.stringify(
            {
                ok: true,
                binary: "rz-admin",
                composition: "monitor-distribution",
                scope: "direct normal/build dependencies",
                dependencies,
            },
            null,
            2,
        ),
    );
} catch (error) {
    console.error(
        JSON.stringify({ ok: false, error: error instanceof Error ? error.message : String(error) }),
    );
    process.exitCode = 1;
}
