import { ProTable } from "@ant-design/pro-components";
import type { ProColumns } from "@ant-design/pro-components";
import { roleAPI } from "@/services";
import type { Role } from "System";
import { Space } from "antd";

const RolePage = () => {
  const columns: ProColumns<Role.Item>[] = [
    {
      title: "ID",
      dataIndex: "id",
      width: 48,
    },
    {
      title: "Role Name",
      dataIndex: "roleName",
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
      title: "Created At",
      dataIndex: "createdAt",
      valueType: "dateTime",
    },
    {
      title: "Updated At",
      dataIndex: "updatedAt",
      valueType: "dateTime",
    },
    {
      title: "Menu IDs",
      dataIndex: "menuIds",
      render: (_, record) => record.menuIds?.join(", ") || "-",
    },
    {
      title: "Actions",
      key: "action",
      render: () => (
        <Space size="middle">
          <a>Edit</a>
          <a>Delete</a>
        </Space>
      ),
    },
  ];

  return (
    <ProTable<Role.Item>
      columns={columns}
      request={roleAPI.getRoleList}
      rowKey="id"
      search={false}
      headerTitle="Role List"
    />
  );
};

export default RolePage;
