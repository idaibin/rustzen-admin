import { Button, Modal } from "antd";
import { useState, type ReactNode } from "react";

import { t } from "@/lib/i18n";

interface ConfirmModalProps {
    open: boolean;
    title: ReactNode;
    description: ReactNode;
    confirmLabel: ReactNode;
    destructive?: boolean;
    confirmTestId?: string;
    modalTestId?: string;
    disabled?: boolean;
    onCancel: () => void;
    onConfirm: () => Promise<void>;
}

export function ConfirmModal({
    open,
    title,
    description,
    confirmLabel,
    destructive = false,
    confirmTestId,
    modalTestId,
    disabled = false,
    onCancel,
    onConfirm,
}: ConfirmModalProps) {
    const [submitting, setSubmitting] = useState(false);

    const hideDialog = () => {
        if (submitting) {
            return;
        }
        onCancel();
    };

    const submit = async () => {
        if (submitting) {
            return;
        }
        setSubmitting(true);
        try {
            await onConfirm();
            onCancel();
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <Modal
            data-testid={modalTestId}
            open={open}
            closable={!disabled}
            confirmLoading={submitting}
            onCancel={hideDialog}
            title={title}
            footer={[
                <Button key="cancel" type="default" onClick={hideDialog}>
                    {t("取消", "Cancel")}
                </Button>,
                <Button
                    data-testid={confirmTestId}
                    key="confirm"
                    type="primary"
                    danger={destructive}
                    loading={submitting}
                    disabled={disabled}
                    onClick={submit}
                >
                    {confirmLabel}
                </Button>,
            ]}
        >
            <div>{description}</div>
        </Modal>
    );
}

interface ConfirmDialogProps {
    trigger: ReactNode;
    title: ReactNode;
    description: ReactNode;
    confirmLabel: ReactNode;
    destructive?: boolean;
    confirmTestId?: string;
    modalTestId?: string;
    disabled?: boolean;
    onConfirm: () => Promise<void>;
}

export function ConfirmDialog({ trigger, disabled = false, ...modalProps }: ConfirmDialogProps) {
    const [open, setOpen] = useState(false);

    return (
        <>
            <span
                onClick={() => {
                    if (!disabled) {
                        setOpen(true);
                    }
                }}
            >
                {trigger}
            </span>
            <ConfirmModal
                {...modalProps}
                disabled={disabled}
                open={open}
                onCancel={() => setOpen(false)}
            />
        </>
    );
}
