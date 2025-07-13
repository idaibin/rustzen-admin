import { ProTable } from "@ant-design/pro-components";
import type { ProColumns, ActionType } from "@ant-design/pro-components";
import type { Role } from "System";
import { roleAPI } from "@/services";
import { Space, Button, Popconfirm } from "antd";
import React, { useRef } from "react";
import RoleModalForm from "./RoleModalForm";

export default function RolePage() {
    const actionRef = useRef<ActionType>(null);

    return (
        <ProTable<Role.Item>
            rowKey="id"
            search={false}
            scroll={{ y: "calc(100vh - 287px)" }}
            headerTitle="Role Management"
            columns={columns}
            request={roleAPI.getRoleList}
            actionRef={actionRef}
            toolBarRender={() => [
                <RoleModalForm
                    mode={"create"}
                    onSuccess={() => {
                        actionRef.current?.reload();
                    }}
                >
                    <Button type="primary">Create Role</Button>
                </RoleModalForm>,
            ]}
        />
    );
}
const columns: ProColumns<Role.Item>[] = [
    {
        title: "ID",
        dataIndex: "id",
        width: 48,
    },
    {
        title: "Role Name",
        dataIndex: "roleName",
        ellipsis: true,
    },
    {
        title: "Role Code",
        dataIndex: "roleCode",
        ellipsis: true,
    },
    {
        title: "Description",
        dataIndex: "description",
        ellipsis: true,
        hideInSearch: true,
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
        title: "Permissions",
        dataIndex: "menuIds",
        render: (_, record) => {
            if (!record.menuIds || record.menuIds.length === 0) {
                return <span style={{ color: "#999" }}>No permissions</span>;
            }
            return (
                <span title={record.menuIds.join(", ")}>
                    {record.menuIds.length} permission(s)
                </span>
            );
        },
    },
    {
        title: "Created At",
        dataIndex: "createdAt",
        valueType: "dateTime",
        hideInSearch: true,
    },
    {
        title: "Updated At",
        dataIndex: "updatedAt",
        valueType: "dateTime",
        hideInSearch: true,
    },
    {
        title: "Actions",
        key: "action",
        width: 110,
        fixed: "right",
        render: (
            _dom: React.ReactNode,
            entity: Role.Item,
            _index,
            action?: ActionType
        ) => (
            <Space size="middle">
                <RoleModalForm
                    mode={"edit"}
                    initialValues={entity}
                    onSuccess={() => {
                        action?.reload();
                    }}
                >
                    <a>Edit</a>
                </RoleModalForm>
                <Popconfirm
                    title="Are you sure you want to delete this role?"
                    description="This action cannot be undone."
                    placement="leftBottom"
                    onConfirm={async () => {
                        try {
                            await roleAPI.deleteRole(entity.id);
                            action?.reload();
                        } catch (error) {
                            console.error("Delete role failed:", error);
                        }
                    }}
                >
                    <a className="text-red-500">Delete</a>
                </Popconfirm>
            </Space>
        ),
    },
];
