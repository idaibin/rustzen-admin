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
    id: number;
    username: string;
    token: string;
  }

  // 注册请求
  export interface RegisterRequest {
    username: string;
    email: string;
    password: string;
    confirmPassword?: string;
  }

  // 注册响应
  export interface RegisterResponse {
    id: number;
    username: string;
  }

  // 用户信息响应
  export interface UserInfoResponse {
    avatarUrl: string;
    id: number;
    realName: string;
    username: string;
    menus: string[];
    roles: string[];
  }

  // 菜单信息
  export interface MenuInfo {
    id: number;
    name: string;
    path?: string;
    icon?: string;
    type: number;
    sortOrder: number;
    parentId?: number;
    children?: MenuInfo[];
  }
}
