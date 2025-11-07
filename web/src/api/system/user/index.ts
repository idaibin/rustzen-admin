import { apiRequest, proTableRequest } from '@/api';

/**
 * 用户管理API服务
 */
export const userAPI = {
  getTableData: (params?: User.QueryParams) =>
    proTableRequest<User.Item, User.QueryParams>({
      url: '/api/system/users',
      params,
    }),

  create: (data: User.CreateRequest) =>
    apiRequest<User.Item, User.CreateRequest>({
      url: '/api/system/users',
      method: 'POST',
      params: data,
    }),

  update: (id: number, data: User.UpdateRequest) =>
    apiRequest<User.Item, User.UpdateRequest>({
      url: `/api/system/users/${id}`,
      method: 'PUT',
      params: data,
    }),

  delete: (id: number) => apiRequest<void>({ url: `/api/system/users/${id}`, method: 'DELETE' }),

  updateStatus: (id: number, status: number) =>
    apiRequest<void>({
      url: `/api/system/users/${id}/status`,
      method: 'PUT',
      params: { status },
    }),

  resetPassword: (id: number, password: string) =>
    apiRequest<void>({
      url: `/api/system/users/${id}/reset-password`,
      method: 'PUT',
      params: { password },
    }),

  getStatusOptions: () =>
    apiRequest<Api.OptionItem[]>({
      url: '/api/system/users/status-options',
    }),
};
