import {
    ModalForm,
    ProTable,
    ProFormSelect,
    ProFormText,
    ProFormTextArea,
    type ActionType,
    type ProColumns,
} from "@ant-design/pro-components";
import { createFileRoute } from "@tanstack/react-router";
import { Form, Button, Space } from "antd";
import React, { useRef } from "react";

import { menuAPI } from "@/api/system/menu";
import { roleAPI } from "@/api/system/role";
import { AuthPopconfirm, AuthWrap } from "@/components/auth";
import { ENABLE_OPTIONS } from "@/constant/options";
import { useApiQuery } from "@/integrations/react-query";

export const Route = createFileRoute("/system/role")({
    component: RolePage,
});

function RolePage() {
    const actionRef = useRef<ActionType>(null);

    return (
        <ProTable<Role.Item>
            rowKey="id"
            scroll={{ y: "calc(100vh - 383px)" }}
            headerTitle="Role Management"
            columns={columns}
            request={roleAPI.listRoles}
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
        search: {
            transform: (value) => ({ roleName: value }),
        },
    },
    {
        title: "Role Code",
        dataIndex: "code",
        width: 200,
        ellipsis: true,
        search: {
            transform: (value) => ({ roleCode: value }),
        },
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
            1: { text: "Enabled", status: "Success" },
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
        render: (_dom: React.ReactNode, entity: Role.Item, _index, action?: ActionType) => {
            const isSystemRole = entity.id === 1;
            return (
                <Space size="middle">
                    <AuthWrap code="system:role:edit" hidden={isSystemRole}>
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
                    <AuthPopconfirm
                        hidden={isSystemRole}
                        code="system:role:delete"
                        title="Are you sure you want to delete this role?"
                        description="This action cannot be undone."
                        onConfirm={async () => {
                            await roleAPI.deleteRole(entity.id);
                            action?.reload();
                        }}
                    >
                        <span className="cursor-pointer text-red-500">Delete</span>
                    </AuthPopconfirm>
                </Space>
            );
        },
    },
];

interface RoleModalFormProps {
    initialValues?: Partial<Role.Item>;
    mode?: "create" | "edit";
    children: React.ReactNode;
    onSuccess?: () => void;
}

const RoleModalForm = ({
    children,
    initialValues,
    mode = "create",
    onSuccess,
}: RoleModalFormProps) => {
    const [form] = Form.useForm();
    const { data: menuOptions = [] } = useApiQuery("system/menus/options", menuAPI.listMenuOptions);

    return (
        <ModalForm<Role.CreateRequest | Role.UpdateRequest>
            form={form}
            width={600}
            layout="horizontal"
            title={mode === "create" ? "Create Role" : "Edit Role"}
            trigger={children}
            labelCol={{ span: 6 }}
            wrapperCol={{ span: 18 }}
            modalProps={{
                destroyOnHidden: true,
                maskClosable: false,
                okText: mode === "create" ? "Create" : "Save",
                cancelText: "Cancel",
            }}
            onOpenChange={(open) => {
                if (open) {
                    const menuIds = initialValues?.menus?.map((menu) => menu.value);
                    form.setFieldsValue({
                        ...initialValues,
                        menuIds,
                    });
                } else {
                    form.resetFields();
                }
            }}
            onFinish={async (values) => {
                if (mode === "create") {
                    await roleAPI.createRole(values as Role.CreateRequest);
                } else if (mode === "edit" && initialValues?.id) {
                    await roleAPI.updateRole(initialValues.id, values as Role.UpdateRequest);
                }
                onSuccess?.();
                return true;
            }}
        >
            <ProFormText
                name="name"
                label="Role Name"
                placeholder="Enter role name"
                rules={[
                    { required: true, message: "Please enter role name" },
                    {
                        min: 2,
                        max: 50,
                        message: "Role name must be 2-50 characters",
                    },
                ]}
            />
            <ProFormText
                name="code"
                label="Role Code"
                placeholder="Enter role code"
                rules={[
                    { required: true, message: "Please enter role code" },
                    {
                        min: 2,
                        max: 50,
                        message: "Role code must be 2-50 characters",
                    },
                    {
                        pattern: /^[A-Z_]+$/,
                        message: "Role code can only contain uppercase letters and underscores",
                    },
                ]}
            />

            <ProFormSelect
                name="status"
                label="Status"
                placeholder="Select status"
                options={ENABLE_OPTIONS}
                rules={[{ required: true, message: "Please select status" }]}
            />
            <ProFormSelect
                name="menuIds"
                label="Permissions"
                placeholder="Select permissions"
                options={[{ label: "Root", value: 0 }, ...menuOptions]}
                mode="multiple"
                rules={[
                    {
                        required: true,
                        message: "Please select at least one permission",
                    },
                ]}
            />
            <ProFormTextArea
                name="description"
                label="Description"
                placeholder="Enter role description"
                fieldProps={{
                    maxLength: 200,
                    showCount: true,
                }}
            />
        </ModalForm>
    );
};
