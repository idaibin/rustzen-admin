import { useMutation } from "@tanstack/react-query";
import { Switch, Typography } from "antd";
import { useRef, useState } from "react";

import { reportsAPI } from "@/api";
import { t } from "@/lib/i18n";

import {
    isCurrentScheduleToggle,
    settleScheduleToggle,
    startScheduleToggle,
    type ScheduleToggleRequest,
    type ScheduleToggleTracker,
} from "../-schedule-toggle";
import { toSaveSchedule } from "./schedule-utils";

export function ScheduleToggle({
    schedule,
    onSaved,
}: {
    schedule: Reports.Schedule;
    onSaved: () => Promise<unknown>;
}) {
    const tracker = useRef<ScheduleToggleTracker>({
        id: schedule.id,
        nextRequestId: 0,
        pendingRequestId: null,
    });
    const [pendingRequestId, setPendingRequestId] = useState<number | null>(null);
    const [errorMessage, setErrorMessage] = useState<string>();
    const mutation = useMutation({
        mutationFn: async ({ id, enabled, requestId }: ScheduleToggleRequest) => {
            if (id !== schedule.id) {
                throw new Error(t("计划标识已变化", "Schedule identity changed"));
            }
            await reportsAPI.updateSchedule(schedule.id, {
                ...toSaveSchedule(schedule),
                enabled,
            });
            return { id: schedule.id, requestId };
        },
        onSuccess: async (request) => {
            if (!isCurrentScheduleToggle(tracker.current, request)) return;
            await onSaved();
        },
        onError: (error, variables) => {
            if (!isCurrentScheduleToggle(tracker.current, variables)) return;
            setErrorMessage(
                error instanceof Error && error.message
                    ? error.message
                    : t("计划状态更新失败", "Failed to update schedule status"),
            );
        },
        onSettled: (_data, error, variables) => {
            if (variables && isCurrentScheduleToggle(tracker.current, variables)) {
                tracker.current = settleScheduleToggle(
                    tracker.current,
                    variables,
                    error instanceof Error && error.message ? error.message : undefined,
                );
                setPendingRequestId(null);
            }
        },
    });

    const pending = pendingRequestId !== null || mutation.isPending;
    const toggle = (enabled: boolean) => {
        if (pending) return;
        const started = startScheduleToggle(tracker.current, enabled);
        if (!started.request) return;
        tracker.current = started.tracker;
        const { requestId } = started.request;
        setPendingRequestId(requestId);
        setErrorMessage(undefined);
        mutation.mutate(started.request);
    };

    return (
        <span className="inline-flex items-center gap-2">
            <Switch
                size="small"
                checked={schedule.enabled}
                checkedChildren={t("启用", "On")}
                unCheckedChildren={t("停用", "Off")}
                loading={pending}
                disabled={pending}
                onChange={toggle}
            />
            {errorMessage ? (
                <Typography.Text type="danger" className="text-xs" aria-live="polite">
                    {errorMessage}
                </Typography.Text>
            ) : null}
        </span>
    );
}
