import { createBrowserRouter, redirect } from "react-router-dom";
import type { RouteObject } from "react-router-dom";
import BasicLayout from "./layouts/BasicLayout";
import AuthGuard from "./components/AuthGuard";
import LoginPage from "./pages/auth/login";
import UserPage from "./pages/system/user";
import RolePage from "./pages/system/role";
import MenuPage from "./pages/system/menu";
import DictPage from "./pages/system/dict";
import LogPage from "./pages/system/log";
import HomePage from "./pages/home";
import { useAuthStore } from "./stores/useAuthStore";
import {
    HistoryOutlined,
    SettingOutlined,
    TeamOutlined,
    UserOutlined,
} from "@ant-design/icons";
import { MenuOutlined } from "@ant-design/icons";
import { BookOutlined } from "@ant-design/icons";

type AppRouter = RouteObject & {
    name?: string;
    icon?: React.ReactNode;
    children?: AppRouter[];
};

const systemRoutes: AppRouter = {
    name: "System",
    icon: <SettingOutlined />,
    path: "/system",
    children: [
        {
            path: "/system/user",
            element: <UserPage />,
            name: "User",
            icon: <UserOutlined />,
        },
        {
            path: "/system/role",
            element: <RolePage />,
            name: "Role",
            icon: <TeamOutlined />,
        },
        {
            path: "/system/menu",
            element: <MenuPage />,
            name: "Menu",
            icon: <MenuOutlined />,
        },
        {
            path: "/system/dict",
            element: <DictPage />,
            name: "Dictionary",
            icon: <BookOutlined />,
        },
        {
            path: "/system/log",
            element: <LogPage />,
            name: "Log",
            icon: <HistoryOutlined />,
        },
    ] as AppRouter[],
};

const pageRoutes: AppRouter[] = [systemRoutes];
const routes: RouteObject[] = [
    {
        path: "/login",
        element: <LoginPage />,
        loader: () => {
            const token = useAuthStore.getState().token;
            return token ? redirect("/") : null;
        },
    },
    {
        path: "/",
        element: (
            <AuthGuard>
                <BasicLayout />
            </AuthGuard>
        ),
        children: [
            {
                index: true,
                element: <HomePage />,
            },
            ...pageRoutes,
        ],
    },
    // {
    //   path: "/register",
    //   element: <RegisterPage />,
    //   loader: () => {
    //     const token = useAuthStore.getState().token;
    //     return token ? redirect("/") : null;
    //   },
    // },
];

export const router = createBrowserRouter(routes);

export const getMenuData = (): AppRouter[] => {
    const { checkPermision } = useAuthStore.getState();

    const getMenuList = (menuList: AppRouter[]): AppRouter[] => {
        return menuList
            .filter((item) => {
                if (!item.path) return false;
                const code = item.path.replace(/\//g, ":").slice(1);
                return checkPermision(code, true);
            })
            .map(
                (item) =>
                    ({
                        ...item,
                        children: item.children
                            ? getMenuList(item.children)
                            : undefined,
                    } as AppRouter)
            );
    };
    return getMenuList(pageRoutes);
};
