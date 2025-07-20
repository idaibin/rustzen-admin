import React, { type JSX } from "react";
import {
    ModalForm,
    ProFormText,
    ProFormSelect,
    ProFormDigit,
} from "@ant-design/pro-components";
import type { Menu } from "System";
import { menuAPI } from "@/services/system/menu";
import { Form } from "antd";

interface MenuModalFormProps {
    initialValues?: Partial<Menu.Item>;
    mode?: "create" | "edit";
    children: JSX.Element;
    onSuccess?: () => void;
}

const MenuModalForm: React.FC<MenuModalFormProps> = ({
    children,
    initialValues,
    mode = "create",
    onSuccess,
}) => {
    const [form] = Form.useForm();
    const isRequired = mode === "create";

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
                    await menuAPI.createMenu(
                        values as Menu.CreateAndUpdateRequest
                    );
                } else if (mode === "edit" && initialValues?.id) {
                    await menuAPI.updateMenu(
                        initialValues.id,
                        values as Menu.CreateAndUpdateRequest
                    );
                }
                onSuccess?.();
                return true;
            }}
        >
            <ProFormSelect
                name="parentId"
                label="Parent Menu"
                placeholder="Select parent menu (optional)"
                request={menuAPI.getMenuOptions}
                allowClear
            />
            <ProFormText
                name="title"
                label="Menu Title"
                placeholder="Enter menu title"
                rules={[
                    {
                        required: isRequired,
                        message: "Please enter menu title",
                    },
                ]}
            />
            <ProFormText
                name="path"
                label="Menu Path"
                placeholder="Enter menu path (e.g., /system/menu)"
            />
            <ProFormText
                name="component"
                label="Component"
                placeholder="Enter component path (e.g., @/pages/system/menu)"
            />
            <ProFormText
                name="icon"
                label="Icon"
                placeholder="Enter icon name (e.g., MenuOutlined)"
            />
            <ProFormDigit
                name="sortOrder"
                label="Sort Order"
                placeholder="Enter sort order"
                min={0}
                fieldProps={{ precision: 0 }}
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
        </ModalForm>
    );
};

export default MenuModalForm;
