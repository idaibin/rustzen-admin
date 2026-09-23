export const publicRouteAllowlist = {
    insights: new Set([
        "GET /api/insights/tracker.js",
        "OPTIONS /api/insights/track",
        "POST /api/insights/track",
    ]),
    monitor: new Set(["POST /api/monitor/agent-reports"]),
    reports: new Set(),
};

export function compareManifestRoutes(module, manifest, clientContract) {
    const runtimeRoutes = new Map(
        (manifest.routes ?? []).map((route) => [
            `${route.method} ${manifest.apiPrefix}${route.path}`,
            route,
        ]),
    );
    const clientRoutes = new Map(
        Object.entries(clientContract).map(([name, route]) => [
            `${route.method} ${route.path}`,
            name,
        ]),
    );
    const allowlistedPublicRoutes = publicRouteAllowlist[module] ?? new Set();
    const protectedRuntimeRoutes = [...runtimeRoutes.entries()].filter(
        ([, route]) => route.access === "protected",
    );
    const missing = [...clientRoutes.entries()]
        .filter(([key]) => !runtimeRoutes.has(key))
        .map(([key, name]) => ({ key, name }));
    const extra = protectedRuntimeRoutes
        .filter(([key]) => !clientRoutes.has(key))
        .map(([key, route]) => ({ key, access: route.access }));
    const unlistedPublicRoutes = [...runtimeRoutes.entries()]
        .filter(
            ([key, route]) =>
                route.access === "public" && !allowlistedPublicRoutes.has(key),
        )
        .map(([key, route]) => ({ key, access: route.access }));
    const clientPublicRoutes = [...clientRoutes.entries()]
        .filter(([key]) => runtimeRoutes.get(key)?.access === "public")
        .map(([key, name]) => ({ key, name }));
    const invalidAllowlist = [...allowlistedPublicRoutes]
        .filter((key) => runtimeRoutes.get(key)?.access !== "public")
        .map((key) => ({ key, access: runtimeRoutes.get(key)?.access }));

    return {
        missing,
        extra: [...extra, ...unlistedPublicRoutes],
        invalidAllowlist,
        clientPublicRoutes,
    };
}
