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

export const exactSseFetchAuthorization = (input: RequestInfo | URL, init: RequestInit | undefined, token: string): boolean => {
    const request = new Request(input, init);
    return new URL(request.url).pathname === "/api/notifications/stream"
        && request.headers.get("authorization") === `Bearer ${token}`;
};

export const sseFetchProbeSource = (token: string) => `(() => {
    const originalFetch = window.fetch;
    Object.defineProperty(window, "__rzP8fbSseBearer", { configurable: true, writable: true, value: [] });
    window.fetch = function(input, init) {
        const request = new Request(input, init);
        if (new URL(request.url, location.href).pathname === "/api/notifications/stream")
            window.__rzP8fbSseBearer.push(request.headers.get("authorization") === ${JSON.stringify(`Bearer ${token}`)});
        return originalFetch.call(this, input, init);
    };
})()`;

export const observeNotificationStreams = (
    requests: CdpRequest[],
    responses: Map<string, CdpStreamResponse>,
    bytes: Map<string, number>,
    bearer: boolean,
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
        bearer,
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
