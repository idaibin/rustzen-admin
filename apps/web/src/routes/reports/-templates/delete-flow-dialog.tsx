import { DeleteOutlined } from "@ant-design/icons";
import { Button } from "antd";

import { appMessage, reportsAPI } from "@/api";
import { ConfirmDialog } from "@/components/feedback/confirm-dialog";
import { t } from "@/lib/i18n";

export function DeleteFlowDialog({
    flow,
    onDeleted,
}: {
    flow: Reports.Flow;
    onDeleted: () => Promise<unknown>;
}) {
    const deleteFlow = async () => {
        await reportsAPI.deleteFlow(flow.id);
        await onDeleted();
        appMessage.success(t("流程已删除", "Template deleted"));
    };

    return (
        <ConfirmDialog
            trigger={
                <Button
                    type="text"
                    danger
                    icon={<DeleteOutlined />}
                    aria-label={t("删除流程", "Delete template")}
                />
            }
            title={t("删除流程？", "Delete template?")}
            description={t(
                "已有填报执行必须不再引用该流程。",
                "Existing report runs must no longer reference this template.",
            )}
            confirmLabel={t("删除", "Delete")}
            destructive
            onConfirm={deleteFlow}
        />
    );
}
