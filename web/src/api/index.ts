import { useAuthStore } from "@/stores/useAuthStore";
import { messageApi } from "@/main";

/**
 * API request adapter
 */
export const apiRequest = <T, P = Api.BaseParams>(props: RequestOptions<P>) => {
    return coreRequest<T, P>(props).then((res) => res.data);
};

/**
 * SWR fetcher
 */
export const swrFetcher = <T, P = Api.BaseParams>(url: string, params?: P) => {
    return coreRequest<T, P>({ url, params }).then((res) => res.data);
};

/**
 * ProTable request adapter
 */
export const proTableRequest = <T, P = Api.BaseParams>(
    props: RequestOptions<P>
): Promise<Api.PageResponse<T>> => {
    return coreRequest<T, P>(props).then((res) => ({
        data: res.data as T[],
        total: res.total || 0,
        success: true,
    }));
};

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

interface RequestOptions<P = Api.BaseParams> extends RequestInit {
    /**
     * Request url
     */
    url: string;
    /**
     * Request params
     */
    params?: P;
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
 * Core request function with unified error and success handling
 */
const coreRequest = async <T, P>({
    url,
    params,
    ...options
}: RequestOptions<P>): Promise<Api.ApiResponse<T>> => {
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

    if (["PUT", "POST"].includes(options.method || "GET")) {
        config.body = options.body || JSON.stringify(params);
    } else {
        url += buildQueryString(params);
    }
    try {
        const response = await fetch(url, config);
        if (!response.ok) {
            throw response;
        }
        const result = await response.json();
        if (result.code !== 0) {
            return Promise.reject(result);
        }
        return result;
    } catch (error) {
        return handleError(error);
    } finally {
        requestPool.delete(controller);
    }
};

/**
 * Handle all errors
 */
const handleError = async (error: unknown) => {
    const response = error as Response;
    const statusCode = response.status;
    if (error instanceof DOMException && error.name === "AbortError") {
        console.warn("Request aborted");
    } else if (statusCode === 401) {
        useAuthStore.getState().clearAuth();
        requestPool.forEach((controller) => {
            if (!controller.signal.aborted) {
                controller.abort();
            }
        });
        messageApi.error(
            "Invalid session or session expired, please login again."
        );
    } else if (statusCode >= 500) {
        messageApi.error(`Server error: ${response.statusText}`);
        return Promise.reject(new Error(response.statusText));
    } else {
        messageApi.error(`Request failed: ${error}`);
        return Promise.reject(error);
    }

    // throw error;
    return Promise.reject(error);
};

/**
 * Safe params conversion
 */
const buildQueryString = <P>(params?: P): string => {
    if (!params) return "";
    const searchParams = new URLSearchParams(params);
    const query = searchParams.toString();
    return query ? `?${query}` : "";
};
