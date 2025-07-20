import React, { type JSX } from "react";
import {
    ModalForm,
    ProFormText,
    ProFormTextArea,
    ProFormSelect,
} from "@ant-design/pro-components";
import type { Role } from "System";
import { roleAPI } from "@/services/system/role";
import { menuAPI } from "@/services/system/menu";
import { Form } from "antd";

interface RoleModalFormProps {
    initialValues?: Partial<Role.Item>;
    mode?: "create" | "edit";
    children: JSX.Element;
    onSuccess?: () => void;
}

const RoleModalForm: React.FC<RoleModalFormProps> = ({
    children,
    initialValues,
    mode = "create",
    onSuccess,
}) => {
    const [form] = Form.useForm();
    const isRequired = mode === "create";

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
                destroyOnClose: true,
                maskClosable: false,
                okText: mode === "create" ? "Create" : "Save",
                cancelText: "Cancel",
            }}
            onOpenChange={(open) => {
                if (open) {
                    const menuIds = initialValues?.menus?.map(
                        (menu) => menu.value
                    );
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
                    await roleAPI.updateRole(
                        initialValues.id,
                        values as Role.UpdateRequest
                    );
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
                    { required: isRequired, message: "Please enter role name" },
                    {
                        min: 2,
                        max: 50,
                        message: "Role name must be 2-50 characters",
                    },
                    {
                        pattern: /^[a-zA-Z0-9_\u4e00-\u9fa5]+$/,
                        message:
                            "Role name can only contain letters, numbers, underscores and Chinese characters",
                    },
                ]}
            />
            <ProFormText
                name="code"
                label="Role Code"
                placeholder="Enter role code"
                rules={[
                    { required: isRequired, message: "Please enter role code" },
                    {
                        min: 2,
                        max: 50,
                        message: "Role code must be 2-50 characters",
                    },
                    {
                        pattern: /^[A-Z_]+$/,
                        message:
                            "Role code can only contain uppercase letters and underscores",
                    },
                ]}
            />

            <ProFormSelect
                name="status"
                label="Status"
                placeholder="Select status"
                options={[
                    { label: "Normal", value: 1 },
                    { label: "Disabled", value: 2 },
                ]}
                rules={[
                    { required: isRequired, message: "Please select status" },
                ]}
            />
            <ProFormSelect
                name="menuIds"
                label="Permissions"
                placeholder="Select permissions"
                request={menuAPI.getMenuOptions}
                mode="multiple"
                rules={[
                    {
                        required: isRequired,
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

export default RoleModalForm;
