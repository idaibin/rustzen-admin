import { ProTable } from "@ant-design/pro-components";
import type { ProColumns } from "@ant-design/pro-components";
import { menuAPI } from "@/services";
import type { Menu } from "System";
import { Space } from "antd";

const MenuPage = () => {
  const columns: ProColumns<Menu.Item>[] = [
    {
      width: 32,
    },
    {
      title: "Title",
      dataIndex: "title",
    },
    {
      title: "Path",
      dataIndex: "path",
    },
    {
      title: "Component",
      dataIndex: "component",
    },
    {
      title: "Icon",
      dataIndex: "icon",
    },
    {
      title: "Status",
      dataIndex: "status",
      valueEnum: {
        1: { text: "Normal", status: "Success" },
        2: { text: "Disabled", status: "Default" },
      },
    },
    {
      title: "Updated At",
      dataIndex: "updatedAt",
      valueType: "dateTime",
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
    <ProTable<Menu.Item>
      columns={columns}
      request={menuAPI.getMenuList}
      rowKey="id"
      search={false}
      headerTitle="Menu List"
      pagination={false}
      expandable={{
        defaultExpandAllRows: true,
      }}
    />
  );
};

export default MenuPage;
