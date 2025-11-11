import { ModalForm, ProFormSelect, ProFormText, ProFormTextArea } from '@ant-design/pro-components';
import { Form } from 'antd';
import React, { type JSX } from 'react';

import { menuAPI } from '@/api/system/menu';
import { roleAPI } from '@/api/system/role';
import { ENABLE_OPTIONS } from '@/constant/options';
import { useApiQuery } from '@/integrations/react-query';

interface RoleModalFormProps {
  initialValues?: Partial<Role.Item>;
  mode?: 'create' | 'edit';
  children: JSX.Element;
  onSuccess?: () => void;
}

export const RoleModalForm: React.FC<RoleModalFormProps> = ({
  children,
  initialValues,
  mode = 'create',
  onSuccess,
}) => {
  const [form] = Form.useForm();
  const { data: menuOptions = [] } = useApiQuery('system/menus/options', menuAPI.getOptions);

  return (
    <ModalForm<Role.CreateRequest | Role.UpdateRequest>
      form={form}
      width={600}
      layout="horizontal"
      title={mode === 'create' ? 'Create Role' : 'Edit Role'}
      trigger={children}
      labelCol={{ span: 6 }}
      wrapperCol={{ span: 18 }}
      modalProps={{
        destroyOnHidden: true,
        maskClosable: false,
        okText: mode === 'create' ? 'Create' : 'Save',
        cancelText: 'Cancel',
      }}
      onOpenChange={(open) => {
        if (open) {
          const menuIds = initialValues?.menus?.map((menu) => menu.value);
          form.setFieldsValue({
            ...initialValues,
            menuIds,
          });
        } else {
          form.resetFields();
        }
      }}
      onFinish={async (values) => {
        if (mode === 'create') {
          await roleAPI.create(values as Role.CreateRequest);
        } else if (mode === 'edit' && initialValues?.id) {
          await roleAPI.update(initialValues.id, values as Role.UpdateRequest);
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
          { required: true, message: 'Please enter role name' },
          {
            min: 2,
            max: 50,
            message: 'Role name must be 2-50 characters',
          },
        ]}
      />
      <ProFormText
        name="code"
        label="Role Code"
        placeholder="Enter role code"
        rules={[
          { required: true, message: 'Please enter role code' },
          {
            min: 2,
            max: 50,
            message: 'Role code must be 2-50 characters',
          },
          {
            pattern: /^[A-Z_]+$/,
            message: 'Role code can only contain uppercase letters and underscores',
          },
        ]}
      />

      <ProFormSelect
        name="status"
        label="Status"
        placeholder="Select status"
        options={ENABLE_OPTIONS}
        rules={[{ required: true, message: 'Please select status' }]}
      />
      <ProFormSelect
        name="menuIds"
        label="Permissions"
        placeholder="Select permissions"
        options={[{ label: 'Root', value: 0 }, ...menuOptions]}
        mode="multiple"
        rules={[
          {
            required: true,
            message: 'Please select at least one permission',
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
