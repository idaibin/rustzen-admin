/** Shared response shape for the Monitor and Reports delivery-health endpoints. */
export interface NotificationDeliveryStatus {
    pendingCount: number;
    pendingBytes: number;
    quarantineCount: number;
    quarantineBytes: number;
    omittedCount: number;
    expiredCount: number;
    unconfirmedCount: number;
    quarantinedCount: number;
    quarantineEvictedCount: number;
    firstGapAt: string | null;
    lastGapAt: string | null;
    lastSuccessAt: string | null;
}
