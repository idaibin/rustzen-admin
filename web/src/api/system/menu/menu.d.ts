// ==================== 菜单管理 ====================
declare namespace Menu {
  // 菜单状态枚举
  enum Status {
    Normal = 1,
    Disabled = 2,
  }

  // 菜单基本信息 - 简化版本
  interface Item {
    id: number;
    parentId: number;
    name: string;
    code: string;
    menuType: number;
    sortOrder: number;
    status: Status;
    isSystem: boolean;
    createdAt: string;
    updatedAt: string;
  }

  // 查询参数
  interface QueryParams {
    current?: number;
    pageSize?: number;
    name?: string;
    code?: string;
  }

  // 创建菜单请求
  interface CreateAndUpdateRequest {
    parentId: number;
    name: string;
    code: string;
    menuType: number;
    sortOrder: number;
    status: Status;
  }
}
