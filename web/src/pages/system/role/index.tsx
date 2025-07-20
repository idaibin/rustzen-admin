import { ProTable } from "@ant-design/pro-components";
import type { ProColumns, ActionType } from "@ant-design/pro-components";
import type { Role } from "System";
import { roleAPI } from "@/services";
import { Space, Button, Popconfirm } from "antd";
import React, { useRef } from "react";
import RoleModalForm from "./RoleModalForm";
import { AuthWrap } from "@/components/auth";

export default function RolePage() {
    const actionRef = useRef<ActionType>(null);

    return (
        <ProTable<Role.Item>
            rowKey="id"
            scroll={{ y: "calc(100vh - 383px)" }}
            headerTitle="Role Management"
            columns={columns}
            request={roleAPI.getRoleList}
            actionRef={actionRef}
            search={{ span: 6 }}
            toolBarRender={() => [
                <AuthWrap code="system:role:create">
                    <RoleModalForm
                        mode={"create"}
                        onSuccess={() => {
                            actionRef.current?.reload();
                        }}
                    >
                        <Button type="primary">Create Role</Button>
                    </RoleModalForm>
                </AuthWrap>,
            ]}
        />
    );
}
const columns: ProColumns<Role.Item>[] = [
    {
        title: "ID",
        dataIndex: "id",
        width: 48,
        search: false,
    },
    {
        title: "Role Name",
        dataIndex: "name",
        width: 200,
        ellipsis: true,
    },
    {
        title: "Role Code",
        dataIndex: "code",
        width: 200,
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
        valueType: "select",
        width: 120,
        valueEnum: {
            1: { text: "Normal", status: "Success" },
            2: { text: "Disabled", status: "Default" },
        },
    },
    {
        title: "Permissions",
        dataIndex: "menus",
        width: 160,
        hideInSearch: true,
        render: (_, record) => {
            if (!record.menus || record.menus.length === 0) {
                return <span style={{ color: "#999" }}>No permissions</span>;
            }
            return (
                <span title={record.menus.map((menu) => menu.label).join(", ")}>
                    {record.menus.length} permission(s)
                </span>
            );
        },
    },
    {
        title: "Updated At",
        dataIndex: "updatedAt",
        valueType: "dateTime",
        width: 160,
        hideInSearch: true,
    },
    {
        title: "Actions",
        key: "action",
        width: 110,
        hideInSearch: true,
        render: (
            _dom: React.ReactNode,
            entity: Role.Item,
            _index,
            action?: ActionType
        ) => {
            if (entity.id === 1) {
                return null;
            }
            return (
                <Space size="middle">
                    <AuthWrap code="system:role:edit">
                        <RoleModalForm
                            mode={"edit"}
                            initialValues={entity}
                            onSuccess={() => {
                                action?.reload();
                            }}
                        >
                            <a>Edit</a>
                        </RoleModalForm>
                    </AuthWrap>
                    <AuthWrap code="system:role:delete">
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
                            <a style={{ color: "#ff4d4f" }}>Delete</a>
                        </Popconfirm>
                    </AuthWrap>
                </Space>
            );
        },
    },
];
