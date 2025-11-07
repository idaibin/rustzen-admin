declare namespace User {
  // 用户状态枚举
  enum Status {
    Normal = 1,
    Disabled = 2,
  }

  // 角色信息
  interface RoleInfo {
    id: number;
    name: string;
    code: string;
  }

  // 用户基本信息
  interface Item {
    id: number;
    username: string;
    email: string;
    realName?: string;
    avatarUrl?: string;
    status: Status;
    lastLoginAt?: string;
    createdAt: string;
    updatedAt: string;
    roles: Api.OptionItem<number>[];
  }

  // 查询参数
  interface QueryParams {
    current?: number;
    pageSize?: number;
    username?: string;
    status?: string; // "1" | "2" | "all"
  }

  // 创建用户请求
  interface CreateRequest {
    username: string;
    email: string;
    password: string;
    realName?: string;
    status?: number;
    roleIds: number[];
  }

  // 更新用户请求
  interface UpdateRequest {
    email?: string;
    realName?: string;
    status?: number;
    roleIds?: number[];
  }
}
