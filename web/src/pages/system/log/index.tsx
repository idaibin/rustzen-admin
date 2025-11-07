import type { ProColumns } from '@ant-design/pro-components';
import { ProTable } from '@ant-design/pro-components';
import { Button, Tag } from 'antd';

import { logAPI } from '@/api/system/log';

export default function LogPage() {
  return (
    <ProTable<Log.Item>
      rowKey="id"
      scroll={{ y: 'calc(100vh - 383px)' }}
      headerTitle={'Operation Log'}
      columns={columns}
      request={logAPI.getTableData}
      toolBarRender={() => [
        <Button
          key="export"
          type="primary"
          onClick={() => {
            logAPI.exportLogList();
          }}
        >
          Export
        </Button>,
      ]}
    />
  );
}

const columns: ProColumns<Log.Item>[] = [
  {
    title: 'ID',
    dataIndex: 'id',
    width: 80,
    hideInSearch: true,
  },
  {
    title: 'User',
    dataIndex: 'username',
    width: 120,
    render: (_, record) => record.username || 'Anonymous User',
  },
  {
    title: 'Action',
    dataIndex: 'action',
    width: 150,
    hideInSearch: true,
    render: (_, record) => {
      const action = record.action;
      let color = 'default';
      if (action.startsWith('HTTP_')) {
        color = 'blue';
      } else if (action.includes('CREATE')) {
        color = 'green';
      } else if (action.includes('UPDATE')) {
        color = 'orange';
      } else if (action.includes('DELETE')) {
        color = 'red';
      }
      return <Tag color={color}>{action}</Tag>;
    },
  },
  {
    title: 'Description',
    dataIndex: 'description',
    ellipsis: true,
  },
  {
    title: 'Status',
    dataIndex: 'status',
    width: 100,
    hideInSearch: true,
    render: (_, record) => {
      const status = record.status;
      const color = status === 'SUCCESS' ? 'success' : 'error';
      return <Tag color={color}>{status}</Tag>;
    },
  },
  {
    title: 'IP Address',
    dataIndex: 'ipAddress',
    width: 120,
    render: (_, record) => record.ipAddress || '-',
  },
  {
    title: 'Duration',
    dataIndex: 'durationMs',
    width: 80,
    hideInSearch: true,
    render: (_, record) => {
      if (!record.durationMs) return '-';
      return `${record.durationMs}ms`;
    },
  },
  {
    title: 'Created At',
    dataIndex: 'createdAt',
    width: 180,
    valueType: 'dateTime',
    hideInSearch: true,
  },
];
