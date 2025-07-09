import { proTableRequest, request } from "../api";
import type { OptionItem } from "Api";
import type { Menu } from "System";

/**
 * 菜单管理API服务
 */
export const menuAPI = {
  // 完整请求方法
  getMenuList: (params?: Menu.QueryParams) =>
    proTableRequest<Menu.Item>("/system/menus", params),

  getMenuById: (id: number) => request.get<Menu.Item>(`/system/menus/${id}`),

  createMenu: (data: Menu.CreateRequest) =>
    request.post<Menu.Item, Menu.CreateRequest>("/system/menus", data),

  updateMenu: (id: number, data: Menu.UpdateRequest) =>
    request.put<Menu.Item, Menu.UpdateRequest>(`/system/menus/${id}`, data),

  deleteMenu: (id: number) => request.del<void>(`/system/menus/${id}`),

  getMenuTree: () => request.get<Menu.Item[]>("/system/menus/tree"),

  getMenuOptions: () => request.get<OptionItem[]>("/system/menus/options"),

  // URL生成器（SWR使用）
  urls: {
    getMenuById: (id: number) => `/system/menus/${id}`,
    getMenuList: () => "/system/menus",
    getMenuTree: () => "/system/menus/tree",
    getMenuOptions: () => "/system/menus/options",
  },
};
