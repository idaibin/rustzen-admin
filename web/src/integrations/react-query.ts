import { useQuery, type UseQueryOptions } from '@tanstack/react-query';

/**
 * Helper function to create query key
 */
export const createQueryKey = (key: string | string[], params?: unknown) => {
  const baseKey = Array.isArray(key) ? key : [key];
  return params ? [...baseKey, params] : baseKey;
};

/**
 * Wrapper for useQuery that works with existing API functions
 */
export function useApiQuery<TData, TError = Error, TParams = unknown>(
  queryKey: string | string[],
  queryFn: (params?: TParams) => Promise<TData>,
  options?: Omit<UseQueryOptions<TData, TError>, 'queryKey' | 'queryFn'> & { params?: TParams },
) {
  const key = createQueryKey(queryKey, options?.params);
  return useQuery<TData, TError>({
    queryKey: key,
    queryFn: () => queryFn(options?.params),
    ...options,
  });
}
