import { Button } from "antd";
import type { ReactNode } from "react";

import { t } from "@/lib/i18n";

interface DialogFooterProps {
    onCancel: () => void;
    cancelLabel?: ReactNode;
    submitLabel?: ReactNode;
    submitting?: boolean;
    submitDisabled?: boolean;
    submitTestId?: string;
    danger?: boolean;
    submitHtmlType?: "submit" | "button";
    onSubmit?: () => void;
}

export function DialogFooter({
    onCancel,
    cancelLabel,
    submitLabel,
    submitting = false,
    submitDisabled = false,
    submitTestId,
    danger = false,
    submitHtmlType = "button",
    onSubmit,
}: DialogFooterProps) {
    return (
        <div className="flex items-center justify-end gap-2">
            <Button type="default" onClick={onCancel}>
                {cancelLabel ?? t("取消", "Cancel")}
            </Button>
            {submitLabel !== undefined ? (
                <Button
                    data-testid={submitTestId}
                    type="primary"
                    danger={danger}
                    loading={submitting}
                    disabled={submitDisabled}
                    htmlType={submitHtmlType}
                    onClick={onSubmit}
                >
                    {submitLabel}
                </Button>
            ) : null}
        </div>
    );
}
