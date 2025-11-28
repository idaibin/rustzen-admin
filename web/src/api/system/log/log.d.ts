// ==================== 日志管理 ====================
declare namespace Log {
  type Action = 'HTTP_GET' | 'HTTP_POST' | 'HTTP_PUT' | 'HTTP_DELETE' | 'AUTH_LOGIN';
  interface Item {
    id: number;
    userId: number;
    username: string;
    action: Action;
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
