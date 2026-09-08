import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createRouter, RouterProvider } from "@tanstack/react-router";

import "antd/dist/reset.css";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { bindAuthenticatedQueryCache } from "@/lib/auth-query-cache";
import { LocaleProvider } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

import { routeTree } from "./routeTree.gen";

const queryClient = new QueryClient({
    defaultOptions: {
        queries: {
            refetchInterval: 1000 * 60 * 10,
            refetchIntervalInBackground: true,
            staleTime: Infinity,
            gcTime: Infinity,
            refetchOnWindowFocus: false,
            refetchOnReconnect: false,
        },
    },
});

bindAuthenticatedQueryCache(queryClient, useAuthStore);

const router = createRouter({
    routeTree,
    context: {
        queryClient,
    },
    defaultPreload: "intent",
    scrollRestoration: true,
    defaultStructuralSharing: true,
    defaultPreloadStaleTime: 0,
});

// Register the router instance for type safety
declare module "@tanstack/react-router" {
    interface Register {
        router: typeof router;
    }
    interface StaticDataRouteOption {
        headerTitle?: string;
        headerHide?: boolean;
        headerBack?: boolean;
    }
}

// Render the root
const rootElement = document.getElementById("root");
if (rootElement && !rootElement.innerHTML) {
    const root = createRoot(rootElement);
    root.render(
        <StrictMode>
            <QueryClientProvider client={queryClient}>
                <LocaleProvider>
                    <RouterProvider router={router} />
                </LocaleProvider>
            </QueryClientProvider>
        </StrictMode>,
    );
}
