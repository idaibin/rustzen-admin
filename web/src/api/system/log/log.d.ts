// ==================== 日志管理 ====================
declare namespace Log {
  interface Item {
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
