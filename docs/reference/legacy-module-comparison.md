# Former Product Comparison

This document records live former-product evidence for future Rustzen Admin
module work. It is a comparison input, not current runtime authority.

## Evidence basis

Verified on 2026-07-23 from the default `main` branches:

| Product source | Revision | Relevant current role |
| --- | --- | --- |
| `rustzen/rustzen-inspect` | `ad06fecddba9a7dd6f3270d27aa9c1385a17cc0a` | Monitoring and managed-node behavior reference. |
| `rustzen/rustzen-analytics` | `0e3f1e9dd876301aaa9fc2a928c1f6d8903f35bb` | Multi-project Analytics behavior reference. |
| `rustzen/rustzen-report` | `69930c9b1e26e7580503e956566b315872167a33` | Browser filling and report workflow behavior reference. |

The current implementation owner is this `rustzen-admin` repository. Former
repositories are not package dependencies, deployment authorities, or proof
that every former capability still belongs in the product.

## Decision matrix

### Monitoring compared with `rustzen-inspect`

| Capability | Former source | Current source | Decision |
| --- | --- | --- | --- |
| Server/agent split, node heartbeat, metrics | `zen-server/src/features/{node,agent}/`, `zen-agent/src/features/metrics/` | `apps/monitor/src/features/{heartbeat,nodes,metrics}/` | Retain the current owner and protocol; reproduce only missing observable behavior. |
| TCP targets, probe scheduling, incidents | `zen-server/src/features/monitor/`, `zen-server/migrations/014_server_monitoring.sql` | `apps/monitor/src/features/{checks,incidents}/` | Retain and extend current features. |
| Alert policies and agent-side alert evaluation | `zen-server/src/features/alert/`, `zen-agent/src/features/alerts/` | Incident lifecycle only | Defer to a dedicated alert-policy feature specification. |
| Agent aggregation, cleaner, time drift, network filtering | `zen-agent/src/task/`, `zen-server/migrations/010_time_offset_monitor.sql` | Bounded current metric and retention behavior | Selective reproduce after an explicit node-runtime requirement; do not copy the task framework. |
| Server-period reports | `zen-server/src/features/reports/server_period/` | Not found in Monitor | Defer as a Monitoring report-producer slice. |
| Auth, RBAC, system pages, project/CORS, deployment | `zen-core/`, `zen-server/src/features/{system,project,cors,deploy}/` | Admin and shared crates | Drop from module migration; reuse current Admin, `crates/auth`, and release topology. |
| `zen-common` and protected runtime layout | `zen-common/`, `/opt/rustzen-inspect` | Current named shared crates and `/opt/rz` | Do not copy. Map each selected mechanism to its current owner. |

### Analytics compared with `rustzen-analytics`

| Capability | Former source | Current source | Decision |
| --- | --- | --- | --- |
| Public tracker and bounded event batch validation | `features/analytics/{tracker_script,handler,service}.rs` | `apps/insights/src/features/tracking/` | Retain the current simpler contract. Extend only through a feature spec. |
| Project lifecycle and stable project key | `features/analytics/`, frontend `routes/project.tsx` | Fixed internal `default` project | Reproduce selectively for a future multi-project slice; requires migration and permission decisions. |
| Browser origin and application package allowlists | `features/analytics/service.rs`, API-key routes | Not found | Reproduce with project identity; do not add an unrelated global CORS manager. |
| Ingest queue, background aggregation, compaction, behavior query tables | `ingest_queue.rs`, `aggregation_worker.rs`, migrations `0015`-`0022` | Transactional writes plus instance-wide queries | Reproduce incrementally only when volume and query acceptance require it. Queue mechanics are not yet a shared platform. |
| Project overview, pages, APIs, users, timeline, push devices | `features/analytics/mod.rs`, frontend analytics routes | Instance-wide overview and details | Reproduce as independently accepted query slices after project identity exists. |
| Scheduled report records | `features/report/`, project report rules | Not found | Defer; do not conflate Analytics records with a cross-module Report Center. |
| Auth, account, dashboard, system, manage, deploy | `features/{auth,account,dashboard,system,manage}/` | Admin | Drop from migration; reuse current Admin capabilities and Web shell. |

### Reports compared with `rustzen-report`

| Capability | Former source | Current source | Decision |
| --- | --- | --- | --- |
| Target systems, flows/templates, queued runs | `features/{system,flow,job}/` | `apps/reports/src/features/automation/` | Retain the current smaller model. |
| Six basic browser actions, step audit, screenshots, live frame, cancellation | `infra/browser/`, `features/job/`, `features/ws/` | `automation/{browser,scheduler,service}.rs` | Retain current behavior and tests. |
| Accounts and protected credentials | `features/account/`, job preparation | Not found in the current contract | Reproduce only with an encryption, permission, and write-only-input feature specification. |
| Datasets, uploads, Excel row selection | `features/dataset/`, `features/uploads/`, job preparation | Generic run input object only | Defer to one dataset-and-upload slice; use a shared file primitive only after another real consumer exists. |
| Expression, `forEach` groups, guards, parse-error policy | `features/flow/`, `docs/reference/template-rule.md` | `{{input.key}}` string substitution only | Reproduce selected DSL semantics behind a versioned Reports-owned contract; do not build a generic form or workflow DSL. |
| Suspend/resume, detailed job events, cron and periodic generation | `features/job/`, `features/ws/`, `features/cron/`, `features/periodic/` | Cancel plus bounded scheduler execution | Defer as separate lifecycle and scheduling feature specifications. |
| Auth, users, operation shell, standalone deploy layout | `features/{auth,user}/`, `zen-web`, `/opt/rustzen-report` | Admin and `/opt/rz` bundle | Drop from migration. |

## Shared extraction result

Use the current owners rather than copying former shared directories:

- `crates/auth/`: authentication and capability policy;
- `crates/ipc/`: health, Manifest, route registration, and delegation;
- `crates/config/`: focused process configuration;
- `crates/storage/`: SQLite connection and maintenance;
- `apps/web/src/components/`: accepted shared page, table, feedback, form, and
  primitive UI;
- `apps/web/src/lib/format-date-time.ts`: the verified nullable locale
  formatter.

Monitoring alert rules, Analytics aggregation, Reports DSL, browser runtime,
artifacts, scheduling, and report summaries remain module-owned until current
consumers prove an identical cross-module contract.

## Required refresh

Before using this comparison for implementation:

1. fetch the named former repository and record the new default-branch SHA;
2. revalidate the cited definition, route registration, representative
   consumer, and tests;
3. compare against the current `rustzen-admin` owner;
4. update only the affected matrix row;
5. keep runtime behavior `Not verified` until the selected slice is exercised.
