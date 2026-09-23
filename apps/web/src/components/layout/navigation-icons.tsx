import {
    Activity,
    BarChart3,
    Boxes,
    CalendarClock,
    CalendarDays,
    ChartNoAxesCombined,
    FileClock,
    FileStack,
    FileText,
    Gauge,
    HeartPulse,
    LayoutDashboard,
    ListTree,
    PackageCheck,
    PanelLeft,
    ScrollText,
    Server,
    Settings,
    ShieldCheck,
    TriangleAlert,
    Users,
    Workflow,
    Wrench,
    type LucideIcon,
} from "lucide-react";
import type { ReactNode } from "react";

export type NavigationIconKey =
    | "/"
    | "/monitoring"
    | "/monitoring/overview"
    | "/monitoring/nodes"
    | "/monitoring/incidents"
    | "/monitoring/summaries"
    | "/analytics"
    | "/analytics/overview"
    | "/analytics/details"
    | "/reports"
    | "/reports/templates"
    | "/reports/runs"
    | "/system"
    | "/system/user"
    | "/system/role"
    | "/system/menu"
    | "/manage/log"
    | "/manage"
    | "/system/module"
    | "/system/status"
    | "/system/module-log"
    | "/manage/task"
    | "/manage/deploy";

const navigationIcons: Record<NavigationIconKey, LucideIcon> = {
    "/": LayoutDashboard,
    "/monitoring": Activity,
    "/monitoring/overview": Gauge,
    "/monitoring/nodes": Server,
    "/monitoring/incidents": TriangleAlert,
    "/monitoring/summaries": CalendarDays,
    "/analytics": ChartNoAxesCombined,
    "/analytics/overview": BarChart3,
    "/analytics/details": ListTree,
    "/reports": FileText,
    "/reports/templates": FileStack,
    "/reports/runs": Workflow,
    "/system": Settings,
    "/system/user": Users,
    "/system/role": ShieldCheck,
    "/system/menu": PanelLeft,
    "/manage/log": ScrollText,
    "/manage": Wrench,
    "/system/module": Boxes,
    "/system/status": HeartPulse,
    "/system/module-log": FileClock,
    "/manage/task": CalendarClock,
    "/manage/deploy": PackageCheck,
};

export const navigationIcon = (key: NavigationIconKey): ReactNode => {
    const Icon = navigationIcons[key];
    return <Icon aria-hidden="true" size={14} strokeWidth={1.75} />;
};
