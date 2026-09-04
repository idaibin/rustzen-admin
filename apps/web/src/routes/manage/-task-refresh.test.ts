import { expect, test } from "bun:test";

import { taskListRefreshInterval, taskQueryKeys, taskRunsRefreshInterval } from "./-task-refresh";

test("task refreshes only while a task is running", () => {
    expect(taskListRefreshInterval([])).toBeFalse();
    expect(taskListRefreshInterval([{ running: false }])).toBeFalse();
    expect(taskListRefreshInterval([{ running: true }])).toBe(1_000);
});

test("run records refresh only while the dialog is open and a run is active", () => {
    const running = [{ status: "running" as Task.RunStatus }];
    expect(taskRunsRefreshInterval(false, running)).toBeFalse();
    expect(taskRunsRefreshInterval(true, [{ status: "success" }])).toBeFalse();
    expect(taskRunsRefreshInterval(true, running)).toBe(1_000);
});

test("task run keys invalidate all pages while keeping page queries distinct", () => {
    expect(taskQueryKeys.list()).toEqual(["manage", "task"]);
    expect(taskQueryKeys.runs("cleanup")).toEqual(["manage", "task", "cleanup", "runs"]);
    expect(taskQueryKeys.runsPage("cleanup", 2)).toEqual(["manage", "task", "cleanup", "runs", 2]);
});
