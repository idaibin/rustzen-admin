export type DigestSource =
    | "resolved-selection"
    | "selected-web-files"
    | "binary-file";
export type Digest = { sha256: string; source: DigestSource; path?: string };
export type BinaryDigest = Digest & { path: string };
export type FileEntry = {
    path: string;
    type: "file";
    mode: "0644" | "0755";
    size: number;
    sha256: string;
};
export type ManifestBase = {
    manifestVersion: 1;
    releaseClass: "production" | "test";
    releaseVersion: string;
    target: string;
    artifactClass: "server" | "node-agent";
    preset: string;
    capabilities: string[];
    services: string[];
    compositionId: string;
    selectionDigest: Digest;
    buildId: string;
    sourceIdentity: string;
    apiDigest: string;
    configDigest: string;
    configOwners: string[];
    binaryDigests: BinaryDigest[];
    files: FileEntry[];
    agentProtocolContractId?: string;
};
export type ServerManifest = ManifestBase & {
    artifactClass: "server";
    schemaFingerprints: Record<string, string>;
    dataContractIds: Record<string, string>;
    webDigest: Digest;
};
export type AgentManifest = ManifestBase & {
    artifactClass: "node-agent";
    agentProtocolContractId: string;
};
export type ReleaseManifest = ServerManifest | AgentManifest;
export type BuildInputs = {
    releaseVersion: string;
    sourceIdentity: string;
    toolchain: string;
    selectedRoutes: string[];
    apiDigest: string;
    schemaDigest: string;
    configDigest: string;
    protocolId?: string;
};
export type ProduceInput = BuildInputs & {
    selection: unknown;
    artifactRoot: string;
    schemaFingerprints?: Record<string, string>;
    dataContractIds?: Record<string, string>;
    webRoot?: string;
};
