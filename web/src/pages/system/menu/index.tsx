import { ProTable } from "@ant-design/pro-components";
import type { ProColumns, ActionType } from "@ant-design/pro-components";
import type { Menu } from "System";
import { menuAPI } from "@/services";
import { Space, Button, Popconfirm } from "antd";
import React, { useRef } from "react";
import MenuModalForm from "./MenuModalForm";

export default function MenuPage() {
    const actionRef = useRef<ActionType>(null);

    return (
        <ProTable<Menu.Item>
            rowKey="id"
            search={false}
            scroll={{ y: "calc(100vh - 287px)" }}
            headerTitle="Menu Management"
            columns={columns}
            request={menuAPI.getMenuList}
            actionRef={actionRef}
            toolBarRender={() => [
                <MenuModalForm
                    mode={"create"}
                    onSuccess={() => {
                        actionRef.current?.reload();
                    }}
                >
                    <Button type="primary">Create Menu</Button>
                </MenuModalForm>,
            ]}
        />
    );
}

const columns: ProColumns<Menu.Item>[] = [
    {
        title: "ID",
        dataIndex: "id",
        width: 48,
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
        ellipsis: true,
    },
    {
        title: "Sort Order",
        dataIndex: "sortOrder",
        ellipsis: true,
    },
    // {
    //     title: "Status",
    //     dataIndex: "status",
    //     valueEnum: {
    //         1: { text: "Show", status: "Success" },
    //         2: { text: "Hidden", status: "Default" },
    //     },
    // },
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
            entity: Menu.Item,
            _index,
            action?: ActionType
        ) => (
            <Space size="middle">
                <MenuModalForm
                    mode={"edit"}
                    initialValues={entity}
                    onSuccess={() => {
                        action?.reload();
                    }}
                >
                    <a>Edit</a>
                </MenuModalForm>
                <Popconfirm
                    title="Are you sure you want to delete this menu?"
                    placement="leftBottom"
                    onConfirm={async () => {
                        try {
                            await menuAPI.deleteMenu(entity.id);
                            action?.reload();
                        } catch (error) {
                            console.error("Delete menu failed:", error);
                        }
                    }}
                >
                    <a className="text-red-500">Delete</a>
                </Popconfirm>
            </Space>
        ),
    },
];
