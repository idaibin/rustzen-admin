import { describe, expect, test } from "bun:test";

import {
    getSchedulePermissionState,
    REPORTS_FLOW_VIEW,
    REPORTS_SCHEDULE_MANAGE,
    REPORTS_SCHEDULE_VIEW,
} from "./-schedule-permissions";

const checkFor =
    (...grants: string[]) =>
    (code: string) =>
        grants.includes(code);

describe("scheduled report permission matrix", () => {
    test.each([
        {
            name: "flow viewer only",
            grants: [REPORTS_FLOW_VIEW],
            expected: { canViewFlows: true, canViewSchedules: false, canManageSchedules: false },
        },
        {
            name: "schedule viewer only",
            grants: [REPORTS_SCHEDULE_VIEW],
            expected: { canViewFlows: false, canViewSchedules: true, canManageSchedules: false },
        },
        {
            name: "schedule manager without flow view",
            grants: [REPORTS_SCHEDULE_VIEW, REPORTS_SCHEDULE_MANAGE],
            expected: { canViewFlows: false, canViewSchedules: true, canManageSchedules: true },
        },
        {
            name: "no report permissions",
            grants: [],
            expected: { canViewFlows: false, canViewSchedules: false, canManageSchedules: false },
        },
    ])("keeps $name independent", ({ grants, expected }) => {
        expect(getSchedulePermissionState(checkFor(...grants))).toEqual(expected);
    });
});
