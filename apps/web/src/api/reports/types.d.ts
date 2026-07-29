declare namespace Reports {
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
    interface SaveFlow {
        systemId: string;
        name: string;
        steps: FlowStep[];
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
