import { EditOutlined } from "@ant-design/icons";
import { useMutation } from "@tanstack/react-query";
import { Alert, Button, Form, Input, Modal, Select, Switch } from "antd";
import { useEffect, useRef, useState } from "react";

import { appMessage, reportsAPI } from "@/api";
import { DialogFooter } from "@/components/feedback/dialog-footer";
import { t } from "@/lib/i18n";

import {
    beginScheduleDialogCycle,
    createScheduleSaveHandlers,
    endScheduleDialogCycle,
    initialScheduleDialogCycle,
    isCurrentScheduleDialogCycle,
    markScheduleDraftInitialized,
    shouldInitializeScheduleDraft,
} from "./schedule-save-state";

export function ScheduleDialog({
    flowOptions,
    schedule,
    onSaved,
}: {
    flowOptions: Reports.FlowOption[];
    schedule?: Reports.Schedule;
    onSaved: () => Promise<unknown>;
}) {
    const [open, setOpen] = useState(false);
    const [flowId, setFlowId] = useState("");
    const [cadence, setCadence] = useState<Reports.ScheduleCadence>("daily");
    const [weekday, setWeekday] = useState<number>(1);
    const [dueTime, setDueTime] = useState("09:00");
    const [inputJson, setInputJson] = useState("{}");
    const [description, setDescription] = useState("");
    const [enabled, setEnabled] = useState(true);
    const [saveError, setSaveError] = useState<string>();
    const cycle = useRef(initialScheduleDialogCycle);

    const closeDialog = () => {
        cycle.current = endScheduleDialogCycle(cycle.current);
        setSaveError(undefined);
        setOpen(false);
    };

    const openDialog = () => {
        cycle.current = beginScheduleDialogCycle(cycle.current);
        setSaveError(undefined);
        setOpen(true);
    };

    useEffect(() => {
        if (!open || !shouldInitializeScheduleDraft(cycle.current)) return;
        setFlowId(schedule?.flowId ?? flowOptions.find((flow) => flow.enabled)?.id ?? "");
        setCadence(schedule?.cadence ?? "daily");
        setWeekday(schedule?.weekday ?? 1);
        setDueTime(schedule?.dueTime ?? "09:00");
        setInputJson(JSON.stringify(schedule?.input ?? {}, null, 2));
        setDescription(schedule?.description ?? "");
        setEnabled(schedule?.enabled ?? true);
        cycle.current = markScheduleDraftInitialized(cycle.current);
    }, [open]);

    const saveHandlers = createScheduleSaveHandlers({
        refresh: onSaved,
        isCurrent: (saveCycle) => isCurrentScheduleDialogCycle(cycle.current, saveCycle),
        showSuccess: () =>
            appMessage.success(
                schedule
                    ? t("计划已更新", "Schedule updated")
                    : t("计划已创建", "Schedule created"),
            ),
        close: closeDialog,
        showError: setSaveError,
        fallbackError: t("计划保存失败，请稍后重试。", "Unable to save the schedule. Try again."),
    });
    const mutation = useMutation({
        mutationFn: ({ input }: { input: Reports.SaveSchedule; cycle: number }) =>
            schedule
                ? reportsAPI.updateSchedule(schedule.id, input)
                : reportsAPI.createSchedule(input),
        ...saveHandlers,
    });

    const save = () => {
        setSaveError(undefined);
        try {
            const input = JSON.parse(inputJson) as Record<string, unknown>;
            if (!input || Array.isArray(input) || typeof input !== "object") throw new Error();
            if (!flowId || !dueTime || (cadence === "weekly" && (weekday < 0 || weekday > 6))) {
                throw new Error();
            }
            const selectedFlow = flowOptions.find((flow) => flow.id === flowId);
            if (!selectedFlow?.enabled) {
                throw new Error(t("所选流程目标已停用。", "The selected flow target is disabled."));
            }
            mutation.mutate({
                input: {
                    flowId,
                    cadence,
                    weekday: cadence === "weekly" ? weekday : undefined,
                    dueTime,
                    input,
                    description: description.trim(),
                    enabled,
                },
                cycle: cycle.current.id,
            });
        } catch {
            appMessage.error(
                t(
                    "请填写完整的计划字段，并提供有效的 JSON 对象。",
                    "Complete the schedule fields and provide a valid JSON object.",
                ),
            );
        }
    };

    return (
        <>
            <Button
                data-testid={schedule ? "schedule-edit" : "schedule-create"}
                type={schedule ? "text" : "primary"}
                icon={schedule ? <EditOutlined /> : undefined}
                aria-label={schedule ? t("编辑计划", "Edit schedule") : undefined}
                onClick={openDialog}
                disabled={!flowOptions.some((flow) => flow.enabled)}
            >
                {schedule ? null : t("新建计划", "New schedule")}
            </Button>
            <Modal
                data-testid="schedule-dialog"
                open={open}
                title={
                    schedule
                        ? t("编辑定时报表计划", "Edit scheduled report")
                        : t("新建定时报表计划", "New scheduled report")
                }
                onCancel={closeDialog}
                footer={null}
                width={760}
                destroyOnHidden
            >
                <Alert
                    className="mb-4"
                    type="warning"
                    showIcon
                    message={t(
                        "不要提交密码、Token、密钥或其他敏感信息。",
                        "Do not submit passwords, tokens, keys, or other sensitive information.",
                    )}
                />
                {saveError ? (
                    <Alert
                        data-testid="schedule-save-error"
                        className="mb-4"
                        type="error"
                        showIcon
                        message={t("计划未保存", "Schedule was not saved")}
                        description={saveError}
                    />
                ) : null}
                <Form layout="vertical">
                    <Form.Item label={t("流程", "Template")} required>
                        <Select
                            data-testid="schedule-flow"
                            value={flowId || undefined}
                            onChange={(value) => setFlowId(value)}
                            options={flowOptions.map((flow) => ({
                                value: flow.id,
                                label: flow.enabled
                                    ? flow.name
                                    : `${flow.name} (${t("目标已停用", "Target disabled")})`,
                                disabled: !flow.enabled,
                            }))}
                            placeholder={t("选择流程", "Select a template")}
                        />
                    </Form.Item>
                    <div className="grid gap-4 sm:grid-cols-2">
                        <Form.Item label={t("频率", "Cadence")} required>
                            <Select
                                data-testid="schedule-cadence"
                                value={cadence}
                                onChange={(value: Reports.ScheduleCadence) => setCadence(value)}
                                options={[
                                    {
                                        value: "daily",
                                        label: (
                                            <span data-testid="schedule-cadence-daily">
                                                {t("每日", "Daily")}
                                            </span>
                                        ),
                                    },
                                    {
                                        value: "weekly",
                                        label: (
                                            <span data-testid="schedule-cadence-weekly">
                                                {t("每周", "Weekly")}
                                            </span>
                                        ),
                                    },
                                ]}
                            />
                        </Form.Item>
                        {cadence === "weekly" ? (
                            <Form.Item label={t("星期", "Weekday")} required>
                                <Select
                                    data-testid="schedule-weekday"
                                    value={weekday}
                                    onChange={(value) => setWeekday(value)}
                                    options={[
                                        {
                                            value: 0,
                                            label: (
                                                <span data-testid="schedule-weekday-0">
                                                    {t("周一", "Monday")}
                                                </span>
                                            ),
                                        },
                                        { value: 1, label: t("周二", "Tuesday") },
                                        { value: 2, label: t("周三", "Wednesday") },
                                        { value: 3, label: t("周四", "Thursday") },
                                        { value: 4, label: t("周五", "Friday") },
                                        { value: 5, label: t("周六", "Saturday") },
                                        { value: 6, label: t("周日", "Sunday") },
                                    ]}
                                />
                            </Form.Item>
                        ) : null}
                    </div>
                    <Form.Item label={t("本地执行时间", "Local due time")} required>
                        <Input
                            data-testid="schedule-due-time"
                            type="time"
                            value={dueTime}
                            onChange={(event) => setDueTime(event.target.value)}
                        />
                    </Form.Item>
                    <Form.Item label={t("安全输入 JSON", "Safe input JSON")}>
                        <Input.TextArea
                            className="font-mono text-xs"
                            rows={7}
                            value={inputJson}
                            onChange={(event) => setInputJson(event.target.value)}
                        />
                    </Form.Item>
                    <Form.Item label={t("描述", "Description")}>
                        <Input
                            data-testid="schedule-description"
                            value={description}
                            onChange={(event) => setDescription(event.target.value)}
                        />
                    </Form.Item>
                    <Form.Item label={t("启用计划", "Enable schedule")}>
                        <Switch
                            data-testid="schedule-enabled"
                            checked={enabled}
                            onChange={setEnabled}
                        />
                    </Form.Item>
                    <DialogFooter
                        onCancel={closeDialog}
                        submitLabel={t("校验并保存", "Validate and save")}
                        submitting={mutation.isPending}
                        submitDisabled={mutation.isPending}
                        submitTestId="schedule-save"
                        onSubmit={save}
                    />
                </Form>
            </Modal>
        </>
    );
}
