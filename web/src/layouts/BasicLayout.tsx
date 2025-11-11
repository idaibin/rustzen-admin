import { DashboardOutlined, LogoutOutlined, UserOutlined } from '@ant-design/icons';
import { ProLayout } from '@ant-design/pro-components';
import { Link, useRouter } from '@tanstack/react-router';
import type { MenuProps } from 'antd';
import { Dropdown } from 'antd';

import { appMessage } from '@/api';
import { authAPI } from '@/api/auth';
import { UserProfileModal } from '@/components/user';
import { getMenuData } from '@/router';

import { useAuthStore } from '../stores/useAuthStore';

interface BasicLayoutProps {
  children: React.ReactNode;
  hidden?: boolean;
}

export const BasicLayout = ({ children, hidden = false }: BasicLayoutProps) => {
  const router = useRouter();
  const { userInfo } = useAuthStore();
  const currentPath = router.state.location.pathname;

  // If hidden, return children
  if (hidden) {
    return children;
  }

  // User dropdown menu items
  const userMenuItems: MenuProps['items'] = [
    {
      key: 'profile',
      icon: <UserOutlined />,
      label: <UserProfileModal />,
    },
    {
      type: 'divider',
    },
    {
      key: 'logout',
      icon: <LogoutOutlined />,
      label: 'Logout',
      onClick: async () => {
        await authAPI.logout();
        useAuthStore.getState().clearAuth();
        appMessage.success('Logout successful');
        router.navigate({ to: '/login' });
      },
    },
  ];

  return (
    <ProLayout
      title="Rustzen Admin"
      logo="/rustzen.png"
      location={{ pathname: currentPath }}
      layout="mix"
      onMenuHeaderClick={() => router.navigate({ to: '/' })}
      menuItemRender={(item, dom) => (
        <Link to={item.path || '/'} className="block">
          {dom}
        </Link>
      )}
      route={{
        path: '/',
        children: [
          {
            path: '/',
            name: 'Dashboard',
            icon: <DashboardOutlined />,
          },
          ...getMenuData(),
        ],
      }}
      avatarProps={{
        src: userInfo?.avatarUrl,
        size: 'small',
        title: userInfo?.realName || userInfo?.username,
        render: (_props, dom) => {
          return <Dropdown menu={{ items: userMenuItems }}>{dom}</Dropdown>;
        },
      }}
    >
      {children}
    </ProLayout>
  );
};
