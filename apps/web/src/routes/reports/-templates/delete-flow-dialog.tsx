import { DeleteOutlined } from "@ant-design/icons";
import { Button, Modal } from "antd";
import { useState } from "react";

import { appMessage, reportsAPI } from "@/api";
import { t } from "@/lib/i18n";

export function DeleteFlowDialog({
    flow,
    onDeleted,
}: {
    flow: Reports.Flow;
    onDeleted: () => Promise<unknown>;
}) {
    const [open, setOpen] = useState(false);
    const [submitting, setSubmitting] = useState(false);

    const deleteFlow = async () => {
        setSubmitting(true);
        try {
            await reportsAPI.deleteFlow(flow.id);
            await onDeleted();
            appMessage.success(t("流程已删除", "Template deleted"));
            setOpen(false);
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <>
            <Button
                type="text"
                danger
                icon={<DeleteOutlined />}
                aria-label={t("删除流程", "Delete template")}
                onClick={() => setOpen(true)}
            />
            <Modal
                open={open}
                title={t("删除流程？", "Delete template?")}
                onCancel={() => setOpen(false)}
                footer={null}
                centered
                destroyOnHidden
            >
                <p className="mb-4">
                    {t(
                        "已有填报执行必须不再引用该流程。",
                        "Existing report runs must no longer reference this template.",
                    )}
                </p>
                <div className="flex justify-end gap-2">
                    <Button type="default" onClick={() => setOpen(false)}>
                        {t("取消", "Cancel")}
                    </Button>
                    <Button type="primary" danger loading={submitting} onClick={deleteFlow}>
                        {t("删除", "Delete")}
                    </Button>
                </div>
            </Modal>
        </>
    );
}
