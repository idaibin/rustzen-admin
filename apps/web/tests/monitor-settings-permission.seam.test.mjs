import { expect, test } from "bun:test";

const source = await Bun.file("src/routes/monitoring/-global-alert-settings.tsx").text();
const nodesSource = await Bun.file("src/routes/monitoring/nodes.tsx").text();
const nodeDetailsSource = await Bun.file("src/routes/monitoring/-node-details.tsx").text();

test("alert settings remain readable but only managers receive enabled controls and save", () => {
    expect(source).toContain('state.checkPermissions("monitor:manage")');
    expect(source).toContain("disabled={!canManage || mutation.isPending}");
    expect(source).toContain("{canManage ? (");
    expect(source).toContain('htmlType="submit"');
    expect(source).toContain("!mutation.isPending");
});

test("Nodes exposes Global settings to node viewers while the form remains manager-only", () => {
    expect(nodesSource).toContain('state.checkPermissions("monitor:node:view")');
    expect(nodesSource).not.toContain('state.checkPermissions("monitor:overview:view")');
    expect(nodesSource).toContain("panel === \"settings\" && canViewSettings");
    expect(source).toContain('state.checkPermissions("monitor:manage")');
    expect(source).toContain("{canManage ? (");
});

test("node policy shows its source and allows managers to save or reset an override", () => {
    expect(nodesSource).toContain('state.checkPermissions("monitor:manage")');
    expect(nodeDetailsSource).toContain("data.isCustom");
    expect(nodeDetailsSource).toContain("Save node policy");
    expect(nodeDetailsSource).toContain("Reset to global defaults");
    expect(nodeDetailsSource).toContain("monitorAPI.updateNodeAlertSettings");
    expect(nodeDetailsSource).toContain("monitorAPI.resetNodeAlertSettings");
    expect(nodeDetailsSource).toContain("const busy = save.isPending || reset.isPending");
    expect(nodeDetailsSource).toContain("if (!busy) reset.mutate()");
    expect(nodeDetailsSource).toContain('body: { overflowY: "auto" }');
    expect(nodeDetailsSource).toContain('wrapper: { maxWidth: "100vw" }');
});
