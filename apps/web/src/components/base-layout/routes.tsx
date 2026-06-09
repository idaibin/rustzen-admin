import {
    BookOutlined,
    ClockCircleOutlined,
    CloudUploadOutlined,
    DashboardOutlined,
    FileUnknownOutlined,
    HistoryOutlined,
    MenuOutlined,
    SettingOutlined,
    StopOutlined,
    TeamOutlined,
    UserOutlined,
} from "@ant-design/icons";
import type { ReactNode } from "react";

export type AppRoutePath =
    | "/"
    | "/profile"
    | "/403"
    | "/404"
    | "/system/user"
    | "/system/role"
    | "/system/menu"
    | "/manage/dict"
    | "/manage/log"
    | "/manage/task"
    | "/manage/deploy";

type AppRouteGroupPath = "/system" | "/manage";

export type AppRouteItem = {
    name: string;
    icon?: ReactNode;
    path?: AppRoutePath | AppRouteGroupPath;
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

const dashboardRoute: AppRouteItem = {
    path: "/",
    name: "Dashboard",
    icon: <DashboardOutlined />,
    requiresPermission: false,
};

const profileRoute: AppRouteItem = {
    path: "/profile",
    name: "Profile",
    icon: <UserOutlined />,
    requiresPermission: false,
};

const forbiddenRoute: AppRouteItem = {
    path: "/403",
    name: "403 Page",
    icon: <StopOutlined />,
    requiresPermission: false,
};

const notFoundRoute: AppRouteItem = {
    path: "/404",
    name: "404 Page",
    icon: <FileUnknownOutlined />,
    requiresPermission: false,
};

const systemRoutes: AppRouteItem = {
    name: "System",
    icon: <SettingOutlined />,
    path: "/system",
    children: [
        {
            path: "/system/user",
            name: "User",
            icon: <UserOutlined />,
        },
        {
            path: "/system/role",
            name: "Role",
            icon: <TeamOutlined />,
        },
        {
            path: "/system/menu",
            name: "Menu",
            icon: <MenuOutlined />,
        },
    ],
};

const manageRoutes: AppRouteItem = {
    name: "Manage",
    icon: <CloudUploadOutlined />,
    path: "/manage",
    children: [
        {
            path: "/manage/dict",
            name: "Dictionary",
            icon: <BookOutlined />,
        },
        {
            path: "/manage/log",
            name: "Log",
            icon: <HistoryOutlined />,
        },
        {
            path: "/manage/task",
            name: "Scheduled Task",
            icon: <ClockCircleOutlined />,
        },
        {
            path: "/manage/deploy",
            name: "Deploy Versions",
            icon: <CloudUploadOutlined />,
        },
    ],
};

const appRoutePaths = new Set<string>([
    "/",
    "/profile",
    "/403",
    "/404",
    "/system/user",
    "/system/role",
    "/system/menu",
    "/manage/dict",
    "/manage/log",
    "/manage/task",
    "/manage/deploy",
]);

export const layoutMenuRoutes: AppRouteItem[] = [
    dashboardRoute,
    forbiddenRoute,
    notFoundRoute,
    systemRoutes,
    manageRoutes,
];

export const layoutSearchRoutes: AppRouteItem[] = [
    dashboardRoute,
    profileRoute,
    forbiddenRoute,
    notFoundRoute,
    systemRoutes,
    manageRoutes,
];

export const getMenuData = (checkMenuPermissions: (path: string) => boolean): AppRouteItem[] => {
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

export const getSearchRouteItems = (
    checkMenuPermissions: (path: string) => boolean,
): SearchRouteItem[] => {
    const flattenRoutes = (routes: AppRouteItem[], groupLabel = "General"): SearchRouteItem[] => {
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
