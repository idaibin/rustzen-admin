import { apiRequest } from '@/api';

/**
 * 认证相关API服务
 */
export const authAPI = {
  /**
   * 用户登录
   */
  login: (data: Auth.LoginRequest) =>
    apiRequest<Auth.LoginResponse, Auth.LoginRequest>({
      url: '/api/auth/login',
      method: 'POST',
      params: data,
    }),

  /**
   * 用户登出
   */
  logout: () => apiRequest<void>({ url: '/api/auth/logout' }),

  /**
   * 获取当前用户信息
   */
  getUserInfo: () => apiRequest<Auth.UserInfoResponse>({ url: '/api/auth/me' }),
};
