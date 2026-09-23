import { expect, test } from "bun:test";

import {
    assertNotificationDeliveryClosure,
    assertSelectedApiAuthority,
} from "./distribution-notification-delivery-closure";

const selected = {
    hasNotifications: true,
    retainedText:
        'import { monitorAPI } from "@/api/monitor/api"; // notification marker',
    generatedText: "<NotificationDeliveryCard />",
    outputText: "/api/monitor/notification-delivery notification-delivery-card",
};
const pure = {
    hasNotifications: false,
    retainedText: 'import { monitorCoreAPI } from "@/api/monitor/core-api";',
    generatedText: "<PageCard />",
    outputText: "monitor overview",
};

test("notification delivery closure requires the three selected layers", () => {
    expect(() => assertNotificationDeliveryClosure(selected)).not.toThrow();
    expect(() =>
        assertNotificationDeliveryClosure({
            ...selected,
            generatedText: "<PageCard />",
        }),
    ).toThrow();
    expect(() =>
        assertNotificationDeliveryClosure({
            ...selected,
            outputText: "notification-delivery-card",
        }),
    ).toThrow();
});

test("pure Monitor rejects retained notification markers and emitted delivery seams", () => {
    expect(() => assertNotificationDeliveryClosure(pure)).not.toThrow();
    expect(() =>
        assertNotificationDeliveryClosure({
            ...pure,
            retainedText: `${pure.retainedText} notification marker`,
        }),
    ).toThrow();
    expect(() =>
        assertNotificationDeliveryClosure({
            ...pure,
            outputText: "/api/monitor/notification-delivery",
        }),
    ).toThrow();
});

test("authoritative API template must exactly match generated and retained bytes", () => {
    const bytes = new TextEncoder().encode("export const api = 'monitor';\n");
    expect(() =>
        assertSelectedApiAuthority({
            authoritative: bytes,
            generated: bytes,
            retained: bytes,
        }),
    ).not.toThrow();
    expect(() =>
        assertSelectedApiAuthority({
            authoritative: bytes,
            generated: new TextEncoder().encode(
                "export const api = 'changed';\n",
            ),
            retained: bytes,
        }),
    ).toThrow("authoritative template");
});
