declare namespace Notifications {
    interface Item {
        id: string;
        producer: string;
        topic: string;
        subjectKind: string;
        subjectId: string;
        subjectRevision: number;
        occurredAt: string;
        acceptedAt: string;
        title: string;
        summary: string;
        readAt?: string | null;
    }

    interface ListQuery {
        cursor?: string;
        limit?: number;
        unreadOnly?: boolean;
    }

    interface ListResponse {
        items: Item[];
        nextCursor?: string | null;
        snapshot: string;
        revision: number;
        retentionDays: number;
    }

    interface UnreadCount {
        count: number;
        revision: number;
    }
    interface ReadResponse {
        id: string;
        readAt: string;
        revision: number;
    }
    interface ReadAllResponse {
        changed: number;
        revision: number;
    }
}
