import catalogJson from "./catalog.json";

type ArtifactClass = "server" | "node-agent";
type ReleaseClass = "production" | "test";
type Target = { package: string; binary: string; status: "blocked"; reason: string };
type Capability = {
    id: string;
    artifactClass: ArtifactClass;
    dependsOn: string[];
    services: string[];
    packageTargets: Target[];
    webRoots: string[];
    schemaOwners: string[];
    schemaOwnerIntersections?: { owner: string; capabilities: string[] }[];
    configOwners: string[];
    units: string[];
};
export type Catalog = {
    schemaVersion: number;
    capabilities: Capability[];
    presets: Record<
        string,
        { artifactClass: ArtifactClass; releaseClass: ReleaseClass; capabilities: string[] }
    >;
};
type Selection = {
    schemaVersion?: number;
    preset: string;
    capabilities?: string[];
    artifactClass?: ArtifactClass;
    releaseClass?: ReleaseClass;
    sourceVersion?: string;
    target?: string;
};

const CATALOG = catalogJson as Catalog;
const SELECTION_KEYS = new Set([
    "schemaVersion",
    "preset",
    "capabilities",
    "artifactClass",
    "releaseClass",
    "sourceVersion",
    "target",
]);
const CATALOG_KEYS = new Set(["schemaVersion", "capabilities", "presets"]);
const CAPABILITY_KEYS = new Set([
    "id",
    "artifactClass",
    "dependsOn",
    "services",
    "packageTargets",
    "webRoots",
    "schemaOwners",
    "schemaOwnerIntersections",
    "configOwners",
    "units",
]);
const TARGET_KEYS = new Set(["package", "binary", "status", "reason"]);
const PRESET_KEYS = new Set(["artifactClass", "releaseClass", "capabilities"]);
// Selection support follows the repository's current native build targets.
const SUPPORTED_TARGETS = new Set(["x86_64-unknown-linux-musl", "aarch64-unknown-linux-gnu"]);
export class SelectionError extends Error {}
const fail = (message: string): never => {
    throw new SelectionError(message);
};
const sorted = (values: Iterable<string>): string[] =>
    [...new Set(values)].sort((a, b) => a.localeCompare(b));
function canonicalJson(value: unknown): string {
    if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
    if (value && typeof value === "object") {
        const record = value as Record<string, unknown>;
        return `{${Object.keys(record)
            .sort()
            .map((key) => `${JSON.stringify(key)}:${canonicalJson(record[key])}`)
            .join(",")}}`;
    }
    return JSON.stringify(value);
}
const sha256 = (value: string): string =>
    new Bun.CryptoHasher("sha256").update(value).digest("hex");
function assertRecord(value: unknown, label: string): asserts value is Record<string, unknown> {
    if (!value || typeof value !== "object" || Array.isArray(value))
        fail(`${label} must be an object`);
}
function assertOnlyKeys(
    record: Record<string, unknown>,
    allowed: Set<string>,
    label: string,
): void {
    for (const key of Object.keys(record))
        if (!allowed.has(key)) fail(`unknown ${label} field: ${key}`);
}
function assertStringArray(value: unknown, label: string): asserts value is string[] {
    if (!Array.isArray(value) || value.some((item) => typeof item !== "string"))
        fail(`${label} must be an array of strings`);
}

export function validateCatalog(catalog: Catalog = CATALOG): void {
    assertRecord(catalog, "catalog");
    assertOnlyKeys(catalog, CATALOG_KEYS, "catalog");
    if (catalog.schemaVersion !== 1 || !Array.isArray(catalog.capabilities))
        fail("catalog must use schemaVersion 1");
    assertRecord(catalog.presets, "catalog presets");
    const byId = new Map<string, Capability>();
    for (const capability of catalog.capabilities) {
        assertRecord(capability, "catalog capability");
        assertOnlyKeys(capability, CAPABILITY_KEYS, "catalog capability");
        if (!capability || typeof capability.id !== "string" || !capability.id)
            fail("catalog capability id is required");
        if (byId.has(capability.id)) fail(`duplicate catalog capability: ${capability.id}`);
        if (capability.artifactClass !== "server" && capability.artifactClass !== "node-agent")
            fail(`invalid artifact class for ${capability.id}`);
        assertStringArray(capability.dependsOn, `catalog capability ${capability.id} dependencies`);
        assertStringArray(capability.services, `catalog capability ${capability.id} services`);
        assertStringArray(capability.webRoots, `catalog capability ${capability.id} webRoots`);
        assertStringArray(
            capability.schemaOwners,
            `catalog capability ${capability.id} schemaOwners`,
        );
        if (capability.schemaOwnerIntersections !== undefined) {
            if (!Array.isArray(capability.schemaOwnerIntersections))
                fail(`catalog capability ${capability.id} schemaOwnerIntersections must be an array`);
            for (const condition of capability.schemaOwnerIntersections) {
                assertRecord(condition, `catalog capability ${capability.id} schema intersection`);
                assertOnlyKeys(
                    condition,
                    new Set(["owner", "capabilities"]),
                    `catalog capability ${capability.id} schema intersection`,
                );
                if (typeof condition.owner !== "string" || !condition.owner)
                    fail(`catalog capability ${capability.id} schema intersection owner is required`);
                assertStringArray(
                    condition.capabilities,
                    `catalog capability ${capability.id} schema intersection capabilities`,
                );
            }
        }
        assertStringArray(
            capability.configOwners,
            `catalog capability ${capability.id} configOwners`,
        );
        assertStringArray(capability.units, `catalog capability ${capability.id} units`);
        if (!Array.isArray(capability.packageTargets))
            fail(`catalog capability ${capability.id} packageTargets must be an array`);
        for (const target of capability.packageTargets) {
            assertRecord(target, `catalog capability ${capability.id} package target`);
            assertOnlyKeys(target, TARGET_KEYS, "catalog package target");
            if (
                typeof target.package !== "string" ||
                typeof target.binary !== "string" ||
                target.status !== "blocked" ||
                typeof target.reason !== "string"
            )
                fail(`invalid catalog package target for ${capability.id}`);
        }
        byId.set(capability.id, capability);
    }
    for (const capability of catalog.capabilities) {
        for (const condition of capability.schemaOwnerIntersections ?? []) {
            if (condition.capabilities.length === 0)
                fail(
                    `catalog capability ${capability.id} schema intersection capabilities must not be empty`,
                );
            if (new Set(condition.capabilities).size !== condition.capabilities.length)
                fail(
                    `catalog capability ${capability.id} schema intersection capabilities must be unique`,
                );
            for (const selectedId of condition.capabilities) {
                const selected = byId.get(selectedId);
                if (!selected)
                    fail(
                        `catalog capability ${capability.id} schema intersection references unknown capability: ${selectedId}`,
                    );
                if (selected.artifactClass !== capability.artifactClass)
                    fail(
                        `catalog capability ${capability.id} schema intersection mixes artifact classes`,
                    );
            }
        }
    }
    const visiting = new Set<string>();
    const visited = new Set<string>();
    const visit = (id: string): void => {
        const capability = byId.get(id);
        if (!capability) fail(`unknown catalog capability: ${id}`);
        if (visiting.has(id))
            fail(`capability dependency cycle: ${[...visiting, id].join(" -> ")}`);
        if (visited.has(id)) return;
        visiting.add(id);
        for (const dependency of capability.dependsOn) visit(dependency);
        visiting.delete(id);
        visited.add(id);
    };
    for (const id of byId.keys()) visit(id);
    for (const [name, preset] of Object.entries(catalog.presets)) {
        assertRecord(preset, `preset ${name}`);
        assertOnlyKeys(preset, PRESET_KEYS, `preset ${name}`);
        assertStringArray(preset.capabilities, `preset ${name} capabilities`);
        if (
            (preset.artifactClass !== "server" && preset.artifactClass !== "node-agent") ||
            (preset.releaseClass !== "production" && preset.releaseClass !== "test")
        )
            fail(`preset ${name} has invalid class`);
        for (const id of preset.capabilities) {
            const capability = byId.get(id);
            if (!capability) fail(`preset ${name} selects unknown capability: ${id}`);
            if (capability.artifactClass !== preset.artifactClass)
                fail(`preset ${name} mixes artifact classes`);
        }
    }
    const full = sorted(catalog.presets.full?.capabilities ?? []);
    const allServerCapabilities = sorted(
        catalog.capabilities
            .filter((capability) => capability.artifactClass === "server")
            .map((capability) => capability.id),
    );
    if (canonicalJson(full) !== canonicalJson(allServerCapabilities))
        fail("full must enumerate every server capability exactly");
}

function parseSelection(value: unknown): Selection {
    if (!value || typeof value !== "object" || Array.isArray(value))
        fail("selection must be an object");
    const record = value as Record<string, unknown>;
    for (const key of Object.keys(record))
        if (!SELECTION_KEYS.has(key)) fail(`unknown selection field: ${key}`);
    if (record.schemaVersion !== undefined && record.schemaVersion !== 1)
        fail("selection schemaVersion must be 1");
    if (typeof record.preset !== "string" || !record.preset) fail("selection preset is required");
    if (
        record.capabilities !== undefined &&
        (!Array.isArray(record.capabilities) ||
            record.capabilities.some((id) => typeof id !== "string"))
    )
        fail("selection capabilities must be an array of strings");
    if (
        record.artifactClass !== undefined &&
        record.artifactClass !== "server" &&
        record.artifactClass !== "node-agent"
    )
        fail("selection artifactClass is invalid");
    if (
        record.releaseClass !== undefined &&
        record.releaseClass !== "production" &&
        record.releaseClass !== "test"
    )
        fail("selection releaseClass is invalid");
    if (record.sourceVersion !== undefined && typeof record.sourceVersion !== "string")
        fail("selection sourceVersion must be a string");
    if (
        record.target !== undefined &&
        (typeof record.target !== "string" || !SUPPORTED_TARGETS.has(record.target))
    )
        fail("selection target must be a supported native target");
    return record as Selection;
}

export function resolveSelection(value: unknown, catalog: Catalog = CATALOG) {
    validateCatalog(catalog);
    const selection = parseSelection(value);
    const byId = new Map(catalog.capabilities.map((capability) => [capability.id, capability]));
    const isCustom = selection.preset === "custom";
    const preset = catalog.presets[selection.preset];
    if (!isCustom && !preset) fail(`unknown preset: ${selection.preset}`);
    if (!isCustom && selection.capabilities !== undefined)
        fail("named presets do not accept explicit capabilities");
    if (isCustom && (!selection.capabilities || selection.capabilities.length === 0))
        fail("custom requires explicit capabilities");
    const requested = isCustom
        ? (selection.capabilities ?? fail("custom requires explicit capabilities"))
        : (preset?.capabilities ?? fail(`unknown preset: ${selection.preset}`));
    if (new Set(requested).size !== requested.length)
        fail("selection capabilities must not repeat");
    const artifactClass =
        selection.artifactClass ??
        (isCustom ? byId.get(requested[0])?.artifactClass : preset?.artifactClass);
    if (!artifactClass) fail("selection artifactClass cannot be determined");
    const releaseClass = selection.releaseClass ?? (isCustom ? "production" : preset?.releaseClass);
    const target = selection.target ?? "x86_64-unknown-linux-musl";
    if (
        !isCustom &&
        (artifactClass !== preset.artifactClass || releaseClass !== preset.releaseClass)
    )
        fail(`preset ${selection.preset} has fixed artifactClass and releaseClass`);
    if (selection.preset === "current-full-regression" && releaseClass !== "test")
        fail("current-full-regression is test-only");
    if (releaseClass === "test" && selection.preset !== "current-full-regression")
        fail("test releaseClass is reserved for current-full-regression");
    const closure = new Set<string>();
    const visit = (id: string): void => {
        const capability = byId.get(id);
        if (!capability) fail(`unknown capability: ${id}`);
        if (capability.artifactClass !== artifactClass)
            fail(`capability ${id} is not valid for ${artifactClass}`);
        if (closure.has(id)) return;
        closure.add(id);
        for (const dependency of capability.dependsOn) visit(dependency);
    };
    for (const id of requested) visit(id);
    const capabilities = sorted(closure);
    const full = sorted(catalog.presets.full.capabilities);
    if (selection.preset === "full" && canonicalJson(capabilities) !== canonicalJson(full))
        fail("full must resolve to the exact declared full capability set");
    if (
        artifactClass === "node-agent" &&
        canonicalJson(capabilities) !== canonicalJson(["monitor-agent"])
    )
        fail("node-agent must resolve exactly monitor-agent");
    const selected = capabilities.map((id) => byId.get(id) ?? fail(`unknown capability: ${id}`));
    const groupedTargets = new Map<string, Target & { capabilities: string[] }>();
    for (const capability of selected)
        for (const target of capability.packageTargets) {
            const key = `${target.package}/${target.binary}`;
            const grouped = groupedTargets.get(key) ?? { ...target, capabilities: [] };
            grouped.capabilities.push(capability.id);
            groupedTargets.set(key, grouped);
        }
    const packageTargets = [...groupedTargets.values()]
        .map((target) => ({ ...target, capabilities: sorted(target.capabilities) }))
        .sort((a, b) => `${a.package}/${a.binary}`.localeCompare(`${b.package}/${b.binary}`));
    const blockers = sorted(
        selected.flatMap((capability) => capability.packageTargets.map((target) => target.reason)),
    );
    return {
        schemaVersion: 1,
        preset: selection.preset,
        releaseClass,
        artifactClass,
        target,
        capabilities,
        compositionId: sha256(
            canonicalJson({
                artifactClass,
                capabilities,
                capabilityContractVersion: catalog.schemaVersion,
            }),
        ),
        services: sorted(selected.flatMap((capability) => capability.services)),
        owners: sorted(selected.map((capability) => capability.id)),
        packageTargets,
        webRoots: sorted(selected.flatMap((capability) => capability.webRoots)),
        schemaOwners: sorted([
            ...selected.flatMap((capability) => capability.schemaOwners),
            ...selected.flatMap((capability) =>
                (capability.schemaOwnerIntersections ?? [])
                    .filter((condition) =>
                        condition.capabilities.every((id) => capabilities.includes(id)),
                    )
                    .map((condition) => condition.owner),
            ),
        ]),
        configOwners: sorted(selected.flatMap((capability) => capability.configOwners)),
        units: sorted(selected.flatMap((capability) => capability.units)),
        producerReadiness: { ready: false, blockers },
        notes: [
            "Route and permission inventories are code-derived from Rust registration.",
            "Resolution is a selection plan, not evidence of physical pruning or a build command.",
        ],
    };
}
export const distributionCatalog = CATALOG;
