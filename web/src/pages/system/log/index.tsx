import { ProTable } from "@ant-design/pro-components";
import type { ProColumns } from "@ant-design/pro-components";
import { Tag } from "antd";
import type { Log } from "System";
import { logAPI } from "@/services";

export default function LogPage() {
    return (
        <ProTable<Log.Item>
            rowKey="id"
            search={false}
            scroll={{ y: "calc(100vh - 287px)" }}
            headerTitle={"Operation Log"}
            columns={columns}
            request={logAPI.getLogList}
        />
    );
}

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
        render: (_, record) => {
            const action = record.action;
            let color = "default";
            if (action.startsWith("HTTP_")) {
                color = "blue";
            } else if (action.includes("CREATE")) {
                color = "green";
            } else if (action.includes("UPDATE")) {
                color = "orange";
            } else if (action.includes("DELETE")) {
                color = "red";
            }
            return <Tag color={color}>{action}</Tag>;
        },
    },
    {
        title: "Description",
        dataIndex: "description",
        ellipsis: true,
        search: false,
    },
    {
        title: "Status",
        dataIndex: "status",
        width: 100,
        render: (_, record) => {
            const status = record.status;
            const color = status === "SUCCESS" ? "success" : "error";
            return <Tag color={color}>{status}</Tag>;
        },
        valueEnum: {
            SUCCESS: { text: "Success", status: "Success" },
            ERROR: { text: "Error", status: "Error" },
            FAILED: { text: "Failed", status: "Error" },
        },
    },
    {
        title: "IP Address",
        dataIndex: "ipAddress",
        width: 120,
        search: false,
        render: (_, record) => record.ipAddress || "-",
    },
    {
        title: "Resource Type",
        dataIndex: "resourceType",
        width: 120,
        search: false,
        render: (_, record) => record.resourceType || "-",
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
        width: 180,
        valueType: "dateTime",
        search: false,
    },
];
