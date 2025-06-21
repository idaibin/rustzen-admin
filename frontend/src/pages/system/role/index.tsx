import { ProTable } from "@ant-design/pro-components";
import type { ProColumns } from "@ant-design/pro-components";
import { requestTable } from "@/services/api";
import type * as Role from "Role";

const RolePage = () => {
  const columns: ProColumns<Role.Item>[] = [
    {
      title: "ID",
      dataIndex: "id",
      width: 48,
    },
    {
      title: "角色名称",
      dataIndex: "roleName",
    },
    {
      title: "角色编码",
      dataIndex: "roleCode",
    },
    {
      title: "备注",
      dataIndex: "remark",
      ellipsis: true,
    },
    {
      title: "操作",
      key: "action",
      render: () => [<a>编辑</a>, <a>删除</a>],
    },
  ];

  return (
    <ProTable<Role.Item>
      columns={columns}
      request={async (params) => {
        return requestTable<Role.Item>("/api/sys/role", params);
      }}
      rowKey="id"
      search={false}
      headerTitle="角色列表"
    />
  );
};

export default RolePage;
