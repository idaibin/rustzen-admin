import { describe, expect, test } from "bun:test";

const route = await Bun.file(new URL("../src/routes/system/user.tsx", import.meta.url)).text();
const actions = await Bun.file(
    new URL("../src/routes/system/-user-actions.tsx", import.meta.url),
).text();
const dialog = await Bun.file(
    new URL("../src/routes/system/-user-dialog.tsx", import.meta.url),
).text();

describe("user route composition", () => {
    test("keeps the route as the list and create-permission orchestrator", () => {
        expect(route.split("\n").length).toBeLessThan(400);
        expect(route).toContain('from "./-user-actions"');
        expect(route).toContain('from "./-user-dialog"');
        expect(route).toContain('code="system:user:create"');
        expect(route).toContain("function UserStatusBadge");
        expect(route).toContain("function getUserInitial");
    });

    test("keeps actions and dialog concerns in their bounded modules", () => {
        expect(actions).toContain('code="system:user:update"');
        expect(actions).toContain('"system:user:status"');
        expect(actions).toContain('"system:user:password"');
        expect(actions).toContain('"system:user:delete"');
        expect(actions).toContain("export function getUserActionItems");
        expect(actions).not.toContain("function RolePicker");
        expect(dialog).toContain("export const UserDialog");
        expect(dialog).toContain("function RolePicker");
        expect(dialog).not.toContain("UserActions");
    });
});
