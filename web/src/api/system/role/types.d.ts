// ==================== 角色管理 ====================
declare namespace Role {
    // 角色状态枚举
    enum Status {
        Normal = 1,
        Disabled = 2,
    }

    // 角色基本信息 - 更新为与后端一致
    interface Item {
        id: number;
        name: string;
        code: string;
        description?: string;
        status: Status;
        createdAt: string;
        updatedAt: string;
        menus: Api.OptionItem<number>[];
    }

    // 查询参数
    interface QueryParams {
        current?: number;
        pageSize?: number;
        roleName?: string;
        roleCode?: string;
        status?: string; // "1" | "2" | "all"
    }

    // 创建角色请求 - 更新为与后端一致
    interface CreateRequest {
        name: string;
        code: string;
        description?: string;
        status: number;
        menuIds: number[];
    }

    // 更新角色请求 - 更新为与后端一致
    interface UpdateRequest {
        name: string;
        code: string;
        description?: string;
        status: number;
        menuIds: number[];
    }
}
