# Product Foundation

Status: current foundation specification.

This document is the single authority for current product positioning,
direction, module purposes, boundaries, and non-goals. Source remains
authoritative for delivered behavior; `docs/architecture.md` remains
authoritative for runtime and repository structure.

## Product boundary

Future distribution direction (authorized 2026-09-03): the complete template
must also support physically pruned product distributions, including server
monitoring alone, with an optional durable message center and realtime delivery.
The [composable-distribution design](features/composable-distribution/spec.md)
defines the candidate boundaries and acceptance. The four-service descriptions
below remain current implementation facts until that design is implemented;
the new direction does not imply an OS, plugin platform or additional product
families. Its ten external review rounds are complete; the linked record separates
the final local corrections from unexecuted implementation/runtime acceptance.

Rustzen Admin is primarily a lightweight, self-hosted operations and
administration product for developer-operators and small technical teams. It is
secondarily a structured Rust full-stack reference implementation; product
journeys take priority over generic framework demonstrations.

The product is one operations console delivered as one signed release. The
release contains four independently restarted server processes but keeps one
version and one rollback boundary:

- Admin owns identity, RBAC, module control, release management, and the Web
  application.
- Monitoring owns managed nodes, resource metrics, alerts, incidents, and daily summaries.
- Analytics owns product-event collection and analysis.
- Reports owns target-backed browser filling templates and runs.

The current product does not contain a fifth service, a dynamic plugin system,
or independently versioned modules.

## Target users, problems, and principles

The primary-user definition remains an adoption assumption to validate. The
current product addresses five concrete problems:

- administer accounts, roles, permissions, and fixed modules from one place;
- understand installation and managed-service health;
- inspect lightweight product-activity summaries and details;
- execute repeatable browser-based reporting with visible failure evidence;
- install and update the complete product with one recovery boundary.

Product decisions follow these principles:

1. Complete setup-to-result, failure, and recovery loops before adding breadth.
2. Keep self-hosting understandable and configuration overhead bounded.
3. Preserve one product experience while isolating module failures and data.
4. Require explicit authorization, visible status, audit evidence, and
   recoverable outcomes for privileged work.
5. Prefer named user problems over template completeness or speculative
   abstractions.
6. Deepen the four fixed product areas before considering extensibility.

## Users and core journeys

- Owner configures the installation, roles, modules, and releases.
- Admin performs day-to-day operations without owner-only release authority.
- Viewer reads the concrete module capabilities granted by the module
  Manifests.
- An operator can inspect module health, monitor nodes and resource metrics, query
  analytics observations, and execute or inspect report-filling runs without
  leaving the Admin console.

Module failure must not prevent Admin login or make an unrelated module
unavailable.

An enabled fixed module's active, visible, reconciled navigation remains
discoverable in the sidebar and page search when its service is unavailable or
its current runtime contract is incompatible. Service health controls whether
the destination can currently serve a request, not whether an otherwise
visible entry exists. Disabling the module removes that navigation. Existing
RBAC grants and manual menu-visibility overrides continue to decide which of
the enabled module's destinations a user may see.

The retained end-to-end journeys are:

- Admin: sign in, inspect system state, manage users and roles, control modules,
  inspect operations, and manage releases within owner/admin/viewer boundaries.
- Monitoring: inspect health, nodes, recent metrics, alerts, and incident state,
  with missing data distinguishable from healthy empty data.
- Analytics: inspect installation-wide activity and raw details without a query
  or collection failure blocking other product areas.
- Reports: define a target-backed template, validate run input, execute browser
  steps, and retain enough live or captured evidence to diagnose failure.

Reports retains non-sensitive run input for execution but omits it from run
responses. Template definitions and run requests that reference password,
token, key, credential, or other recognized secret fields are rejected. Users
must not submit secrets until a separate protected-storage, encryption, and
write-only-input boundary is specified, implemented, and verified.

Across those journeys, loading, empty, validation, permission, business-error,
retry, audit, interruption, and recovery results must remain explicit.

## Console interaction requirements

Confirmed product requirements for the current console:

- Existing list filters apply automatically; operators do not submit a Search or
  Reset action. Text input applies after a 300 ms pause, composition candidates do
  not query, clearing applies immediately, and selection/date changes apply immediately.
- A changed applied filter starts at page one before the request. Loading and failures
  preserve editable filters and keyboard focus; failures expose retry rather than false empty data.
- User management offers username and account-status filters only. Removing email
  and real-name filters does not remove those user fields from forms or results.
- Role, menu, operation-log, incident, analytics-detail, and module-log lists retain
  their current filter meanings. Log export uses the applied list conditions.
- Daily summaries present the retained per-node reports without a node-ID search.
- The Nodes page owns Add node onboarding and Global settings actions. The independent
  Alert Settings menu/page is removed. Add node presents the signed offline Agent
  installation prerequisites and verifies registration through real reports; it creates no
  empty records and never displays a token or a direct Agent-start command.
- The four global alert settings form one drawer configuration task and one Save operation.
  Node overrides, inheritance, permission gates, validation, and alert evaluation are unchanged.
- Analytics overview/details show observed activity without a collection-policy status
  panel or a policy-status read solely for display. Removing that panel does not disable
  ingestion policy, allowed-origin enforcement, or backend management permissions.
- Dashboard keeps four account totals, a permission-gated CPU/memory/disk summary,
  and textual availability for the three modules. No extra metrics or trends are implied.

Acceptance IDs: CON-01 automatic filters and immediate clearing; CON-02 page-one
queries and retained input focus; CON-03 user filter scope and no Daily Summaries
search; CON-04 one alert configuration task; CON-05 activity-only Analytics surfaces;
CON-06 factual Dashboard scope. The [UI index](../ui/index.md) maps these behaviors
to route composition. Root [DESIGN.md](../../DESIGN.md) owns shared appearance,
placement, responsive layout, and component semantics. These requirements do not
change backend APIs, permissions, release scope, or independently bump the version.

## Product language

Confirmed display names: **自动化 / Automation** for the Reports-owned module,
and **告警事件 / Alert incidents** for `/monitoring/incidents`. Navigation,
page search, module status, headings and state messages use these names.
Internal service names, routes, permissions, child-page names and browser
execution scope remain unchanged.

| Product term | Current meaning | Product owner |
| --- | --- | --- |
| Admin | Control plane, identity, RBAC, release, and Web host. | Admin |
| Monitoring | Node, resource metric, alert, incident, and daily-summary operations. | Monitoring |
| Analytics | Instance-wide event overview and detail queries. | Analytics |
| Automation | Target systems, browser filling templates, runs, artifacts, and live frames. Display name of the Reports-owned module, not a separately shipped module. | Reports |
| Report Center | A possible future cross-module report catalog. It is not implemented. | Deferred |

Technical ownership and stable internal names are defined in
[`architecture.md`](../architecture.md) and
[`project-map.md`](../project-map.md).

## Module purposes and direction

| Product area | Internal name | Current purpose | Direction | Explicit non-goal |
| --- | --- | --- | --- | --- |
| Admin | Admin | Trusted control plane for identity, RBAC, module state, system status, operations, and releases. | Clarify installation, access, diagnosis, update, and recovery. | ERP, generic CRUD generation, low-code admin, or workflow builder. |
| Monitoring | Monitor | Managed-node, resource metric, alert, incident, and daily-summary operations. | Improve the path from signal to actionable incident for small installations. | Full APM, tracing, log warehouse, or cloud orchestrator. |
| Analytics | Insights | Lightweight installation-wide activity collection, overview, detail, and retention. | Make the retained signals useful before adding event families or segmentation. | Marketing automation, general BI, warehouse, or multi-tenant analytics. |
| Automation | Reports | Controlled browser-filling targets, templates, runs, steps, live frames, artifacts, cancellation, and recovery. | Strengthen authoring, validation, credential boundaries, visibility, and recovery. | Unrestricted scripts, general RPA, document editor, or open-ended browser agent. |

## Confirmed decisions

1. Current modules keep independent failure and data-ownership boundaries.
2. A new module starts from a live comparison with the relevant former
   standalone repository. The comparison preserves product behavior, not old
   directory layout or duplicated platform code.
3. Existing Admin identity, RBAC, deployment, navigation, and Web shell are
   reused. A module must not import a second copy from a former repository.
4. Cross-module code moves to a shared owner only after it has compatible real
   consumers and an independently verifiable contract. Security and protocol
   consistency may justify extraction earlier than ordinary presentation code.
5. Module business meanings, status lifecycles, calculations, data selection,
   and failure semantics remain with the module.
6. Root `DESIGN.md` is the sole current authority for shared visual semantics,
   component semantics, and design-approval records. A new or changed product
   surface still requires a scoped UI specification before frontend
   implementation; `docs/ui/features/*` may retain slice-local behavior and
   acceptance evidence. Structured `docs/ui/` packages such as
   `evaluation.yaml` and `artifact-manifest.yaml` are historical or task-local
   evidence only and do not govern current approval.
7. Dashboard is a control-plane landing page: it shows account totals, module
   health, and a permission-gated summary of the Admin host's CPU, memory, and
   disk usage. Detailed storage and host-resource diagnostics remain owned by
   System Status, while module metrics and trends remain on their Monitoring
   and Analytics pages.
8. Menu management shows the navigation inventory used by the sidebar and page
   search. Core Admin navigation is read-only there; only module-owned
   presentation may be edited. It does not create capability definitions.
   Admin route registration and module Manifests generate the capability
   catalog, while role management is the only product surface that assigns
   those capabilities to custom roles.
9. Dictionary management is not a current product capability because no
   in-repository workflow consumes it. Its HTTP surface, page, navigation, and
   permission are removed. The dormant SQLite table is not a supported
   integration or upgrade-compatibility contract and may be removed from the
   resettable baseline when schema cleanup is in scope.

## Legacy-product decisions

The evidence and path-level comparison are recorded in
[`legacy-module-comparison.md`](../reference/legacy-module-comparison.md).

### Monitoring

Monitoring is defined by
[`features/monitoring/spec.md`](./features/monitoring/spec.md). It uses
the current Monitoring surface with a lightweight fixed-metric Agent and a
Controller-owned 30-day data, policy, incident, and report model. Agents report
CPU, memory, and per-mount disk usage every 30 seconds; they do not receive
configuration or execute configurable checks.

The former `rustzen-inspect` remains only a behavior and failure-scenario
reference. Do not copy its Admin, system, project, deployment, permission,
runtime-layout, protocol, or database layers. Monitoring alert policies and
reports are authorized only within the Monitoring central ownership and
retention rules. Notification delivery and reports beyond the 30-day data
window are not Monitoring capabilities.

### Analytics

Retain the current single-project, instance-wide Analytics behavior. The former
`rustzen-analytics` is the reference for a possible multi-project evolution:
project lifecycle, stable project keys, browser-origin and application-package
allowlists, bounded ingestion, aggregation, and richer project queries.

Multi-project behavior changes event identity, permissions, navigation, and
data ownership. It requires one dedicated feature specification and migration
plan before implementation. The old repository's duplicate Admin and deploy
features must not return.

### Reports and Automation

Retain the current target, flow, run, step, screenshot, artifact, cancellation,
and live-frame loop. The former `rustzen-report` is the behavior reference for
possible account credentials, datasets, upload processing, a richer expression
and group DSL, suspend/resume semantics, scheduled runs, and detailed live job
events.

Each capability is a separate feature slice. The old authentication, users,
system settings, deployment, and application shell are rejected because Admin
already owns them. The browser runtime stays Reports-owned until another real
module needs the same semantics; it must not become a generic workflow engine
in advance.

The selected bounded Reports automation slice is
[`scheduled-report-automation`](./features/scheduled-report-automation/spec.md):
daily and weekly schedules around existing target-backed flows, installation
timezone, missed-occurrence skip semantics, and one ordinary run per enqueued
occurrence. Credentials, datasets, expression/group DSL, notifications, and
webhooks remain deferred; they require separate protected-storage, data, or
delivery contracts.

### Report Center

A cross-module report catalog, common report summary, document renderer, and
artifact catalog are deferred. They require at least two real report producers,
an explicit permission model, and a product specification for degraded module
behavior. They do not justify a fifth process or a new contract crate today.

## Non-goals

- Whole-repository source copying from a former product.
- Compatibility wrappers for former HTTP paths, database names, binaries, or
  deployment layouts.
- A universal business status enum, CRUD framework, form DSL, dashboard
  builder, repository layer, or workflow engine.
- A generic dictionary administration surface without a current product
  consumer.
- Cross-database joins or one module reading another module's SQLite database.
- Moving a capability to a shared package before current consumers prove the
  same contract.

## Decision status

- Confirmed: the current product has Admin, Monitoring, Analytics, and Reports
  in one release; Automation remains part of Reports.
- Confirmed: former products are capability evidence, not whole-product
  migration targets.
- Confirmed: Dashboard includes account totals, module health, and the three
  key Admin-host resource percentages for operators who can view System
  Status; detailed resource and product telemetry stay on their owning pages.
- Confirmed: capability definitions cannot be added manually. Admin routes and
  module Manifests generate them, and role management assigns them.
- Confirmed: Dictionary management is removed from the current surface; its
  dormant SQLite table is not an upgrade-compatibility contract.
- Confirmed: the primary positioning is a lightweight self-hosted operations
  product; the reference-implementation role is secondary.
- Confirmed: SQLite and one coherent signed release are the current supported
  operating boundary.
- Confirmed: Reports has selected the bounded daily/weekly schedule slice; it
  does not authorize credentials, datasets, expression/group DSL,
  notifications, or webhooks.
- Assumption: the primary adopter is a developer-operator or small technical
  team managing one installation.
- Assumption: the named former repositories remain the best product-behavior
  references. Revalidate their live default branches before each slice.
- Open: which retained journey currently causes the most user friction and
  should receive the next bounded feature specification.
- Open: which deferred capability becomes the first implementation slice.
- Open: whether Report Center will become an Admin projection, a Reports
  capability, or a separately approved module.
- Rejected: copying a former repository's Admin shell, authentication, RBAC,
  deployment, or directory layout into a module.
- Deferred: multi-project Analytics, Reports credentials/datasets/
  expression-group DSL/notifications/webhooks, Monitoring notification delivery
  and reports beyond its 30-day window, Report Center, and a fifth process.
  Monitoring global/node thresholds and daily summaries are implemented within
  the current central-monitoring scope.

## Development horizon and success signals

Now, consolidate onboarding, permission safety, understandable states, update
recovery, and the four retained journeys. Next, deepen only the highest-friction
steps proven by adoption evidence. Later, evaluate bounded notifications,
exports, or target-specific helpers; hosted SaaS, multi-tenancy, fleet
administration, dynamic plugins, and a broader extension model require a new
product-boundary decision.

The product is succeeding when a new owner can install, sign in, understand
health, and complete one useful journey in every enabled module; ordinary
operators see only permitted actions; module failures remain diagnosable
without losing Admin; and runs and updates retain understandable success,
failure, and recovery evidence. Quantitative targets remain open until real
usage or user feedback exists.

## New-module acceptance

Before implementation begins, a module proposal must provide:

1. a fixed live source revision for every former repository used as evidence;
2. a capability matrix with `retain`, `reproduce`, `reuse`, `extend`, `wrap`,
   `new`, `defer`, or `drop` for every selected behavior;
3. declared product, data, permission, and navigation ownership;
4. a bounded user journey with normal and failure acceptance;
5. a reuse search through the current shared Rust and frontend owners;
6. verification for the selected behavior and every new shared contract;
7. synchronized architecture, project map, product, UI, command, and deployment
   documentation when those boundaries change.

## Ready verdict

Ready for legacy comparison and feature-specification slices.

Not ready for direct implementation of multi-project Analytics, expanded
Reports automation, Report Center, or a fifth module. Each remains deferred
until its named feature specification resolves behavior, permissions,
migration, failure states, and acceptance.
