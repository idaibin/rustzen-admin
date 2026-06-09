import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Input, Segmented, Tag } from "antd";
import { useMemo, useState } from "react";

import { manageAPI } from "@/api";
import { AuthWrap } from "@/components/base-auth";
import { useLocalStore } from "@/store/useLocalStore";

export const Route = createFileRoute("/manage/log")({
    component: LogPage,
});

const DEFAULT_ACTION = "AUTH_LOGIN";
const ALL_ACTION = "all";

const actionOptions: Array<{ label: string; value: string }> = [
    { label: "All", value: ALL_ACTION },
    { label: "Login", value: DEFAULT_ACTION },
    { label: "GET", value: "HTTP_GET" },
    { label: "POST", value: "HTTP_POST" },
    { label: "PUT", value: "HTTP_PUT" },
    { label: "DELETE", value: "HTTP_DELETE" },
];

function LogPage() {
    const [savedActionType, setActionType] = useLocalStore("log-action", DEFAULT_ACTION);
    const actionType = savedActionType || DEFAULT_ACTION;
    const selectedAction = actionType === ALL_ACTION ? undefined : actionType;
    const [searchInput, setSearchInput] = useState("");
    const [searchKeyword, setSearchKeyword] = useState("");
    const params = useMemo(
        () => ({
            action: selectedAction,
            search: searchKeyword || undefined,
        }),
        [searchKeyword, selectedAction],
    );

    return (
        <div className="manage-log-page">
            <ProTable<Log.Item>
                rowKey="id"
                search={false}
                scroll={{ y: "calc(100vh - 287px)" }}
                columns={columns}
                params={params}
                request={manageAPI.log.list}
                headerTitle="Log"
                toolBarRender={() => [
                    <Segmented
                        key="action"
                        value={actionType}
                        options={actionOptions}
                        onChange={(value) => {
                            setActionType(value as string);
                        }}
                    />,
                    <Input.Search
                        key="search"
                        allowClear
                        placeholder="Search user or IP"
                        value={searchInput}
                        style={{ width: 240 }}
                        onChange={(event) => {
                            const value = event.target.value;
                            setSearchInput(value);
                            if (!value) {
                                setSearchKeyword("");
                            }
                        }}
                        onSearch={(value) => {
                            setSearchKeyword(value.trim());
                        }}
                    />,
                    <AuthWrap key="export" code="manage:log:export">
                        <Button
                            type="primary"
                            onClick={() => {
                                void manageAPI.log.export();
                            }}
                        >
                            Export
                        </Button>
                    </AuthWrap>,
                ]}
            />
        </div>
    );
}

const actionColorMap: Record<string, string> = {
    HTTP_GET: "default",
    HTTP_POST: "processing",
    HTTP_PUT: "warning",
    HTTP_DELETE: "error",
    AUTH_LOGIN: "success",
};
const columns: ProColumns<Log.Item>[] = [
    {
        title: "ID",
        dataIndex: "id",
        width: 80,
        search: false,
    },
    {
        title: "User",
        dataIndex: "username",
        width: 120,
        render: (_, record) => record.username || "Anonymous User",
    },
    {
        title: "Action",
        dataIndex: "action",
        width: 150,
        search: false,
        render: (_, record) => {
            const action = record.action;
            const color = actionColorMap[action];
            return (
                <Tag color={color} variant="outlined">
                    {action}
                </Tag>
            );
        },
    },
    {
        title: "Description",
        dataIndex: "description",
        ellipsis: true,
    },
    {
        title: "Status",
        dataIndex: "status",
        width: 100,
        search: false,
        render: (_, record) => {
            const status = record.status;
            const color = status === "SUCCESS" ? "success" : "error";
            return (
                <Tag color={color} variant="solid">
                    {status}
                </Tag>
            );
        },
    },
    {
        title: "IP Address",
        dataIndex: "ipAddress",
        width: 120,
        render: (_, record) => record.ipAddress || "-",
    },
    {
        title: "Duration",
        dataIndex: "durationMs",
        width: 80,
        search: false,
        render: (_, record) => {
            if (!record.durationMs) return "-";
            return `${record.durationMs}ms`;
        },
    },
    {
        title: "Created At",
        dataIndex: "createdAt",
        valueType: "dateTime",
        width: 160,
        ellipsis: true,
        className: "whitespace-nowrap",
        search: false,
    },
];
