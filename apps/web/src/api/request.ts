import { appMessage } from "@/api/runtime";
import { localizeApiError } from "@/lib/builtin-i18n";
import { t } from "@/lib/i18n";
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
    const result = await executeJsonRequest<Api.ApiResponse<T>>(url, config);

    return props.raw ? result : result.data;
}

/** Orval mutator: generated clients retain the same auth/error semantics. */
export const generatedApiRequest = <T>(url: string, options: RequestInit): Promise<T> =>
    executeJsonRequest<T>(url, withDefaultAndAuthHeaders(options));

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

export const apiBlob = async (options: RequestOptions): Promise<Blob | null> => {
    const { url, config } = formatFetchConfig(options);
    const response = await fetch(url, config);
    if (response.status === 204) {
        return null;
    }
    if (!response.ok) {
        return handleError(response);
    }
    return response.blob();
};

const getAuthHeaders = (): Record<string, string> => {
    const token = useAuthStore.getState().token;
    return token ? { Authorization: `Bearer ${token}` } : {};
};

export const apiUpload = async <T>(url: string, formData: FormData): Promise<T> => {
    const response = await fetch(url, {
        method: "POST",
        headers: getAuthHeaders(),
        body: formData,
    });
    if (!response.ok) {
        return handleError(response);
    }

    const result = (await response.json()) as Api.ApiResponse<T>;
    if (result.code !== 0) {
        const message = localizeApiError(
            result.code,
            result.message || response.statusText || t("上传失败", "Upload failed"),
        );
        appMessage.error(message);
        return Promise.reject(new Error(message));
    }
    return result.data;
};

const defaultHeaders = {
    "Content-Type": "application/json",
};

interface RequestOptions<P = Api.BaseParams> extends RequestInit {
    url: string;
    params?: P;
    query?: Api.BaseParams;
    raw?: boolean;
}

const withDefaultAndAuthHeaders = (options: RequestInit): RequestInit => {
    const headers = new Headers(defaultHeaders);
    new Headers(options.headers).forEach((value, key) => {
        headers.set(key, value);
    });
    new Headers(getAuthHeaders()).forEach((value, key) => {
        headers.set(key, value);
    });

    return { ...options, headers };
};

const formatFetchConfig = <T>({ params, query, url, ...options }: RequestOptions<T>) => {
    const config = withDefaultAndAuthHeaders(options);
    url = appendQueryString(url, query);
    if (["PUT", "POST", "PATCH"].includes(options.method || "GET")) {
        config.body = options.body || JSON.stringify(params);
    } else {
        url = appendQueryString(url, params);
    }
    return { url, config };
};

const executeJsonRequest = async <T>(url: string, config: RequestInit): Promise<T> => {
    const response = await fetch(url, config);
    if (!response.ok) return handleError(response);
    const result = (await response.json()) as T;
    if (typeof result === "object" && result !== null && "code" in result && result.code !== 0) {
        const envelope = result as { code: number; message?: string };
        const message = localizeApiError(
            envelope.code,
            envelope.message || response.statusText || t("请求失败", "Request failed"),
        );
        appMessage.error(message);
        return Promise.reject(new Error(message));
    }
    return result;
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
    const fallbackMessage = payload?.message || error.statusText || t("请求失败", "Request failed");
    const message = localizeApiError(payload?.code, fallbackMessage);

    if (error.status === 401) {
        if (requestUrl.includes("/api/auth/login")) {
            appMessage.error(message || t("用户名或密码错误。", "Invalid username or password."));
            return Promise.reject(error);
        }

        useAuthStore.getState().clearAuth();
        if (window.location.pathname !== "/login") {
            window.location.replace("/login");
        }
        return Promise.reject(error);
    }

    if (error.status >= 500 && requestUrl.includes("/api/auth/")) {
        useAuthStore.getState().clearAuth();
        if (window.location.pathname !== "/login") {
            window.location.replace("/login");
        }
        appMessage.error(message);
        return Promise.reject(error);
    }

    appMessage.error(message);
    return Promise.reject(error);
};

const readErrorPayload = async (
    response: Response,
): Promise<{ code?: number; message?: string } | null> => {
    try {
        return (await response.clone().json()) as { code?: number; message?: string };
    } catch {
        const text = await response.clone().text();
        return text ? { message: text } : null;
    }
};

const buildQueryString = <P>(params?: P): string => {
    if (!params) return "";
    const searchParams = new URLSearchParams();

    const appendQueryValue = (key: string, value: unknown): void => {
        if (value === undefined || value === null) return;

        if (Array.isArray(value)) {
            value.forEach((item) => {
                appendQueryValue(key, item);
            });
            return;
        }

        if (value instanceof Date) {
            searchParams.append(key, value.toISOString());
            return;
        }

        if (typeof value === "number" || typeof value === "boolean" || typeof value === "bigint") {
            searchParams.append(key, `${value}`);
            return;
        }

        if (typeof value === "string") {
            searchParams.append(key, value);
            return;
        }

        if (
            Object.prototype.toString.call(value) === "[object Object]" ||
            value instanceof String ||
            value instanceof Number ||
            value instanceof Boolean
        ) {
            searchParams.append(key, JSON.stringify(value));
            return;
        }

        if (typeof value === "object") {
            throw new Error(`Unsupported query param value for "${key}".`);
        }

        throw new Error(`Unsupported query param value for "${key}".`);
    };

    Object.entries(params as Record<string, unknown>).forEach(([key, value]) => {
        appendQueryValue(key, value);
    });

    const query = searchParams.toString();
    return query ? `?${query}` : "";
};

const appendQueryString = <P>(url: string, params?: P): string => {
    const query = buildQueryString(params);
    if (!query) return url;
    return url.includes("?") ? `${url}&${query.slice(1)}` : `${url}${query}`;
};
