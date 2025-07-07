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
            width: 48,
        },
        {
            title: "Level",
            dataIndex: "level",
            render: (_, record) => {
                const level = record.level?.toUpperCase();
                if (level === "INFO")
                    return <Tag color="processing">{level}</Tag>;
                if (level === "WARN") return <Tag color="warning">{level}</Tag>;
                if (level === "ERROR") return <Tag color="error">{level}</Tag>;
                return <Tag>{level}</Tag>;
            },
        },
        {
            title: "Message",
            dataIndex: "message",
            ellipsis: true,
        },
        {
            title: "Created At",
            dataIndex: "createdAt",
            valueType: "dateTime",
        },
    ];

    return (
        <ProTable<Log.Item>
            search={false}
            columns={columns}
            request={logAPI.getLogList}
            rowKey="id"
            headerTitle="Log List"
        />
    );
}
