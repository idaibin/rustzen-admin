import { ProTable } from "@ant-design/pro-components";
import type { ProColumns } from "@ant-design/pro-components";
import type { Dict } from "System";
import { dictAPI } from "@/services";
import { Tag, Space } from "antd";

const DictPage = () => {
  const columns: ProColumns<Dict.Item>[] = [
    {
      title: "ID",
      dataIndex: "id",
      width: 48,
    },
    {
      title: "Dict Type",
      dataIndex: "dictType",
    },
    {
      title: "Label",
      dataIndex: "label",
    },
    {
      title: "Value",
      dataIndex: "value",
    },
    {
      title: "Default",
      dataIndex: "isDefault",
      render: (_, record) =>
        record.isDefault ? <Tag color="success">Yes</Tag> : "No",
    },
    {
      title: "Actions",
      key: "action",
      width: 110,
      render: () => (
        <Space size="middle">
          <a>Edit</a>
          <a>Delete</a>
        </Space>
      ),
    },
  ];

  return (
    <ProTable<Dict.Item>
      search={false}
      columns={columns}
      request={dictAPI.getDictList}
      rowKey="id"
      headerTitle="Dictionary List"
    />
  );
};

export default DictPage;
