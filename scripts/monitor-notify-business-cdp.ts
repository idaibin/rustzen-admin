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

const persistedAuthToken = (serialized: string | null): string | undefined => {
    try {
        const value = JSON.parse(serialized ?? "") as { state?: { token?: unknown } };
        return typeof value.state?.token === "string" ? value.state.token : undefined;
    } catch {
        return undefined;
    }
};

export const exactPersistedSseAuthorization = (input: RequestInfo | URL, init: RequestInit | undefined, serializedAuth: string | null): boolean => {
    const request = new Request(input, init);
    const token = persistedAuthToken(serializedAuth);
    return new URL(request.url).pathname === "/api/notifications/stream"
        && typeof token === "string"
        && request.headers.get("authorization") === `Bearer ${token}`;
};

export const sseFetchProbeSource = () => `(() => {
    const originalFetch = window.fetch;
    Object.defineProperty(window, "__rzP8fbSseBearer", { configurable: true, writable: true, value: [] });
    Object.defineProperty(window, "__rzP8fbDocumentGeneration", { configurable: true, value: String(Date.now()) + ":" + String(Math.random()) });
    window.fetch = function(input, init) {
        const request = new Request(input, init);
        if (new URL(request.url, location.href).pathname === "/api/notifications/stream") {
            let token;
            try { token = JSON.parse(localStorage.getItem("auth-store") || "").state?.token; } catch {}
            window.__rzP8fbSseBearer.push(typeof token === "string" && request.headers.get("authorization") === "Bearer " + token);
        }
        return originalFetch.call(this, input, init);
    };
})()`;

export const cdpMilliseconds = (seconds: unknown): number => {
    const value = typeof seconds === "number" ? seconds : Number.NaN;
    const milliseconds = Math.round(value * 1_000);
    if (!Number.isFinite(value) || !Number.isSafeInteger(milliseconds)) throw Error("CDP timestamp differs");
    return milliseconds;
};

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
