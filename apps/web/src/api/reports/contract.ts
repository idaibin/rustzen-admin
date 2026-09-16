import { reportsCoreAPIContract } from "./core-contract";
import { reportsNotificationAPIContract } from "./notification-contract";

export const reportsAPIContract = { ...reportsCoreAPIContract, ...reportsNotificationAPIContract };
