import { expect, test } from "bun:test";
const route = await Bun.file(new URL("../src/routes/system/role.tsx", import.meta.url)).text();
const dialog = await Bun.file(new URL("../src/routes/system/-role-dialog.tsx", import.meta.url)).text();
const actions = await Bun.file(new URL("../src/routes/system/-role-actions.tsx", import.meta.url)).text();
test("role route delegates controls while retaining list authorization", () => {
    expect(route.split("\n").length).toBeLessThan(400);
    expect(route).toContain('code="system:role:create"');
    expect(route).toContain('from "./-role-actions"');
    expect(route).toContain('from "./-role-dialog"');
    expect(dialog).toContain("export function RoleDialog");
    expect(actions).toContain('code="system:role:delete"');
});
