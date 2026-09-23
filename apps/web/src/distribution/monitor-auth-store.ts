import { createDistributionAuthStore } from "@/distribution/auth-store";

const { getRouteCapabilityCodes, useAuthStore } = createDistributionAuthStore({
    "/": "monitor:overview:view",
    "/monitoring": "monitor:overview:view",
    "/monitoring/overview": "monitor:overview:view",
    "/monitoring/nodes": "monitor:node:view",
    "/monitoring/incidents": "monitor:incident:view",
    "/monitoring/summaries": "monitor:node:view",
    "/system/user": "system:user:list",
    "/system/role": "system:role:list",
});

export { getRouteCapabilityCodes, useAuthStore };
