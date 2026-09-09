import { admitted, json, revalidatedNativeEvidence } from "./monitor-load-admission.ts";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { preparedContext } from "./prepared-monitor-load-context.ts";
import { certifiedOwner, listener, service } from "./monitor-load-runtime.ts";
import { inspected } from "./verify-selected-web-runtime-attestation.ts";

const path = Bun.argv[2];
if (!path || Bun.argv.length !== 3) throw Error("usage: <prepared context>");
const context = await preparedContext(path);
const native = await revalidatedNativeEvidence(String(context.nativeEvidence), String(context.releaseResult)) as unknown as Record<string, unknown>, browser = await json(String(context.browserReceipt)), admission = admitted(native, browser), run = async (args: string[]) => {
    const result = Bun.spawnSync(["docker", ...args], { stdout: "pipe", stderr: "pipe" });
    if (result.exitCode) throw Error(new TextDecoder().decode(result.stderr));
    return new TextDecoder().decode(result.stdout).trim();
}, raw = JSON.parse(await run(["inspect", String(context.containerName)])), mapped = inspected(raw, "127.0.0.1", Number(context.hostPort));
if (context.expectedSourceIdentity !== admission.source.expected) throw Error("prepared source differs");
if (mapped.containerId !== context.containerId || mapped.imageId !== context.imageId || mapped.containerPort !== 19801 || await run(["inspect", "--format", "{{index .Config.Labels \"io.rustzen.p8g-owner\"}}", String(context.containerName)]) !== context.ownerToken || await run(["inspect", "--format", "{{.HostConfig.NanoCpus}}", String(context.containerName)]) !== "4000000000" || await run(["inspect", "--format", "{{.HostConfig.Memory}}", String(context.containerName)]) !== String(512 * 1024 * 1024) || await run(["inspect", "--format", "{{.HostConfig.PidsLimit}}", String(context.containerName)]) !== "256") throw Error("prepared runtime caps differ");
const binaries = new Map(((native.release as Record<string, unknown>).binaryDigests as Array<Record<string, unknown>>).map(value => [value.path, value.sha256])), admin = await service(run, String(context.containerName), "rz-admin.service", "/opt/rz/current/bin/rz-admin"), monitor = await service(run, String(context.containerName), "rz-monitor.service", "/opt/rz/current/bin/rz-monitor"), owner = await listener(run, String(context.containerName), 19801, admin.sha256), browserRuntime = admission.runtime as Record<string, unknown>;
certifiedOwner(admin, binaries.get("bin/rz-admin")); certifiedOwner(monitor, binaries.get("bin/rz-monitor"));
if (!["containerId", "imageId", "hostPort", "containerPort"].every(key => browserRuntime[key] === context[key]) || !["pid", "dev", "ino", "sha256"].every(key => browserRuntime[key] === admin[key] && owner[key] === admin[key])) throw Error("prepared Admin tuple differs");
console.log(canonicalJson({containerId: context.containerId, hostPort: context.hostPort, nativeEvidence: context.nativeEvidence, status: "passed"}));
