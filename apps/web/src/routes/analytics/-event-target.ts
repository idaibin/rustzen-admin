export type EventTargetKind = "page" | "api" | "other";

export interface EventTargetProjection {
    kind: EventTargetKind;
    eventName: string;
    pagePath: string | null;
    referrer: string | null;
    apiPath: string | null;
    apiMethod: string | null;
}

export function projectEventTarget(event: Insights.Event): EventTargetProjection {
    return {
        kind: event.eventName === "page_view" ? "page" : event.eventName === "api_request" ? "api" : "other",
        eventName: event.eventName,
        pagePath: event.pagePath,
        referrer: event.referrer,
        apiPath: event.apiPath,
        apiMethod: event.apiMethod,
    };
}
