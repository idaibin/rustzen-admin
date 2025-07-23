import React, { type JSX } from "react";
import {
    ModalForm,
    ProFormText,
    ProFormSelect,
} from "@ant-design/pro-components";
import type { User } from "System";
import { userAPI } from "@/api/system/user";
import { roleAPI } from "@/api";
import { Form } from "antd";

interface UserModalFormProps {
    initialValues?: Partial<User.Item>;
    mode?: "create" | "edit";
    children: JSX.Element;
    onSuccess?: () => void;
}

const UserModalForm: React.FC<UserModalFormProps> = ({
    children,
    initialValues,
    mode = "create",
    onSuccess,
}) => {
    const [form] = Form.useForm();

    return (
        <ModalForm<User.CreateRequest | User.UpdateRequest>
            form={form}
            width={500}
            layout="horizontal"
            title={mode === "create" ? "Create User" : "Edit User"}
            trigger={children}
            labelCol={{ span: 5 }}
            modalProps={{ destroyOnHidden: true, maskClosable: false }}
            onOpenChange={(open) => {
                if (open) {
                    const roleIds = initialValues?.roles?.map(
                        (role) => role.value
                    );
                    form.setFieldsValue({
                        ...initialValues,
                        roleIds,
                    });
                }
            }}
            submitter={{
                searchConfig: {
                    submitText: mode === "create" ? "Create" : "Save",
                },
            }}
            onFinish={async (values) => {
                if (mode === "create") {
                    await userAPI.create(values as User.CreateRequest);
                } else if (mode === "edit" && initialValues?.id) {
                    await userAPI.update(
                        initialValues.id,
                        values as User.UpdateRequest
                    );
                }
                onSuccess?.();
                return true;
            }}
        >
            <ProFormText
                name="username"
                label="Username"
                placeholder="Enter username"
                rules={[
                    { required: true, message: "Please enter username" },
                    { min: 3, message: "At least 3 characters" },
                ]}
                disabled={mode === "edit"}
            />
            <ProFormText
                name="email"
                label="Email"
                placeholder="Enter email"
                rules={[
                    { required: true, message: "Please enter email" },
                    { type: "email", message: "Invalid email format" },
                ]}
            />
            <ProFormText
                name="realName"
                label="Real Name"
                placeholder="Enter real name"
                rules={[{ required: true, message: "Please enter real name" }]}
            />
            {mode === "create" && (
                <ProFormText.Password
                    name="password"
                    label="Password"
                    placeholder="Enter password"
                    rules={[
                        {
                            required: true,
                            message: "Please enter password",
                        },
                        { min: 6, message: "At least 6 characters" },
                    ]}
                />
            )}

            <ProFormSelect
                name="status"
                label="Status"
                placeholder="Select status"
                request={userAPI.getStatusOptions}
                rules={[{ required: true, message: "Please select status" }]}
            />
            <ProFormSelect
                name="roleIds"
                label="Roles"
                placeholder="Select roles"
                request={roleAPI.getOptions}
                mode="multiple"
                rules={[
                    {
                        required: true,
                        message: "Please select at least one role",
                    },
                ]}
            />
        </ModalForm>
    );
};

export default UserModalForm;
