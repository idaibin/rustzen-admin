import { ModalForm, ProFormText, ProFormTextArea } from "@ant-design/pro-components";
import type { ActionType, ProColumns } from "@ant-design/pro-components";
import { ProTable } from "@ant-design/pro-components";
import { createFileRoute } from "@tanstack/react-router";
import { Form } from "antd";
import { Button, Space, Tag } from "antd";
import React, { useRef } from "react";

import { dictAPI } from "@/api/system/dict";
import { AuthPopconfirm, AuthWrap } from "@/components/auth";

export const Route = createFileRoute("/system/dict")({
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
            request={dictAPI.listDicts}
            actionRef={actionRef}
            toolBarRender={() => [
                <AuthWrap code="system:dict:create">
                    <DictModalForm
                        mode={"create"}
                        onSuccess={() => {
                            actionRef.current?.reload();
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
        width: 110,
        fixed: "right",
        render: (_dom: React.ReactNode, entity: Dict.Item, _index, action?: ActionType) => (
            <Space size="middle">
                <AuthWrap code="system:dict:edit">
                    <DictModalForm
                        mode={"edit"}
                        initialValues={entity}
                        onSuccess={() => {
                            action?.reload();
                        }}
                    >
                        <a>Edit</a>
                    </DictModalForm>
                </AuthWrap>
                <AuthPopconfirm
                    code="system:dict:delete"
                    title="Are you sure you want to delete this dictionary?"
                    description="This action cannot be undone."
                    onConfirm={async () => {
                        await dictAPI.deleteDict(entity.id);
                        action?.reload();
                    }}
                >
                    <span className="cursor-pointer text-red-500">Delete</span>
                </AuthPopconfirm>
            </Space>
        ),
    },
];

interface DictModalFormProps {
    initialValues?: Partial<Dict.Item>;
    mode?: "create" | "edit";
    children: React.ReactNode;
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
                    await dictAPI.createDict(values as Dict.CreateRequest);
                } else if (mode === "edit" && initialValues?.id) {
                    await dictAPI.updateDict(initialValues.id, values as Dict.UpdateRequest);
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
