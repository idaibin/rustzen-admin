import { apiRequest } from "@/api/request";

export const notificationQueryKeys = {
    all: ["notifications"] as const,
    generation: (generation: number) => ["notifications", generation] as const,
    inbox: (generation: number) => ["notifications", generation, "inbox"] as const,
    count: (generation: number) => ["notifications", generation, "inbox", "unread-count"] as const,
    list: (generation: number, unreadOnly: boolean) =>
        ["notifications", generation, "inbox", "list", unreadOnly] as const,
    detail: (generation: number, id: string) =>
        ["notifications", generation, "detail", id] as const,
};

export const notificationAPI = {
    list: (query: Notifications.ListQuery, signal?: AbortSignal) =>
        apiRequest<Notifications.ListResponse, Notifications.ListQuery>({
            url: "/api/notifications",
            params: query,
            silent: true,
            signal,
        }),
    unreadCount: (signal?: AbortSignal) =>
        apiRequest<Notifications.UnreadCount>({
            url: "/api/notifications/unread-count",
            silent: true,
            signal,
        }),
    detail: (id: string, signal?: AbortSignal) =>
        apiRequest<Notifications.Item>({
            url: `/api/notifications/${encodeURIComponent(id)}`,
            silent: true,
            signal,
        }),
    markRead: (id: string) =>
        apiRequest<Notifications.ReadResponse>({
            url: `/api/notifications/${encodeURIComponent(id)}/read`,
            method: "PUT",
            silent: true,
        }),
    markAllRead: (snapshot: string) =>
        apiRequest<Notifications.ReadAllResponse, { snapshot: string }>({
            url: "/api/notifications/read-all",
            method: "POST",
            params: { snapshot },
            silent: true,
        }),
};
