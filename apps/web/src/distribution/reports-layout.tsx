import { navigationIcon } from "@/components/layout/navigation-icons";
import { createPortalBaseLayout } from "@/distribution/portal-layout";

export const BaseLayout = createPortalBaseLayout([
    {
        path: "/reports/templates",
        chinese: "模板",
        english: "Templates",
        permission: "reports:flow:view",
        icon: navigationIcon("/reports/templates"),
    },
    {
        path: "/reports/runs",
        chinese: "填报执行",
        english: "Runs",
        permission: "reports:run:view",
        icon: navigationIcon("/reports/runs"),
    },
    {
        path: "/system/user",
        chinese: "用户",
        english: "Users",
        permission: "system:user:list",
        icon: navigationIcon("/system/user"),
    },
    {
        path: "/system/role",
        chinese: "角色",
        english: "Roles",
        permission: "system:role:list",
        icon: navigationIcon("/system/role"),
    },
]);
