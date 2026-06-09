import { DeleteOutlined, EditOutlined } from "@ant-design/icons";
import {
    ModalForm,
    ProFormText,
    ProFormTextArea,
    ProTable,
    type ActionType,
    type ProColumns,
} from "@ant-design/pro-components";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Form, Space, Tag } from "antd";
import React, { useRef } from "react";

import { manageAPI } from "@/api";
import { AuthPopconfirm, AuthWrap } from "@/components/base-auth";
import {
    TABLE_ACTION_SPACE_SIZE,
    TableActionButton,
} from "@/components/base-button";

export const Route = createFileRoute("/manage/dict")({
    component: DictPage,
});

function DictPage() {
    const actionRef = useRef<ActionType>(null);

    return (
        <ProTable<Dict.Item>
            rowKey="id"
            search={false}
            scroll={{ y: "calc(100vh - 287px)" }}
            headerTitle="Dictionary Management"
            columns={columns}
            request={manageAPI.dict.list}
            actionRef={actionRef}
            toolBarRender={() => [
                <AuthWrap key="create" code="manage:dict:create">
                    <DictModalForm
                        mode={"create"}
                        onSuccess={() => {
                            void actionRef.current?.reload();
                        }}
                    >
                        <Button type="primary">Create Dictionary</Button>
                    </DictModalForm>
                </AuthWrap>,
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
        title: "Description",
        dataIndex: "description",
        ellipsis: true,
    },
    {
        title: "Actions",
        key: "action",
        width: 68,
        align: "left",
        fixed: "right",
        render: (_dom: React.ReactNode, entity: Dict.Item, _index, action?: ActionType) => (
            <Space size={TABLE_ACTION_SPACE_SIZE} align="center">
                <AuthWrap code="manage:dict:update">
                    <DictModalForm
                        mode={"edit"}
                        initialValues={entity}
                        onSuccess={() => {
                            void action?.reload();
                        }}
                    >
                        <TableActionButton color="blue" label="Edit" icon={<EditOutlined />} />
                    </DictModalForm>
                </AuthWrap>
                <AuthPopconfirm
                    code="manage:dict:delete"
                    title="Are you sure you want to delete this dictionary?"
                    description="This action cannot be undone."
                    onConfirm={async () => {
                        await manageAPI.dict.delete(entity.id);
                        void action?.reload();
                    }}
                >
                    <TableActionButton color="danger" label="Delete" icon={<DeleteOutlined />} />
                </AuthPopconfirm>
            </Space>
        ),
    },
];

interface DictModalFormProps {
    initialValues?: Partial<Dict.Item>;
    mode?: "create" | "edit";
    children: React.JSX.Element;
    onSuccess?: () => void;
}

const DictModalForm = ({
    children,
    initialValues,
    mode = "create",
    onSuccess,
}: DictModalFormProps) => {
    const [form] = Form.useForm();

    return (
        <ModalForm<Dict.CreateRequest | Dict.UpdateRequest>
            form={form}
            width={500}
            layout="horizontal"
            title={mode === "create" ? "Create Dictionary" : "Edit Dictionary"}
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
                    await manageAPI.dict.create(values as Dict.CreateRequest);
                } else if (mode === "edit" && initialValues?.id) {
                    await manageAPI.dict.update(initialValues.id, values as Dict.UpdateRequest);
                }
                onSuccess?.();
                return true;
            }}
        >
            <ProFormText
                name="dictType"
                label="Dict Type"
                placeholder="Enter dictionary type (e.g., user_status)"
                rules={[
                    {
                        required: true,
                        message: "Please enter dictionary type",
                    },
                    {
                        pattern: /^[a-z_]+$/,
                        message:
                            "Dictionary type can only contain lowercase letters and underscores",
                    },
                ]}
            />
            <ProFormText
                name="label"
                label="Label"
                placeholder="Enter display label (e.g., Active)"
                rules={[{ required: true, message: "Please enter label" }]}
            />
            <ProFormText
                name="value"
                label="Value"
                placeholder="Enter value (e.g., 1)"
                rules={[{ required: true, message: "Please enter value" }]}
            />
            <ProFormTextArea
                name="description"
                label="Description"
                placeholder="Enter description"
            />
        </ModalForm>
    );
};
