import { expect, test } from "bun:test";

import { renderToStaticMarkup } from "react-dom/server";

import { formatDateTime } from "@/lib/format-date-time";
import { setLocale } from "@/lib/i18n";

import { NotificationDeliveryCardView } from "./notification-delivery-card";
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

const render = (state: Parameters<typeof NotificationDeliveryCardView>[0]["state"]) =>
    renderToStaticMarkup(<NotificationDeliveryCardView state={state} onRetry={() => undefined} />);

test("notification delivery view has stable localized state seams", () => {
    setLocale("zh-CN");
    expect(render(notificationDeliveryState({ isPending: true }))).toContain(
        "notification-delivery-loading",
    );
    const healthy = render(notificationDeliveryState({ data: status, isPending: false }));
    expect(healthy).toContain("notification-delivery-healthy");
    expect(healthy).toContain("待投递 2（64 B）");
    expect(healthy).toContain("隔离 3（96 B）");
    expect(healthy).toContain("首个缺口 -");
    expect(healthy).toContain(`最后成功 ${formatDateTime(status.lastSuccessAt)}`);
    const gap = render(
        notificationDeliveryState({
            data: {
                ...status,
                omittedCount: 1,
                expiredCount: 1,
                unconfirmedCount: 1,
                quarantinedCount: 1,
                quarantineEvictedCount: 1,
            },
            isPending: false,
        }),
    );
    expect(gap).toContain("notification-delivery-gap");
    expect(gap).toContain("5 个不可恢复缺口");
    const permission = render(
        notificationDeliveryState({ error: { status: 403 }, isPending: false }),
    );
    expect(permission).toContain("notification-delivery-permission");
    expect(permission).toMatch(/重\s*试/);
    const error = render(notificationDeliveryState({ error: new Error(), isPending: false }));
    expect(error).toContain("notification-delivery-error");
    expect(error).toMatch(/重\s*试/);
    setLocale("en-US");
    const english = render(notificationDeliveryState({ data: status, isPending: false }));
    expect(english).toContain("Pending 2 (64 B)");
    expect(english).toContain("Quarantine 3 (96 B)");
    setLocale("zh-CN");
});
