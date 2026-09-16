import type { NotificationDeliveryStatus } from "@/api/notification-delivery";
import { reportsCoreAPI } from "@/api/reports/core-api";
import { reportsNotificationAPIContract } from "@/api/reports/notification-contract";
import { apiRequest } from "@/api/request";

export { reportsCoreAPI } from "@/api/reports/core-api";

export const reportsAPI = {
    ...reportsCoreAPI,
    notificationDelivery: () =>
        apiRequest<NotificationDeliveryStatus>({
            url: reportsNotificationAPIContract.notificationDelivery.path,
            silent: true,
        }),
};
