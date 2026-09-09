import { expect, test } from "bun:test";

const [pageSource, actionSource] = await Promise.all([
    Bun.file("src/routes/manage/task.tsx").text(),
    Bun.file("src/routes/manage/-task-console-actions.tsx").text(),
]);
const source = `${pageSource}\n${actionSource}`;

test("maintenance task console keeps the fixed control and observation seams", () => {
    for (const selector of [
        "maintenance-task-table",
        "maintenance-task-records-${taskKey}",
        "maintenance-task-run-${record.taskKey}",
        "maintenance-task-records-dialog-${taskKey}",
        "maintenance-task-run-dialog-${record.taskKey}",
        "maintenance-task-status-${row.taskKey}",
        "maintenance-task-run-status-${row.id}",
    ]) {
        expect(source).toContain(selector);
    }
    expect(source).toContain('code="manage:task:run"');
    expect(source).toContain('disabled={isFetching || record.running}');
    expect(source).toContain('data-status={status ?? "never"}');
    expect(source).toContain('kind="empty"');
    expect(source).toContain('kind="error"');
    expect(source).toContain('scroll={{ x: "max-content", y: "100%" }}');
    expect(source).not.toContain("Schedule editor");
    expect(source).not.toContain("enable task");
});
