export const REPORTS_FLOW_VIEW = "reports:flow:view";
export const REPORTS_SCHEDULE_VIEW = "reports:schedule:view";
export const REPORTS_SCHEDULE_MANAGE = "reports:schedule:manage";

export type ReportsPermissionCheck = (code: string) => boolean;

export type SchedulePermissionState = {
    canViewFlows: boolean;
    canViewSchedules: boolean;
    canManageSchedules: boolean;
};

export function getSchedulePermissionState(
    checkPermissions: ReportsPermissionCheck,
): SchedulePermissionState {
    return {
        canViewFlows: checkPermissions(REPORTS_FLOW_VIEW),
        canViewSchedules: checkPermissions(REPORTS_SCHEDULE_VIEW),
        canManageSchedules: checkPermissions(REPORTS_SCHEDULE_MANAGE),
    };
}
