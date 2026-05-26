import { appMessage } from "@/api/runtime";
import { useAuthStore } from "@/store/useAuthStore";

export function apiRequest<T, P = Api.BaseParams>(
    props: RequestOptions<P> & { raw: true },
): Promise<Api.ApiResponse<T>>;
export function apiRequest<T, P = Api.BaseParams>(
    props: RequestOptions<P> & { raw?: false },
): Promise<T>;
export async function apiRequest<T, P = Api.BaseParams>(
    props: RequestOptions<P>,
): Promise<T | Api.ApiResponse<T>> {
    const { url, config } = formatFetchConfig(props);
    const response = await fetch(url, config);
    if (!response.ok) {
        return handleError(response);
    }

    const result = (await response.json()) as Api.ApiResponse<T>;
    if (result.code !== 0) {
        appMessage.error(result.message || response.statusText || "Request failed");
        return Promise.reject(new Error(result.message || response.statusText));
    }

    return props.raw ? result : result.data;
}

export const apiDownload = async ({
    filename,
    ...options
}: RequestOptions & { filename?: string }): Promise<string> => {
    const { url, config } = formatFetchConfig(options);
    const response = await fetch(url, config);
    if (!response.ok) {
        return handleError(response);
    }
    const blob = await response.blob();
    const contentDisposition = response.headers.get("content-disposition");
    const fileName = contentDisposition?.split("filename=")[1] || filename;
    const downloadName = fileName || `${Date.now()}.bin`;
    const downloadUrl = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = downloadUrl;
    link.download = downloadName;
    document.body.appendChild(link);
    link.click();
    URL.revokeObjectURL(downloadUrl);
    document.body.removeChild(link);
    return downloadName;
};

const getAuthHeaders = (): Record<string, string> => {
    const token = useAuthStore.getState().token;
    return token ? { Authorization: `Bearer ${token}` } : {};
};

const defaultHeaders = {
    "Content-Type": "application/json",
};

interface RequestOptions<P = Api.BaseParams> extends RequestInit {
    url: string;
    params?: P;
    raw?: boolean;
}

const formatFetchConfig = <T>({ params, url, ...options }: RequestOptions<T>) => {
    const headers = new Headers(defaultHeaders);
    new Headers(options.headers).forEach((value, key) => {
        headers.set(key, value);
    });
    new Headers(getAuthHeaders()).forEach((value, key) => {
        headers.set(key, value);
    });

    const config: RequestInit = {
        ...options,
        headers,
    };
    if (["PUT", "POST"].includes(options.method || "GET")) {
        config.body = options.body || JSON.stringify(params);
    } else {
        url += buildQueryString(params);
    }
    return { url, config };
};

const handleError = async (error: unknown) => {
    if (error instanceof DOMException && error.name === "AbortError") {
        return Promise.reject(error);
    }

    if (!(error instanceof Response)) {
        return Promise.reject(error);
    }

    const payload = await readErrorPayload(error);
    const requestUrl = error.url || "";
    const message = payload?.message || error.statusText || "Request failed";

    if (error.status === 401) {
        if (requestUrl.includes("/api/auth/login")) {
            appMessage.error(message || "Invalid username or password.");
            return Promise.reject(error);
        }

        useAuthStore.getState().clearAuth();
        if (window.location.pathname !== "/login") {
            window.location.replace("/login");
        }
        return Promise.reject(error);
    }

    appMessage.error(message);
    return Promise.reject(error);
};

const readErrorPayload = async (response: Response): Promise<{ message?: string } | null> => {
    try {
        return (await response.clone().json()) as { message?: string };
    } catch {
        return null;
    }
};

const buildQueryString = <P>(params?: P): string => {
    if (!params) return "";
    const searchParams = new URLSearchParams(params as Record<string, string>);
    const query = searchParams.toString();
    return query ? `?${query}` : "";
};
