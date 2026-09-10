const login = [
    { action: "goto", url: "/login" },
    { action: "waitFor", selector: "#login_username" },
    { action: "fill", selector: "#login_username", value: "owner" },
    { action: "fill", selector: "#login_password", value: "rustzen@123" },
    { action: "click", selector: "button[type=submit]" },
    { action: "waitFor", selector: ".shell-content" },
];

const routes = {
    overview: {
        path: "/monitoring/overview",
        loading: "Loading overview",
        permission: "You do not have permission to view monitoring",
        failure: "Failed to load monitoring overview",
        marker: ".grid > .ant-card:nth-child(4) .ant-statistic",
        markerText: "21",
    },
    nodes: {
        path: "/monitoring/nodes",
        loading: "Loading nodes",
        permission: "You do not have permission to view nodes",
        failure: "Failed to load nodes",
        marker: ".ant-table-row",
        markerText: "fixture-node",
    },
    incidents: {
        path: "/monitoring/incidents",
        loading: "Loading alert incidents",
        permission: "You do not have permission to view alert incidents",
        failure: "Failed to load alert incidents",
        marker: ".ant-table-row",
        markerText: "Fixture incident",
    },
    summaries: {
        path: "/monitoring/summaries",
        loading: "Loading daily summaries",
        permission: "You do not have permission to view daily summaries",
        failure: "Failed to load daily summaries",
        marker: ".ant-table-row",
        markerText: "fixture-node",
    },
};

const desktop = (steps) => [
    { action: "setUiPreferences", theme: "dark", locale: "en-US" },
    { action: "setViewport", width: 1440, height: 900 },
    ...login,
    ...steps,
];

const mobile = (steps) => [
    { action: "setUiPreferences", theme: "light", locale: "zh-CN" },
    { action: "setViewport", width: 390, height: 844 },
    ...login,
    ...steps,
];

const errorAlert = "[role=alert]:not([data-testid^=notification-delivery])";

const loading = (route, screenshotName) =>
    desktop([
        { action: "goto", url: route.path },
        { action: "waitFor", selector: ".shell-content" },
        { action: "assertText", selector: ".shell-content", text: route.loading },
        { action: "waitFor", selector: route.marker },
        { action: "assertText", selector: route.marker, text: route.markerText },
        { action: "assertNoHorizontalOverflow" },
        ...(screenshotName
            ? [{ action: "screenshotViewport", name: screenshotName }]
            : []),
    ]);

const initialFailure = (route, status) =>
    desktop([
        { action: "goto", url: route.path },
        { action: "waitFor", selector: errorAlert },
        {
            action: "assertText",
            selector: errorAlert,
            text: status === 403 ? route.permission : route.failure,
        },
        { action: "assertAbsent", selector: route.marker },
        { action: "assertNoHorizontalOverflow" },
    ]);

const backgroundFailure = (route, status) =>
    desktop([
        { action: "goto", url: route.path },
        { action: "waitFor", selector: route.marker },
        { action: "assertText", selector: route.marker, text: route.markerText },
        { action: "pause", durationMs: 15_000 },
        { action: "pause", durationMs: 16_000 },
        { action: "waitFor", selector: errorAlert },
        {
            action: "assertText",
            selector: errorAlert,
            text:
                status === 403
                    ? route.permission
                    : "Background refresh failed. The last successfully loaded data remains visible.",
        },
        ...(status === 403
            ? [{ action: "assertAbsent", selector: route.marker }]
            : [
                  { action: "waitFor", selector: route.marker },
                  {
                      action: "assertText",
                      selector: route.marker,
                      text: route.markerText,
                  },
              ]),
        { action: "assertNoHorizontalOverflow" },
    ]);

const steps = {
    overviewLoading: loading(routes.overview, "monitoring-overview-desktop-dark-en"),
    overview403: initialFailure(routes.overview, 403),
    overview500: initialFailure(routes.overview, 500),
    overviewBackground403: backgroundFailure(routes.overview, 403),
    overviewBackground500: backgroundFailure(routes.overview, 500),
    nodesLoading: loading(routes.nodes),
    nodes403: initialFailure(routes.nodes, 403),
    nodes500: initialFailure(routes.nodes, 500),
    nodesBackground403: backgroundFailure(routes.nodes, 403),
    nodesBackground500: backgroundFailure(routes.nodes, 500),
    incidentsLoading: loading(routes.incidents),
    incidents403: initialFailure(routes.incidents, 403),
    incidents500: initialFailure(routes.incidents, 500),
    incidentsBackground403: backgroundFailure(routes.incidents, 403),
    incidentsBackground500: backgroundFailure(routes.incidents, 500),
    summariesLoading: loading(routes.summaries),
    summaries403: initialFailure(routes.summaries, 403),
    summaries500: initialFailure(routes.summaries, 500),
    summariesBackground403: backgroundFailure(routes.summaries, 403),
    summariesBackground500: backgroundFailure(routes.summaries, 500),
    incidentsPaging: desktop([
        { action: "goto", url: routes.incidents.path },
        { action: "waitFor", selector: ".ant-pagination-item-2" },
        { action: "click", selector: ".ant-pagination-item-2" },
        { action: "pause", durationMs: 350 },
        { action: "assertText", selector: ".ant-table-row", text: "Fixture incident 21" },
        { action: "assertNoHorizontalOverflow" },
    ]),
    incidentsFilters: desktop([
        { action: "goto", url: routes.incidents.path },
        { action: "waitFor", selector: ".ant-pagination-item-2" },
        { action: "click", selector: ".ant-pagination-item-2" },
        { action: "click", selector: "[aria-label='Incident status']" },
        { action: "click", selector: ".ant-select-item-option[title='Active']" },
        { action: "pause", durationMs: 350 },
        { action: "assertText", selector: ".ant-pagination-item-active", text: "1" },
        { action: "click", selector: "[aria-label='Incident kind']" },
        { action: "click", selector: ".ant-select-item-option[title='CPU']" },
        { action: "pause", durationMs: 350 },
        { action: "assertText", selector: ".ant-pagination-item-active", text: "1" },
        { action: "assertNoHorizontalOverflow" },
    ]),
    summariesPaging: mobile([
        { action: "goto", url: routes.summaries.path },
        { action: "waitFor", selector: ".ant-pagination-item-2" },
        { action: "click", selector: ".ant-pagination-item-2" },
        { action: "pause", durationMs: 350 },
        { action: "assertText", selector: ".ant-table-row", text: "fixture-node" },
        { action: "assertText", selector: ".ant-table-row", text: "%" },
        {
            action: "assertElementLayout",
            selector: ".ant-table-thead th:not(.ant-table-cell-scrollbar)",
            visibleCount: 3,
        },
        {
            action: "assertElementLayout",
            selector: ".ant-table-thead .monitoring-summary-detail-column",
            visibleCount: 0,
        },
        {
            action: "assertElementLayout",
            selector: ".ant-table-thead th:first-child",
            visibleCount: 1,
            maxHeight: 64,
            withinViewportRight: true,
        },
        {
            action: "assertElementLayout",
            selector: ".ant-table-tbody > tr.ant-table-row",
            visibleCount: 1,
            maxHeight: 72,
            withinViewportRight: true,
        },
        {
            action: "assertElementLayout",
            selector: ".ant-table-body > table",
            visibleCount: 1,
            withinViewportRight: true,
        },
        {
            action: "assertElementLayout",
            selector: ".data-table-pagination .ant-pagination",
            visibleCount: 1,
            withinViewportRight: true,
        },
        { action: "assertNoHorizontalOverflow" },
        {
            action: "screenshotViewport",
            name: "monitoring-summaries-mobile-light-zh",
        },
    ]),
};

if (Object.keys(steps).length !== 23) {
    const count = Object.keys(steps).length;
    throw new Error(`Monitoring UI state matrix must contain 23 runs, got ${count}`);
}

process.stdout.write(`${JSON.stringify(steps, null, 2)}\n`);
