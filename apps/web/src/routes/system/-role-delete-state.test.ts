import { describe, expect, test } from "bun:test";

import { deriveRoleDeletionState } from "./-role-delete-state";

describe("role deletion state", () => {
    test("keeps an unassigned deletable custom role enabled", () => {
        expect(
            deriveRoleDeletionState({
                assignedUserCount: 0,
                deletable: true,
            }),
        ).toEqual({ disabled: false, blockedByAssignments: false });
    });

    test("disables a custom role assigned to users", () => {
        expect(
            deriveRoleDeletionState({
                assignedUserCount: 2,
                deletable: false,
            }),
        ).toEqual({ disabled: true, blockedByAssignments: true });
    });
});
