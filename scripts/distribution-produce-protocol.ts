import { join, resolve } from "node:path";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import {
    produceSelectedProtocol,
    supportsSelectedProtocol,
} from "../distribution/selected-protocol.ts";
import { resolveSelection } from "../distribution/resolver.ts";

const root = resolve(import.meta.dir, "..");
const args = values(
    Bun.argv.slice(2),
    new Set(["--selection", "--binary-root", "--output-root"]),
);
const selectionPath = args.get("--selection");
const binaryRootInput = args.get("--binary-root");
const outputRootInput = args.get("--output-root");
if (!selectionPath || !binaryRootInput || !outputRootInput || args.size !== 3)
    throw new Error(
        "usage: --selection <selection.json> --binary-root <release-binary-root> --output-root <output-root>",
    );
const binaryRoot = resolve(root, binaryRootInput);
const outputRoot = resolve(root, outputRootInput);

const run = (name: string) => {
    const result = Bun.spawnSync(
        [join(binaryRoot, name), "contract", "protocol"],
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
if (!supportsSelectedProtocol(plan))
    throw new Error("selected protocol production is unavailable for this selection");
const produced = await produceSelectedProtocol(
    selection,
    outputRoot,
    ...(plan.preset === "analytics"
        ? [run("rz-admin"), run("rz-insights")]
        : [run("rz-monitor"), run("rz-monitor-agent")]),
);

function values(input: string[], allowed: Set<string>) {
    if (input.length % 2 !== 0) return new Map<string, string>();
    const result = new Map<string, string>();
    for (let index = 0; index < input.length; index += 2) {
        const name = input[index], value = input[index + 1];
        if (!name || !value || !allowed.has(name) || result.has(name))
            return new Map<string, string>();
        result.set(name, value);
    }
    return result;
}
console.log(
    canonicalJson({
        artifactClass: plan.artifactClass,
        compositionId: plan.compositionId,
        path: join(outputRoot, "protocol.json"),
        sha256: produced.sha256,
    }),
);
