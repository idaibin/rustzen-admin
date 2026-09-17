import {
    AppstoreOutlined,
    ClockCircleOutlined,
    CloudUploadOutlined,
    DashboardOutlined,
    FileTextOutlined,
    HistoryOutlined,
    MenuOutlined,
    MonitorOutlined,
    SettingOutlined,
    TeamOutlined,
    UserOutlined,
} from "@ant-design/icons";
import type { ReactNode } from "react";

import { localizeModuleMenuName, localizeModuleName } from "@/lib/builtin-i18n";
import { t } from "@/lib/i18n";

import { dedupeModuleNavigation } from "./module-navigation";

export type AppRoutePath =
    | "/"
    | "/profile"
    | SystemModule.RoutePath
    | "/403"
    | "/404"
    | "/system/user"
    | "/system/role"
    | "/system/menu"
    | "/system/module"
    | "/system/status"
    | "/system/module-log"
    | "/manage/log"
    | "/manage/task"
    | "/manage/deploy";

type AppRouteGroupPath = "/monitoring" | "/analytics" | "/reports" | "/system" | "/manage";

export type AppRouteItem = {
    name: string;
    icon?: ReactNode;
    path?: AppRoutePath | AppRouteGroupPath;
    permission?: string;
    children?: AppRouteItem[];
    requiresPermission?: boolean;
};

export type SearchRouteItem = {
    path: AppRoutePath;
    label: string;
    groupLabel: string;
    icon?: ReactNode;
    searchText: string;
};

const dashboardRoute = (): AppRouteItem => ({
    path: "/",
    name: t("仪表盘", "Dashboard"),
    icon: <DashboardOutlined />,
    permission: "dashboard:view",
});

const profileRoute = (): AppRouteItem => ({
    path: "/profile",
    name: t("个人资料", "Profile"),
    icon: <UserOutlined />,
    requiresPermission: false,
});

const moduleGroupPaths: Record<SystemModule.Id, AppRouteGroupPath> = {
    monitor: "/monitoring",
    insights: "/analytics",
    reports: "/reports",
};

const moduleIcons: Record<SystemModule.Icon, ReactNode> = {
    monitor: <MonitorOutlined />,
    "chart-no-axes-combined": <AppstoreOutlined />,
    "file-text": <FileTextOutlined />,
};

const getModuleRoutes = (navigation: SystemModule.NavigationItem[]): AppRouteItem[] => {
    const groups = new Map<SystemModule.Id, AppRouteItem>();
    dedupeModuleNavigation(navigation).forEach((item) => {
        const icon = moduleIcons[item.icon];
        if (!registeredModuleRoutePaths.has(item.path) || !icon) {
            return;
        }
        const group = groups.get(item.module) ?? {
            path: moduleGroupPaths[item.module],
            name: localizeModuleName(item.module, item.moduleName),
            icon,
            children: [],
        };
        group.children?.push({
            path: item.path,
            name: localizeModuleMenuName(item.module, item.code, item.title),
            icon,
            permission: item.permission,
            requiresPermission: false,
        });
        groups.set(item.module, group);
    });
    return Array.from(groups.values());
};

const systemRoutes = (): AppRouteItem => ({
    name: t("系统", "System"),
    icon: <SettingOutlined />,
    path: "/system",
    children: [
        {
            path: "/system/user",
            name: t("用户", "Users"),
            icon: <UserOutlined />,
            permission: "system:user:list",
        },
        {
            path: "/system/role",
            name: t("角色", "Roles"),
            icon: <TeamOutlined />,
            permission: "system:role:list",
        },
        {
            path: "/system/menu",
            name: t("菜单", "Menus"),
            icon: <MenuOutlined />,
            permission: "system:menu:list",
        },
        {
            path: "/manage/log",
            name: t("日志", "Logs"),
            icon: <HistoryOutlined />,
            permission: "manage:log:list",
        },
    ],
});

const manageRoutes = (): AppRouteItem => ({
    name: t("管理", "Management"),
    icon: <CloudUploadOutlined />,
    path: "/manage",
    children: [
        {
            path: "/system/module",
            name: t("系统模块", "System modules"),
            icon: <AppstoreOutlined />,
            permission: "system:module:list",
        },
        {
            path: "/system/status",
            name: t("系统状态", "System status"),
            icon: <MonitorOutlined />,
            permission: "system:status:view",
        },
        {
            path: "/system/module-log",
            name: t("模块日志", "Module logs"),
            icon: <FileTextOutlined />,
            permission: "system:module:log:view",
        },
        {
            path: "/manage/task",
            name: t("定时任务", "Scheduled tasks"),
            icon: <ClockCircleOutlined />,
            permission: "manage:task:list",
        },
        {
            path: "/manage/deploy",
            name: t("部署版本", "Deploy versions"),
            icon: <CloudUploadOutlined />,
            permission: "manage:deploy:list",
        },
    ],
});

const appRoutePaths = new Set<string>([
    "/",
    "/profile",
    "/monitoring/overview",
    "/monitoring/nodes",
    "/monitoring/incidents",
    "/monitoring/summaries",
    "/analytics/overview",
    "/analytics/details",
    "/reports/templates",
    "/reports/runs",
    "/403",
    "/404",
    "/system/user",
    "/system/role",
    "/system/menu",
    "/system/module",
    "/system/status",
    "/system/module-log",
    "/manage/log",
    "/manage/task",
    "/manage/deploy",
]);

const registeredModuleRoutePaths = new Set<SystemModule.RoutePath>([
    "/monitoring/overview",
    "/monitoring/nodes",
    "/monitoring/incidents",
    "/monitoring/summaries",
    "/analytics/overview",
    "/analytics/details",
    "/reports/templates",
    "/reports/runs",
]);

export const getMenuData = (
    checkMenuPermissions: (path: string) => boolean,
    moduleNavigation: SystemModule.NavigationItem[],
): AppRouteItem[] => {
    const layoutMenuRoutes: AppRouteItem[] = [
        dashboardRoute(),
        ...getModuleRoutes(moduleNavigation),
        systemRoutes(),
        manageRoutes(),
    ];
    const getMenuList = (menuList: AppRouteItem[]): AppRouteItem[] => {
        return menuList
            .filter((item) => {
                if (!item.path) return false;
                if (item.requiresPermission === false) return true;
                if (item.children) return true;
                return checkMenuPermissions(item.path);
            })
            .map((item) => ({
                ...item,
                children: item.children ? getMenuList(item.children) : undefined,
            }))
            .filter((item) => {
                if (item.children?.length === 0) {
                    return false;
                }
                return true;
            });
    };

    return getMenuList(layoutMenuRoutes);
};

export const getCoreNavigationItems = (): AppRouteItem[] => {
    const flattenRoutes = (routes: AppRouteItem[]): AppRouteItem[] =>
        routes.flatMap((route) =>
            route.children
                ? flattenRoutes(route.children)
                : isAppRoutePath(route.path) && route.permission
                  ? [route]
                  : [],
        );

    return flattenRoutes([dashboardRoute(), systemRoutes(), manageRoutes()]);
};

export const getSearchRouteItems = (
    checkMenuPermissions: (path: string) => boolean,
    moduleNavigation: SystemModule.NavigationItem[],
): SearchRouteItem[] => {
    const layoutSearchRoutes: AppRouteItem[] = [
        dashboardRoute(),
        profileRoute(),
        ...getModuleRoutes(moduleNavigation),
        systemRoutes(),
        manageRoutes(),
    ];
    const flattenRoutes = (
        routes: AppRouteItem[],
        groupLabel = t("通用", "General"),
    ): SearchRouteItem[] => {
        return routes.flatMap((route) => {
            if (route.children) {
                return flattenRoutes(route.children, route.name);
            }

            if (!isAppRoutePath(route.path)) {
                return [];
            }

            if (route.requiresPermission !== false && !checkMenuPermissions(route.path)) {
                return [];
            }

            return [
                {
                    path: route.path,
                    label: route.name,
                    groupLabel,
                    icon: route.icon,
                    searchText: [route.name, route.path, groupLabel].join(" ").toLowerCase(),
                },
            ];
        });
    };

    return flattenRoutes(layoutSearchRoutes);
};

const isAppRoutePath = (path: AppRouteItem["path"]): path is AppRoutePath => {
    return Boolean(path && appRoutePaths.has(path));
};
