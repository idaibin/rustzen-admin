import type { NotificationDeliveryStatus } from "@/api/notification-delivery";
import { formatDateTime } from "@/lib/format-date-time";

export type { NotificationDeliveryStatus } from "@/api/notification-delivery";
export type NotificationDeliveryViewState =
    | { kind: "loading" }
    | { kind: "permission" }
    | { kind: "error" }
    | {
          kind: "healthy" | "gap";
          status: NotificationDeliveryStatus;
          gaps: number;
          firstGap: string;
          lastGap: string;
          lastSuccess: string;
      };
export function notificationDeliveryView(
    status: NotificationDeliveryStatus,
): Extract<NotificationDeliveryViewState, { kind: "healthy" | "gap" }> {
    const gaps =
        status.omittedCount +
        status.expiredCount +
        status.unconfirmedCount +
        status.quarantinedCount +
        status.quarantineEvictedCount;
    return {
        kind: gaps ? "gap" : "healthy",
        status,
        gaps,
        firstGap: formatDateTime(status.firstGapAt),
        lastGap: formatDateTime(status.lastGapAt),
        lastSuccess: formatDateTime(status.lastSuccessAt),
    };
}
export function notificationDeliveryState({
    data,
    error,
    isPending,
}: {
    data?: NotificationDeliveryStatus;
    error?: unknown;
    isPending: boolean;
}): NotificationDeliveryViewState {
    if (isPending) return { kind: "loading" };
    if (error)
        return { kind: (error as { status?: number }).status === 403 ? "permission" : "error" };
    return data ? notificationDeliveryView(data) : { kind: "error" };
}
export async function retryDelivery(refetch: () => Promise<unknown> | unknown) {
    await refetch();
}

export function formatDeliveryBytes(value: number) {
    return `${value} B`;
}
