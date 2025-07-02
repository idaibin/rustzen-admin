import { ProTable } from "@ant-design/pro-components";
import type { ProColumns } from "@ant-design/pro-components";
import type { User } from "System";
import { userAPI } from "@/services";
import { Space } from "antd";

const UserPage = () => {
  const columns: ProColumns<User.Item>[] = [
    {
      title: "ID",
      dataIndex: "id",
      width: 48,
    },
    {
      title: "Username",
      dataIndex: "username",
    },
    {
      title: "Email",
      dataIndex: "email",
    },
    {
      title: "Real Name",
      dataIndex: "realName",
    },
    {
      title: "Avatar",
      dataIndex: "avatarUrl",
      render: (_: React.ReactNode, record: User.Item) =>
        record.avatarUrl && record.avatarUrl.length > 0 ? (
          <img
            src={record.avatarUrl}
            alt="avatar"
            style={{ width: 32, height: 32, borderRadius: "50%" }}
          />
        ) : null,
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
      title: "Last Login",
      dataIndex: "lastLoginAt",
      valueType: "dateTime",
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
    <ProTable<User.Item>
      columns={columns}
      request={userAPI.getUserList}
      rowKey="id"
      search={false}
      headerTitle="User List"
    />
  );
};

export default UserPage;
