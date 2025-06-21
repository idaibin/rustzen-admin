import { ProTable } from "@ant-design/pro-components";
import type { ProColumns } from "@ant-design/pro-components";
import { requestTable } from "@/services/api";
import { Tag } from "antd";
import type * as Log from "Log";

const LogPage = () => {
  const columns: ProColumns<Log.Item>[] = [
    {
      title: "ID",
      dataIndex: "id",
      width: 48,
    },
    {
      title: "日志级别",
      dataIndex: "level",
      render: (_, record) => {
        if (record.level === "INFO")
          return <Tag color="processing">{record.level}</Tag>;
        if (record.level === "WARN")
          return <Tag color="warning">{record.level}</Tag>;
        if (record.level === "ERROR")
          return <Tag color="error">{record.level}</Tag>;
        return <Tag>{record.level}</Tag>;
      },
    },
    {
      title: "消息",
      dataIndex: "message",
      ellipsis: true,
    },
    {
      title: "创建时间",
      dataIndex: "createdAt",
      valueType: "dateTime",
    },
  ];

  return (
    <ProTable<Log.Item>
      columns={columns}
      request={async (params) => {
        return requestTable<Log.Item>("/api/sys/log", params);
      }}
      rowKey="id"
      search={{
        labelWidth: "auto",
      }}
      headerTitle="日志列表"
    />
  );
};

export default LogPage;
