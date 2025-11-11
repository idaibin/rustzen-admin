import {
  BookOutlined,
  HistoryOutlined,
  MenuOutlined,
  SettingOutlined,
  TeamOutlined,
  UserOutlined,
} from '@ant-design/icons';

import { useAuthStore } from '../stores/useAuthStore';

type AppRouter = {
  name?: string;
  icon?: React.ReactNode;
  path?: string;
  children?: AppRouter[];
};

const systemRoutes: AppRouter = {
  name: 'System',
  icon: <SettingOutlined />,
  path: '/system',
  children: [
    {
      path: '/system/user',
      name: 'User',
      icon: <UserOutlined />,
    },
    {
      path: '/system/role',
      name: 'Role',
      icon: <TeamOutlined />,
    },
    {
      path: '/system/menu',
      name: 'Menu',
      icon: <MenuOutlined />,
    },
    {
      path: '/system/dict',
      name: 'Dictionary',
      icon: <BookOutlined />,
    },
    {
      path: '/system/log',
      name: 'Log',
      icon: <HistoryOutlined />,
    },
  ],
};

const pageRoutes: AppRouter[] = [systemRoutes];

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
