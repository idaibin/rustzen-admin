import { expect, test } from "bun:test";
import { rm } from "node:fs/promises";

const gateLibPath = new URL("./monitoring-ui-state-gate-lib.sh", import.meta.url).pathname;
const runValidator = (provenancePath) =>
    Bun.spawnSync(
        [
            "bash",
            "-c",
            '. "$1"; verify_build_provenance "$2" "$3" "$4" "$5" "$6" "$7" "$8"',
            "provenance-validator",
            gateLibPath,
            provenancePath,
            "0123456789abcdef",
            "dirty",
            "abcdef0123456789",
            "aarch64",
            "aarch64-unknown-linux-musl",
            "linux/arm64",
        ],
        { stdout: "pipe", stderr: "pipe" },
    );

test("POSIX awk provenance validator accepts exact fields and rejects drift", async () => {
    const root = "/tmp/rz-monitoring-provenance-" + crypto.randomUUID();
    const provenancePath = root + "/build-provenance.txt";
    const valid = [
        "schemaVersion\t1",
        "gitHead\t0123456789abcdef",
        "sourceTreeState\tdirty",
        "sourceTreeSha256\tabcdef0123456789",
        "architecture\taarch64",
        "targetTriple\taarch64-unknown-linux-musl",
        "platform\tlinux/arm64",
        "distribution\tfull",
    ];
    try {
        await Bun.write(provenancePath, valid.join("\n") + "\n");
        expect(runValidator(provenancePath).exitCode).toBe(0);

        await Bun.write(provenancePath, valid.slice(0, -1).join("\n") + "\n");
        expect(runValidator(provenancePath).exitCode).not.toBe(0);

        const wrongHash = valid.map((line) =>
            line.startsWith("sourceTreeSha256\t") ? "sourceTreeSha256\twrong" : line,
        );
        await Bun.write(provenancePath, wrongHash.join("\n") + "\n");
        expect(runValidator(provenancePath).exitCode).not.toBe(0);
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});
