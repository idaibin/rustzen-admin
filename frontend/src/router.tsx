import { createBrowserRouter } from "react-router-dom";
import type { RouteObject } from "react-router-dom";
import BasicLayout from "./layouts/BasicLayout";
import UserPage from "./pages/system/user";
import RolePage from "./pages/system/role";
import MenuPage from "./pages/system/menu";
import DictPage from "./pages/system/dict";
import LogPage from "./pages/system/log";
import HomePage from "./pages/home";

const routes: RouteObject[] = [
  {
    path: "/",
    element: <BasicLayout />,
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
