import { EditOutlined, PlusCircleOutlined } from "@ant-design/icons";
import { useMutation } from "@tanstack/react-query";
import { Button, Form, Input, Modal, Select } from "antd";
import { useEffect, useId, useState } from "react";

import { appMessage, reportsAPI } from "@/api";
import { DialogFooter } from "@/components/feedback/dialog-footer";
import { useSubmission } from "@/hooks/use-submission";
import { t } from "@/lib/i18n";

const example: Reports.FlowStep[] = [
    { action: "goto", url: "/report" },
    { action: "fill", selector: "#value", value: "{{input.value}}" },
    { action: "click", selector: "button[type=submit]" },
    { action: "waitFor", selector: "form" },
    { action: "screenshot", name: "submitted" },
];

export function FlowDialog({
    systems,
    flow,
    onSaved,
}: {
    systems: Reports.System[];
    flow?: Reports.Flow;
    onSaved: () => Promise<unknown>;
}) {
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
    const [systemId, setSystemId] = useState("");
    const [json, setJson] = useState("");

    useEffect(() => {
        if (open) {
            setName(flow?.name ?? "");
            setSystemId(flow?.systemId ?? systems[0]?.id ?? "");
            setJson(JSON.stringify(flow?.steps ?? example, null, 2));
        }
    }, [open, flow, systems]);

    const mutation = useMutation({
        onSettled: finishSubmission,
        onError: failSubmission,
        mutationFn: (input: Reports.SaveFlow) =>
            flow ? reportsAPI.updateFlow(flow.id, input) : reportsAPI.createFlow(input),
        onSuccess: async () => {
            await onSaved();
            appMessage.success(
                flow ? t("流程已更新", "Template updated") : t("流程已创建", "Template created"),
            );
            setOpen(false);
        },
    });

    const save = () => {
        try {
            const steps = JSON.parse(json) as Reports.FlowStep[];
            if (!Array.isArray(steps)) {
                throw new Error();
            }
            if (!name.trim() || !systemId || !beginSubmission()) return;
            mutation.mutate({ name, systemId, steps });
        } catch {
            appMessage.error(t("步骤必须是有效的 JSON 数组", "Steps must be a valid JSON array"));
        }
    };

    return (
        <>
            <Button
                type={flow ? "text" : "primary"}
                icon={flow ? <EditOutlined /> : <PlusCircleOutlined />}
                aria-label={flow ? t("编辑模板", "Edit template") : undefined}
                disabled={!systems.length}
                onClick={() => {
                    clearSubmissionError();
                    setOpen(true);
                }}
            >
                {flow ? null : t("新建模板", "New template")}
            </Button>
            <Modal
                open={open}
                closable={!submitting}
                keyboard={!submitting}
                mask={{ closable: !submitting }}
                title={
                    flow ? t("编辑模板", "Edit template") : t("新建报表模板", "New report template")
                }
                onCancel={() => {
                    if (!submitting) setOpen(false);
                }}
                footer={null}
                width={760}
                destroyOnHidden
            >
                <p className="mb-4 text-sm text-muted-foreground">
                    {t(
                        "支持的动作：goto、fill、click、waitFor、assertText、assertValue、assertAbsent、screenshot、screenshotViewport、setViewport、setUiPreferences、assertNoHorizontalOverflow、assertElementLayout、guardExists、pressKey、pause。",
                        "Supported actions: goto, fill, click, waitFor, assertText, assertValue, assertAbsent, screenshot, screenshotViewport, setViewport, setUiPreferences, assertNoHorizontalOverflow, assertElementLayout, guardExists, pressKey, pause.",
                    )}
                </p>
                <Form layout="vertical" disabled={submitting} onFinish={save}>
                    <div className="grid gap-4 sm:grid-cols-2">
                        <Form.Item label={t("名称", "Name")} htmlFor={`${fieldId}-name`}>
                            <Input
                                id={`${fieldId}-name`}
                                value={name}
                                onChange={(event) => setName(event.target.value)}
                            />
                        </Form.Item>
                        <Form.Item label={t("系统", "System")} htmlFor={`${fieldId}-system`}>
                            <Select
                                id={`${fieldId}-system`}
                                value={systemId}
                                onChange={(value) => setSystemId(value)}
                                options={systems.map((s) => ({ value: s.id, label: s.name }))}
                                placeholder={t("选择系统", "Select a system")}
                                allowClear={false}
                            />
                        </Form.Item>
                    </div>
                    <Form.Item label={t("步骤 JSON", "Steps JSON")} htmlFor={`${fieldId}-steps`}>
                        <Input.TextArea
                            id={`${fieldId}-steps`}
                            className="font-mono text-xs"
                            rows={15}
                            value={json}
                            onChange={(event) => setJson(event.target.value)}
                        />
                    </Form.Item>
                    <DialogFooter
                        error={submissionError}
                        onCancel={() => {
                            if (!submitting) setOpen(false);
                        }}
                        submitLabel={t("校验并保存", "Validate and save")}
                        submitting={submitting}
                        submitDisabled={!name || !systemId}
                        submitHtmlType="submit"
                    />
                </Form>
            </Modal>
        </>
    );
}
