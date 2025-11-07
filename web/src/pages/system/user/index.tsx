import type { ActionType, ProColumns } from '@ant-design/pro-components';
import { ProTable } from '@ant-design/pro-components';
import { Button, Space } from 'antd';
import React, { useRef } from 'react';

import { userAPI } from '@/api/system/user';
import { AuthConfirm, AuthWrap } from '@/components/auth';
import { MoreButton } from '@/components/button';
import { useAuthStore } from '@/stores/useAuthStore';

import { UserModalForm } from './UserModalForm';

export default function UserPage() {
  const actionRef = useRef<ActionType>(null);
  return (
    <ProTable<User.Item>
      rowKey="id"
      scroll={{ y: 'calc(100vh - 383px)' }}
      headerTitle="User List"
      columns={columns}
      request={userAPI.getTableData}
      actionRef={actionRef}
      search={{ span: 6 }}
      toolBarRender={() => [
        <AuthWrap code="system:user:create">
          <UserModalForm
            mode={'create'}
            onSuccess={() => {
              actionRef.current?.reload();
            }}
          >
            <Button type="primary">Create User</Button>
          </UserModalForm>
        </AuthWrap>,
      ]}
    />
  );
}

const columns: ProColumns<User.Item>[] = [
  {
    title: 'ID',
    dataIndex: 'id',
    width: 48,
    search: false,
  },
  {
    title: 'Avatar',
    dataIndex: 'avatarUrl',
    width: 60,
    search: false,
    render: (_, record) => {
      if (!record.avatarUrl) {
        return null;
      }
      return (
        <img
          src={record.avatarUrl}
          alt="avatar"
          className="object-fit mx-auto h-5 w-5 rounded-full"
        />
      );
    },
  },
  {
    title: 'Username',
    dataIndex: 'username',
  },
  {
    title: 'Email',
    dataIndex: 'email',
  },
  {
    title: 'Real Name',
    dataIndex: 'realName',
  },
  {
    title: 'Status',
    dataIndex: 'status',
    valueEnum: {
      1: { text: 'Enabled', status: 'Success' },
      2: { text: 'Disabled', status: 'Default' },
    },
  },
  {
    title: 'Roles',
    dataIndex: 'roles',
    search: false,
    render: (_: React.ReactNode, record: User.Item) =>
      record.roles.map((role) => role.label).join(', '),
  },
  {
    title: 'Last Login',
    dataIndex: 'lastLoginAt',
    valueType: 'dateTime',
    search: false,
  },
  {
    title: 'Updated At',
    dataIndex: 'updatedAt',
    valueType: 'dateTime',
    search: false,
  },
  {
    title: 'Actions',
    key: 'action',
    width: 110,
    search: false,
    render: (_dom: React.ReactNode, entity: User.Item, _index, action?: ActionType) => {
      const cur = useAuthStore.getState().userInfo;
      if (entity.id === cur?.id || entity.id === 1) {
        return null;
      }
      const status = entity.status === 1 ? 'Disable' : 'Enable';
      return (
        <Space size="middle">
          <AuthWrap code="system:user:edit">
            <UserModalForm
              mode={'edit'}
              initialValues={entity}
              onSuccess={() => {
                action?.reload();
              }}
            >
              <a>Edit</a>
            </UserModalForm>
          </AuthWrap>
          <MoreButton>
            <AuthConfirm
              key="status"
              code="system:user:status"
              title={`Are you sure you want to ${status} this user?`}
              children={status}
              onConfirm={async () => {
                await userAPI.updateStatus(entity.id, entity.status === 1 ? 2 : 1);
                action?.reload();
              }}
            />
            <AuthConfirm
              key="password"
              code="system:user:password"
              title="Are you sure you want to reset the password of this user?"
              children="Reset Password"
              onConfirm={async () => {
                await userAPI.resetPassword(entity.id, `${entity.username}@123456`);
                action?.reload();
              }}
            />
            <AuthConfirm
              key="delete"
              code="system:user:delete"
              title="Are you sure you want to delete this user?"
              className="text-red-500"
              children={'Delete User'}
              onConfirm={async () => {
                await userAPI.delete(entity.id);
                action?.reload();
              }}
            />
          </MoreButton>
        </Space>
      );
    },
  },
];
