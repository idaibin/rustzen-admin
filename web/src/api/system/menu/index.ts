import { apiRequest, proTableRequest } from '@/api';

/**
 * 菜单管理API服务
 */
export const menuAPI = {
  getTableData: (params?: Menu.QueryParams) => {
    return proTableRequest<Menu.Item, Menu.QueryParams>({
      url: '/api/system/menus',
      params,
    }).then((res) => {
      return {
        ...res,
        data: buildMenuTree(res.data),
      };
    });
  },

  create: (data: Menu.CreateAndUpdateRequest) =>
    apiRequest<Menu.Item, Menu.CreateAndUpdateRequest>({
      url: '/api/system/menus',
      method: 'POST',
      params: data,
    }),

  update: (id: number, data: Menu.CreateAndUpdateRequest) =>
    apiRequest<Menu.Item, Menu.CreateAndUpdateRequest>({
      url: `/api/system/menus/${id}`,
      method: 'PUT',
      params: data,
    }),

  delete: (id: number) => apiRequest<void>({ url: `/api/system/menus/${id}`, method: 'DELETE' }),

  getOptions: () =>
    apiRequest<Api.OptionItem[]>({ url: '/api/system/menus/options' }).then((res) => [
      { label: 'Root', value: 0 },
      ...res,
    ]),
};

function buildMenuTree(list: Menu.Item[], parentId = 0): Menu.Item[] {
  return list
    .filter((item) => item.parentId === parentId)
    .map((item) => {
      const child = buildMenuTree(list, item.id);
      return {
        ...item,
        children: child.length > 0 ? child : null,
      };
    });
}
