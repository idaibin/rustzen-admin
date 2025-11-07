import type { ActionType, ProColumns } from '@ant-design/pro-components';
import { ProTable } from '@ant-design/pro-components';
import { Button, Space } from 'antd';
import React, { useRef } from 'react';

import { roleAPI } from '@/api/system/role';
import { AuthPopconfirm, AuthWrap } from '@/components/auth';

import { RoleModalForm } from './RoleModalForm';

export default function RolePage() {
  const actionRef = useRef<ActionType>(null);

  return (
    <ProTable<Role.Item>
      rowKey="id"
      scroll={{ y: 'calc(100vh - 383px)' }}
      headerTitle="Role Management"
      columns={columns}
      request={roleAPI.getTableData}
      actionRef={actionRef}
      search={{ span: 6 }}
      toolBarRender={() => [
        <AuthWrap code="system:role:create">
          <RoleModalForm
            mode={'create'}
            onSuccess={() => {
              actionRef.current?.reload();
            }}
          >
            <Button type="primary">Create Role</Button>
          </RoleModalForm>
        </AuthWrap>,
      ]}
    />
  );
}
const columns: ProColumns<Role.Item>[] = [
  {
    title: 'ID',
    dataIndex: 'id',
    width: 48,
    search: false,
  },
  {
    title: 'Role Name',
    dataIndex: 'name',
    width: 200,
    ellipsis: true,
  },
  {
    title: 'Role Code',
    dataIndex: 'code',
    width: 200,
    ellipsis: true,
  },
  {
    title: 'Description',
    dataIndex: 'description',
    ellipsis: true,
    hideInSearch: true,
  },
  {
    title: 'Status',
    dataIndex: 'status',
    valueType: 'select',
    width: 120,
    valueEnum: {
      1: { text: 'Enabled', status: 'Success' },
      2: { text: 'Disabled', status: 'Default' },
    },
  },
  {
    title: 'Permissions',
    dataIndex: 'menus',
    width: 160,
    hideInSearch: true,
    render: (_, record) => {
      if (!record.menus || record.menus.length === 0) {
        return <span style={{ color: '#999' }}>No permissions</span>;
      }
      return (
        <span title={record.menus.map((menu) => menu.label).join(', ')}>
          {record.menus.length} permission(s)
        </span>
      );
    },
  },
  {
    title: 'Updated At',
    dataIndex: 'updatedAt',
    valueType: 'dateTime',
    width: 160,
    hideInSearch: true,
  },
  {
    title: 'Actions',
    key: 'action',
    width: 110,
    hideInSearch: true,
    render: (_dom: React.ReactNode, entity: Role.Item, _index, action?: ActionType) => {
      const isSystemRole = entity.id === 1;
      return (
        <Space size="middle">
          <AuthWrap code="system:role:edit" hidden={isSystemRole}>
            <RoleModalForm
              mode={'edit'}
              initialValues={entity}
              onSuccess={() => {
                action?.reload();
              }}
            >
              <a>Edit</a>
            </RoleModalForm>
          </AuthWrap>
          <AuthPopconfirm
            hidden={isSystemRole}
            code="system:role:delete"
            title="Are you sure you want to delete this role?"
            description="This action cannot be undone."
            onConfirm={async () => {
              await roleAPI.delete(entity.id);
              action?.reload();
            }}
          >
            <span className="cursor-pointer text-red-500">Delete</span>
          </AuthPopconfirm>
        </Space>
      );
    },
  },
];
