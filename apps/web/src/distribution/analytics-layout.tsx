import { navigationIcon } from "@/components/layout/navigation-icons";
import { createPortalBaseLayout } from "@/distribution/portal-layout";

export const BaseLayout = createPortalBaseLayout([
    {
        path: "/analytics/overview",
        chinese: "分析概览",
        english: "Analytics overview",
        permission: "insights:overview:view",
        icon: navigationIcon("/analytics/overview"),
    },
    {
        path: "/analytics/details",
        chinese: "分析明细",
        english: "Analytics details",
        permission: "insights:event:view",
        icon: navigationIcon("/analytics/details"),
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
