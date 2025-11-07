import { ModalForm, ProFormText } from '@ant-design/pro-components';
import { Form } from 'antd';

import { useAuthStore } from '@/stores/useAuthStore';

import { UserAvatar } from './avatar';

export const UserProfileModal = () => {
  const { userInfo } = useAuthStore();
  const [form] = Form.useForm();
  return (
    <ModalForm
      readonly
      form={form}
      title="User Profile"
      trigger={<span>Profile</span>}
      layout="horizontal"
      labelCol={{ span: 6 }}
      submitter={false}
      onOpenChange={(visible) => {
        if (visible) {
          form.setFieldsValue(userInfo);
        }
      }}
    >
      <div className="flex pt-5">
        <div className="flex-1">
          <ProFormText name="username" label="Username" readonly />
          <ProFormText name="email" label="Email" readonly />
          <ProFormText name="phone" label="Phone" />
          <ProFormText name="realName" label="Real Name" />
        </div>
        <div className="flex w-70 flex-none flex-col items-center p-10">
          <UserAvatar />
        </div>
      </div>
    </ModalForm>
  );
};
