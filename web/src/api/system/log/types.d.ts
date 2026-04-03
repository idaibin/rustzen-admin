// ==================== 日志管理 ====================
declare namespace Log {
    type Action = string;
    interface Item {
        id: number;
        userId: number;
        username: string;
        action: Action;
        description?: string;
        data?: unknown;
        status: string;
        durationMs: number;
        ipAddress: string;
        userAgent: string;
        createdAt: string;
    }

    interface QueryParams {
        current?: number;
        pageSize?: number;
        search?: string;
        username?: string;
        action?: string;
        description?: string;
        ipAddress?: string;
    }
}
