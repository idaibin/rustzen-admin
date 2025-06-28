import { proTableRequest, request } from "../api";
import type { Log } from "System";

/**
 * 日志管理API服务
 */
export const logAPI = {
  // 完整请求方法
  getLogList: (params?: Log.QueryParams) =>
    proTableRequest<Log.Item>("/system/logs", params),

  getLogById: (id: number) => request.get<Log.Item>(`/system/logs/${id}`),

  deleteLog: (id: number) => request.del<void>(`/system/logs/${id}`),

  clearLogs: () => request.del<void>("/system/logs/clear"),

  exportLogs: (params?: Log.QueryParams) =>
    request.get<Blob>("/system/logs/export", params),

  // URL生成器（SWR使用）
  urls: {
    getLogById: (id: number) => `/system/logs/${id}`,
    getLogList: () => "/system/logs",
  },
};
