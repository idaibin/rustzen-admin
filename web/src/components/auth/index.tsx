import { Popconfirm } from 'antd';
import React from 'react';

import { appModal } from '@/api';
import { useAuthStore } from '@/stores/useAuthStore';

interface AuthWrapProps {
  code: string;
  children: React.ReactNode;
  hidden?: boolean;
  fallback?: React.ReactNode;
}

export const AuthWrap: React.FC<AuthWrapProps> = ({ code, children, hidden = false, fallback = null }) => {
  const isPermission = useAuthStore.getState().checkPermissions(code);
  if (isPermission && !hidden) {
    return children;
  }
  return fallback;
};

interface AuthPopconfirmProps extends AuthWrapProps {
  title: React.ReactNode;
  description?: React.ReactNode;
  onConfirm: () => Promise<void>;
  onCancel?: () => Promise<void>;
}

export const AuthPopconfirm: React.FC<AuthPopconfirmProps> = ({
  code,
  children,
  hidden = false,
  title,
  description,
  onConfirm,
  onCancel,
}) => {
  return (
    <AuthWrap code={code} hidden={hidden}>
      <Popconfirm
        placement="leftBottom"
        title={title}
        description={description}
        onConfirm={onConfirm}
        onCancel={onCancel}
      >
        {children}
      </Popconfirm>
    </AuthWrap>
  );
};

interface AuthConfirmProps extends AuthPopconfirmProps {
  className?: string;
}

export const AuthConfirm: React.FC<AuthConfirmProps> = (props) => {
  const handleConfirm = () => {
    appModal.confirm({
      title: props.title,
      content: props.description,
      onOk: props.onConfirm,
      onCancel: props.onCancel,
    });
  };

  return (
    <AuthWrap code={props.code} hidden={props.hidden}>
      <span onClick={handleConfirm} className={props.className}>
        {props.children}
      </span>
    </AuthWrap>
  );
};
