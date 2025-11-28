import type { ProColumns } from '@ant-design/pro-components';
import { ProTable } from '@ant-design/pro-components';
import { Button, Segmented, Tag } from 'antd';

import { logAPI } from '@/api/system/log';
import { useLocalStore } from '@/stores/useLocalStore';

const actionOptions = [
  { label: 'All', value: '' },
  { label: 'Login', value: 'AUTH_LOGIN' },
  { label: 'GET', value: 'HTTP_GET' },
  { label: 'POST', value: 'HTTP_POST' },
  { label: 'PUT', value: 'HTTP_PUT' },
  { label: 'DELETE', value: 'HTTP_DELETE' },
];
export default function LogPage() {
  const [actionType, setActionType] = useLocalStore('log-action');
  return (
    <ProTable<Log.Item>
      rowKey="id"
      scroll={{ y: 'calc(100vh - 383px)' }}
      columns={columns}
      params={{ action: actionType }}
      request={logAPI.getTableData}
      headerTitle={
        <Segmented
          value={actionType}
          options={actionOptions}
          onChange={(val) => {
            setActionType(val);
          }}
        />
      }
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

const actionColorMap: Record<Log.Action, string> = {
  HTTP_GET: 'default',
  HTTP_POST: 'processing',
  HTTP_PUT: 'warning',
  HTTP_DELETE: 'error',
  AUTH_LOGIN: 'success',
};
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
      const color = actionColorMap[action];
      return (
        <Tag color={color} variant="outlined">
          {action}
        </Tag>
      );
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
      return (
        <Tag color={color} variant="solid">
          {status}
        </Tag>
      );
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
