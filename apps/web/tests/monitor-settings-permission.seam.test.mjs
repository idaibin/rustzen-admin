import { expect, test } from "bun:test";

const source = await Bun.file("src/routes/monitoring/-global-alert-settings.tsx").text();
const nodesSource = await Bun.file("src/routes/monitoring/nodes.tsx").text();
const nodeDetailsSource = await Bun.file("src/routes/monitoring/-node-details.tsx").text();
const nodePolicySource = await Bun.file("src/routes/monitoring/-node-alert-policy.tsx").text();

test("alert settings remain readable but only managers receive enabled controls and save", () => {
    expect(source).toContain('state.checkPermissions("monitor:manage")');
    expect(source).toContain("disabled={!canManage || mutation.isPending}");
    expect(source).toContain("{canManage ? (");
    expect(source).toContain('htmlType="submit"');
    expect(source).toContain("canManage && !mutation.isPending");
    expect(source).toContain("retryFailedNetworkAction(canManage, failedSave");
    expect(source).toContain("if (!canManage) setFailedSave(undefined)");
    expect(source).toContain('failedNetworkAction(error, { type: "save", values })');
    expect(source).toContain("The monitoring service could not be reached");
    expect(source).not.toContain("appMessage.error");
});

test("Nodes exposes Global settings to node viewers while the form remains manager-only", () => {
    expect(nodesSource).toContain('state.checkPermissions("monitor:node:view")');
    expect(nodesSource).not.toContain('state.checkPermissions("monitor:overview:view")');
    expect(nodesSource).toContain('panel === "settings" && canViewSettings');
    expect(source).toContain('state.checkPermissions("monitor:manage")');
    expect(source).toContain("{canManage ? (");
});

test("node policy shows its source and allows managers to save or reset an override", () => {
    expect(nodesSource).toContain('state.checkPermissions("monitor:manage")');
    expect(nodePolicySource).toContain("data.isCustom");
    expect(nodePolicySource).toContain("Save node policy");
    expect(nodePolicySource).toContain("Reset to global defaults");
    expect(nodePolicySource).toContain("monitorAPI.updateNodeAlertSettings");
    expect(nodePolicySource).toContain("monitorAPI.resetNodeAlertSettings");
    expect(nodePolicySource).toContain("const busy = save.isPending || reset.isPending");
    expect(nodePolicySource).toContain("if (canManage && !busy) reset.mutate()");
    expect(nodePolicySource).toContain("retryFailedNetworkAction(canManage, failedAction");
    expect(nodePolicySource).toContain("if (!canManage) setFailedAction(undefined)");
    expect(nodeDetailsSource).toContain("<NodeAlertPolicy key={node.nodeId}");
    expect(nodePolicySource).toContain("shouldHydrateNodePolicy");
    expect(nodePolicySource).toContain('failedNetworkAction(error, { type: "save", values })');
    expect(nodePolicySource).toContain("The monitoring service could not be reached");
    expect(nodePolicySource).not.toContain("appMessage.error");
    expect(nodeDetailsSource).toContain('body: { overflowY: "auto" }');
    expect(nodeDetailsSource).toContain('wrapper: { maxWidth: "100vw" }');
    expect(nodeDetailsSource).toContain('<div className="h-56 w-full">');
    expect(nodeDetailsSource).toContain('<ResponsiveContainer width="100%" height="100%">');
    expect(nodeDetailsSource.match(/dot=\{data\.points\.length === 1\}/g)).toHaveLength(2);
});

test("Nodes keeps cached rows visible and provides a background-refresh retry", () => {
    expect(nodesSource).toContain("hasMonitorBackgroundRefreshFailure(data, error)");
    expect(nodesSource).toContain("<BackgroundRefreshNotice");
    expect(nodesSource).toContain("onRetry={() => void refetch()}");
});
