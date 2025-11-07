import { messageApi } from '@/main';
import { useAuthStore } from '@/stores/useAuthStore';

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
 * Download file
 */
export const apiDownload = async <T>({
  filename,
  ...options
}: RequestOptions<T> & { filename?: string }): Promise<string> => {
  const { url, config, reqDelete } = formatFetchConfig(options);
  try {
    const response = await fetch(url, config);
    if (!response.ok) {
      throw response;
    }
    return downloadFile(response, filename);
  } catch (error) {
    return handleError(error);
  } finally {
    reqDelete();
  }
};
/**
 * ProTable request adapter
 */
export const proTableRequest = <T, P = Api.BaseParams>(
  props: RequestOptions<P>,
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
export const getAuthHeaders = (): Record<string, string> => {
  const token = useAuthStore.getState().token;
  return token ? { Authorization: `Bearer ${token}` } : {};
};

/**
 * Default request headers
 */
const defaultHeaders = {
  'Content-Type': 'application/json',
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
const coreRequest = async <T, P>(options: RequestOptions<P>): Promise<Api.ApiResponse<T>> => {
  const { url, config, reqDelete } = formatFetchConfig(options);
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
    reqDelete();
  }
};

const formatFetchConfig = <T>({ params, url, ...options }: RequestOptions<T>) => {
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
  if (['PUT', 'POST'].includes(options.method || 'GET')) {
    config.body = options.body || JSON.stringify(params);
  } else {
    url += buildQueryString(params);
  }
  return {
    url,
    config,
    reqDelete() {
      requestPool.delete(controller);
    },
  };
};

/**
 * Handle all errors
 */
const handleError = async (error: unknown) => {
  const response = error as Response;
  const statusCode = response.status;
  if (error instanceof DOMException && error.name === 'AbortError') {
    console.warn('Request aborted');
  } else if (statusCode === 401) {
    useAuthStore.getState().clearAuth();
    requestPool.forEach((controller) => {
      if (!controller.signal.aborted) {
        controller.abort();
      }
    });
    messageApi.error('Invalid session or session expired, please login again.');
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
  if (!params) return '';
  const searchParams = new URLSearchParams(params);
  const query = searchParams.toString();
  return query ? `?${query}` : '';
};

/**
 * Download file
 */
const downloadFile = async (response: Response, defaultName?: string): Promise<string> => {
  const blob = await response.blob();
  const contentDisposition = response.headers.get('content-disposition');
  const filename = contentDisposition?.split('filename=')[1] || defaultName;
  const downloadName = filename || `${Date.now()}${getFileExt(blob.type)}`;
  return new Promise((resolve) => {
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = downloadName;
    document.body.appendChild(a);
    a.click();
    URL.revokeObjectURL(url);
    document.body.removeChild(a);
    resolve(downloadName);
  });
};

// 根据 blob 类型确定文件扩展名
const getFileExt = (mimeType: string): string => {
  const mimeToExt: Record<string, string> = {
    // 文本文件
    'text/plain': '.txt',
    'text/csv': '.csv',
    'text/html': '.html',
    'text/css': '.css',
    'text/javascript': '.js',
    'text/xml': '.xml',
    'text/markdown': '.md',

    // 文档文件
    'application/pdf': '.pdf',
    'application/msword': '.doc',
    'application/vnd.openxmlformats-officedocument.wordprocessingml.document': '.docx',
    'application/vnd.ms-excel': '.xls',
    'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet': '.xlsx',
    'application/vnd.ms-powerpoint': '.ppt',
    'application/vnd.openxmlformats-officedocument.presentationml.presentation': '.pptx',

    // 压缩文件
    'application/zip': '.zip',
    'application/x-rar-compressed': '.rar',
    'application/x-7z-compressed': '.7z',
    'application/gzip': '.gz',
    'application/x-tar': '.tar',

    // 图片文件
    'image/jpeg': '.jpg',
    'image/png': '.png',
    'image/gif': '.gif',
    'image/svg+xml': '.svg',
    'image/webp': '.webp',
    'image/bmp': '.bmp',
    'image/tiff': '.tiff',

    // 音频文件
    'audio/mpeg': '.mp3',
    'audio/wav': '.wav',
    'audio/ogg': '.ogg',
    'audio/mp4': '.m4a',

    // 视频文件
    'video/mp4': '.mp4',
    'video/avi': '.avi',
    'video/quicktime': '.mov',
    'video/x-msvideo': '.avi',
    'video/webm': '.webm',

    // JSON 和 XML
    'application/json': '.json',
    'application/xml': '.xml',

    // 其他常见类型
    'application/octet-stream': '.bin',
    'application/x-binary': '.bin',
  };

  return mimeToExt[mimeType] || '.bin';
};
