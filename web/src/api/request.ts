import { useAuthStore } from "@/stores/useAuthStore";
import type { ApiResponse, BaseParams, PageResponse } from "Api";
import { message } from "antd";

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
     * Request url
     */
    url: string;
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
const coreRequest = async <T>({
    url,
    ...options
}: RequestOptions): Promise<ApiResponse<T>> => {
    const controller = new AbortController();
    requestPool.add(controller);

    const config: RequestInit = {
        ...options,
        signal: controller.signal,
        headers: {
            ...defaultHeaders,
            ...options.headers,
            ...getAuthHeaders(),
        },
    };
    try {
        const response = await fetch(url, config);
        if (!response.ok) {
            throw response;
        }
        const result = await response.json();
        if (result.code !== 0) {
            if (options.silent) {
                return Promise.reject(result);
            }
            throw result;
        }
        return result;
    } catch (error) {
        return handleError(error);
    } finally {
        requestPool.delete(controller);
    }
};

type BaseRequestProps<T> = {
    url: string;
    method?: "GET" | "POST" | "PUT" | "DELETE";
    params?: T;
    mode?: "normal" | "proTable";
};
type RequestProps<T> = Omit<BaseRequestProps<T>, "mode">;

/**
 * base request adapter
 */
const baseRequest = async <T, P = BaseParams>({
    url,
    method = "GET",
    params,
    mode = "normal",
}: BaseRequestProps<P>): Promise<any> => {
    let query = "";
    let body = undefined;

    if (["PUT", "POST"].includes(method)) {
        body = JSON.stringify(params);
    } else if (params) {
        query = `?${new URLSearchParams(
            params as Record<string, any>
        ).toString()}`;
    }

    const res = await coreRequest<T | T[]>({
        url: `${url}${query}`,
        body,
        method,
    });

    if (mode === "proTable") {
        // 适配 ProTable 结构
        return {
            data: (res.data as T[]) || [],
            total: res.total || 0,
            success: true,
        };
    }
    // 普通结构
    return res.data;
};

/**
 * SWR fetcher for GET requests
 */
export const swrFetcher = <T, P = BaseParams>(url: string, params?: P) => {
    return baseRequest<T, P>({ url, params });
};

/**
 * API request adapter
 */
export const apiRequest = <T, P = BaseParams>(props: RequestProps<P>) => {
    return baseRequest<T, P>(props);
};

/**
 * ProTable request adapter
 */
export const proTableRequest = async <T, P>(props: RequestProps<P>) => {
    return baseRequest<T, P>({ ...props, mode: "proTable" }) as Promise<
        PageResponse<T>
    >;
};
