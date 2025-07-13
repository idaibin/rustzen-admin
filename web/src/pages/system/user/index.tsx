import { ProTable } from "@ant-design/pro-components";
import type { ProColumns, ActionType } from "@ant-design/pro-components";
import type { User } from "System";
import { userAPI } from "@/services";
import { Space, Button, Popconfirm } from "antd";
import React, { useRef } from "react";
import UserModalForm from "./UserModalForm";

export default function UserPage() {
    const actionRef = useRef<ActionType>(null);
    return (
        <ProTable<User.Item>
            rowKey="id"
            search={false}
            scroll={{ y: "calc(100vh - 287px)" }}
            headerTitle="User List"
            columns={columns}
            request={userAPI.getUserList}
            actionRef={actionRef}
            toolBarRender={() => [
                <UserModalForm
                    mode={"create"}
                    onSuccess={() => {
                        actionRef.current?.reload();
                    }}
                >
                    <Button type="primary">Create User</Button>
                </UserModalForm>,
            ]}
        />
    );
}

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
        title: "Roles",
        dataIndex: "roles",
        render: (_: React.ReactNode, record: User.Item) =>
            record.roles.map((role) => role.name).join(", "),
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
                    <UserModalForm
                        mode={"edit"}
                        initialValues={entity}
                        onSuccess={() => {
                            action?.reload();
                        }}
                    >
                        <a>Edit</a>
                    </UserModalForm>
                    <Popconfirm
                        title="Are you sure you want to delete this user?"
                        placement="leftBottom"
                        onConfirm={async () => {
                            await userAPI.deleteUser(entity.id);
                            action?.reload();
                        }}
                    >
                        <a className="text-red-500">Delete</a>
                    </Popconfirm>
                </Space>
            );
        },
    },
];
