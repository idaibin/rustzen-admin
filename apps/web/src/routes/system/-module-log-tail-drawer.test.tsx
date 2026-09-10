import { expect, test } from "bun:test";
import type { ReactElement, ReactNode } from "react";
import { Drawer } from "antd";

import { ModuleLogTailDrawer } from "./-module-log-tail-drawer";

const file = { module: "admin", date: "2026-09-11", fileName: "admin.2026-09-11", sizeBytes: 1, modifiedAt: "2026-09-11T00:00:00Z", readable: true, active: true };
const data = { ...file, content: "line", lineCount: 2, byteCount: 2048, truncated: true, nextCursor: "older" };

function query(overrides = {}) {
    return { data, error: null, isPending: false, isFetching: false, refetch: () => Promise.resolve(), ...overrides } as never;
}
type Node = ReactElement<Record<string, any>>;
function nodes(value: ReactNode): Node[] {
    if (!value || typeof value !== "object") return [];
    if (Array.isArray(value)) return value.flatMap(nodes);
    const element = value as Node;
    if (!element.props) return [];
    return [element, ...nodes(element.props.children)];
}
function tailNodes(tree: ReactElement) {
    const tail = nodes(tree).find((node) => (node.type as { name?: string }).name === "TailContent")!;
    return nodes((tail.type as (props: typeof tail.props) => ReactElement)(tail.props));
}
function render(props: Partial<Parameters<typeof ModuleLogTailDrawer>[0]> = {}) {
    const onClose = props.onClose ?? (() => {});
    const onLoadOlder = props.onLoadOlder ?? (() => {});
    return ModuleLogTailDrawer({ error: null, file, open: true, query: query(), ...props, onClose, onLoadOlder });
}

test("tail drawer renders nothing without a file and keeps Drawer focus lifecycle props", () => {
    expect(render({ file: null })).toBeNull();
    let closed = false;
    const tree = render({ onClose: () => { closed = true; } })!;
    expect(tree.type).toBe(Drawer);
    expect(tree.props.open).toBeTrue();
    expect(tree.props.destroyOnHidden).toBeTrue();
    tree.props.onClose();
    expect(closed).toBeTrue();
});

test("tail drawer prioritizes error and refetches", () => {
    let refetched = false;
    const tree = render({ error: "failed", query: query({ isPending: true, refetch: () => { refetched = true; } }) })!;
    const state = nodes(tree).find((node) => node.props.kind === "error")!;
    expect(state.props.description).toBe("failed");
    const action = state.props.action as ReactElement<{ onClick: () => void }>;
    action.props.onClick();
    expect(refetched).toBeTrue();
});

test("tail drawer renders pending, absent, empty, truncated, and older cursor states", () => {
    expect(nodes(render({ query: query({ data: undefined, isPending: true }) })!).some((node) => node.props.kind === "loading")).toBeTrue();
    expect(nodes(render({ query: query({ data: undefined }) })!).some((node) => node.props.kind === "empty")).toBeTrue();
    expect(tailNodes(render({ query: query({ data: { ...data, content: "", nextCursor: undefined } }) })!).some((node) => node.props.kind === "empty")).toBeTrue();
    let cursor = "";
    const tree = render({ query: query({ isFetching: true }), onLoadOlder: (value) => { cursor = value; } })!;
    const content = tailNodes(tree);
    expect(content.some((node) => node.props.type === "warning")).toBeTrue();
    expect(content.some((node) => String(node.props.children).includes("2 KB"))).toBeTrue();
    const button = content.find((node) => node.props.loading === true && typeof node.props.onClick === "function")!;
    button.props.onClick();
    expect(cursor).toBe("older");
});

test("tail drawer keeps closed, absent-content, and exact bounded-content states distinct", () => {
    const closed = render({ open: false })!;
    expect(closed.props.open).toBeFalse();

    const absent = nodes(render({ query: query({ data: undefined }) })!).find((node) => node.props.kind === "empty")!;
    expect(absent.props.title).toBe("未找到日志内容");

    const emptyTree = render({ query: query({ data: { ...data, content: "", nextCursor: undefined } }) })!;
    const empty = tailNodes(emptyTree).find((node) => node.props.kind === "empty")!;
    expect(empty.props.title).toBe("日志文件为空");
    expect(tailNodes(emptyTree).some((node) => typeof node.props.onClick === "function")).toBeFalse();

    const contentTree = render({ query: query({ data: { ...data, content: "exact line", nextCursor: undefined } }) })!;
    const content = tailNodes(contentTree).find((node) => node.props["data-testid"] === "module-log-tail-content")!;
    expect(content).toBeDefined();
    const pre = nodes(content).find((node) => node.type === "pre")!;
    expect(pre.props.children).toBe("exact line");
    expect(tailNodes(contentTree).some((node) => typeof node.props.onClick === "function")).toBeFalse();
});
