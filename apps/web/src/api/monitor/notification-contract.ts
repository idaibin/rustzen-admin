import type { ModuleApiRoute } from "@/api/module-contract";
export const monitorNotificationAPIContract = { notificationDelivery: { method: "GET", path: "/api/monitor/notification-delivery" } } as const satisfies Record<string, ModuleApiRoute>;
