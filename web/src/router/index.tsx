import type { RouteObject } from 'react-router-dom';
import { createBrowserRouter, redirect } from 'react-router-dom';

import { AuthGuard } from '@/components/auth';

import { BasicLayout } from '../layouts/BasicLayout';
import LoginPage from '../pages/auth/login';
import DashboardPage from '../pages/dashboard';
import { useAuthStore } from '../stores/useAuthStore';
import { systemRoutes } from './system';

export type AppRouter = RouteObject & {
  name?: string;
  icon?: React.ReactNode;
  children?: AppRouter[];
};

const pageRoutes: AppRouter[] = [systemRoutes];

export const router = createBrowserRouter([
  {
    path: '/',
    element: (
      <AuthGuard>
        <BasicLayout />
      </AuthGuard>
    ),
    children: [
      {
        index: true,
        element: <DashboardPage />,
      },
      ...pageRoutes,
    ],
  },
  {
    path: '/login',
    element: <LoginPage />,
    loader: () => {
      const token = useAuthStore.getState().token;
      return token ? redirect('/') : null;
    },
  },
  {
    path: '*',
    loader: () => {
      const token = useAuthStore.getState().token;
      return token ? redirect('/') : redirect('/login');
    },
  },
  // {
  //   path: "/register",
  //   element: <RegisterPage />,
  //   loader: () => {
  //     const token = useAuthStore.getState().token;
  //     return token ? redirect("/") : null;
  //   },
  // },
]);

export const getMenuData = (): AppRouter[] => {
  const { checkMenuPermissions } = useAuthStore.getState();

  const getMenuList = (menuList: AppRouter[]): AppRouter[] => {
    return menuList
      .filter((item) => {
        if (!item.path) return false;
        if (item.children) return true;
        return checkMenuPermissions(item.path);
      })
      .map((item) => {
        return {
          ...item,
          children: item.children ? getMenuList(item.children) : undefined,
        } as AppRouter;
      })
      .filter((item) => {
        // if none children, to hide the item
        if (item.children?.length === 0) {
          return false;
        }
        return true;
      });
  };
  return getMenuList(pageRoutes);
};
