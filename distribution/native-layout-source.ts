import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import { distributionCatalog, resolveSelection } from "./resolver.ts";
import { completeSelectedConfigForTest } from "./selected-config-validator.ts";

export type NativeUnit = { path: string; sha256: string };
export type NativeConfig = {
    path: string;
    owner: string;
    consumer: string;
    keys: string[];
};
export type NativeLayout = {
    version: 1;
    artifactClass: "server" | "node-agent";
    compositionId: string;
    preset: "monitor" | "node-agent";
    configOwners: string[];
    units: NativeUnit[];
    configs: NativeConfig[];
};

const unit = (path: string, text: string): NativeUnit => ({
    path,
    sha256: sha256(text),
});
const service = (
    description: string,
    identity: string,
    config: string,
    command: string,
    target: boolean,
    agent = false,
) =>
    `[Unit]\nDescription=${description}\nWants=network-online.target\nAfter=network-online.target${target ? "\nPartOf=rz.target" : ""}\nStartLimitIntervalSec=60\nStartLimitBurst=5\n\n[Service]\nType=${agent ? "notify" : "simple"}\nUser=${identity}\nGroup=${identity}\nUMask=0027${agent ? "\nNotifyAccess=main\nTimeoutStartSec=infinity" : ""}\nEnvironmentFile=/opt/rz/config/${config}${target ? "\nEnvironmentFile=/opt/rz/config/rz-release.env" : ""}${agent ? "\nStateDirectory=rustzen-monitor-agent\nLogsDirectory=rustzen-monitor-agent\nEnvironment=RUSTZEN_RUNTIME_ROOT=/var/lib/rustzen-monitor-agent" : ""}\nExecStart=/opt/rz/current/bin/${command}\nWorkingDirectory=/opt/rz\nRestart=on-failure\nRestartSec=5\n\n[Install]\nWantedBy=${target ? "rz.target" : "multi-user.target"}\n`;

type Descriptor = { consumer: string; fields: Array<{ key: string }> };
const config = (selection: unknown, owner: string): NativeConfig => {
    const owners = completeSelectedConfigForTest(selection).owners as Record<
        string,
        Descriptor
    >;
    const descriptor = owners[owner];
    if (
        !descriptor ||
        !descriptor.consumer ||
        !Array.isArray(descriptor.fields)
    )
        throw new Error(
            `native layout missing selected config descriptor: ${owner}`,
        );
    return {
        path: `config/${descriptor.consumer}.env`,
        owner,
        consumer: descriptor.consumer,
        keys: descriptor.fields.map((field) => field.key),
    };
};

export function nativeUnitBytes(
    selection: unknown,
    catalog: typeof distributionCatalog = distributionCatalog,
): Record<string, string> {
    const plan = resolveSelection(selection, catalog);
    if (plan.preset === "monitor" && plan.artifactClass === "server")
        return {
            "systemd/rz-admin.service": service(
                "Rustzen Admin",
                "rz-admin",
                "rz-admin.env",
                "rz-admin serve",
                true,
            ),
            "systemd/rz-monitor.service": service(
                "Rustzen Monitor Controller",
                "rz-monitor",
                "rz-monitor.env",
                "rz-monitor controller",
                true,
            ),
            "systemd/rz.target":
                "[Unit]\nDescription=Rustzen Monitor Services\nWants=rz-admin.service rz-monitor.service\nAfter=network.target\n\n[Install]\nWantedBy=multi-user.target\n",
        };
    if (plan.preset === "node-agent" && plan.artifactClass === "node-agent")
        return {
            "systemd/rz-monitor-agent.service": service(
                "Rustzen Monitor Agent",
                "rz-monitor-agent",
                "rz-monitor-agent.env",
                "rz-monitor-agent",
                false,
                true,
            ),
        };
    throw new Error("native layout supports only monitor server or node-agent");
}

export function generatedNativeLayout(
    selection: unknown,
    catalog: typeof distributionCatalog = distributionCatalog,
): NativeLayout {
    const plan = resolveSelection(selection, catalog);
    const server = plan.preset === "monitor" && plan.artifactClass === "server";
    const agent =
        plan.preset === "node-agent" && plan.artifactClass === "node-agent";
    if (!server && !agent)
        throw new Error(
            "native layout supports only monitor server or node-agent",
        );
    const owners = plan.configOwners;
    const expectedOwners = server ? ["access", "monitor"] : ["monitor-agent"];
    if (canonicalJson(owners) !== canonicalJson(expectedOwners))
        throw new Error(
            "native layout config owners differ from resolved selection",
        );
    const bytes = nativeUnitBytes(selection, catalog);
    const serviceUnits = Object.keys(bytes)
        .filter((path) => path.endsWith(".service"))
        .map((path) => path.slice("systemd/".length))
        .sort();
    if (canonicalJson(serviceUnits) !== canonicalJson(plan.units))
        throw new Error("native layout units differ from resolved selection");
    const units = Object.entries(bytes)
        .map(([path, text]) => unit(path, text))
        .sort((left, right) => left.path.localeCompare(right.path));
    return server
        ? {
              version: 1,
              artifactClass: "server",
              compositionId: plan.compositionId,
              preset: "monitor",
              configOwners: owners,
              units,
              configs: [
                  config(selection, "access"),
                  config(selection, "monitor"),
              ],
          }
        : {
              version: 1,
              artifactClass: "node-agent",
              compositionId: plan.compositionId,
              preset: "node-agent",
              configOwners: owners,
              units,
              configs: [config(selection, "monitor-agent")],
          };
}
