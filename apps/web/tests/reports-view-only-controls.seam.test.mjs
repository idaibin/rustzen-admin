import { expect, test } from "bun:test";

const panel = await Bun.file(
    new URL("../src/routes/reports/-templates/schedule-panel.tsx", import.meta.url),
).text();
const columns = await Bun.file(
    new URL("../src/routes/reports/-templates/schedule-columns.tsx", import.meta.url),
).text();
const scheduleDialog = await Bun.file(
    new URL("../src/routes/reports/-templates/schedule-dialog.tsx", import.meta.url),
).text();
const toggle = await Bun.file(
    new URL("../src/routes/reports/-templates/schedule-toggle.tsx", import.meta.url),
).text();
const runs = await Bun.file(new URL("../src/routes/reports/runs.tsx", import.meta.url)).text();
const runDialog = await Bun.file(
    new URL("../src/routes/reports/-runs/run-dialog.tsx", import.meta.url),
).text();
const retry = await Bun.file(
    new URL("../src/routes/reports/-runs/retry-run-button.tsx", import.meta.url),
).text();

test("view-only Reports controls stay behind schedule and run management permissions", () => {
    expect(panel).toContain("createScheduleColumns({ flowOptions, canManageSchedules, onSaved: refresh })");
    expect(columns).toContain("if (canManageSchedules)");
    expect(columns).toContain('data-testid="schedule-actions-column"');
    expect(columns).toContain("<ScheduleDialog");
    expect(columns).toContain("<ScheduleToggle");
    expect(columns).toContain('data-testid="schedule-delete"');
    expect(scheduleDialog).toContain('schedule ? "schedule-edit" : "schedule-create"');
    expect(toggle).toContain('data-testid="schedule-toggle"');
    expect(runDialog).toContain('data-testid="run-create"');
    expect(runs).toContain('code="reports:run:manage"');
    expect(runs).toContain('data-testid={`run-cancel-${row.id}`}');
    expect(retry).toContain('code="reports:run:manage"');
});
