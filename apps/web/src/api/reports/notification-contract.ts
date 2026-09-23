import type { ModuleApiRoute } from "@/api/module-contract";

export const reportsNotificationAPIContract = {
    notificationDelivery: { method: "GET", path: "/api/reports/notification-delivery" },
} as const satisfies Record<string, ModuleApiRoute>;
