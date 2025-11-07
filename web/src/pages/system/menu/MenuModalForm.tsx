import { ModalForm, ProFormDigit, ProFormSelect, ProFormText } from '@ant-design/pro-components';
import { Form } from 'antd';
import React, { type JSX } from 'react';

import { menuAPI } from '@/api/system/menu';
import { ENABLE_OPTIONS, MENU_TYPE_OPTIONS } from '@/constant/options';

interface MenuModalFormProps {
  initialValues?: Partial<Menu.Item>;
  mode?: 'create' | 'edit';
  children: JSX.Element;
  onSuccess?: () => void;
}

export const MenuModalForm: React.FC<MenuModalFormProps> = ({
  children,
  initialValues,
  mode = 'create',
  onSuccess,
}) => {
  const [form] = Form.useForm();

  return (
    <ModalForm<Menu.CreateAndUpdateRequest>
      form={form}
      width={600}
      layout="horizontal"
      title={mode === 'create' ? 'Create Menu' : 'Edit Menu'}
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
          await menuAPI.create(values as Menu.CreateAndUpdateRequest);
        } else if (mode === 'edit' && initialValues?.id) {
          await menuAPI.update(initialValues.id, values as Menu.CreateAndUpdateRequest);
        }
        onSuccess?.();
        return true;
      }}
    >
      <ProFormSelect
        name="parentId"
        label="Parent Menu"
        placeholder="Select parent menu (optional)"
        request={menuAPI.getOptions}
        fieldProps={{
          // showSearch: true,
          optionFilterProp: 'label',
        }}
        rules={[{ required: true, message: 'Please select parent menu' }]}
      />
      <ProFormText
        name="name"
        label="Menu Name"
        placeholder="Enter menu name"
        rules={[{ required: true, message: 'Please enter menu name' }]}
      />
      <ProFormText
        name="code"
        label="Permission Code"
        placeholder="Enter permission code (e.g., system:menu:list)"
        rules={[{ required: true, message: 'Please enter permission code' }]}
      />
      <ProFormSelect
        label="Type"
        name="menuType"
        options={MENU_TYPE_OPTIONS}
        rules={[{ required: true, message: 'Please select menu type' }]}
      />
      <ProFormSelect
        name="status"
        label="Status"
        placeholder="Select status"
        options={ENABLE_OPTIONS}
        rules={[{ required: true, message: 'Please select status' }]}
      />
      <ProFormDigit
        name="sortOrder"
        label="Sort Order"
        placeholder="Enter sort order"
        min={0}
        fieldProps={{ precision: 0 }}
      />
    </ModalForm>
  );
};
