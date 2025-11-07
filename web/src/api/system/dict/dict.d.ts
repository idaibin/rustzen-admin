// ==================== 字典管理 ====================
declare namespace Dict {
  // 字典基本信息
  interface Item {
    id: number;
    dictType: string;
    label: string;
    value: string;
    isDefault: boolean;
  }

  // 查询参数
  interface QueryParams {
    current?: number;
    pageSize?: number;
    dictType?: string;
    q?: string;
    limit?: number;
  }

  // 创建字典请求
  interface CreateRequest {
    dictType: string;
    label: string;
    value: string;
    isDefault?: boolean;
  }

  // 更新字典请求
  interface UpdateRequest {
    dictType?: string;
    label?: string;
    value?: string;
    isDefault?: boolean;
  }
}
