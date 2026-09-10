export type CdpRequest = { requestId: string; method: string; url: string };
export type CdpStreamResponse = { status: number; contentType?: unknown };

export type StreamObservation = {
    requestId: string;
    method: string;
    query: boolean;
    status: number | null;
    eventStream: boolean;
    bytes: number;
    bearer: boolean | null;
};

const streamUrl = (url: string): URL | undefined => {
    try {
        const parsed = new URL(url);
        return parsed.pathname === "/api/notifications/stream" ? parsed : undefined;
    } catch {
        return undefined;
    }
};

export const hasExactBearerAuthorization = (headers: unknown, token: string): boolean => {
    if (!headers || typeof headers !== "object" || Array.isArray(headers)) return false;
    const value = (headers as Record<string, unknown>).Authorization
        ?? (headers as Record<string, unknown>).authorization;
    return typeof value === "string" && value === `Bearer ${token}`;
};

export const observeNotificationStreams = (
    requests: CdpRequest[],
    responses: Map<string, CdpStreamResponse>,
    bytes: Map<string, number>,
    bearer: Map<string, boolean>,
): StreamObservation[] => requests.flatMap((request) => {
    const parsed = streamUrl(request.url);
    if (!parsed) return [];
    const response = responses.get(request.requestId);
    return [{
        requestId: request.requestId,
        method: request.method,
        query: parsed.search.length > 0,
        status: response?.status ?? null,
        eventStream: typeof response?.contentType === "string" && response.contentType.toLowerCase().includes("text/event-stream"),
        bytes: bytes.get(request.requestId) ?? 0,
        bearer: bearer.has(request.requestId) ? bearer.get(request.requestId)! : null,
    }];
});

export const isReadyNotificationStream = (value: StreamObservation | undefined): value is StreamObservation => Boolean(
    value
    && value.method === "GET"
    && !value.query
    && value.status === 200
    && value.eventStream
    && value.bytes > 0
    && value.bearer === true,
);

export const streamDiagnostics = (observations: StreamObservation[]) => observations.map(({ requestId, ...value }) => value);
