import { ProTable } from "@ant-design/pro-components";
import type { ProColumns } from "@ant-design/pro-components";
import { menuAPI } from "@/services";
import { Badge } from "antd";
import type { Menu } from "System";

const MenuPage = () => {
  const columns: ProColumns<Menu.Item>[] = [
    {
      title: "菜单名称",
      dataIndex: "name",
    },
    {
      title: "路径",
      dataIndex: "path",
    },
    {
      title: "组件",
      dataIndex: "component",
    },
    {
      title: "类型",
      dataIndex: "type",
      render: (_, record) => {
        if (record.type === 0) return <Badge status="processing" text="目录" />;
        if (record.type === 1) return <Badge status="success" text="菜单" />;
        if (record.type === 2) return <Badge status="warning" text="按钮" />;
        return "-";
      },
    },
    {
      title: "操作",
      key: "action",
      render: () => [<a>编辑</a>, <a>删除</a>],
    },
  ];

  return (
    <ProTable<Menu.Item>
      columns={columns}
      request={menuAPI.getMenuList}
      rowKey="id"
      search={false}
      headerTitle="菜单列表"
      pagination={false}
      expandable={{
        defaultExpandAllRows: true,
      }}
    />
  );
};

export default MenuPage;
