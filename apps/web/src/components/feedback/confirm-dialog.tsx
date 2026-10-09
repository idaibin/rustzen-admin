import { Alert, Button, Modal } from "antd";
import { useEffect, useRef, useState, type ReactNode, type RefObject } from "react";

import { useSubmission } from "@/hooks/use-submission";
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
    returnFocusRef?: RefObject<HTMLElement | null>;
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
    returnFocusRef,
    onCancel,
    onConfirm,
}: ConfirmModalProps) {
    const { submitting, beginSubmission, finishSubmission } = useSubmission();
    const [error, setError] = useState<string>();
    useEffect(() => {
        if (!open) setError(undefined);
    }, [open]);
    useEffect(() => {
        const trigger = returnFocusRef?.current;
        const main = trigger?.closest("main");
        const fallback = main?.querySelector<HTMLElement>("h1") ?? main;
        return () => {
            // A successful deletion can unmount the dialog together with its row.
            // Ant Design restores surviving triggers; only cover the removed-trigger case.
            queueMicrotask(() => {
                if (
                    trigger &&
                    !trigger.isConnected &&
                    fallback?.isConnected &&
                    document.activeElement === document.body
                ) {
                    fallback.tabIndex = -1;
                    fallback.focus({ preventScroll: true });
                }
            });
        };
    }, [open, returnFocusRef]);

    const hideDialog = () => {
        if (submitting) {
            return;
        }
        onCancel();
    };

    const submit = async () => {
        if (disabled || !beginSubmission()) {
            return;
        }
        setError(undefined);
        try {
            await onConfirm();
            onCancel();
        } catch (error) {
            setError(
                error instanceof Error
                    ? error.message
                    : t("操作失败，请重试。", "The action failed. Please retry."),
            );
        } finally {
            finishSubmission();
        }
    };

    return (
        <Modal
            data-testid={modalTestId}
            open={open}
            closable={!submitting}
            keyboard={!submitting}
            mask={{ closable: !submitting }}
            confirmLoading={submitting}
            onCancel={hideDialog}
            title={title}
            footer={[
                <Button key="cancel" type="default" disabled={submitting} onClick={hideDialog}>
                    {t("取消", "Cancel")}
                </Button>,
                <Button
                    data-testid={confirmTestId}
                    key="confirm"
                    type="primary"
                    danger={destructive}
                    loading={submitting}
                    disabled={disabled}
                    aria-busy={submitting}
                    onClick={submit}
                >
                    {confirmLabel}
                </Button>,
            ]}
        >
            <div>{description}</div>
            {error ? (
                <Alert type="error" showIcon title={error} role="alert" className="mt-4" />
            ) : null}
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
    const triggerRef = useRef<HTMLSpanElement>(null);

    return (
        <>
            <span
                ref={triggerRef}
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
                returnFocusRef={triggerRef}
                disabled={disabled}
                open={open}
                onCancel={() => setOpen(false)}
            />
        </>
    );
}
