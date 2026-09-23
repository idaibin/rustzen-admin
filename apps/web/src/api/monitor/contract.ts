import { monitorCoreAPIContract } from "./core-contract";
import { monitorNotificationAPIContract } from "./notification-contract";
export const monitorAPIContract = { ...monitorCoreAPIContract, ...monitorNotificationAPIContract };
