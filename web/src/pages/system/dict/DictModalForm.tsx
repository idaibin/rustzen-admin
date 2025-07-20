import React, { type JSX } from "react";
import {
    ModalForm,
    ProFormText,
    ProFormTextArea,
} from "@ant-design/pro-components";
import type { Dict } from "System";
import { dictAPI } from "@/services/system/dict";
import { Form } from "antd";

interface DictModalFormProps {
    initialValues?: Partial<Dict.Item>;
    mode?: "create" | "edit";
    children: JSX.Element;
    onSuccess?: () => void;
}

const DictModalForm: React.FC<DictModalFormProps> = ({
    children,
    initialValues,
    mode = "create",
    onSuccess,
}) => {
    const [form] = Form.useForm();
    const isRequired = mode === "create";

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
                destroyOnClose: true,
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
                    await dictAPI.updateDict(
                        initialValues.id,
                        values as Dict.UpdateRequest
                    );
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
                        required: isRequired,
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
                rules={[
                    { required: isRequired, message: "Please enter label" },
                ]}
            />
            <ProFormText
                name="value"
                label="Value"
                placeholder="Enter value (e.g., 1)"
                rules={[
                    { required: isRequired, message: "Please enter value" },
                ]}
            />
            <ProFormTextArea
                name="description"
                label="Description"
                placeholder="Enter description"
            />
        </ModalForm>
    );
};

export default DictModalForm;
