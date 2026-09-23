import { createDistributionAuthStore } from "@/distribution/auth-store";

const { getRouteCapabilityCodes, useAuthStore } = createDistributionAuthStore({
    "/": "reports:flow:view",
    "/reports": "reports:flow:view",
    "/reports/templates": "reports:flow:view",
    "/reports/runs": "reports:run:view",
    "/system/user": "system:user:list",
    "/system/role": "system:role:list",
});

export { getRouteCapabilityCodes, useAuthStore };
