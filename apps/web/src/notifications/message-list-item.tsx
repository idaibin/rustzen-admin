import { Button, List, Space, Typography } from "antd";

import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

export const MessageListItem = ({
    item,
    readPending,
    readDisabled = false,
    onOpen,
    onRead,
}: {
    item: Notifications.Item;
    readPending: boolean;
    readDisabled?: boolean;
    onOpen: () => void;
    onRead: () => void;
}) => (
    <List.Item
        actions={
            !item.readAt
                ? [
                      <Button
                          key="read"
                          type="link"
                          loading={readPending}
                          disabled={readDisabled}
                          onClick={onRead}
                      >
                          {t("已读", "Read")}
                      </Button>,
                  ]
                : undefined
        }
    >
        <Button
            type="text"
            className="min-w-0 flex-1 !h-auto !justify-start !whitespace-normal !text-start"
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
        </Button>
    </List.Item>
);
