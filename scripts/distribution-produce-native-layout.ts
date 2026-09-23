import { join, resolve } from "node:path";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { produceNativeLayout } from "../distribution/native-layout.ts";
import { resolveSelection } from "../distribution/resolver.ts";

const root = resolve(import.meta.dir, "..");
const args = values(Bun.argv.slice(2), new Set(["--selection", "--output-root"]));
const selectionPath = args.get("--selection");
if (!selectionPath || ![1, 2].includes(args.size))
    throw new Error(
        "usage: --selection <selection.json> [--output-root <output-root>]",
    );

const selection = await Bun.file(resolve(root, selectionPath)).json();
const plan = resolveSelection(selection);
const outputRoot = args.get("--output-root") ?? join(
    root,
    "target/distributions",
    plan.compositionId,
    "contracts/native",
);
const produced = await produceNativeLayout(selection, outputRoot);
console.log(
    canonicalJson({
        artifactClass: produced.layout.artifactClass,
        compositionId: produced.layout.compositionId,
        path: join(outputRoot, "native-layout.json"),
        sha256: produced.sha256,
    }),
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
