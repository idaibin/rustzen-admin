import { ProTable } from "@ant-design/pro-components";
import type { ProColumns, ActionType } from "@ant-design/pro-components";
import type { Menu } from "System";
import { menuAPI } from "@/api";
import { Space, Button, Popconfirm, Tag } from "antd";
import React, { useRef } from "react";
import MenuModalForm from "./MenuModalForm";
import { AuthPopconfirm, AuthWrap } from "@/components/auth";

export default function MenuPage() {
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
        render: (
            _dom: React.ReactNode,
            entity: Menu.Item,
            _index,
            action?: ActionType
        ) => (
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
                    <span className="text-red-500 cursor-pointer">Delete</span>
                </AuthPopconfirm>
            </Space>
        ),
    },
];
