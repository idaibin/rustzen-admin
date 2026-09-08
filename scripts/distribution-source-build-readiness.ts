import { resolve } from "node:path";

import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { auditSourceBuildReadiness } from "../distribution/source-build-readiness.ts";

const root = resolve(import.meta.dir, "..");
const args = Bun.argv.slice(2);
const selectionIndex = args.indexOf("--selection");
const allowed = new Set(["--selection", "--require-ready"]);

try {
    if (
        selectionIndex < 0 ||
        !args[selectionIndex + 1] ||
        args.some((arg, index) => index !== selectionIndex + 1 && !allowed.has(arg))
    )
        throw new Error(
            "usage: bun scripts/distribution-source-build-readiness.ts " +
                "--selection <selection.json> [--require-ready]",
        );
    const selection = await Bun.file(resolve(root, args[selectionIndex + 1])).json();
    const result = auditSourceBuildReadiness(selection);
    console.log(canonicalJson(result));
    if (args.includes("--require-ready") && !result.admissionReady) process.exitCode = 1;
} catch (error) {
    console.error(
        canonicalJson({ error: error instanceof Error ? error.message : String(error) }),
    );
    process.exitCode = 1;
}
