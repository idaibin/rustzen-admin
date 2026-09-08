import { join, resolve } from "node:path";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { produceSelectedContract } from "../distribution/selected-contract.ts";
import { resolveSelection } from "../distribution/resolver.ts";
import { produceSchemaContract } from "../distribution/schema-contract.ts";
import { produceSelectedConfig } from "../distribution/selected-config.ts";
import { selectedCargoBuilds } from "../distribution/selected-cargo-producer.ts";

const root = resolve(import.meta.dir, "..");
const [flag, selectionPath] = Bun.argv.slice(2);
if (flag !== "--selection" || !selectionPath) {
    throw new Error(
        "usage: bun scripts/distribution-produce-contracts.ts --selection <selection.json>",
    );
}

const selection = await Bun.file(resolve(root, selectionPath)).json();
const plan = resolveSelection(selection);
const producerTarget = resolve(
    process.env.RUSTZEN_CONTRACT_TARGET_DIR ??
        join(root, "target/distribution-contract-producers", plan.compositionId),
);
const contractRoot = resolve(
    process.env.RUSTZEN_CONTRACT_OUTPUT_ROOT ??
        join(root, "target/distributions", plan.compositionId, "contracts"),
);

buildSelectedProducers();

const run = (kind: "admin" | "monitor" | "notifications") => {
    const binary = join(
        producerTarget,
        "debug",
        kind === "monitor" ? "rz-monitor" : "rz-admin",
    );
    const args =
        kind === "monitor"
            ? [binary, "contract", "selected"]
            : [binary, "contract", "selected", kind];
    const result = Bun.spawnSync(args, {
        cwd: "/tmp",
        env: { PATH: process.env.PATH ?? "" },
        stdout: "pipe",
        stderr: "pipe",
    });
    if (result.exitCode !== 0) {
        throw new Error(new TextDecoder().decode(result.stderr));
    }
    return JSON.parse(new TextDecoder().decode(result.stdout));
};
const runConfig = (
    kind: "admin" | "monitor" | "agent",
    owner: "access" | "monitor" | "monitor-agent" | "notifications",
) => {
    const name =
        kind === "admin"
            ? "rz-admin"
            : kind === "agent"
              ? "rz-monitor-agent"
              : "rz-monitor";
    const args = [join(producerTarget, "debug", name), "contract", "config", "selected"];
    if (kind === "admin") args.push(owner);
    const result = Bun.spawnSync(
        args,
        {
            cwd: "/tmp",
            env: { PATH: process.env.PATH ?? "" },
            stdout: "pipe",
            stderr: "pipe",
        },
    );
    if (result.exitCode !== 0)
        throw new Error(new TextDecoder().decode(result.stderr));
    return JSON.parse(new TextDecoder().decode(result.stdout));
};

const config = await produceSelectedConfig(
    selection,
    join(contractRoot, "config"),
    runConfig,
);
if (plan.artifactClass === "node-agent") {
    console.log(canonicalJson({ config }));
} else {
    const api = await produceSelectedContract(
        selection,
        join(contractRoot, "api"),
        run,
    );
    const schema = await produceSchemaContract(
        selection,
        root,
        join(contractRoot, "schema"),
    );
    console.log(canonicalJson({ api, config, schema }));
}

function buildSelectedProducers() {
    const builds = selectedCargoBuilds(plan);
    for (const command of builds) {
        const result = Bun.spawnSync(command, {
            cwd: root,
            env: { ...process.env, CARGO_TARGET_DIR: producerTarget },
            stdout: "pipe",
            stderr: "pipe",
        });
        if (result.exitCode !== 0) {
            throw new Error(new TextDecoder().decode(result.stderr));
        }
    }
}
