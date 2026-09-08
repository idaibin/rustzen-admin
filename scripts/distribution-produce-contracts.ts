import { join, resolve } from "node:path";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { produceSelectedContract } from "../distribution/selected-contract.ts";
import { resolveSelection } from "../distribution/resolver.ts";
import { produceSchemaContract } from "../distribution/schema-contract.ts";
import { produceSelectedConfig } from "../distribution/selected-config.ts";

const root = resolve(import.meta.dir, "..");
const args = Bun.argv.slice(2);
const parsed = parseArgs(args, ["--selection", "--binary-root"]);
const selectionPath = parsed.get("--selection");
const binaryRootValue = parsed.get("--binary-root");
if (!selectionPath || !binaryRootValue) {
    throw new Error(
        "usage: bun scripts/distribution-produce-contracts.ts " +
            "--selection <selection.json> --binary-root <binary-root>",
    );
}

const selection = await Bun.file(resolve(root, selectionPath)).json();
const plan = resolveSelection(selection);
const producerRoot = resolve(root, binaryRootValue);
const contractRoot = resolve(
    process.env.RUSTZEN_CONTRACT_OUTPUT_ROOT ??
        join(root, "target/distributions", plan.compositionId, "contracts"),
);

const run = (kind: "admin" | "monitor" | "notifications") => {
    const binary = join(
        producerRoot,
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
    const args = [join(producerRoot, name), "contract", "config", "selected"];
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

function parseArgs(values: string[], allowed: string[]): Map<string, string> {
    if (values.length !== allowed.length * 2) return new Map();
    const result = new Map<string, string>();
    for (let index = 0; index < values.length; index += 2) {
        const name = values[index];
        const value = values[index + 1];
        if (!allowed.includes(name) || result.has(name) || !value?.length)
            return new Map();
        result.set(name, value);
    }
    return result;
}
