declare namespace Reports {
    interface InstallationSettings {
        timezone: string;
    }

    interface System {
        id: string;
        name: string;
        baseUrl: string;
        enabled: boolean;
        notes: string;
        createdAt: string;
        updatedAt: string;
    }
    interface SaveSystem {
        name: string;
        baseUrl: string;
        enabled?: boolean;
        notes?: string;
    }
    type FlowStep =
        | { action: "goto"; url: string }
        | { action: "fill"; selector: string; value: string }
        | { action: "click"; selector: string }
        | { action: "waitFor"; selector: string }
        | { action: "assertText"; selector: string; text: string }
        | { action: "screenshot"; name?: string };
    interface Flow {
        id: string;
        systemId: string;
        name: string;
        steps: FlowStep[];
        createdAt: string;
        updatedAt: string;
    }
    interface FlowOption {
        id: string;
        name: string;
        enabled: boolean;
    }
    interface SaveFlow {
        systemId: string;
        name: string;
        steps: FlowStep[];
    }
    type ScheduleCadence = "daily" | "weekly";
    type ScheduleDecision = "enqueued" | "skipped";
    interface SaveSchedule {
        flowId: string;
        cadence: ScheduleCadence;
        weekday?: number;
        dueTime: string;
        input: Record<string, unknown>;
        description?: string;
        enabled?: boolean;
    }
    interface ScheduleOccurrence {
        id: number;
        scheduleId: string;
        occurrenceKey: string;
        dueLocal: string;
        dueAt: string | null;
        decidedAt: string;
        decision: ScheduleDecision;
        reason: string | null;
        runId: string | null;
    }
    interface Schedule {
        id: string;
        flowId: string;
        cadence: ScheduleCadence;
        weekday: number | null;
        dueTime: string;
        input: Record<string, unknown>;
        description: string;
        enabled: boolean;
        timezone: string;
        nextDue: string | null;
        lastOccurrence: ScheduleOccurrence | null;
        lastRun: Run | null;
        createdAt: string;
        updatedAt: string;
    }
    interface Run {
        id: string;
        flowId: string;
        status: "queued" | "running" | "cancelling" | "succeeded" | "failed" | "cancelled";
        error: string | null;
        createdAt: string;
        startedAt: string | null;
        finishedAt: string | null;
    }
    interface CreateRun {
        flowId: string;
        input: Record<string, unknown>;
    }
    interface RunStep {
        id: number;
        runId: string;
        stepIndex: number;
        action: string;
        status: string;
        durationMs: number | null;
        message: string | null;
        createdAt: string;
    }
    interface Artifact {
        id: string;
        runId: string;
        kind: string;
        fileName: string;
        createdAt: string;
    }
    interface Page<T> {
        data: T[];
        total: number;
        success: boolean;
    }
}
