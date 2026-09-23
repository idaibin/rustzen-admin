import { SseParser, type SseEvent } from "./sse-parser";

export type StreamState = "connecting" | "connected" | "reconnecting" | "forbidden" | "error";

export interface RealtimeCallbacks {
    currentGeneration: () => number;
    onInvalidate: (revision?: number) => void;
    onState: (state: StreamState) => void;
    onUnauthorized: () => void;
}

type Fetcher = (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>;

export const retryDelay = (attempt: number, random: () => number): number => {
    const base = Math.min(30_000, 1_000 * 2 ** Math.min(attempt, 5));
    return Math.max(1_000, Math.min(30_000, Math.round(base * (0.8 + random() * 0.4))));
};

export class NotificationRealtimeClient {
    private stopped = false;
    private request?: AbortController;
    private timer?: ReturnType<typeof setTimeout>;
    private finishWait?: () => void;
    private lastEventId?: string;

    constructor(
        private readonly token: string,
        private readonly generation: number,
        private readonly callbacks: RealtimeCallbacks,
        private readonly fetcher: Fetcher = (input, init) => globalThis.fetch(input, init),
        private readonly random: () => number = Math.random,
    ) {}

    start(): void {
        this.stopped = false;
        void this.run();
    }

    stop(): void {
        this.stopped = true;
        this.request?.abort();
        this.finishWait?.();
    }

    private async run(): Promise<void> {
        let attempt = 0;
        while (!this.stopped && this.isCurrent()) {
            this.callbacks.onState(attempt ? "reconnecting" : "connecting");
            const outcome = await this.connect();
            if (outcome === "stop" || this.stopped || !this.isCurrent()) return;
            this.callbacks.onState("reconnecting");
            const delay = outcome.retryAfter ?? retryDelay(attempt++, this.random);
            await this.wait(delay);
        }
    }

    private async connect(): Promise<"stop" | { retryAfter?: number }> {
        const controller = new AbortController();
        this.request = controller;
        let watchdog = setTimeout(() => controller.abort("heartbeat-timeout"), 45_000);
        const resetWatchdog = () => {
            clearTimeout(watchdog);
            watchdog = setTimeout(() => controller.abort("heartbeat-timeout"), 45_000);
        };
        try {
            const headers: Record<string, string> = {
                Accept: "text/event-stream",
                Authorization: `Bearer ${this.token}`,
                "Cache-Control": "no-cache",
            };
            if (this.lastEventId) headers["Last-Event-ID"] = this.lastEventId;
            const response = await this.fetcher("/api/notifications/stream", {
                method: "GET",
                headers,
                credentials: "same-origin",
                cache: "no-store",
                redirect: "manual",
                signal: controller.signal,
            });
            if (this.stopped || !this.isCurrent()) return "stop";
            if (response.status === 401) {
                this.callbacks.onUnauthorized();
                return "stop";
            }
            if (response.status === 403) {
                this.callbacks.onState("forbidden");
                return "stop";
            }
            if (response.status === 204) {
                this.callbacks.onInvalidate();
                return "stop";
            }
            if (response.status === 429 || response.status === 503) {
                this.callbacks.onInvalidate();
                const retry = Number(response.headers.get("retry-after"));
                return {
                    retryAfter:
                        (Number.isFinite(retry) ? Math.max(60, retry) : 60) * 1_000 +
                        retryDelay(0, this.random),
                };
            }
            if (
                !response.ok ||
                !response.headers.get("content-type")?.toLowerCase().startsWith("text/event-stream")
            ) {
                this.callbacks.onState("error");
                return "stop";
            }
            if (!response.body) {
                this.callbacks.onState("error");
                return "stop";
            }
            this.callbacks.onState("connected");
            const parser = new SseParser((event) => this.handleEvent(event, controller));
            const reader = response.body.getReader();
            for (;;) {
                const chunk = await reader.read();
                if (chunk.done) break;
                if (!this.isCurrent()) {
                    controller.abort();
                    return "stop";
                }
                resetWatchdog();
                parser.push(chunk.value);
            }
            parser.finish();
            return {};
        } catch (error) {
            if (this.stopped || !this.isCurrent()) return "stop";
            if (error instanceof Error && /SSE (line|event|ended)/.test(error.message)) {
                this.callbacks.onState("error");
                return "stop";
            }
            return {};
        } finally {
            clearTimeout(watchdog);
            if (this.request === controller) this.request = undefined;
        }
    }

    private handleEvent(event: SseEvent, controller: AbortController): void {
        if (!this.isCurrent()) {
            controller.abort();
            return;
        }
        if (event.id) this.lastEventId = event.id;
        if (event.event === "inbox.changed") {
            const payload = this.parsePayload(event.data) as {
                schemaVersion?: number;
                revision?: number;
            };
            if (payload.schemaVersion !== 1 || !Number.isSafeInteger(payload.revision))
                throw new Error("SSE event is invalid");
            this.callbacks.onInvalidate(payload.revision);
        }
        if (event.event === "reconcile.required") {
            const payload = this.parsePayload(event.data) as {
                schemaVersion?: number;
                reason?: string;
            };
            if (
                payload.schemaVersion !== 1 ||
                !["connected", "lagged"].includes(payload.reason ?? "")
            )
                throw new Error("SSE event is invalid");
            this.callbacks.onInvalidate();
            if (payload.reason === "lagged") controller.abort("reconcile-required");
        }
    }

    private parsePayload(data: string): object {
        try {
            const value = JSON.parse(data) as unknown;
            if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error();
            return value;
        } catch {
            throw new Error("SSE event is invalid");
        }
    }

    private wait(delay: number): Promise<void> {
        return new Promise((resolve) => {
            const finish = () => {
                if (this.timer) clearTimeout(this.timer);
                this.timer = undefined;
                this.finishWait = undefined;
                resolve();
            };
            this.finishWait = finish;
            this.timer = setTimeout(finish, delay);
        });
    }

    private isCurrent(): boolean {
        return this.callbacks.currentGeneration() === this.generation;
    }
}
