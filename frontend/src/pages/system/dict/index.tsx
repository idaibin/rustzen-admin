import { ProTable } from "@ant-design/pro-components";
import type { ProColumns } from "@ant-design/pro-components";
import { requestTable } from "@/services/api";
import type * as Dict from "Dict";
import { Tag } from "antd";

const DictPage = () => {
  const columns: ProColumns<Dict.Item>[] = [
    {
      title: "ID",
      dataIndex: "id",
      width: 48,
    },
    {
      title: "字典类型",
      dataIndex: "dictType",
    },
    {
      title: "标签",
      dataIndex: "label",
    },
    {
      title: "值",
      dataIndex: "value",
    },
    {
      title: "默认",
      dataIndex: "isDefault",
      render: (_, record) =>
        record.isDefault ? <Tag color="success">是</Tag> : "否",
    },
    {
      title: "操作",
      key: "action",
      render: () => [<a>编辑</a>, <a>删除</a>],
    },
  ];

  return (
    <ProTable<Dict.Item>
      columns={columns}
      request={async (params) => {
        return requestTable<Dict.Item>("/api/sys/dict", params);
      }}
      rowKey="id"
      search={{
        labelWidth: "auto",
      }}
      headerTitle="字典列表"
    />
  );
};

export default DictPage;
