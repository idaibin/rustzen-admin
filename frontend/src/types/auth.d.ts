// 认证模块类型定义

declare module "Auth" {
  // 登录请求
  export interface LoginRequest {
    username: string;
    password: string;
    rememberMe?: boolean;
  }

  // 登录响应
  export interface LoginResponse {
    token: string;
    username: string;
    userId: number;
  }

  // 用户信息响应
  export interface UserInfoResponse {
    id: number;
    username: string;
    realName?: string;
    avatarUrl?: string;
    menus: AuthMenuInfoEntity[];
    permissions: string[];
  }

  // 认证菜单信息
  export interface AuthMenuInfoEntity {
    id: number;
    parentId?: number;
    title: string;
    path: string;
    component?: string;
    icon?: string;
    orderNum?: number;
    visible?: boolean;
    keepAlive?: boolean;
    menuType?: number;
  }
}
