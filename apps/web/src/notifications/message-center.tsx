import { BellOutlined } from "@ant-design/icons";
import { useQuery } from "@tanstack/react-query";
import { Badge, Button, Tooltip } from "antd";
import { useRef, useState } from "react";

import { notificationAPI, notificationQueryKeys } from "@/api/notifications/api";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

import { MessageDrawer } from "./message-drawer";
import { useNotificationRealtime } from "./use-notification-realtime";

export const MessageCenter = () => {
    const [open, setOpen] = useState(false);
    const bell = useRef<HTMLButtonElement>(null);
    const generation = useAuthStore((state) => state.authGeneration);
    const token = useAuthStore((state) => state.token);
    const realtime = useNotificationRealtime();
    const unread = useQuery({
        queryKey: notificationQueryKeys.count(generation),
        queryFn: ({ signal }) => notificationAPI.unreadCount(signal),
        enabled: Boolean(token),
        retry: false,
    });
    const close = () => {
        setOpen(false);
        window.setTimeout(() => bell.current?.focus(), 0);
    };
    return (
        <>
            <Tooltip title={t("消息中心", "Message center")}>
                <Badge count={unread.data?.count ?? 0} overflowCount={99} size="small">
                    <Button
                        ref={bell}
                        type="text"
                        icon={<BellOutlined />}
                        aria-label={t("打开消息中心", "Open message center")}
                        onClick={() => setOpen(true)}
                    />
                </Badge>
            </Tooltip>
            <MessageDrawer
                open={open}
                generation={generation}
                onClose={close}
                streamState={realtime.state}
                onRetryRealtime={realtime.restart}
            />
        </>
    );
};
