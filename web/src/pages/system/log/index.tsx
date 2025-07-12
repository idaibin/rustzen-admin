import { ProTable } from "@ant-design/pro-components";
import type { ProColumns } from "@ant-design/pro-components";
import { Tag } from "antd";
import type { Log } from "System";
import { logAPI } from "@/services";

export default function LogPage() {
    const columns: ProColumns<Log.Item>[] = [
        {
            title: "ID",
            dataIndex: "id",
            width: 80,
            search: false,
        },
        {
            title: "用户",
            dataIndex: "username",
            width: 120,
            render: (_, record) => record.username || "匿名用户",
        },
        {
            title: "操作",
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
            title: "描述",
            dataIndex: "description",
            ellipsis: true,
            search: false,
        },
        {
            title: "状态",
            dataIndex: "status",
            width: 100,
            render: (_, record) => {
                const status = record.status;
                const color = status === "SUCCESS" ? "success" : "error";
                return <Tag color={color}>{status}</Tag>;
            },
            valueEnum: {
                SUCCESS: { text: "成功", status: "Success" },
                ERROR: { text: "错误", status: "Error" },
                FAILED: { text: "失败", status: "Error" },
            },
        },
        {
            title: "IP地址",
            dataIndex: "ipAddress",
            width: 120,
            search: false,
            render: (_, record) => record.ipAddress || "-",
        },
        {
            title: "资源类型",
            dataIndex: "resourceType",
            width: 100,
            search: false,
            render: (_, record) => record.resourceType || "-",
        },
        {
            title: "耗时",
            dataIndex: "durationMs",
            width: 80,
            search: false,
            render: (_, record) => {
                if (!record.durationMs) return "-";
                return `${record.durationMs}ms`;
            },
        },
        {
            title: "创建时间",
            dataIndex: "createdAt",
            width: 180,
            valueType: "dateTime",
            search: false,
        },
    ];

    return (
        <ProTable<Log.Item>
            rowKey="id"
            headerTitle="操作日志"
            columns={columns}
            request={logAPI.getLogList}
            search={false}
        />
    );
}
