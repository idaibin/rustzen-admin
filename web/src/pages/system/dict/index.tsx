import { ProTable } from "@ant-design/pro-components";
import type { ProColumns, ActionType } from "@ant-design/pro-components";
import type { Dict } from "System";
import { dictAPI } from "@/services";
import { Tag, Space, Button, Popconfirm } from "antd";
import React, { useRef } from "react";
import DictModalForm from "./DictModalForm";

export default function DictPage() {
    const actionRef = useRef<ActionType>(null);

    return (
        <ProTable<Dict.Item>
            rowKey="id"
            search={false}
            scroll={{ y: "calc(100vh - 287px)" }}
            headerTitle="Dictionary Management"
            columns={columns}
            request={dictAPI.getDictList}
            actionRef={actionRef}
            toolBarRender={() => [
                <DictModalForm
                    mode={"create"}
                    onSuccess={() => {
                        actionRef.current?.reload();
                    }}
                >
                    <Button type="primary">Create Dictionary</Button>
                </DictModalForm>,
            ]}
        />
    );
}

const columns: ProColumns<Dict.Item>[] = [
    {
        title: "ID",
        dataIndex: "id",
        width: 48,
    },
    {
        title: "Dict Type",
        dataIndex: "dictType",
        ellipsis: true,
        render: (text) => <Tag color="blue">{text}</Tag>,
    },
    {
        title: "Label",
        dataIndex: "label",
        ellipsis: true,
        search: {
            transform: (value) => ({ q: value }),
        },
    },
    {
        title: "Value",
        dataIndex: "value",
        ellipsis: true,
    },
    {
        title: "Default",
        dataIndex: "isDefault",
        render: (_, record) =>
            record.isDefault ? <Tag color="success">Yes</Tag> : <Tag>No</Tag>,
    },
    {
        title: "Actions",
        key: "action",
        width: 110,
        fixed: "right",
        render: (
            _dom: React.ReactNode,
            entity: Dict.Item,
            _index,
            action?: ActionType
        ) => (
            <Space size="middle">
                <DictModalForm
                    mode={"edit"}
                    initialValues={entity}
                    onSuccess={() => {
                        action?.reload();
                    }}
                >
                    <a>Edit</a>
                </DictModalForm>
                <Popconfirm
                    title="Are you sure you want to delete this dictionary?"
                    placement="leftBottom"
                    onConfirm={async () => {
                        try {
                            await dictAPI.deleteDict(entity.id);
                            action?.reload();
                        } catch (error) {
                            console.error("Delete dictionary failed:", error);
                        }
                    }}
                >
                    <a style={{ color: "#ff4d4f" }}>Delete</a>
                </Popconfirm>
            </Space>
        ),
    },
];
