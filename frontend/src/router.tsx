import { createBrowserRouter, redirect } from "react-router-dom";
import type { RouteObject } from "react-router-dom";
import BasicLayout from "./layouts/BasicLayout";
import AuthGuard from "./components/AuthGuard";
import LoginPage from "./pages/auth/login";
import RegisterPage from "./pages/auth/register";
import UserPage from "./pages/system/user";
import RolePage from "./pages/system/role";
import MenuPage from "./pages/system/menu";
import DictPage from "./pages/system/dict";
import LogPage from "./pages/system/log";
import HomePage from "./pages/home";
import { useAuthStore } from "./stores/useAuthStore";

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
    path: "/register",
    element: <RegisterPage />,
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
      {
        path: "/system/user",
        element: <UserPage />,
      },
      {
        path: "/system/role",
        element: <RolePage />,
      },
      {
        path: "/system/menu",
        element: <MenuPage />,
      },
      {
        path: "/system/dict",
        element: <DictPage />,
      },
      {
        path: "/system/log",
        element: <LogPage />,
      },
    ],
  },
];

export const router = createBrowserRouter(routes);
