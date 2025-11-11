// 认证模块类型定义
declare namespace Auth {
  // 登录请求
  interface LoginRequest {
    username: string;
    password: string;
    rememberMe?: boolean;
  }

  // 登录响应
  interface LoginResponse {
    token: string;
    userInfo: UserInfoResponse;
  }

  // 用户信息响应
  interface UserInfoResponse {
    id: number;
    username: string;
    realName?: string;
    avatarUrl?: string;
    permissions: string[];
    isSystem: boolean;
  }
}
