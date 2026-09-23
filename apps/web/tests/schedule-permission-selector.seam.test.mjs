import { describe, expect, test } from "bun:test";

const routeSource = await Bun.file(
    new URL("../src/routes/reports/templates.tsx", import.meta.url),
).text();
const schedulePanelSource = await Bun.file(
    new URL("../src/routes/reports/-templates/schedule-panel.tsx", import.meta.url),
).text();
const scheduleColumnsSource = await Bun.file(
    new URL("../src/routes/reports/-templates/schedule-columns.tsx", import.meta.url),
).text();

describe("scheduled report auth-store selector seam", () => {
    test("subscribes to the stable permission function instead of an object snapshot", () => {
        expect(routeSource).toContain(
            "const checkPermissions = useAuthStore((state) => state.checkPermissions);",
        );
        expect(routeSource).toContain("getSchedulePermissionState(checkPermissions)");
        expect(routeSource).not.toContain(
            "useAuthStore((state) =>\n        getSchedulePermissionState",
        );
    });

    test("keeps schedule-only readers and managers inside their capability gates", () => {
        expect(routeSource).toContain("if (!canViewFlows)");
        expect(routeSource).toContain('code="reports:schedule:view"');
        expect(routeSource).toContain("<SchedulePanel />");
        expect(schedulePanelSource).toContain("state.checkPermissions(REPORTS_SCHEDULE_MANAGE)");
        expect(schedulePanelSource).toContain(
            "createScheduleColumns({ flowOptions, canManageSchedules",
        );
        expect(scheduleColumnsSource).toContain("if (canManageSchedules)");
        expect(scheduleColumnsSource).toContain("occurrence.runId ? (");
    });
});
