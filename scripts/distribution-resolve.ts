import { resolveSelection, SelectionError } from "../distribution/resolver.ts";

function usage(): never {
    throw new SelectionError(
        "usage: bun scripts/distribution-resolve.ts <validate|resolve|release-gate> --selection <selection.json>",
    );
}

try {
    const [command, ...args] = Bun.argv.slice(2);
    if (!command || args.length !== 2 || args[0] !== "--selection") usage();
    let selection: unknown;
    try {
        selection = await Bun.file(args[1]).json();
    } catch {
        throw new SelectionError(`cannot read selection JSON: ${args[1]}`);
    }
    const plan = resolveSelection(selection);
    if (command === "release-gate") {
        if (plan.releaseClass === "test")
            throw new SelectionError(
                "production release gate rejects test-only current-full-regression",
            );
        throw new SelectionError(
            `release gate closed: ${plan.producerReadiness.blockers.join(" ")}`,
        );
    }
    if (command !== "validate" && command !== "resolve") usage();
    const output =
        command === "validate"
            ? { ok: true, preset: plan.preset, compositionId: plan.compositionId }
            : plan;
    process.stdout.write(`${JSON.stringify(output, null, 2)}\n`);
} catch (error) {
    process.stderr.write(
        `${JSON.stringify({ ok: false, error: error instanceof Error ? error.message : String(error) })}\n`,
    );
    process.exitCode = 1;
}
