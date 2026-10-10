import { BarChart3, ListTree, ShieldCheck, Users, type LucideIcon } from "lucide-react";
import type { ReactNode } from "react";

import { createPortalBaseLayout } from "@/distribution/portal-layout";

// Portal menus carry their own icons instead of the shared route-keyed icon
// map: the shared map lists every composition's route strings, which the
// selected Web inventory policy rightly keeps out of a portal bundle.
const navigationIcon = (Icon: LucideIcon): ReactNode => (
    <Icon aria-hidden="true" size={14} strokeWidth={1.75} />
);

export const BaseLayout = createPortalBaseLayout([
    {
        path: "/analytics/overview",
        chinese: "分析概览",
        english: "Analytics overview",
        permission: "insights:overview:view",
        icon: navigationIcon(BarChart3),
    },
    {
        path: "/analytics/details",
        chinese: "分析明细",
        english: "Analytics details",
        permission: "insights:event:view",
        icon: navigationIcon(ListTree),
    },
    {
        path: "/system/user",
        chinese: "用户",
        english: "Users",
        permission: "system:user:list",
        icon: navigationIcon(Users),
    },
    {
        path: "/system/role",
        chinese: "角色",
        english: "Roles",
        permission: "system:role:list",
        icon: navigationIcon(ShieldCheck),
    },
]);
