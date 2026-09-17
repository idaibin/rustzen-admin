# UI Route Audit Matrix

> Current route inventory. Shared visual semantics remain in
> [`DESIGN.md`](../../DESIGN.md); [local verification](../guides/local-verification.md)
> defines observed coverage. This matrix is not a claim that all states were tested.

This matrix covers the current 21 leaf frontend routes plus the Nodes drawers. The application `layout/`
owns the global width boundary, `PageHeader` or `PageCard` owns page hierarchy,
and `DataState` owns global list feedback while each route owns its `ProTable`
for table surface, sorting, filtering, and paging. Every route supports the
standard light and dark themes.

| # | Route | Surface | Current audit result |
| ---: | --- | --- | --- |
| 1 | `/login` | Sign-in | Uses one compact sign-in form with localized copy and no horizontal overflow. |
| 2 | `/` | Dashboard | One heading; four tone count cards, permission-gated CPU/memory/disk summary and textual module health; storage details stay on System Status. |
| 3 | `/profile` | Detail | Uses one page heading with consistent account cards and localized dialogs. |
| 4 | `/403` | Permission status | Reuses the permission-state component and preserves the return action. |
| 5 | `/404` | Error status | Reuses the error-state component and preserves the return action. |
| 6 | `/monitoring/overview` | Overview | Reuses `MetricCard` with explicit empty, loading, and error states. |
| 7 | `/monitoring/nodes` | List and detail | Uses the shared page card, `DataState`, actions, and node-detail states. |
| 8 | `/monitoring/incidents` | Filtered list and detail | Uses `ProTable`, semantic status tags, background-refresh feedback, and an Ant Design detail Drawer. |
| 9 | `/monitoring/nodes` drawers | Onboarding and global configuration | Agent setup and connection check; one permission-aware four-control global form and Save. |
| 10 | `/monitoring/summaries` | List | No search input; `DataState` and retained report pagination. |
| 11 | `/analytics/overview` | Overview | Reuses `MetricCard` with consistent metric density and chart surfaces. |
| 12 | `/analytics/details` | Filtered list | Upper-right automatic type/path filters; other clears path; filling table and bottom pagination. |
| 13 | `/reports/templates` | List and form | Uses consistent template, target-system, action, and `DataState` feedback. |
| 14 | `/reports/runs` | List and workflow | Uses consistent fill actions, workflow states, run details, and `DataState` feedback. |
| 15 | `/system/user` | Filtered list | Upper-right username/status only; automatic search preserves focus; localized account actions. |
| 16 | `/system/role` | Filtered list | Upper-right automatic name/code/status filters; permission summaries and dialogs preserved. |
| 17 | `/system/menu` | Flat inventory | Upper-right automatic name/code/status filters, readable permission codes, flat rows and table-body scrolling. |
| 18 | `/system/module` | Status list | Uses consistent module health, start/stop confirmation, and `DataState` feedback. |
| 19 | `/system/status` | Resource overview | Storage and local-resource telemetry only. |
| 22 | `/system/module-log` | Diagnostics | Owner-only module log list, tail, backup, and cleanup. |
| 20 | `/manage/log` | Filtered list | Uses consistent filters, status labels, common log descriptions, and `DataState` feedback. |
| 21 | `/manage/task` | List and detail | Task/run-log states and confirmation retained; compact 72px operation column. |
| 22 | `/manage/deploy` | List and workflow | Uses consistent upload, deployment, expiry, cleanup, and `DataState` feedback. |

## Acceptance

- No page-level horizontal overflow at 1920x1080.
- No page-level horizontal overflow on representative routes at 1440x900.
- Core shell, form, dialog, and table surfaces retain clear hierarchy and
  readable solid surfaces in standard light and dark themes.
- Query-backed lists expose distinct initial loading, empty, error, and populated
  states, with retry available after errors.
- User-visible copy defaults to Simplified Chinese and retains only product names,
  protocols, methods, formats, and technical abbreviations untranslated.
