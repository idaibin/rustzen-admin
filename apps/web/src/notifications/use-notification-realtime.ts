import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";

import { notificationQueryKeys } from "@/api/notifications/api";
import { useAuthStore } from "@/store/useAuthStore";

import { NotificationRealtimeClient, type StreamState } from "./realtime-client";

export const useNotificationRealtime = () => {
    const token = useAuthStore((state) => state.token);
    const generation = useAuthStore((state) => state.authGeneration);
    const clearAuth = useAuthStore((state) => state.clearAuth);
    const queryClient = useQueryClient();
    const client = useRef<NotificationRealtimeClient | undefined>(undefined);
    const [state, setState] = useState<StreamState>("connecting");
    const [restartKey, setRestartKey] = useState(0);

    useEffect(() => {
        let active = true;
        const reconcile = () => {
            if (!active) return;
            void queryClient.invalidateQueries({
                queryKey: notificationQueryKeys.inbox(generation),
            });
        };
        const stop = () => {
            void queryClient.cancelQueries({
                queryKey: notificationQueryKeys.generation(generation),
            });
            client.current?.stop();
            client.current = undefined;
        };
        const start = () => {
            stop();
            if (!token || document.visibilityState === "hidden" || !active) return;
            const next = new NotificationRealtimeClient(token, generation, {
                currentGeneration: () => useAuthStore.getState().authGeneration,
                onInvalidate: reconcile,
                onState: (nextState) => {
                    setState(nextState);
                    if (nextState === "forbidden")
                        queryClient.removeQueries({ queryKey: notificationQueryKeys.all });
                },
                onUnauthorized: () => {
                    queryClient.removeQueries({ queryKey: notificationQueryKeys.all });
                    clearAuth();
                    window.location.replace("/login");
                },
            });
            client.current = next;
            next.start();
        };
        const visibility = () => (document.visibilityState === "hidden" ? stop() : start());
        const pageHide = () => stop();
        const pageShow = () => start();
        start();
        const interval = window.setInterval(() => {
            if (document.visibilityState === "visible") reconcile();
        }, 60_000);
        document.addEventListener("visibilitychange", visibility);
        window.addEventListener("pagehide", pageHide);
        window.addEventListener("pageshow", pageShow);
        return () => {
            active = false;
            window.clearInterval(interval);
            document.removeEventListener("visibilitychange", visibility);
            window.removeEventListener("pagehide", pageHide);
            window.removeEventListener("pageshow", pageShow);
            stop();
        };
    }, [clearAuth, generation, queryClient, restartKey, token]);

    return { state, restart: () => setRestartKey((value) => value + 1) };
};
