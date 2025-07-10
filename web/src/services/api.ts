import { useAuthStore } from "@/stores/useAuthStore";
import type { ApiResponse, BaseParams, PageResponse } from "Api";
import { message } from "antd";

/**
 * Get API base URL depending on environment
 */
const getApiBaseUrl = (): string => {
    if (import.meta.env.DEV) {
        return "/api";
    }
    return import.meta.env.VITE_API_BASE_URL || "/api";
};

const API_BASE_URL = getApiBaseUrl();
const requestPool = new Set<AbortController>();

/**
 * Get auth headers from localStorage
 */
function getAuthHeaders(): Record<string, string> {
    const token = useAuthStore.getState().token;
    return token ? { Authorization: `Bearer ${token}` } : {};
}

/**
 * Default request headers
 */
const defaultHeaders = {
    "Content-Type": "application/json",
};

interface RequestOptions extends RequestInit {
    /**
     * Custom success message
     */
    successMessage?: string;

    /**
     * Custom error message
     */
    errorMessage?: string;

    /**
     * If true, disables all messages
     */
    silent?: boolean;
}

/**
 * Handle all errors
 */
const handleError = async (error: unknown) => {
    if (error instanceof Response) {
        try {
            const res = await error.json();
            if (error.status === 401) {
                useAuthStore.getState().clearAuth();
                requestPool.forEach((controller) => {
                    if (!controller.signal.aborted) {
                        controller.abort();
                    }
                });
            }
            message.error(res.message || error.statusText);
        } catch {
            message.error(error.statusText);
        }
    } else if (error instanceof DOMException && error.name === "AbortError") {
        console.log("abort controller do nothing");

        // abort controller do nothing
    } else {
        message.error(error instanceof Error ? error.message : "Network error");
    }
    throw error;
};

/**
 * Core request function with unified error and success handling
 */
async function coreRequest<T>(
    url: string,
    options: RequestOptions = {}
): Promise<ApiResponse<T>> {
    const fullUrl = `${API_BASE_URL}${url}`;
    const controller = new AbortController();
    requestPool.add(controller);

    const config: RequestInit = {
        ...options,
        signal: controller.signal,
        headers: {
            ...defaultHeaders,
            ...getAuthHeaders(),
            ...options.headers,
        },
    };
    try {
        const response = await fetch(fullUrl, config);
        if (!response.ok) {
            throw response;
            // handleNetworkAndHttpError(null, response);
        }
        const result = await response.json();
        if (result.code !== 0) {
            throw result;
        }
        return result;
    } catch (error) {
        return handleError(error);
        // if (error instanceof SyntaxError) {
        //   handleBusinessError(
        //     { code: -1, message: "Response format error" },
        //     options
        //   );
        // } else {
        //   handleNetworkAndHttpError(error);
        // }
    } finally {
        requestPool.delete(controller);
    }
}

/**
 * GET request
 */
const get = <T, P = BaseParams>(url: string, params?: P): Promise<T> => {
    const query = params
        ? `?${new URLSearchParams(
              params as unknown as Record<string, string>
          ).toString()}`
        : "";
    return coreRequest<T>(`${url}${query}`, { method: "GET" }).then(
        (res) => res.data
    );
};

/**
 * POST request with success message
 */
const post = <T, P = BaseParams>(
    url: string,
    data?: P,
    options?: RequestOptions
): Promise<T> => {
    return coreRequest<T>(url, {
        method: "POST",
        body: JSON.stringify(data),
    }).then((res) => {
        if (!options?.silent && res.code === 0) {
            message.success(options?.successMessage || "Created successfully");
        }
        return res.data;
    });
};

/**
 * PUT request with success message
 */
const put = <T, P = BaseParams>(
    url: string,
    data?: P,
    options?: RequestOptions
): Promise<T> => {
    return coreRequest<T>(url, {
        method: "PUT",
        body: JSON.stringify(data),
    }).then((res) => {
        if (!options?.silent && res.code === 0) {
            message.success(options?.successMessage || "Updated successfully");
        }
        return res.data;
    });
};

/**
 * DELETE request with success message
 */
const del = <T, P = BaseParams>(
    url: string,
    data?: P,
    options?: RequestOptions
): Promise<T> => {
    return coreRequest<T>(url, {
        method: "DELETE",
        body: JSON.stringify(data),
    }).then((res) => {
        if (!options?.silent && res.code === 0) {
            message.success(options?.successMessage || "Deleted successfully");
        }
        return res.data;
    });
};

/**
 * SWR fetcher for GET requests
 */
export const swrFetcher = get;

/**
 * ProTable request adapter
 */
export const proTableRequest = async <T>(
    url: string,
    params?: unknown
): Promise<PageResponse<T>> => {
    try {
        const query = params
            ? `?${new URLSearchParams(
                  params as unknown as Record<string, string>
              ).toString()}`
            : "";
        const res = await coreRequest<T[]>(`${url}${query}`, {
            method: "GET",
        });
        return {
            data: res.data || [],
            total: res.total || 0,
            success: true,
        };
    } catch {
        return {
            data: [],
            total: 0,
            success: false,
        };
    }
};

export const request = {
    get,
    post,
    put,
    del,
};

/**
 * Get full API URL (for debugging or special use)
 */
export const getFullApiUrl = (path: string): string => {
    return `${API_BASE_URL}${path}`;
};
