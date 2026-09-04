import { join, resolve } from "node:path";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { produceNativeLayout } from "../distribution/native-layout.ts";
import { resolveSelection } from "../distribution/resolver.ts";

const root = resolve(import.meta.dir, "..");
const [flag, selectionPath] = Bun.argv.slice(2);
if (flag !== "--selection" || !selectionPath)
    throw new Error(
        "usage: bun scripts/distribution-produce-native-layout.ts --selection <selection.json>",
    );

const selection = await Bun.file(resolve(root, selectionPath)).json();
const plan = resolveSelection(selection);
const outputRoot = join(
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
