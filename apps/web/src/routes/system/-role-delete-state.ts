export function deriveRoleDeletionState({
    assignedUserCount,
    deletable,
}: Pick<Role.Item, "assignedUserCount" | "deletable">) {
    return {
        disabled: assignedUserCount > 0 || !deletable,
        blockedByAssignments: assignedUserCount > 0,
    };
}
