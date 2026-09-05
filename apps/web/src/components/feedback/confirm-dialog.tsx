import { Button, Modal } from "antd";
import { useState, type ReactNode } from "react";

import { t } from "@/lib/i18n";

interface ConfirmDialogProps {
    trigger: ReactNode;
    title: ReactNode;
    description: ReactNode;
    confirmLabel: ReactNode;
    destructive?: boolean;
    confirmTestId?: string;
    disabled?: boolean;
    onConfirm: () => Promise<void>;
}

export function ConfirmDialog({
    trigger,
    title,
    description,
    confirmLabel,
    destructive = false,
    confirmTestId,
    disabled = false,
    onConfirm,
}: ConfirmDialogProps) {
    const [open, setOpen] = useState(false);
    const [submitting, setSubmitting] = useState(false);

    const showDialog = () => {
        if (disabled || submitting) {
            return;
        }
        setOpen(true);
    };

    const hideDialog = () => {
        if (submitting) {
            return;
        }
        setOpen(false);
    };

    const submit = async () => {
        if (submitting) {
            return;
        }
        setSubmitting(true);
        try {
            await onConfirm();
            setOpen(false);
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <>
            <span onClick={showDialog}>{trigger}</span>
            <Modal
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
        </>
    );
}
