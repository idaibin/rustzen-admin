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
            name: string;
            code: string;
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
            roles: Api.OptionItem<number>[];
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
            name: string;
            code: string;
            description?: string;
            status: Status;
            sortOrder?: number;
            createdAt: string;
            updatedAt: string;
            menus: Api.OptionItem<number>[];
        }

        // 查询参数
        export interface QueryParams {
            current?: number;
            pageSize?: number;
            name?: string;
            code?: string;
            status?: string; // "1" | "2" | "all"
        }

        // 创建角色请求 - 更新为与后端一致
        export interface CreateRequest {
            name: string;
            code: string;
            description?: string;
            status?: number;
            sortOrder?: number;
            menuIds: Api.Api.OptionItem<number>[];
        }

        // 更新角色请求 - 更新为与后端一致
        export interface UpdateRequest {
            name?: string;
            code?: string;
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
        export interface QueryParams {
            current?: number;
            pageSize?: number;
            name?: string;
            code?: string;
        }

        // 创建菜单请求
        export interface CreateAndUpdateRequest {
            parentId: number;
            name: string;
            code: string;
            menuType: number;
            sortOrder: number;
            status: Status;
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
        export interface Item {
            id: number;
            userId: number;
            username: string;
            action: string;
            description?: string;
            data?: string;
            status: string;
            durationMs: number;
            ipAddress: string;
            userAgent: string;
            createdAt: string;
        }

        export interface QueryParams {
            current?: number;
            pageSize?: number;
            search?: string;
            username?: string;
            action?: string;
            description?: string;
            ipAddress?: string;
        }
    }
}
