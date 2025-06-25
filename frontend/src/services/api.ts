import type { ApiResponse, BaseParams } from "Api";

/**
 * 获取认证头
 */
function getAuthHeaders(): Record<string, string> {
  const token = localStorage.getItem("token");
  return token ? { Authorization: `Bearer ${token}` } : {};
}

/**
 * 默认请求头
 */
const defaultHeaders = {
  "Content-Type": "application/json",
};

/**
 * 核心请求函数
 * @param url - 请求地址
 * @param options - fetch 的配置选项
 * @returns Promise<T> - 返回处理后的数据
 */
async function coreRequest<T>(
  url: string,
  options: RequestInit = {}
): Promise<ApiResponse<T>> {
  const config: RequestInit = {
    ...options,
    headers: {
      ...defaultHeaders,
      ...getAuthHeaders(),
      ...options.headers,
    },
  };

  const response = await fetch(url, config);

  if (!response.ok) {
    const error = new Error("An HTTP error occurred");
    try {
      (error as Error & { info: unknown }).info = await response.json();
    } catch {
      (error as Error & { info: { message: string } }).info = {
        message: "Response not in JSON format",
      };
    }
    (error as Error & { status: number }).status = response.status;
    throw error;
  }

  // 处理 204 No Content 等没有响应体的成功请求
  if (response.status === 204) {
    return {
      code: 204,
      message: "Success (No Content)",
      data: null as T,
    };
  }

  return response.json();
}

/**
 * GET 请求
 * @param url - 请求地址
 * @param params - URL 查询参数
 */
const get = <T, P = BaseParams>(
  url: string,
  params?: P
): Promise<ApiResponse<T>> => {
  const query = params
    ? `?${new URLSearchParams(
        params as unknown as Record<string, string>
      ).toString()}`
    : "";
  return coreRequest<T>(`${url}${query}`, { method: "GET" });
};

/**
 * POST 请求
 * @param url - 请求地址
 * @param data - 请求体数据
 */
const post = <T, P = BaseParams>(
  url: string,
  data?: P
): Promise<ApiResponse<T>> => {
  return coreRequest<T>(url, {
    method: "POST",
    body: JSON.stringify(data),
  });
};

/**
 * PUT 请求
 * @param url - 请求地址
 * @param data - 请求体数据
 */
const put = <T, P = BaseParams>(
  url: string,
  data?: P
): Promise<ApiResponse<T>> => {
  return coreRequest<T>(url, {
    method: "PUT",
    body: JSON.stringify(data),
  });
};

/**
 * DELETE 请求
 * @param url - 请求地址
 */
const del = <T, P = BaseParams>(
  url: string,
  data?: P
): Promise<ApiResponse<T>> => {
  return coreRequest<T>(url, {
    method: "DELETE",
    body: JSON.stringify(data),
  });
};
/**
 * SWR 适配器：它就是最纯粹的 GET 请求
 * 所以可以直接复用 get 函数
 */
export const swrFetcher = get;

/**
 * ProTable 适配器：封装了数据请求和格式转换
 */
export async function requestTable<T, P = BaseParams>(url: string, params?: P) {
  // 复用基础的 get 方法
  const res = await get<T[], P>(url, params);
  return {
    data: res.data,
    success: res.code === 200,
    // 后端最好能直接返回 total，这里是前端模拟
    total: res.total || res.data.length,
  };
}

export const request = {
  get,
  post,
  put,
  del,
};
