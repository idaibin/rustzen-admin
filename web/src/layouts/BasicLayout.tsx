import { DashboardOutlined, LogoutOutlined, UserOutlined } from '@ant-design/icons';
import { ProLayout } from '@ant-design/pro-components';
import type { MenuProps } from 'antd';
import { Dropdown } from 'antd';
import { Link, Outlet, useLocation, useNavigate } from 'react-router-dom';

import { authAPI } from '@/api/auth';
import { UserProfileModal } from '@/components/user';
import { messageApi } from '@/main';
import { getMenuData } from '@/router';

import { useAuthStore } from '../stores/useAuthStore';

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
      messageApi.success('Logout successful');
    },
  },
];

export const BasicLayout = () => {
  const location = useLocation();
  const navigate = useNavigate();
  const { userInfo } = useAuthStore();

  return (
    <ProLayout
      title="Rustzen Admin"
      logo="/rustzen.png"
      location={location}
      layout="mix"
      onMenuHeaderClick={() => navigate('/')}
      menuItemRender={(item, dom) => <Link to={item.path || '/'}>{dom}</Link>}
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
      <Outlet />
    </ProLayout>
  );
};
