import { GlobalOutlined } from "@ant-design/icons";
import { useMutation } from "@tanstack/react-query";
import { Button, Form, Input, Modal } from "antd";
import { useId, useState } from "react";

import { appMessage, reportsAPI } from "@/api";
import { DialogFooter } from "@/components/feedback/dialog-footer";
import { useSubmission } from "@/hooks/use-submission";
import { t } from "@/lib/i18n";

export function TargetDialog({ onSaved }: { onSaved: () => Promise<unknown> }) {
    const [open, setOpen] = useState(false);
    const fieldId = useId();
    const {
        submitting,
        beginSubmission,
        finishSubmission,
        submissionError,
        failSubmission,
        clearSubmissionError,
    } = useSubmission();
    const [name, setName] = useState("");
    const [baseUrl, setBaseUrl] = useState("");
    const mutation = useMutation({
        onSettled: finishSubmission,
        onError: failSubmission,
        mutationFn: () =>
            reportsAPI.createSystem({
                name: name.trim(),
                baseUrl: baseUrl.trim(),
                enabled: true,
            }),
        onSuccess: async () => {
            await onSaved();
            appMessage.success(t("目标系统已添加", "Target system added"));
            setOpen(false);
        },
    });

    const save = () => {
        if (!name.trim() || !baseUrl.trim() || !beginSubmission()) return;
        mutation.mutate();
    };

    return (
        <>
            <Button
                type="default"
                icon={<GlobalOutlined />}
                onClick={() => {
                    clearSubmissionError();
                    setOpen(true);
                }}
            >
                {t("添加目标系统", "Add target system")}
            </Button>
            <Modal
                open={open}
                closable={!submitting}
                keyboard={!submitting}
                mask={{ closable: !submitting }}
                title={t("添加报表目标", "Add report target")}
                onCancel={() => {
                    if (submitting) return;
                    setOpen(false);
                    setName("");
                    setBaseUrl("");
                }}
                footer={null}
                destroyOnHidden
            >
                <p className="mb-3 text-sm text-muted-foreground">
                    {t(
                        "模板只能在这个可信来源内导航。",
                        "Templates can only navigate within this trusted origin.",
                    )}
                </p>
                <Form layout="vertical" disabled={submitting} onFinish={save}>
                    <Form.Item label={t("名称", "Name")} htmlFor={`${fieldId}-name`}>
                        <Input
                            id={`${fieldId}-name`}
                            value={name}
                            onChange={(event) => setName(event.target.value)}
                        />
                    </Form.Item>
                    <Form.Item label={t("基础地址", "Base URL")} htmlFor={`${fieldId}-base-url`}>
                        <Input
                            id={`${fieldId}-base-url`}
                            value={baseUrl}
                            placeholder="https://example.com"
                            onChange={(event) => setBaseUrl(event.target.value)}
                        />
                    </Form.Item>
                    <DialogFooter
                        error={submissionError}
                        onCancel={() => {
                            if (!submitting) setOpen(false);
                        }}
                        submitLabel={t("添加目标系统", "Add target system")}
                        submitting={submitting}
                        submitDisabled={!name.trim() || !baseUrl.trim()}
                        submitHtmlType="submit"
                    />
                </Form>
            </Modal>
        </>
    );
}
