import { supportsNativeLayout } from "./native-layout-source.ts";
import { distributionCatalog, resolveSelection, type Catalog } from "./resolver.ts";
import { supportsSelectedApiContract } from "./selected-contract-validator.ts";
import { supportsSelectedCargo } from "./selected-cargo-producer.ts";
import { supportsSelectedConfig } from "./selected-config-validator.ts";
import { supportsSelectedProtocol } from "./selected-protocol.ts";
import { supportsSelectedSchema } from "./schema-contract.ts";
import { supportsSelectedWeb } from "./selected-web-producer.ts";
import type { SourceBuildPlan } from "./source-build-plan.ts";

export type ProducerFamily =
    | "cargo"
    | "web"
    | "api"
    | "schema"
    | "config"
    | "native-layout"
    | "protocol";

const SERVER_REQUIRED: ProducerFamily[] = [
    "cargo",
    "web",
    "api",
    "schema",
    "config",
    "native-layout",
    "protocol",
];
const AGENT_REQUIRED: ProducerFamily[] = [
    "cargo",
    "config",
    "native-layout",
    "protocol",
];
const supports: Record<ProducerFamily, (plan: SourceBuildPlan) => boolean> = {
    cargo: supportsSelectedCargo,
    web: supportsSelectedWeb,
    api: supportsSelectedApiContract,
    schema: supportsSelectedSchema,
    config: supportsSelectedConfig,
    "native-layout": supportsNativeLayout,
    protocol: supportsSelectedProtocol,
};

export function assertCompleteReadinessInventory(
    catalog: Catalog = distributionCatalog,
): void {
    for (const preset of Object.keys(catalog.presets)) resolveSelection({ preset }, catalog);
}

export function auditSourceBuildReadiness(
    selection: unknown,
    catalog: Catalog = distributionCatalog,
) {
    assertCompleteReadinessInventory(catalog);
    const plan = resolveSelection(selection, catalog);
    const required = plan.artifactClass === "node-agent" ? AGENT_REQUIRED : SERVER_REQUIRED;
    const available = required.filter((producer) => supports[producer](plan));
    const missing = required.filter((producer) => !available.includes(producer));
    const productionEligible = plan.releaseClass === "production" && plan.preset !== "custom";
    const blockers = [
        ...missing.map((producer) => `${producer} producer does not support this exact closure`),
        ...(plan.preset === "custom"
            ? ["custom selections require an explicit composition-specific assessment"]
            : []),
        ...(plan.releaseClass === "test"
            ? ["test-only regression selections are never production-certification inputs"]
            : []),
    ];
    return {
        schemaVersion: 1 as const,
        kind: "source-build-certification-admission" as const,
        preset: plan.preset,
        compositionId: plan.compositionId,
        artifactClass: plan.artifactClass,
        releaseClass: plan.releaseClass,
        target: plan.target,
        capabilities: plan.capabilities,
        requiredProducers: required,
        availableProducers: available,
        missingProducers: missing,
        blockers,
        productionEligible,
        admissionReady: productionEligible && missing.length === 0,
        certified: false as const,
    };
}
