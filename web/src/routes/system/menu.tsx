import type { ActionType, ProColumns } from "@ant-design/pro-components";
import { ProTable } from "@ant-design/pro-components";
import { ModalForm, ProFormDigit, ProFormSelect, ProFormText } from "@ant-design/pro-components";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Space, Tag } from "antd";
import { Form } from "antd";
import React, { useRef } from "react";

import { menuAPI } from "@/api/system/menu";
import { AuthPopconfirm, AuthWrap } from "@/components/auth";
import { ENABLE_OPTIONS, MENU_TYPE_OPTIONS } from "@/constant/options";

export const Route = createFileRoute("/system/menu")({
    component: MenuPage,
});

function MenuPage() {
    const actionRef = useRef<ActionType>(null);

    return (
        <ProTable<Menu.Item>
            rowKey="id"
            search={false}
            scroll={{ y: "calc(100vh - 287px)" }}
            headerTitle="Menu Management"
            columns={columns}
            request={menuAPI.getTableData}
            actionRef={actionRef}
            pagination={false}
            toolBarRender={() => [
                <AuthWrap code="system:menu:create">
                    <MenuModalForm
                        mode={"create"}
                        onSuccess={() => {
                            actionRef.current?.reload();
                        }}
                    >
                        <Button type="primary">Create Menu</Button>
                    </MenuModalForm>
                </AuthWrap>,
            ]}
        />
    );
}

const menuTypeEnum: Record<number, { text: string; color: string }> = {
    1: { text: "Directory", color: "cyan" },
    2: { text: "Menu", color: "purple" },
    3: { text: "Button", color: "warning" },
};

const columns: ProColumns<Menu.Item>[] = [
    {
        title: "",
        dataIndex: "",
        width: 60,
    },
    {
        title: "Name",
        dataIndex: "name",
        ellipsis: true,
    },
    {
        title: "Code",
        dataIndex: "code",
        ellipsis: true,
    },
    {
        title: "Menu Type",
        dataIndex: "menuType",
        width: 120,
        ellipsis: true,
        render: (_, record) => {
            const item = menuTypeEnum[record.menuType];
            return <Tag color={item.color}>{item.text}</Tag>;
        },
    },
    {
        title: "Sort Order",
        dataIndex: "sortOrder",
        width: 120,
        ellipsis: true,
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
        width: 120,
        fixed: "right",
        render: (_dom: React.ReactNode, entity: Menu.Item, _index, action?: ActionType) => (
            <Space size="middle">
                <AuthWrap code="system:menu:edit">
                    <MenuModalForm
                        mode={"edit"}
                        initialValues={entity}
                        onSuccess={() => {
                            action?.reload();
                        }}
                    >
                        <a>Edit</a>
                    </MenuModalForm>
                </AuthWrap>
                <AuthPopconfirm
                    code="system:menu:delete"
                    title="Are you sure you want to delete this menu?"
                    description="This action cannot be undone."
                    hidden={entity.isSystem}
                    onConfirm={async () => {
                        await menuAPI.delete(entity.id);
                        action?.reload();
                    }}
                >
                    <span className="cursor-pointer text-red-500">Delete</span>
                </AuthPopconfirm>
            </Space>
        ),
    },
];

interface MenuModalFormProps {
    initialValues?: Partial<Menu.Item>;
    mode?: "create" | "edit";
    children: React.ReactNode;
    onSuccess?: () => void;
}

const MenuModalForm = ({
    children,
    initialValues,
    mode = "create",
    onSuccess,
}: MenuModalFormProps) => {
    const [form] = Form.useForm();

    return (
        <ModalForm<Menu.CreateAndUpdateRequest>
            form={form}
            width={600}
            layout="horizontal"
            title={mode === "create" ? "Create Menu" : "Edit Menu"}
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
                    form.setFieldsValue(initialValues);
                } else {
                    form.resetFields();
                }
            }}
            onFinish={async (values) => {
                if (mode === "create") {
                    await menuAPI.create(values as Menu.CreateAndUpdateRequest);
                } else if (mode === "edit" && initialValues?.id) {
                    await menuAPI.update(initialValues.id, values as Menu.CreateAndUpdateRequest);
                }
                onSuccess?.();
                return true;
            }}
        >
            <ProFormSelect
                name="parentId"
                label="Parent Menu"
                placeholder="Select parent menu (optional)"
                request={menuAPI.getOptions}
                fieldProps={{
                    // showSearch: true,
                    optionFilterProp: "label",
                }}
                rules={[{ required: true, message: "Please select parent menu" }]}
            />
            <ProFormText
                name="name"
                label="Menu Name"
                placeholder="Enter menu name"
                rules={[{ required: true, message: "Please enter menu name" }]}
            />
            <ProFormText
                name="code"
                label="Permission Code"
                placeholder="Enter permission code (e.g., system:menu:list)"
                rules={[{ required: true, message: "Please enter permission code" }]}
            />
            <ProFormSelect
                label="Type"
                name="menuType"
                options={MENU_TYPE_OPTIONS}
                rules={[{ required: true, message: "Please select menu type" }]}
            />
            <ProFormSelect
                name="status"
                label="Status"
                placeholder="Select status"
                options={ENABLE_OPTIONS}
                rules={[{ required: true, message: "Please select status" }]}
            />
            <ProFormDigit
                name="sortOrder"
                label="Sort Order"
                placeholder="Enter sort order"
                min={0}
                fieldProps={{ precision: 0 }}
            />
        </ModalForm>
    );
};
