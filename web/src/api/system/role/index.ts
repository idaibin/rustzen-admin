import { apiRequest, proTableRequest } from '@/api';

/**
 * 角色管理API服务
 */
export const roleAPI = {
  getTableData: (params?: Role.QueryParams) =>
    proTableRequest<Role.Item, Role.QueryParams>({
      url: '/api/system/roles',
      params,
    }),

  create: (data: Role.CreateRequest) =>
    apiRequest<Role.Item, Role.CreateRequest>({
      url: '/api/system/roles',
      method: 'POST',
      params: data,
    }),

  update: (id: number, data: Role.UpdateRequest) =>
    apiRequest<Role.Item, Role.UpdateRequest>({
      url: `/api/system/roles/${id}`,
      method: 'PUT',
      params: data,
    }),

  delete: (id: number) => apiRequest<void>({ url: `/api/system/roles/${id}`, method: 'DELETE' }),

  getOptions: () => apiRequest<Api.OptionItem[]>({ url: '/api/system/roles/options' }),
};
