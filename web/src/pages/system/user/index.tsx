import { ProTable } from "@ant-design/pro-components";
import type { ProColumns, ActionType } from "@ant-design/pro-components";
import type { User } from "System";
import { userAPI } from "@/services";
import { Space, Button, Popconfirm } from "antd";
import React, { useRef } from "react";
import UserModalForm from "./UserModalForm";
import { AuthWrap } from "@/components/auth";

export default function UserPage() {
    const actionRef = useRef<ActionType>(null);
    return (
        <ProTable<User.Item>
            rowKey="id"
            scroll={{ y: "calc(100vh - 383px)" }}
            headerTitle="User List"
            columns={columns}
            request={userAPI.getUserList}
            actionRef={actionRef}
            search={{ span: 6 }}
            toolBarRender={() => [
                <AuthWrap code="system:user:create">
                    <UserModalForm
                        mode={"create"}
                        onSuccess={() => {
                            actionRef.current?.reload();
                        }}
                    >
                        <Button type="primary">Create User</Button>
                    </UserModalForm>
                </AuthWrap>,
            ]}
        />
    );
}

const columns: ProColumns<User.Item>[] = [
    {
        title: "ID",
        dataIndex: "id",
        width: 48,
        search: false,
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
        title: "Status",
        dataIndex: "status",
        valueEnum: {
            1: { text: "Normal", status: "Success" },
            2: { text: "Disabled", status: "Default" },
        },
    },
    {
        title: "Roles",
        dataIndex: "roles",
        search: false,
        render: (_: React.ReactNode, record: User.Item) =>
            record.roles.map((role) => role.label).join(", "),
    },
    {
        title: "Last Login",
        dataIndex: "lastLoginAt",
        valueType: "dateTime",
        search: false,
    },
    {
        title: "Updated At",
        dataIndex: "updatedAt",
        valueType: "dateTime",
        search: false,
    },
    {
        title: "Actions",
        key: "action",
        width: 110,
        search: false,
        render: (
            _dom: React.ReactNode,
            entity: User.Item,
            _index,
            action?: ActionType
        ) => {
            if (entity.id === 1) {
                return null;
            }
            return (
                <Space size="middle">
                    <AuthWrap code="system:user:edit">
                        <UserModalForm
                            mode={"edit"}
                            initialValues={entity}
                            onSuccess={() => {
                                action?.reload();
                            }}
                        >
                            <a>Edit</a>
                        </UserModalForm>
                    </AuthWrap>
                    <AuthWrap code="system:user:delete">
                        <Popconfirm
                            title="Are you sure you want to delete this user?"
                            placement="leftBottom"
                            onConfirm={async () => {
                                await userAPI.deleteUser(entity.id);
                                action?.reload();
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
