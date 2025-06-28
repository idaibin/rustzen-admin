import { ProTable } from "@ant-design/pro-components";
import type { ProColumns } from "@ant-design/pro-components";
import type { User } from "System";
import { userAPI } from "@/services";

const UserPage = () => {
  const columns: ProColumns<User.Item>[] = [
    {
      title: "ID",
      dataIndex: "id",
      width: 48,
    },
    {
      title: "用户名",
      dataIndex: "userName",
    },
    {
      title: "角色ID",
      dataIndex: "roleIds",
      // renderText: (text: number[]) => text.join(", "),
    },
    {
      title: "操作",
      key: "action",
      render: () => [<a>编辑</a>, <a>删除</a>],
    },
  ];

  return (
    <ProTable<User.Item>
      columns={columns}
      request={userAPI.getUserList}
      rowKey="id"
      search={false}
      headerTitle="用户列表"
    />
  );
};

export default UserPage;
