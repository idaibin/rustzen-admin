import { Popconfirm } from "antd";
import React from "react";

import { appModal } from "@/api";
import { useAuthStore } from "@/store/useAuthStore";

interface AuthWrapProps {
    code: string;
    children: React.ReactNode;
    hidden?: boolean;
    fallback?: React.ReactNode;
}

export const AuthWrap: React.FC<AuthWrapProps> = ({
    code,
    children,
    hidden = false,
    fallback = null,
}) => {
    const isPermission = useAuthStore((state) => state.checkPermissions(code));
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
    const handleConfirm = (event: React.MouseEvent<HTMLButtonElement>) => {
        event.preventDefault();
        event.stopPropagation();

        appModal.confirm({
            title: props.title,
            content: props.description,
            onOk: props.onConfirm,
            onCancel: props.onCancel,
        });
    };

    return (
        <AuthWrap code={props.code} hidden={props.hidden}>
            <button
                type="button"
                onClick={handleConfirm}
                className={`rounded border-0 bg-transparent p-0 text-left ${props.className || ""}`}
            >
                {props.children}
            </button>
        </AuthWrap>
    );
};
