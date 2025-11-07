import type { ActionType, ProColumns } from '@ant-design/pro-components';
import { ProTable } from '@ant-design/pro-components';
import { Button, Space, Tag } from 'antd';
import React, { useRef } from 'react';

import { dictAPI } from '@/api/system/dict';
import { AuthPopconfirm, AuthWrap } from '@/components/auth';

import { DictModalForm } from './DictModalForm';

export default function DictPage() {
  const actionRef = useRef<ActionType>(null);

  return (
    <ProTable<Dict.Item>
      rowKey="id"
      search={false}
      scroll={{ y: 'calc(100vh - 287px)' }}
      headerTitle="Dictionary Management"
      columns={columns}
      request={dictAPI.getTableData}
      actionRef={actionRef}
      toolBarRender={() => [
        <AuthWrap code="system:dict:create">
          <DictModalForm
            mode={'create'}
            onSuccess={() => {
              actionRef.current?.reload();
            }}
          >
            <Button type="primary">Create Dictionary</Button>
          </DictModalForm>
        </AuthWrap>,
      ]}
    />
  );
}

const columns: ProColumns<Dict.Item>[] = [
  {
    title: 'ID',
    dataIndex: 'id',
    width: 48,
  },
  {
    title: 'Dict Type',
    dataIndex: 'dictType',
    ellipsis: true,
    render: (text) => <Tag color="blue">{text}</Tag>,
  },
  {
    title: 'Label',
    dataIndex: 'label',
    ellipsis: true,
    search: {
      transform: (value) => ({ q: value }),
    },
  },
  {
    title: 'Value',
    dataIndex: 'value',
    ellipsis: true,
  },
  {
    title: 'Description',
    dataIndex: 'description',
    ellipsis: true,
  },
  {
    title: 'Actions',
    key: 'action',
    width: 110,
    fixed: 'right',
    render: (_dom: React.ReactNode, entity: Dict.Item, _index, action?: ActionType) => (
      <Space size="middle">
        <AuthWrap code="system:dict:edit">
          <DictModalForm
            mode={'edit'}
            initialValues={entity}
            onSuccess={() => {
              action?.reload();
            }}
          >
            <a>Edit</a>
          </DictModalForm>
        </AuthWrap>
        <AuthPopconfirm
          code="system:dict:delete"
          title="Are you sure you want to delete this dictionary?"
          description="This action cannot be undone."
          onConfirm={async () => {
            await dictAPI.delete(entity.id);
            action?.reload();
          }}
        >
          <span className="cursor-pointer text-red-500">Delete</span>
        </AuthPopconfirm>
      </Space>
    ),
  },
];
