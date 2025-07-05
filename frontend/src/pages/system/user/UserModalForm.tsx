import React, { type JSX } from "react";
import {
    ModalForm,
    ProFormText,
    ProFormSelect,
} from "@ant-design/pro-components";
import type { User } from "System";
import { userAPI } from "@/services/system/user";
import { roleAPI } from "@/services";
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
    const isRequired = mode === "create";

    return (
        <ModalForm<User.CreateRequest | User.UpdateRequest>
            form={form}
            width={500}
            layout="horizontal"
            title={mode === "create" ? "Create User" : "Edit User"}
            trigger={children}
            labelCol={{ span: 5 }}
            modalProps={{ destroyOnClose: true, maskClosable: false }}
            onOpenChange={(open) => {
                if (open) {
                    form.setFieldsValue({
                        ...initialValues,
                        roleIds: initialValues?.roles?.map((role) => role.id),
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
                    await userAPI.createUser(values as User.CreateRequest);
                } else if (mode === "edit" && initialValues?.id) {
                    await userAPI.updateUser(
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
                    { required: isRequired, message: "Please enter username" },
                    { min: 3, message: "At least 3 characters" },
                ]}
                disabled={mode === "edit"}
            />
            <ProFormText
                name="email"
                label="Email"
                placeholder="Enter email"
                rules={[
                    { required: isRequired, message: "Please enter email" },
                    { type: "email", message: "Invalid email format" },
                ]}
            />
            <ProFormText
                name="realName"
                label="Real Name"
                placeholder="Enter real name"
            />
            <ProFormText.Password
                name="password"
                label="Password"
                placeholder="Enter password"
                rules={[
                    { required: isRequired, message: "Please enter password" },
                    { min: 6, message: "At least 6 characters" },
                ]}
            />
            <ProFormSelect
                name="status"
                label="Status"
                placeholder="Select status"
                request={userAPI.getUserStatusOptions}
                rules={[
                    { required: isRequired, message: "Please select status" },
                ]}
            />
            <ProFormSelect
                name="roleIds"
                label="Roles"
                placeholder="Select roles"
                request={roleAPI.getRoleOptions}
                mode="multiple"
                rules={[
                    {
                        required: isRequired,
                        message: "Please select at least one role",
                    },
                ]}
            />
        </ModalForm>
    );
};

export default UserModalForm;
