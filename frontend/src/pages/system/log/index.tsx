import { ProTable } from "@ant-design/pro-components";
import type { ProColumns } from "@ant-design/pro-components";
import { Tag } from "antd";
import type { Log } from "System";
import { logAPI } from "@/services";

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
        if (record.level === "info")
          return <Tag color="processing">{record.level}</Tag>;
        if (record.level === "warn")
          return <Tag color="warning">{record.level}</Tag>;
        if (record.level === "error")
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
      request={logAPI.getLogList}
      rowKey="id"
      search={{
        labelWidth: "auto",
      }}
      headerTitle="日志列表"
    />
  );
};

export default LogPage;
