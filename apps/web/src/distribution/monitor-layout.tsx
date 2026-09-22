import { navigationIcon } from "@/components/layout/navigation-icons";
import { createPortalBaseLayout } from "@/distribution/portal-layout";

export const BaseLayout = createPortalBaseLayout([
    {
        path: "/monitoring/overview",
        chinese: "监控概览",
        english: "Monitoring overview",
        permission: "monitor:overview:view",
        icon: navigationIcon("/monitoring/overview"),
    },
    {
        path: "/monitoring/nodes",
        chinese: "节点",
        english: "Nodes",
        permission: "monitor:node:view",
        icon: navigationIcon("/monitoring/nodes"),
    },
    {
        path: "/monitoring/incidents",
        chinese: "告警事件",
        english: "Alert incidents",
        permission: "monitor:incident:view",
        icon: navigationIcon("/monitoring/incidents"),
    },
    {
        path: "/monitoring/summaries",
        chinese: "监控日报",
        english: "Daily summaries",
        permission: "monitor:node:view",
        icon: navigationIcon("/monitoring/summaries"),
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
