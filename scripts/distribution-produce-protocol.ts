import { join, resolve } from "node:path";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { produceSelectedProtocol } from "../distribution/selected-protocol.ts";
import { resolveSelection } from "../distribution/resolver.ts";

const root = resolve(import.meta.dir, "..");
const [flag, selectionPath] = Bun.argv.slice(2);
if (flag !== "--selection" || !selectionPath)
    throw new Error(
        "usage: bun scripts/distribution-produce-protocol.ts --selection <selection.json>",
    );

const run = (name: string) => {
    const result = Bun.spawnSync(
        [join(root, "target/debug", name), "contract", "protocol"],
        {
            cwd: "/tmp",
            env: { PATH: process.env.PATH ?? "" },
            stdout: "pipe",
            stderr: "pipe",
        },
    );
    if (result.exitCode !== 0)
        throw new Error(new TextDecoder().decode(result.stderr));
    return new TextDecoder().decode(result.stdout);
};

const selection = await Bun.file(resolve(root, selectionPath)).json();
const plan = resolveSelection(selection);
const rootPath = join(
    root,
    "target/distributions",
    plan.compositionId,
    "contracts/protocol",
);
const produced = await produceSelectedProtocol(
    selection,
    rootPath,
    run("rz-monitor"),
    run("rz-monitor-agent"),
);
console.log(
    canonicalJson({
        artifactClass: plan.artifactClass,
        compositionId: plan.compositionId,
        path: join(rootPath, "protocol.json"),
        sha256: produced.sha256,
    }),
);
