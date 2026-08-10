export type ScheduleToggleRequest = {
    id: string;
    requestId: number;
    enabled: boolean;
};

export type ScheduleToggleTracker = {
    id: string;
    nextRequestId: number;
    pendingRequestId: number | null;
    errorMessage?: string;
};

export function startScheduleToggle(
    tracker: ScheduleToggleTracker,
    enabled: boolean,
): { tracker: ScheduleToggleTracker; request?: ScheduleToggleRequest } {
    if (tracker.pendingRequestId !== null) {
        return { tracker };
    }
    const requestId = tracker.nextRequestId + 1;
    return {
        tracker: {
            ...tracker,
            nextRequestId: requestId,
            pendingRequestId: requestId,
            errorMessage: undefined,
        },
        request: { id: tracker.id, requestId, enabled },
    };
}

export function isCurrentScheduleToggle(
    tracker: ScheduleToggleTracker,
    request: Pick<ScheduleToggleRequest, "id" | "requestId">,
): boolean {
    return tracker.id === request.id && tracker.pendingRequestId === request.requestId;
}

export function settleScheduleToggle(
    tracker: ScheduleToggleTracker,
    request: Pick<ScheduleToggleRequest, "id" | "requestId">,
    errorMessage?: string,
): ScheduleToggleTracker {
    if (!isCurrentScheduleToggle(tracker, request)) {
        return tracker;
    }
    return { ...tracker, pendingRequestId: null, errorMessage };
}
