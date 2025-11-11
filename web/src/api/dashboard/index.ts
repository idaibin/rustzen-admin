import { apiRequest } from '@/api';

export const dashboardAPI = {
  getStats: () => apiRequest<Dashboard.Stats>({ url: '/api/dashboard/stats' }),
  getHealth: () => apiRequest<Dashboard.SystemHealth>({ url: '/api/dashboard/health' }),
  getMetrics: () => apiRequest<Dashboard.SystemMetricsData>({ url: '/api/dashboard/metrics' }),
  getTrends: () => apiRequest<Dashboard.UserActivityChart>({ url: '/api/dashboard/trends' }),
};
