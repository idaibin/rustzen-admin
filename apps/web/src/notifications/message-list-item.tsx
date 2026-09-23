import { Button, List, Space, Typography } from "antd";

import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

export const MessageListItem = ({
    item,
    readPending,
    onOpen,
    onRead,
}: {
    item: Notifications.Item;
    readPending: boolean;
    onOpen: () => void;
    onRead: () => void;
}) => (
    <List.Item
        actions={
            !item.readAt
                ? [
                      <Button key="read" type="link" loading={readPending} onClick={onRead}>
                          {t("已读", "Read")}
                      </Button>,
                  ]
                : undefined
        }
    >
        <button
            type="button"
            className="min-w-0 flex-1 cursor-pointer border-0 bg-transparent p-0 text-start"
            aria-label={t(`打开消息：${item.title}`, `Open message: ${item.title}`)}
            onClick={onOpen}
        >
            <List.Item.Meta
                title={
                    <Space>
                        <Typography.Text strong={!item.readAt}>{item.title}</Typography.Text>
                        {!item.readAt ? (
                            <span
                                aria-label={t("未读", "Unread")}
                                className="size-2 rounded-full bg-primary"
                            />
                        ) : null}
                    </Space>
                }
                description={
                    <>
                        <Typography.Text type="secondary">{item.producer}</Typography.Text>
                        <Typography.Paragraph ellipsis={{ rows: 2 }} className="mb-1!">
                            {item.summary}
                        </Typography.Paragraph>
                        <Typography.Text type="secondary">
                            {formatDateTime(item.occurredAt)}
                        </Typography.Text>
                    </>
                }
            />
        </button>
    </List.Item>
);
