import { ModalForm, ProFormSelect, ProFormText } from '@ant-design/pro-components';
import { Form } from 'antd';
import React, { type JSX } from 'react';
import useSWR from 'swr';

import { userAPI } from '@/api/system/user';
import { ENABLE_OPTIONS } from '@/constant/options';
import { ROLE_OPTIONS_URL } from '@/constant/urls';

interface UserModalFormProps {
  initialValues?: Partial<User.Item>;
  mode?: 'create' | 'edit';
  children: JSX.Element;
  onSuccess?: () => void;
}

export const UserModalForm: React.FC<UserModalFormProps> = ({
  children,
  initialValues,
  mode = 'create',
  onSuccess,
}) => {
  const [form] = Form.useForm();
  const { data: roleOptions } = useSWR(ROLE_OPTIONS_URL);

  return (
    <ModalForm<User.CreateRequest | User.UpdateRequest>
      form={form}
      width={500}
      layout="horizontal"
      title={mode === 'create' ? 'Create User' : 'Edit User'}
      trigger={children}
      labelCol={{ span: 5 }}
      modalProps={{ destroyOnHidden: true, maskClosable: false }}
      onOpenChange={(open) => {
        if (open) {
          const roleIds = initialValues?.roles?.map((role) => role.value);
          form.setFieldsValue({
            ...initialValues,
            roleIds,
          });
        }
      }}
      submitter={{
        searchConfig: {
          submitText: mode === 'create' ? 'Create' : 'Save',
        },
      }}
      onFinish={async (values) => {
        if (mode === 'create') {
          await userAPI.create(values as User.CreateRequest);
        } else if (mode === 'edit' && initialValues?.id) {
          await userAPI.update(initialValues.id, values as User.UpdateRequest);
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
          { required: true, message: 'Please enter username' },
          { min: 3, message: 'At least 3 characters' },
        ]}
        disabled={mode === 'edit'}
      />
      <ProFormText
        name="email"
        label="Email"
        placeholder="Enter email"
        rules={[
          { required: true, message: 'Please enter email' },
          { type: 'email', message: 'Invalid email format' },
        ]}
      />
      <ProFormText
        name="realName"
        label="Real Name"
        placeholder="Enter real name"
        rules={[{ required: true, message: 'Please enter real name' }]}
      />
      {mode === 'create' && (
        <ProFormText.Password
          name="password"
          label="Password"
          placeholder="Enter password"
          rules={[
            {
              required: true,
              message: 'Please enter password',
            },
            { min: 6, message: 'At least 6 characters' },
          ]}
        />
      )}
      <ProFormSelect
        name="status"
        label="Status"
        placeholder="Select status"
        options={ENABLE_OPTIONS}
        rules={[{ required: true, message: 'Please select status' }]}
      />
      <ProFormSelect
        name="roleIds"
        label="Roles"
        placeholder="Select roles"
        options={roleOptions}
        mode="multiple"
        rules={[
          {
            required: true,
            message: 'Please select at least one role',
          },
        ]}
      />
    </ModalForm>
  );
};
