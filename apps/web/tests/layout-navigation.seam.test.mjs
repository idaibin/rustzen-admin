import { expect, test } from "bun:test";

const source = await Bun.file("src/components/layout/index.tsx").text();

test("Menu keys navigate the full row and close mobile navigation", () => {
    expect(source).toContain("const handleNavigationSelect = (key: string)");
    expect(source).toContain("setMobileOpen(false)");
    expect(source).toContain("router.navigate({ to: key as AppRoutePath })");
    expect(source).toContain("onClick={({ key }) => handleNavigationSelect(String(key))}");
    expect(source).toContain('data-testid="navigation-reports-schedules"');
    expect(source).toContain("{item.name}");
});

test("profile uses the same Menu handler", () => {
    expect(source).toContain('onNavigate("/profile")');
});
