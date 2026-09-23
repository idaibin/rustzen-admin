import type { NotificationDeliveryStatus } from "@/api/notification-delivery";
import { apiRequest } from "@/api/request";

import { monitorCoreAPI } from "./core-api";
import { monitorNotificationAPIContract } from "./notification-contract";
export { monitorCoreAPI } from "./core-api";
export const monitorAPI = {
    ...monitorCoreAPI,
    notificationDelivery: () =>
        apiRequest<NotificationDeliveryStatus>({
            url: monitorNotificationAPIContract.notificationDelivery.path,
            silent: true,
        }),
};
