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

        // 角色基本信息 - 更新为与后端一致
        export interface Item {
            id: number;
            roleName: string;
            roleCode: string;
            description?: string;
            status: Status;
            sortOrder?: number;
            createdAt: string;
            updatedAt: string;
            menuIds: number[];
        }

        // 查询参数
        export interface QueryParams {
            current?: number;
            pageSize?: number;
            roleName?: string;
            roleCode?: string;
            status?: string; // "1" | "2" | "all"
        }

        // 创建角色请求 - 更新为与后端一致
        export interface CreateRequest {
            roleName: string;
            roleCode: string;
            description?: string;
            status?: number;
            sortOrder?: number;
            menuIds: number[];
        }

        // 更新角色请求 - 更新为与后端一致
        export interface UpdateRequest {
            roleName?: string;
            roleCode?: string;
            description?: string;
            status?: number;
            sortOrder?: number;
            menuIds?: number[];
        }
    }

    // ==================== 菜单管理 ====================
    export namespace Menu {
        // 菜单状态枚举
        export enum Status {
            Normal = 1,
            Disabled = 2,
        }

        // 菜单基本信息 - 简化版本
        export interface Item {
            id: number;
            parentId?: number;
            title: string;
            path?: string;
            component?: string;
            icon?: string;
            sortOrder: number;
            status: Status;
            createdAt: string;
            updatedAt: string;
            permissionCode?: string;
        }

        // 查询参数
        export interface QueryParams {
            current?: number;
            pageSize?: number;
            title?: string;
            status?: string; // "1" | "2" | "all"
        }

        // 创建菜单请求
        export interface CreateRequest {
            parentId?: number;
            title: string;
            path?: string;
            component?: string;
            icon?: string;
            sortOrder?: number;
            status?: number;
        }

        // 更新菜单请求
        export interface UpdateRequest {
            parentId?: number;
            title?: string;
            path?: string;
            component?: string;
            icon?: string;
            sortOrder?: number;
            status?: number;
        }
    }

    // ==================== 字典管理 ====================
    export namespace Dict {
        // 字典基本信息 - 更新为与后端一致
        export interface Item {
            id: number;
            dictType: string;
            label: string;
            value: string;
            isDefault: boolean;
        }

        // 查询参数
        export interface QueryParams {
            current?: number;
            pageSize?: number;
            dictType?: string;
            q?: string;
            limit?: number;
        }

        // 创建字典请求
        export interface CreateRequest {
            dictType: string;
            label: string;
            value: string;
            isDefault?: boolean;
        }

        // 更新字典请求
        export interface UpdateRequest {
            dictType?: string;
            label?: string;
            value?: string;
            isDefault?: boolean;
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
