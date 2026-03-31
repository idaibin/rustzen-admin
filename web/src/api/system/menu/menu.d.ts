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
        children?: Item[] | null;
    }

    // 查询参数
    interface QueryParams {
        current?: number;
        pageSize?: number;
        name?: string;
        code?: string;
        status?: string;
    }

    // 创建菜单请求
    interface CreateRequest {
        parentId: number;
        name: string;
        code: string;
        menuType: number;
        sortOrder: number;
        status: number;
    }

    interface UpdateRequest {
        parentId: number;
        name: string;
        code: string;
        menuType: number;
        sortOrder: number;
        status: number;
    }
}
