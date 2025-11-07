import {
  BookOutlined,
  HistoryOutlined,
  MenuOutlined,
  SettingOutlined,
  TeamOutlined,
  UserOutlined,
} from '@ant-design/icons';

import DictPage from '@/pages/system/dict';
import LogPage from '@/pages/system/log';
import MenuPage from '@/pages/system/menu';
import RolePage from '@/pages/system/role';
import UserPage from '@/pages/system/user';

import type { AppRouter } from '.';

export const systemRoutes: AppRouter = {
  name: 'System',
  icon: <SettingOutlined />,
  path: '/system',
  children: [
    {
      path: '/system/user',
      element: <UserPage />,
      name: 'User',
      icon: <UserOutlined />,
    },
    {
      path: '/system/role',
      element: <RolePage />,
      name: 'Role',
      icon: <TeamOutlined />,
    },
    {
      path: '/system/menu',
      element: <MenuPage />,
      name: 'Menu',
      icon: <MenuOutlined />,
    },
    {
      path: '/system/dict',
      element: <DictPage />,
      name: 'Dictionary',
      icon: <BookOutlined />,
    },
    {
      path: '/system/log',
      element: <LogPage />,
      name: 'Log',
      icon: <HistoryOutlined />,
    },
  ] as AppRouter[],
};
