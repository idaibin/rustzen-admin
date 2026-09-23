declare namespace Insights {
    interface TimeQuery {
        from?: string;
        to?: string;
    }

    interface OverviewQuery extends TimeQuery {}

    interface TrendPoint {
        date: string;
        pv: number;
        uv: number;
        requestCount: number;
        errorCount: number;
    }

    interface Overview {
        pv: number;
        uv: number;
        eventCount: number;
        requestCount: number;
        errorCount: number;
        averageDurationMs: number;
        p95DurationMs: number;
        trend: TrendPoint[];
    }

    interface Page<T> {
        data: T[];
        total: number;
        success: boolean;
    }

    interface ListQuery extends TimeQuery {
        current?: number;
        pageSize?: number;
    }

    interface EventQuery extends ListQuery {
        eventName?: string;
        eventKind?: "page" | "api" | "other";
        path?: string;
        visitorId?: string;
        platform?: string;
    }
    interface Event {
        id: number;
        eventName: string;
        visitorId: string;
        userId: string | null;
        sessionId: string | null;
        platform: string | null;
        pagePath: string | null;
        referrer: string | null;
        apiPath: string | null;
        apiMethod: string | null;
        statusCode: number | null;
        durationMs: number | null;
        isError: boolean;
        properties: Record<string, unknown>;
        occurredAt: string;
        receivedAt: string;
    }
}
