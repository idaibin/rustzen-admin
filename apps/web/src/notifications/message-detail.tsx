import { useQuery } from "@tanstack/react-query";
import { useRouter } from "@tanstack/react-router";
import { Alert, Button, Divider, Space, Spin, Typography } from "antd";
import { useEffect } from "react";

import { notificationAPI, notificationQueryKeys } from "@/api/notifications/api";
import { ApiRequestError } from "@/api/request";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

import { safeSubjectDestination } from "./subject-navigation";

interface Props {
    id: string;
    generation: number;
    readPending: boolean;
    onRead: (id: string) => void;
    onDismiss: () => void;
}

export const MessageDetail = ({ id, generation, readPending, onRead, onDismiss }: Props) => {
    const router = useRouter();
    const detail = useQuery({
        queryKey: notificationQueryKeys.detail(generation, id),
        queryFn: ({ signal }) => notificationAPI.detail(id, signal),
        retry: false,
        staleTime: 0,
        refetchOnMount: "always",
    });
    useEffect(() => {
        if (
            detail.error instanceof ApiRequestError &&
            [403, 404].includes(detail.error.status ?? 0)
        )
            onDismiss();
    }, [detail.error, onDismiss]);
    const destination = detail.data ? safeSubjectDestination(detail.data) : undefined;
    const navigateToSubject = () => {
        if (!destination) return;
        if (destination.to === "/monitoring/incidents")
            void router.navigate({ to: destination.to, search: destination.search });
        else void router.navigate({ to: destination.to, search: destination.search });
    };
    return (
        <>
            <Divider />
            <section aria-label={t("消息详情", "Message details")}>
                {detail.isPending ? <Spin /> : null}
                {detail.error ? (
                    <Alert
                        type="error"
                        message={t("详情不可用", "Message details are unavailable")}
                    />
                ) : null}
                {detail.data && !detail.error ? (
                    <Space direction="vertical" className="w-full">
                        <Typography.Title level={5}>{detail.data.title}</Typography.Title>
                        <Typography.Text type="secondary">{detail.data.producer}</Typography.Text>
                        <Typography.Paragraph>{detail.data.summary}</Typography.Paragraph>
                        <Typography.Text type="secondary">
                            {formatDateTime(detail.data.occurredAt)}
                        </Typography.Text>
                        <Space>
                            {!detail.data.readAt ? (
                                <Button
                                    loading={readPending}
                                    onClick={() => onRead(detail.data!.id)}
                                >
                                    {t("标为已读", "Mark as read")}
                                </Button>
                            ) : null}
                            {destination ? (
                                <Button
                                    type="primary"
                                    aria-label={t("查看相关记录", "View related record")}
                                    onClick={navigateToSubject}
                                >
                                    {t("查看相关记录", "View related record")}
                                </Button>
                            ) : null}
                        </Space>
                    </Space>
                ) : null}
            </section>
        </>
    );
};
