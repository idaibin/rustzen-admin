import { expect, test } from "bun:test";

import { renderToStaticMarkup } from "react-dom/server";

import { formatDateTime } from "@/lib/format-date-time";
import { setLocale } from "@/lib/i18n";

import {
    NotificationDeliveryCardView,
    NotificationDeliveryDetails,
} from "./notification-delivery-card";
import { notificationDeliveryState } from "./notification-delivery-model";

const status = {
    pendingCount: 2,
    pendingBytes: 64,
    quarantineCount: 3,
    quarantineBytes: 96,
    omittedCount: 0,
    expiredCount: 0,
    unconfirmedCount: 0,
    quarantinedCount: 0,
    quarantineEvictedCount: 0,
    firstGapAt: null,
    lastGapAt: null,
    lastSuccessAt: "2026-09-10T03:04:05.000Z",
};

type DeliveryState = ReturnType<typeof notificationDeliveryState>;

const renderTrigger = (state: DeliveryState) =>
    renderToStaticMarkup(<NotificationDeliveryCardView state={state} onRetry={() => undefined} />);

const renderDetails = (state: DeliveryState) =>
    renderToStaticMarkup(<NotificationDeliveryDetails state={state} onRetry={() => undefined} />);

test("notification delivery view has stable localized state seams", () => {
    setLocale("zh-CN");
    expect(renderTrigger(notificationDeliveryState({ isPending: true }))).toContain(
        "notification-delivery-loading",
    );
    const healthyState = notificationDeliveryState({ data: status, isPending: false });
    const healthy = renderTrigger(healthyState);
    expect(healthy).toContain("notification-delivery-healthy");
    expect(healthy).toContain("通知投递健康");
    const details = renderDetails(healthyState);
    expect(details).toContain("待投递 2（64 B）");
    expect(details).toContain("隔离 3（96 B）");
    expect(details).toContain("首个缺口 -");
    expect(details).toContain(`最后成功 ${formatDateTime(status.lastSuccessAt)}`);
    const gapState = notificationDeliveryState({
        data: {
            ...status,
            omittedCount: 1,
            expiredCount: 1,
            unconfirmedCount: 1,
            quarantinedCount: 1,
            quarantineEvictedCount: 1,
        },
        isPending: false,
    });
    const gap = renderTrigger(gapState);
    expect(gap).toContain("notification-delivery-gap");
    expect(gap).toContain("5 个不可恢复缺口");
    const permissionState = notificationDeliveryState({ error: { status: 403 }, isPending: false });
    expect(renderTrigger(permissionState)).toContain("notification-delivery-permission");
    expect(renderDetails(permissionState)).toMatch(/重\s*试/);
    const errorState = notificationDeliveryState({ error: new Error(), isPending: false });
    expect(renderTrigger(errorState)).toContain("notification-delivery-error");
    expect(renderDetails(errorState)).toMatch(/重\s*试/);
    setLocale("en-US");
    const englishDetails = renderDetails(
        notificationDeliveryState({ data: status, isPending: false }),
    );
    expect(englishDetails).toContain("Pending 2 (64 B)");
    expect(englishDetails).toContain("Quarantine 3 (96 B)");
    setLocale("zh-CN");
});
