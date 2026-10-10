import {
    CalendarDays,
    Gauge,
    Server,
    ShieldCheck,
    TriangleAlert,
    Users,
    type LucideIcon,
} from "lucide-react";
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
        path: "/monitoring/overview",
        chinese: "监控概览",
        english: "Monitoring overview",
        permission: "monitor:overview:view",
        icon: navigationIcon(Gauge),
    },
    {
        path: "/monitoring/nodes",
        chinese: "节点",
        english: "Nodes",
        permission: "monitor:node:view",
        icon: navigationIcon(Server),
    },
    {
        path: "/monitoring/incidents",
        chinese: "告警事件",
        english: "Alert incidents",
        permission: "monitor:incident:view",
        icon: navigationIcon(TriangleAlert),
    },
    {
        path: "/monitoring/summaries",
        chinese: "监控日报",
        english: "Daily summaries",
        permission: "monitor:node:view",
        icon: navigationIcon(CalendarDays),
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
