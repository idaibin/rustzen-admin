// 系统管理模块统一类型定义

declare module "System" {
  // ==================== 用户管理 ====================
  export namespace User {
    // 用户状态枚举
    export enum Status {
      Normal = 1,
      Disabled = 2,
    }

    // 角色信息
    export interface RoleInfo {
      id: number;
      roleName: string;
    }

    // 用户基本信息
    export interface Item {
      id: number;
      username: string;
      email: string;
      realName?: string;
      avatarUrl?: string;
      status: Status;
      lastLoginAt?: string;
      createdAt: string;
      updatedAt: string;
      roles: RoleInfo[];
    }

    // 查询参数
    export interface QueryParams {
      current?: number;
      pageSize?: number;
      username?: string;
      status?: string; // "1" | "2" | "all"
    }

    // 创建用户请求
    export interface CreateRequest {
      username: string;
      email: string;
      password: string;
      realName?: string;
      status?: number;
      roleIds: number[];
    }

    // 更新用户请求
    export interface UpdateRequest {
      email?: string;
      realName?: string;
      status?: number;
      roleIds?: number[];
    }
  }

  // ==================== 角色管理 ====================
  export namespace Role {
    // 角色状态枚举
    export enum Status {
      Normal = 1,
      Disabled = 2,
    }

    // 角色基本信息
    export interface Item {
      id: number;
      roleName: string;
      roleCode: string;
      remark?: string | null;
      status: Status;
      sort: number;
      createdAt: string;
      updatedAt: string;
      menuIds?: number[];
    }

    // 查询参数
    export interface QueryParams {
      current?: number;
      pageSize?: number;
      roleName?: string;
      roleCode?: string;
      status?: string; // "1" | "2" | "all"
    }

    // 创建角色请求
    export interface CreateRequest {
      roleName: string;
      roleCode: string;
      remark?: string;
      status?: number;
      sort?: number;
      menuIds?: number[];
    }

    // 更新角色请求
    export interface UpdateRequest {
      roleName?: string;
      roleCode?: string;
      remark?: string;
      status?: number;
      sort?: number;
      menuIds?: number[];
    }
  }

  // ==================== 菜单管理 ====================
  export namespace Menu {
    // 菜单类型枚举
    export enum Type {
      Directory = 0, // 目录
      Menu = 1, // 菜单
      Button = 2, // 按钮
    }

    // 菜单状态枚举
    export enum Status {
      Normal = 1,
      Disabled = 2,
    }

    // 菜单基本信息
    export interface Item {
      id: number;
      parentId: number;
      name: string;
      path?: string | null;
      component?: string | null;
      icon?: string | null;
      type: Type;
      status: Status;
      sort: number;
      permission?: string | null;
      visible: boolean;
      keepAlive: boolean;
      createdAt: string;
      updatedAt: string;
      children?: Item[];
    }

    // 查询参数
    export interface QueryParams {
      current?: number;
      pageSize?: number;
      name?: string;
      type?: string; // "0" | "1" | "2" | "all"
      status?: string; // "1" | "2" | "all"
      parentId?: number;
    }

    // 创建菜单请求
    export interface CreateRequest {
      parentId: number;
      name: string;
      path?: string;
      component?: string;
      icon?: string;
      type: number;
      status?: number;
      sort?: number;
      permission?: string;
      visible?: boolean;
      keepAlive?: boolean;
    }

    // 更新菜单请求
    export interface UpdateRequest {
      parentId?: number;
      name?: string;
      path?: string;
      component?: string;
      icon?: string;
      type?: number;
      status?: number;
      sort?: number;
      permission?: string;
      visible?: boolean;
      keepAlive?: boolean;
    }
  }

  // ==================== 字典管理 ====================
  export namespace Dict {
    // 状态枚举
    export enum Status {
      Normal = 1,
      Disabled = 2,
    }

    // 字典基本信息
    export interface Item {
      id: number;
      dictType: string;
      dictCode: string;
      label: string;
      value: string;
      sort: number;
      status: Status;
      isDefault: boolean;
      remark?: string;
      createdAt: string;
      updatedAt: string;
    }

    // 查询参数
    export interface QueryParams {
      current?: number;
      pageSize?: number;
      dictType?: string;
      dictCode?: string;
      label?: string;
      status?: string; // "1" | "2" | "all"
    }

    // 创建字典请求
    export interface CreateRequest {
      dictType: string;
      dictCode: string;
      label: string;
      value: string;
      sort?: number;
      status?: number;
      isDefault?: boolean;
      remark?: string;
    }

    // 更新字典请求
    export interface UpdateRequest {
      dictType?: string;
      dictCode?: string;
      label?: string;
      value?: string;
      sort?: number;
      status?: number;
      isDefault?: boolean;
      remark?: string;
    }
  }

  // ==================== 日志管理 ====================
  export namespace Log {
    // 日志级别枚举
    export enum Level {
      DEBUG = "debug",
      INFO = "info",
      WARN = "warn",
      ERROR = "error",
    }

    // 日志类型枚举
    export enum Type {
      Login = "login",
      Logout = "logout",
      Create = "create",
      Update = "update",
      Delete = "delete",
      Query = "query",
      Export = "export",
      Import = "import",
    }

    // 日志基本信息
    export interface Item {
      id: number;
      level: Level;
      type: Type;
      module: string;
      action: string;
      message: string;
      requestUrl?: string;
      requestMethod?: string;
      requestParams?: string;
      responseCode?: number;
      responseTime?: number;
      userAgent?: string;
      clientIp?: string;
      userId?: number;
      username?: string;
      createdAt: string;
    }

    // 查询参数
    export interface QueryParams {
      current?: number;
      pageSize?: number;
      level?: string;
      type?: string;
      module?: string;
      username?: string;
      startTime?: string;
      endTime?: string;
    }
  }
}
