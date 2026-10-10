import { FileStack, ShieldCheck, Users, Workflow, type LucideIcon } from "lucide-react";
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
        path: "/reports/templates",
        chinese: "模板",
        english: "Templates",
        permission: "reports:flow:view",
        icon: navigationIcon(FileStack),
    },
    {
        path: "/reports/runs",
        chinese: "填报执行",
        english: "Runs",
        permission: "reports:run:view",
        icon: navigationIcon(Workflow),
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
