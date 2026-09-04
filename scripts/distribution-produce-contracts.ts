import { join, resolve } from "node:path";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { produceSelectedContract } from "../distribution/selected-contract.ts";
import { resolveSelection } from "../distribution/resolver.ts";
import { produceSchemaContract } from "../distribution/schema-contract.ts";

const root = resolve(import.meta.dir, "..");
const [flag, selectionPath] = Bun.argv.slice(2);
if (flag !== "--selection" || !selectionPath) {
    throw new Error(
        "usage: bun scripts/distribution-produce-contracts.ts --selection <selection.json>",
    );
}

const run = (kind: "admin" | "monitor") => {
    const binary = join(
        root,
        "target/debug",
        kind === "admin" ? "rz-admin" : "rz-monitor",
    );
    const result = Bun.spawnSync([binary, "contract", "selected"], {
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

const selection = await Bun.file(resolve(root, selectionPath)).json();
const composition = resolveSelection(selection).compositionId;
const contractRoot = join(
    root,
    "target/distributions",
    composition,
    "contracts",
);
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
console.log(canonicalJson({ api, schema }));
