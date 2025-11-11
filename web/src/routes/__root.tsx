import { useQuery } from '@tanstack/react-query';
import { createRootRoute, Navigate, Outlet, redirect } from '@tanstack/react-router';
import { App, ConfigProvider } from 'antd';
import enUS from 'antd/locale/en_US';
import { useEffect } from 'react';

import { MessageContent } from '@/api';
import { authAPI } from '@/api/auth';
import { TanStackDevtoolsLayout } from '@/integrations/tanstack-query/layout';
import { BasicLayout } from '@/layouts/BasicLayout';
import { useAuthStore } from '@/stores/useAuthStore';

export const Route = createRootRoute({
  beforeLoad: (ctx) => {
    const curPath = ctx.location.pathname;
    const { token, checkMenuPermissions } = useAuthStore.getState();

    // Redirect to login if no token
    if (!token) {
      if (curPath !== '/login') {
        throw redirect({ to: '/login' });
      }
      return null;
    }

    // Redirect to home if already logged in
    if (curPath === '/login') {
      console.log('Redirect to home');
      throw redirect({ to: '/' });
    }

    // Redirect to home skip permissions check
    if (curPath === '/') {
      return null;
    }
    const isPermission = checkMenuPermissions(curPath);
    console.log('Permissions check', curPath, isPermission);

    // Redirect to 403 if no permission
    if (token && !isPermission) {
      throw redirect({ to: '/403' });
    }
  },
  component: RootLayout,
  notFoundComponent: () => <Navigate to="/404" />,
});

function RootLayout() {
  const { token, updateUserInfo } = useAuthStore();
  const { data: userInfo } = useQuery({ queryKey: ['auth-userInfo'], queryFn: authAPI.getUserInfo });

  useEffect(() => {
    if (userInfo) {
      updateUserInfo(userInfo);
    }
  }, [userInfo, updateUserInfo]);

  return (
    <ConfigProvider locale={enUS}>
      <App>
        <BasicLayout hidden={!token}>
          <Outlet />
        </BasicLayout>
        <MessageContent />
      </App>
      <TanStackDevtoolsLayout />
    </ConfigProvider>
  );
}
