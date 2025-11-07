import { ModalForm, ProFormText, ProFormTextArea } from '@ant-design/pro-components';
import { Form } from 'antd';
import React, { type JSX } from 'react';

import { dictAPI } from '@/api/system/dict';

interface DictModalFormProps {
  initialValues?: Partial<Dict.Item>;
  mode?: 'create' | 'edit';
  children: JSX.Element;
  onSuccess?: () => void;
}

export const DictModalForm: React.FC<DictModalFormProps> = ({
  children,
  initialValues,
  mode = 'create',
  onSuccess,
}) => {
  const [form] = Form.useForm();

  return (
    <ModalForm<Dict.CreateRequest | Dict.UpdateRequest>
      form={form}
      width={500}
      layout="horizontal"
      title={mode === 'create' ? 'Create Dictionary' : 'Edit Dictionary'}
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
          form.setFieldsValue(initialValues);
        } else {
          form.resetFields();
        }
      }}
      onFinish={async (values) => {
        if (mode === 'create') {
          await dictAPI.create(values as Dict.CreateRequest);
        } else if (mode === 'edit' && initialValues?.id) {
          await dictAPI.update(initialValues.id, values as Dict.UpdateRequest);
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
            message: 'Please enter dictionary type',
          },
          {
            pattern: /^[a-z_]+$/,
            message: 'Dictionary type can only contain lowercase letters and underscores',
          },
        ]}
      />
      <ProFormText
        name="label"
        label="Label"
        placeholder="Enter display label (e.g., Active)"
        rules={[{ required: true, message: 'Please enter label' }]}
      />
      <ProFormText
        name="value"
        label="Value"
        placeholder="Enter value (e.g., 1)"
        rules={[{ required: true, message: 'Please enter value' }]}
      />
      <ProFormTextArea name="description" label="Description" placeholder="Enter description" />
    </ModalForm>
  );
};
