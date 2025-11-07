declare namespace Dashboard {
  // 顶部统计卡片
  interface Stats {
    totalUsers: number; // 总用户数
    activeUsers: number; // 活跃用户数（7天内登录）
    totalRoles: number; // 角色数量
    systemUptime: string; // 系统运行时间
    todayLogins: number; // 今日登录次数
    pendingUsers: number; // 待审核用户数
  }

  // 系统健康状态
  interface SystemHealth {
    memoryTotal: number;
    memoryUsed: number;
    memoryFree: number;
    cpuTotal: number;
    cpuUsed: number;
    cpuFree: number;
    diskTotal: number;
    diskUsed: number;
    diskFree: number;
  }

  // 性能指标
  interface SystemMetricsData {
    avgResponseTime: number;
    errorRate: number;
    totalRequests: number;
  }

  // 用户活动统计
  interface UserActivityChart {
    dailyLogins: Array<{
      date: string;
      count: number;
    }>; // 最近30天登录趋势
    hourlyActive: Array<{
      date: string;
      count: number;
    }>;
  }

  // 实时操作日志
  interface RecentOperations {
    id: string;
    user: string;
    action: string;
    resource: string;
    timestamp: string;
    status: 'success' | 'failed';
    ip: string;
  }

  // 常用操作快捷入口
  interface QuickActions {
    createUser: () => void;
    createRole: () => void;
    systemSettings: () => void;
    exportLogs: () => void;
    clearCache: () => void;
  }
}
