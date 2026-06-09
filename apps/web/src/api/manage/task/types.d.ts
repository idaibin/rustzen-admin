declare namespace Task {
    type TriggerType = "scheduled" | "manual";
    type RunStatus = "running" | "success" | "failed" | "skipped";

    type Schedule = {
        type: "cron";
        expression: string;
    };

    interface Item {
        taskKey: string;
        name: string;
        description?: string | null;
        enabled: boolean;
        schedule: Schedule;
        running: boolean;
        lastRunId?: number | null;
        lastTriggerType?: TriggerType | null;
        lastStatus?: RunStatus | null;
        lastStartedAt?: string | null;
        lastFinishedAt?: string | null;
        lastErrorMessage?: string | null;
        nextRunAt?: string | null;
        createdAt: string;
        updatedAt: string;
    }

    interface RunItem {
        id: number;
        taskKey: string;
        triggerType: TriggerType;
        status: RunStatus;
        scheduledFor?: string | null;
        startedAt: string;
        finishedAt?: string | null;
        errorMessage?: string | null;
        createdAt: string;
        updatedAt: string;
    }

    interface RunQuery {
        current?: number;
        pageSize?: number;
    }
}
