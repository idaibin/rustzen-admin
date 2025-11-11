import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

// eslint-disable-next-line react-refresh/only-export-components
export function getContext() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        refetchInterval: 1000 * 60 * 10,
        refetchIntervalInBackground: true,
        staleTime: Infinity, // 数据始终 fresh（避免 focus 触发额外请求）
        gcTime: Infinity, // 即使卸载也别清理（大屏一般不会卸载，但这样最保险）
        refetchOnWindowFocus: false, // 大屏没必要聚焦时刷新
        refetchOnReconnect: false, // 网络抖动时避免打爆接口
      },
    },
  });
  return {
    queryClient,
  };
}

export function Provider({ children, queryClient }: { children: React.ReactNode; queryClient: QueryClient }) {
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}
