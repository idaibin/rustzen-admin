# UI Route Audit Matrix

This matrix covers the current 20 frontend routes. The application `layout/`
owns the global width boundary, `PageHeader` or `PageCard` owns page hierarchy,
and `DataState` owns global list feedback while each route owns its `ProTable`
for table surface, sorting, filtering, and paging. Every route supports the
standard light and dark themes.

| # | Route | Surface | Current audit result |
| ---: | --- | --- | --- |
| 1 | `/login` | Sign-in | Uses one compact sign-in form with localized copy and no horizontal overflow. |
| 2 | `/` | Dashboard | Uses one heading owner with account totals and module-health cards; detailed resources and trends stay on their owning routes. |
| 3 | `/profile` | Detail | Uses one page heading with consistent account cards and localized dialogs. |
| 4 | `/403` | Permission status | Reuses the permission-state component and preserves the return action. |
| 5 | `/404` | Error status | Reuses the error-state component and preserves the return action. |
| 6 | `/monitoring/overview` | Overview | Reuses `MetricCard` with explicit empty, loading, and error states. |
| 7 | `/monitoring/nodes` | List and detail | Uses the shared page card, `DataState`, actions, and node-detail states. |
| 8 | `/monitoring/checks` | List and form | Uses `DataState`, the TCP-check form, and test feedback. |
| 9 | `/analytics/overview` | Overview | Reuses `MetricCard` with consistent metric density and chart surfaces. |
| 10 | `/analytics/details` | Filtered list | Uses the shared filter toolbar, accessible names, `DataState`, and route-local ProTable paging. |
| 11 | `/reports/templates` | List and form | Uses consistent template, target-system, action, and `DataState` feedback. |
| 12 | `/reports/runs` | List and workflow | Uses consistent fill actions, workflow states, run details, and `DataState` feedback. |
| 13 | `/system/user` | Filtered list | Uses consistent filters, account status, `DataState` feedback, and localized actions. |
| 14 | `/system/role` | Filtered list | Uses consistent filters, permission summaries, `DataState` feedback, and dialog states. |
| 15 | `/system/menu` | Tree list | Uses consistent tree hierarchy, `DataState` feedback, and localized actions. |
| 16 | `/system/module` | Status list | Uses consistent module health, start/stop confirmation, and `DataState` feedback. |
| 17 | `/system/status` | Resource overview | Uses one page heading with consistent storage and resource cards. |
| 18 | `/manage/log` | Filtered list | Uses consistent filters, status labels, common log descriptions, and `DataState` feedback. |
| 19 | `/manage/task` | List and detail | Uses consistent task and run-log states, route-local ProTable paging, and confirmation. |
| 20 | `/manage/deploy` | List and workflow | Uses consistent upload, deployment, expiry, cleanup, and `DataState` feedback. |

## Acceptance

- No page-level horizontal overflow at 1920x1080.
- No page-level horizontal overflow on representative routes at 1440x900.
- Core shell, form, dialog, and table surfaces retain clear hierarchy and
  readable solid surfaces in standard light and dark themes.
- Query-backed lists expose distinct initial loading, empty, error, and populated
  states, with retry available after errors.
- User-visible copy defaults to Simplified Chinese and retains only product names,
  protocols, methods, formats, and technical abbreviations untranslated.
