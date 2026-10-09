import { useInfiniteQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Alert, Button, Drawer, Empty, List, Segmented, Skeleton, Space, Typography } from "antd";
import { useEffect, useMemo, useRef, useState } from "react";

import { notificationAPI, notificationQueryKeys } from "@/api/notifications/api";
import { ApiRequestError } from "@/api/request";
import { useSubmission } from "@/hooks/use-submission";
import { t } from "@/lib/i18n";

import { MessageDetail } from "./message-detail";
import { MessageListItem } from "./message-list-item";
import { classifyWriteFailure, MessageWriteError, visibleMessages } from "./message-write-error";
import type { StreamState } from "./realtime-client";

interface Props {
    open: boolean;
    generation: number;
    onClose: () => void;
    streamState: StreamState;
    onRetryRealtime: () => void;
}

export const MessageDrawer = ({
    open,
    generation,
    onClose,
    streamState,
    onRetryRealtime,
}: Props) => {
    const [unreadOnly, setUnreadOnly] = useState(false);
    const [selectedId, setSelectedId] = useState<string>();
    const detailTrigger = useRef<HTMLElement | null>(null);
    const drawerContent = useRef<HTMLDivElement>(null);
    const restoreDetailFocus = useRef(false);
    const dismissDetail = () => {
        restoreDetailFocus.current = true;
        setSelectedId(undefined);
    };
    useEffect(() => {
        if (selectedId || !restoreDetailFocus.current) return;
        restoreDetailFocus.current = false;
        if (detailTrigger.current?.isConnected) {
            detailTrigger.current.focus();
        } else {
            drawerContent.current
                ?.querySelector<HTMLElement>("input:not(:disabled), button:not(:disabled)")
                ?.focus();
        }
    }, [selectedId]);
    const [hiddenIds, setHiddenIds] = useState<Set<string>>(() => new Set());
    const client = useQueryClient();
    const { submitting, beginSubmission, finishSubmission } = useSubmission();
    const list = useInfiniteQuery({
        queryKey: notificationQueryKeys.list(generation, unreadOnly),
        queryFn: ({ pageParam, signal }) =>
            notificationAPI.list({ cursor: pageParam ?? undefined, limit: 20, unreadOnly }, signal),
        initialPageParam: null as string | null,
        getNextPageParam: (last) => last.nextCursor ?? undefined,
        enabled: open,
        retry: false,
    });
    const refresh = async () => {
        await client.invalidateQueries({ queryKey: notificationQueryKeys.generation(generation) });
    };
    const markRead = useMutation({
        onSettled: finishSubmission,
        mutationFn: notificationAPI.markRead,
        onSuccess: refresh,
        onError: (error, id) => {
            const failure = classifyWriteFailure(error);
            if (failure === "missing") {
                setHiddenIds((current) => new Set(current).add(id));
                if (selectedId === id) dismissDetail();
            }
            if (failure === "forbidden") dismissDetail();
        },
    });
    const markAll = useMutation({
        onSettled: finishSubmission,
        mutationFn: notificationAPI.markAllRead,
        onSuccess: refresh,
    });
    useEffect(() => {
        setSelectedId(undefined);
        setHiddenIds(new Set());
        markRead.reset();
        markAll.reset();
    }, [generation, unreadOnly]);

    const status =
        streamState === "reconnecting"
            ? t(
                  "实时连接正在恢复，当前内容来自持久收件箱。",
                  "Realtime connection is recovering; durable inbox data remains available.",
              )
            : streamState === "forbidden"
              ? t(
                    "当前权限不可使用消息中心。",
                    "The message center is unavailable for the current permissions.",
                )
              : streamState === "error"
                ? t("实时更新已停止，请手动重试。", "Realtime updates stopped. Retry manually.")
                : undefined;
    const listForbidden = list.error instanceof ApiRequestError && list.error.status === 403;
    const writeForbidden = [markRead.error, markAll.error].some(
        (error) => error && classifyWriteFailure(error) === "forbidden",
    );
    const forbidden = listForbidden || writeForbidden;
    const items = useMemo(
        () =>
            visibleMessages(
                list.data?.pages.flatMap((page) => page.items) ?? [],
                forbidden,
                hiddenIds,
            ),
        [forbidden, hiddenIds, list.data],
    );
    const first = forbidden ? undefined : list.data?.pages[0];
    const initialError = Boolean(list.error && (!list.data || forbidden));
    const readMessage = (id: string) => {
        if (!forbidden && beginSubmission()) markRead.mutate(id);
    };
    const readAll = (snapshot: string) => {
        if (!forbidden && beginSubmission()) markAll.mutate(snapshot);
    };

    return (
        <Drawer title={t("消息中心", "Message center")} open={open} onClose={onClose} width={440}>
            <Space ref={drawerContent} direction="vertical" size="middle" className="w-full">
                {status ? (
                    <Alert
                        type={streamState === "forbidden" ? "warning" : "info"}
                        showIcon
                        message={status}
                        action={
                            streamState === "error" ? (
                                <Button size="small" onClick={onRetryRealtime}>
                                    {t("重试", "Retry")}
                                </Button>
                            ) : undefined
                        }
                    />
                ) : null}
                <div className="flex items-center justify-between gap-3">
                    <Segmented
                        aria-label={t("消息筛选", "Message filter")}
                        value={unreadOnly ? "unread" : "all"}
                        onChange={(value) => setUnreadOnly(value === "unread")}
                        options={[
                            { label: t("全部", "All"), value: "all" },
                            { label: t("未读", "Unread"), value: "unread" },
                        ]}
                    />
                    <Button
                        aria-label={t("全部已读", "Mark all read")}
                        disabled={!first?.snapshot || submitting}
                        loading={markAll.isPending}
                        onClick={() => first?.snapshot && readAll(first.snapshot)}
                    >
                        {t("全部已读", "Mark all read")}
                    </Button>
                </div>
                {initialError ? (
                    <Alert
                        type={forbidden ? "warning" : "error"}
                        showIcon
                        message={
                            forbidden
                                ? t("消息中心不可用", "Message center unavailable")
                                : t("消息加载失败", "Messages could not be loaded")
                        }
                        action={
                            !forbidden ? (
                                <Button
                                    size="small"
                                    loading={list.isFetching}
                                    onClick={() => void list.refetch()}
                                >
                                    {t("重试", "Retry")}
                                </Button>
                            ) : undefined
                        }
                    />
                ) : null}
                {list.error && list.data && !forbidden ? (
                    <Alert
                        type="warning"
                        showIcon
                        message={t(
                            "后台刷新失败，正在显示上次结果。",
                            "Background refresh failed; showing the last result.",
                        )}
                        action={
                            <Button
                                size="small"
                                loading={list.isFetching}
                                onClick={() => void list.refetch()}
                            >
                                {t("重试", "Retry")}
                            </Button>
                        }
                    />
                ) : null}
                <MessageWriteError
                    error={markRead.error}
                    pending={submitting}
                    onRetry={() => markRead.variables && readMessage(markRead.variables)}
                />
                <MessageWriteError
                    error={markAll.error}
                    pending={submitting}
                    onRetry={() => markAll.variables && readAll(markAll.variables)}
                />
                {list.isPending ? <Skeleton active paragraph={{ rows: 5 }} /> : null}
                {!list.isPending && !initialError && items.length === 0 ? (
                    <Empty
                        description={
                            unreadOnly
                                ? t("没有未读消息", "No unread messages")
                                : t("没有消息", "No messages")
                        }
                    />
                ) : null}
                {items.length ? (
                    <List
                        loading={list.isFetching && !list.isFetchingNextPage}
                        dataSource={items}
                        renderItem={(item) => (
                            <MessageListItem
                                item={item}
                                readPending={markRead.isPending && markRead.variables === item.id}
                                readDisabled={submitting}
                                onOpen={() => {
                                    detailTrigger.current = document.activeElement as HTMLElement;
                                    setSelectedId(item.id);
                                }}
                                onRead={() => readMessage(item.id)}
                            />
                        )}
                    />
                ) : null}
                {list.hasNextPage ? (
                    <Button
                        block
                        loading={list.isFetchingNextPage}
                        disabled={list.isFetchingNextPage}
                        onClick={() => void list.fetchNextPage()}
                    >
                        {t("加载更多", "Load more")}
                    </Button>
                ) : null}
                {!list.hasNextPage && items.length ? (
                    <Typography.Text type="secondary">
                        {t(
                            `仅保留最近 ${first?.retentionDays ?? 30} 天消息`,
                            `Messages are retained for ${first?.retentionDays ?? 30} days`,
                        )}
                    </Typography.Text>
                ) : null}
                {selectedId && !forbidden ? (
                    <MessageDetail
                        id={selectedId}
                        generation={generation}
                        readPending={markRead.isPending && markRead.variables === selectedId}
                        readDisabled={submitting}
                        onRead={readMessage}
                        onDismiss={dismissDetail}
                        onNavigate={onClose}
                    />
                ) : null}
            </Space>
        </Drawer>
    );
};
