import { createDistributionAuthStore } from "@/distribution/auth-store";

const { getRouteCapabilityCodes, useAuthStore } = createDistributionAuthStore({
    "/": "insights:overview:view",
    "/analytics": "insights:overview:view",
    "/analytics/overview": "insights:overview:view",
    "/analytics/details": "insights:event:view",
    "/system/user": "system:user:list",
    "/system/role": "system:role:list",
});

export { getRouteCapabilityCodes, useAuthStore };
