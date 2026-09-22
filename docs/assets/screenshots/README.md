# Release screenshots

This directory contains the 1920x1080 candidate screenshots for every authenticated
business route. Each image is captured after authentication from the full Linux runtime,
with the route, page title, loading state, visible error state, and viewport checked
before capture.

| Route | Screenshot |
| --- | --- |
| `/` | [`dashboard.png`](./dashboard.png) |
| `/profile` | [`profile.png`](./profile.png) |
| `/monitoring/overview` | [`monitoring-overview.png`](./monitoring-overview.png) |
| `/monitoring/nodes` | [`monitoring-nodes.png`](./monitoring-nodes.png) |
| `/monitoring/incidents` | [`monitoring-incidents.png`](./monitoring-incidents.png) |
| `/monitoring/summaries` | [`monitoring-summaries.png`](./monitoring-summaries.png) |
| `/analytics/overview` | [`analytics-overview.png`](./analytics-overview.png) |
| `/analytics/details` | [`analytics-details.png`](./analytics-details.png) |
| `/reports/templates` | [`reports-templates.png`](./reports-templates.png) |
| `/reports/runs` | [`reports-runs.png`](./reports-runs.png) |
| `/system/user` | [`system-users.png`](./system-users.png) |
| `/system/role` | [`system-roles.png`](./system-roles.png) |
| `/system/menu` | [`system-menus.png`](./system-menus.png) |
| `/system/module` | [`system-modules.png`](./system-modules.png) |
| `/system/status` | [`system-status.png`](./system-status.png) |
| `/system/module-log` | [`system-module-logs.png`](./system-module-logs.png) |
| `/manage/log` | [`management-operation-logs.png`](./management-operation-logs.png) |
| `/manage/task` | [`management-scheduled-tasks.png`](./management-scheduled-tasks.png) |
| `/manage/deploy` | [`management-deployments.png`](./management-deployments.png) |

Authentication and status routes are verified as behavior, not included as business-page
release screenshots. The source, build, package, deployed service, and screenshot evidence
remain separate acceptance layers.
