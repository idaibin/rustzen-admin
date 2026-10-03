import { expect, test } from "bun:test";

import { ProTable } from "@ant-design/pro-components";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Button, Form, Switch } from "antd";
import type { ReactElement, ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";

import { RunDialog } from "../-runs/run-dialog";
import { FlowDialog } from "./flow-dialog";
import { ScheduleDialog } from "./schedule-dialog";
import { ScheduleToggle } from "./schedule-toggle";
import { TargetDialog } from "./target-dialog";
import { TemplatesContent } from "./templates-content";

type Node = ReactElement<Record<string, any>>;

function nodes(value: ReactNode): Node[] {
    if (!value || typeof value !== "object") return [];
    if (Array.isArray(value)) return value.flatMap(nodes);
    const node = value as Node;
    if (!node.props) return [];
    return [node, ...Object.values(node.props).flatMap((prop) => nodes(prop as ReactNode))];
}

function controlTree(render: () => ReactNode) {
    let tree: ReactNode;
    const client = new QueryClient();
    function Capture() {
        tree = render();
        return null;
    }
    renderToStaticMarkup(
        <QueryClientProvider client={client}>
            <Capture />
        </QueryClientProvider>,
    );
    client.clear();
    return nodes(tree);
}

const onSaved = async () => {};
const flow: Reports.Flow = {
    id: "flow-1",
    name: "Example",
    systemId: "system-1",
    steps: [],
    createdAt: "2026-10-03T00:00:00Z",
    updatedAt: "2026-10-03T00:00:00Z",
};

test("report dialog fields explicitly associate their visible labels with unique controls", () => {
    const dialogs = [
        () => FlowDialog({ systems: [], flow, onSaved }),
        () => TargetDialog({ onSaved }),
        () => ScheduleDialog({ flowOptions: [], onSaved }),
        () => RunDialog({ flows: [] }),
    ];
    for (const render of dialogs) {
        const tree = controlTree(render);
        const items = tree.filter((node) => node.type === Form.Item && node.props.label);
        expect(items.length).toBeGreaterThan(0);
        const ids = new Set<string>();
        for (const item of items) {
            const id = item.props.htmlFor;
            expect(typeof id).toBe("string");
            expect(ids.has(id)).toBeFalse();
            ids.add(id);
            expect(nodes(item.props.children).some((node) => node.props.id === id)).toBeTrue();
        }
        const form = tree.find((node) => node.type === Form)!;
        expect(typeof form.props.onFinish).toBe("function");
        expect(form.props.disabled).toBeFalse();
    }
});

test("template edit and schedule toggle expose action names independent of their icons or state", () => {
    const edit = controlTree(() => FlowDialog({ systems: [], flow, onSaved })).find(
        (node) => node.type === Button,
    )!;
    expect(edit.props["aria-label"]).toBe("编辑模板");
    const toggle = controlTree(() =>
        ScheduleToggle({
            schedule: { id: "schedule-1", enabled: true } as Reports.Schedule,
            onSaved,
        }),
    ).find((node) => node.type === Switch)!;
    expect(toggle.props["aria-label"]).toBe("启用计划");
    expect(toggle.props.checked).toBeTrue();
    expect(toggle.props["aria-busy"]).toBeFalse();
});

test("template cloning disables activation and identifies the pending source row", () => {
    const tree = nodes(
        TemplatesContent({
            systems: [],
            flows: [flow],
            error: null,
            isPending: false,
            refetch: () => {},
            onClone: () => {},
            cloning: true,
            cloningId: flow.id,
            onRefresh: onSaved,
            onRefreshSystems: onSaved,
        }),
    );
    const table = tree.find((node) => node.type === ProTable)!;
    const actions = table.props.columns.find((column: { key: string }) => column.key === "actions");
    const copy = nodes(actions.render(undefined, flow)).find(
        (node) => node.type === Button && node.props["aria-label"] === "复制流程",
    )!;
    expect(copy.props.disabled).toBeTrue();
    expect(copy.props.loading).toBeTrue();
    expect(copy.props["aria-busy"]).toBeTrue();
});
