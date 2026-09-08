import { lstat, mkdir, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { readSingleArtifactFile } from "./release-manifest-artifacts.ts";
import { canonicalJson, sha256, validHash } from "./release-manifest-core.ts";
import { resolveSelection } from "./resolver.ts";
import {
    isExactSupportedPlan,
    type SourceBuildPlan,
} from "./source-build-plan.ts";

export type SelectedProtocol = {
    version: 1;
    artifactClass: "server" | "node-agent";
    compositionId: string;
    preset: "monitor" | "monitor-notify" | "node-agent";
    descriptor: string;
    digest: string;
};

/** Both Controller and Agent emit this descriptor for these exact closures. */
export const supportsSelectedProtocol = (plan: SourceBuildPlan): boolean =>
    isExactSupportedPlan(plan, ["monitor", "monitor-notify", "node-agent"]);

const goldenFile = await Bun.file(
    new URL("./fixtures/monitor-protocol.json", import.meta.url),
).text();
if (!goldenFile.endsWith("\n"))
    throw new Error("reviewed protocol descriptor must end with one newline");
const reviewedDescriptor = goldenFile.slice(0, -1);
JSON.parse(reviewedDescriptor);

export const reviewedProtocolOutput = () =>
    `${reviewedDescriptor}\n${sha256(reviewedDescriptor)}\n`;

export function completeSelectedProtocol(selection: unknown): SelectedProtocol {
    return { ...identity(selection), ...parsedGolden() };
}

export async function produceSelectedProtocol(
    selection: unknown,
    outputRoot: string,
    controllerOutput: string,
    agentOutput: string,
) {
    await rejectSymlink(outputRoot);
    const controller = parseCommandOutput(controllerOutput);
    const agent = parseCommandOutput(agentOutput);
    if (
        controller.descriptor !== agent.descriptor ||
        controller.digest !== agent.digest
    )
        throw new Error("Controller and Agent protocol outputs differ");
    const protocol = parseSelectedProtocol(
        { ...identity(selection), ...controller },
        selection,
    );
    await rm(outputRoot, { recursive: true, force: true });
    await mkdir(outputRoot, { recursive: true });
    await writeFile(
        join(outputRoot, "protocol.json"),
        canonicalJson(protocol),
        {
            mode: 0o644,
        },
    );
    return readSelectedProtocol(outputRoot, selection);
}

export async function readSelectedProtocol(root: string, selection: unknown) {
    const file = await readSingleArtifactFile(root, "protocol.json");
    return parseSelectedProtocolBytes(file.bytes, selection);
}

export function parseSelectedProtocolBytes(
    bytes: Uint8Array,
    selection: unknown,
) {
    const text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    let value: unknown;
    try {
        value = JSON.parse(text);
    } catch {
        throw new Error("selected protocol artifact is not JSON");
    }
    const protocol = parseSelectedProtocol(value, selection);
    if (text !== canonicalJson(protocol))
        throw new Error("selected protocol artifact is not canonical");
    return { protocol, sha256: sha256(bytes) };
}

export function parseSelectedProtocol(
    value: unknown,
    selection: unknown,
): SelectedProtocol {
    const expected = completeSelectedProtocol(selection);
    if (!value || typeof value !== "object" || Array.isArray(value))
        throw new Error("selected protocol artifact must be an object");
    if (canonicalJson(value) !== canonicalJson(expected))
        throw new Error(
            "selected protocol artifact differs from reviewed descriptor",
        );
    return expected;
}

function parseCommandOutput(output: string) {
    if (!output.endsWith("\n"))
        throw new Error("protocol command must end with one newline");
    const lines = output.slice(0, -1).split("\n");
    if (lines.length !== 2 || lines.some((line) => !line))
        throw new Error(
            "protocol command must emit exactly descriptor and digest",
        );
    try {
        JSON.parse(lines[0]);
    } catch {
        throw new Error("protocol command descriptor is not JSON");
    }
    const digest = validHash(lines[1]);
    if (digest !== sha256(lines[0]))
        throw new Error("protocol command digest differs from descriptor");
    if (output !== `${lines[0]}\n${digest}\n`)
        throw new Error("protocol command wire format is invalid");
    return { descriptor: lines[0], digest };
}

function parsedGolden() {
    return {
        descriptor: reviewedDescriptor,
        digest: sha256(reviewedDescriptor),
    };
}

function identity(selection: unknown) {
    const plan = resolveSelection(selection);
    if (
        !(
            (supportsSelectedProtocol(plan) &&
                ["monitor", "monitor-notify"].includes(plan.preset) &&
                plan.artifactClass === "server") ||
            (supportsSelectedProtocol(plan) && plan.preset === "node-agent" &&
                plan.artifactClass === "node-agent")
        )
    )
        throw new Error(
            "selected protocol supports only monitor server compositions or node-agent",
        );
    return {
        version: 1 as const,
        artifactClass: plan.artifactClass,
        compositionId: plan.compositionId,
        preset: plan.preset as "monitor" | "monitor-notify" | "node-agent",
    };
}

async function rejectSymlink(path: string) {
    try {
        if ((await lstat(path)).isSymbolicLink())
            throw new Error(
                "protocol artifact output root must not be symlink",
            );
    } catch (error: unknown) {
        if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
}
