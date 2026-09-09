import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { resolve } from "node:path";
import { json } from "./monitor-load-admission.ts";
const keys = ["adminBin", "adminUrl", "agentTokenFile", "browserReceipt", "certificate", "containerId", "containerName", "containerPort", "expectedSourceIdentity", "exportRoot", "hostPort", "imageId", "nativeEvidence", "ownerToken", "passwordFile", "publicKey", "releaseResult"];
export async function preparedContext(path: string) {
    const value = await json(path);
    if (canonicalJson(Object.keys(value).sort()) !== canonicalJson(keys) || value.containerPort !== 19801 || value.adminUrl !== `http://127.0.0.1:${value.hostPort}` || !Number.isInteger(value.hostPort) || !["adminBin", "agentTokenFile", "browserReceipt", "certificate", "containerId", "containerName", "expectedSourceIdentity", "exportRoot", "imageId", "nativeEvidence", "ownerToken", "passwordFile", "publicKey", "releaseResult"].every(key => typeof value[key] === "string") || !/^[a-f0-9-]{32,64}$/.test(String(value.ownerToken))) throw Error("prepared context differs");
    if (value.passwordFile !== `${resolve(path)}.password` || value.agentTokenFile !== `${resolve(path)}.agent-token`) throw Error("prepared credential path differs");
    return value;
}
if (import.meta.main) { const path = Bun.argv[2]; if (!path || Bun.argv.length !== 3) throw Error("usage: <context>"); console.log(canonicalJson(await preparedContext(path))); }
