# Product Foundation

Status: current foundation specification.

This document is the single authority for current product positioning,
direction, module purposes, boundaries, and non-goals. Source remains
authoritative for delivered behavior; `docs/architecture.md` remains
authoritative for runtime and repository structure.

## Developer adoption priority

The primary user is a developer building a business administration application
with Rust and React. The first useful outcome is to run the Admin foundation,
understand an existing permission-protected feature, and adapt it to a business
need. SQLite keeps local development independent of a database service.

Prioritize developer onboarding, a complete user/role permission journey, the
existing core/full delivery contract, and a reusable business-feature walkthrough,
in that order. Monitor, Insights and Reports remain complete extension examples
under the existing composition contract. Their committed acceptance work remains
owned by the current feature ledgers.

Measure adoption through independent developer attempts, first useful outcomes,
help required, and subsequent use. GitHub views, clones and stars are discovery
signals, not proof of installation or recurring use. Department data scopes and
tenant isolation require separate product decisions before implementation.

## Template-first and English-default slice (2026-10-09)

Approved scope: restore general-purpose admin template positioning and English
as the initial UI language. Preserve explicit Chinese preferences, existing
module implementations and the complete-distribution release boundary.

Acceptance:
- TPL-01: English README.md and Chinese README-zh.md describe equivalent facts
  and commands; README-en.md becomes a link to the canonical English entry.
- TPL-02: distinguish delivered authentication, instance-level RBAC, audit,
  shared layout and feature-layer examples from unimplemented department data
  scopes and multi-tenant isolation.
- TPL-03: login copy identifies the reusable Rust+React admin template and
  points developers to accounts, role permissions, and audit as the starting
  point for business-specific development.
- TPL-04: Dashboard copy describes an account and role-permission business
  administration overview; only enabled, connected extension examples appear.
  A connected example remains visible when offline, while disabled or never-
  connected examples are omitted.
- LANG-01: first visit, missing/invalid storage and no-window rendering use
  en-US regardless of browser language; initialization never overwrites preferences.
- LANG-02: explicit zh-CN/en-US selections persist across reloads; shared copy,
  HTML lang, Ant Design and date formatting follow the selected locale.
- SCOPE-01: existing modules stay installed; runtime disablement does not imply
  optional builds or independent releases. No backend/schema/deployment changes.

Validation: focused locale regressions, existing localization/date tests, Web
format/lint/typecheck/tests/build, README link checks and independent review.
Browser acceptance covers initial English, Chinese selection/reload and invalid
storage in the unchanged login layout. Production and full backend verification
are outside this slice.

Local verification (2026-10-09): Bun 1.3.14 ran all 174 Web tests successfully,
including fresh-process locale/HTML bootstrap checks and existing date/copy
coverage. Web formatting, lint, TypeScript and production build passed; lint
still reports warnings in existing test/generated code, and the build retains
its large-chunk warning. All 30 README relative link occurrences resolve and
English/Chinese command blocks match. An isolated Chrome session at 1440x900
against the production Web preview confirmed English on fresh storage despite
navigator.language=zh-CN, Chinese after selection/reload, and English on invalid
storage without overwriting that value. No backend or production claim follows.
Independent review identified a stale default-language acceptance sentence and
an English README install-path omission; both were corrected before commit.


## Approved composition: optional operations examples

The approved target is a default Admin-only `core` template and an explicit `full`
complete-example composition. The [OE task ledger](features/composable-distribution/tasks.md)
records stage acceptance for core runtime/Web and packaging, independently accepted
installer work, and the 2026-10-24 native and browser acceptance of current-source
Linux fixtures.
The [distribution plan](features/composable-distribution/implementation.md#optional-examples-stages)
owns execution; [OE acceptance](features/composable-distribution/validation.md#optional-examples-acceptance)
owns its exit criteria. Genuine preceding-full 0.5.1 old-worker compatibility remains
unverified and blocks release readiness under OE-T10; current-source fixtures do not
substitute for that release.

Current Cargo defaults and `just build` still select full. The target core default
and `just build-full` interface become supported only with implemented command wiring
and OE-5 closure, including core installation/update gates. Stage acceptance does
not establish overall release readiness; the complete-release instructions below
describe the current default path.

Core must run authentication, instance-level RBAC, audit and shared layout without
Monitor, Insights, Reports, their databases, Chromium or their configuration.
Retain Admin-native account, role, audit browsing, diagnostics and release/recovery
support where this avoids a second Admin implementation. Do not remove audit just
because the historical access-only prototype omitted its browsing surface.
The first supported choices are `core` and `full`, not arbitrary module subsets.
Department scopes and multi-tenancy remain separate, unimplemented capabilities.
The user's compatibility requirement applies to the immediately preceding supported
full release and its data; it does not authorize arbitrary historical migration,
existing-data resets, production deployment, publication or main-branch merge.

## Product boundary

The supported first release is one complete signed distribution containing Admin,
Monitor, Insights, Reports, the Web application and the durable message center.
All three product modules are installed and enabled by default. The owner may disable
an installed module at runtime, which removes its navigation and gateway access but
does not change the signed installed inventory. The first-release acceptance boundary
is the complete distribution.

Rustzen Admin is primarily a general-purpose Rust full-stack admin template
for developers building business administration applications. Its runnable
reference application provides authentication, instance-level RBAC, audit logs,
shared layout and layered feature implementations. Department data scopes,
tenant membership and multi-tenant isolation are not implemented.

Monitor, Insights and Reports are implemented extension examples in `full`.
The approved `core` composition excludes them; its implementation and remaining
acceptance follow the composition boundary above. Independent module releases and
arbitrary subsets are outside the approved two-composition scope.
The UI defaults to English, preserves explicit Chinese preferences and does
not infer the application locale from browser language.

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

The complete distribution uses one Ed25519 release signature and one deployment
transaction. Its signed identity records separate `frontend` and `backend`
digests so operators can distinguish the embedded Web build from the server
executable set. These digests are audit identities, not independently deployable
artifacts; frontend-only, backend-only, and mixed-version updates remain unsupported.

The supported installation journey is deliberately short: build the complete
bundle, copy it to a server, run its installer, and execute `rz start`. The
installer does not ask for an administrator password, a signing key, a
verification key, or database commands. It uses the release verification key
packaged by the trusted build, generates service-to-service secrets locally,
and prepares the fresh runtime layout. Each service owns creation and validation
of its fresh SQLite database during startup. Operator configuration is limited
to ordinary runtime overrides such as bind addresses, ports, paths and timezone.

Admin API updates use the same packaged verification key and the existing
owner-only deployment permission. A candidate is one complete release. It
becomes current only after all four services pass health checks; otherwise the
previous release and its database snapshots are restored.

## Target users, problems, and principles

The primary audience is developers extending the template for their own admin
applications. The usefulness of the bundled operations examples remains an
adoption assumption to validate. The current reference application addresses
five concrete problems:

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
5. Prioritize reusable administration foundations and demonstrated developer
   needs over speculative abstractions.
6. Keep extension examples bounded; assess separation before adding modules.

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
  and textual availability for connected, enabled module examples. No extra
  metrics or trends are implied.

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

| Product term  | Current meaning                                                                                                                                         | Product owner |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| Admin         | Control plane, identity, RBAC, release, and Web host.                                                                                                   | Admin         |
| Monitoring    | Node, resource metric, alert, incident, and daily-summary operations.                                                                                   | Monitoring    |
| Analytics     | Instance-wide event overview and detail queries.                                                                                                        | Analytics     |
| Automation    | Target systems, browser filling templates, runs, artifacts, and live frames. Display name of the Reports-owned module, not a separately shipped module. | Reports       |
| Report Center | A possible future cross-module report catalog. It is not implemented.                                                                                   | Deferred      |

Technical ownership and stable internal names are defined in
[`architecture.md`](../architecture.md) and
[`project-map.md`](../project-map.md).

## Module purposes and direction

| Product area | Internal name | Current purpose                                                                                                 | Direction                                                                          | Explicit non-goal                                                                |
| ------------ | ------------- | --------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| Admin        | Admin         | Trusted control plane for identity, RBAC, module state, system status, operations, and releases.                | Clarify installation, access, diagnosis, update, and recovery.                     | ERP, generic CRUD generation, low-code admin, or workflow builder.               |
| Monitoring   | Monitor       | Managed-node, resource metric, alert, incident, and daily-summary operations.                                   | Improve the path from signal to actionable incident for small installations.       | Full APM, tracing, log warehouse, or cloud orchestrator.                         |
| Analytics    | Insights      | Lightweight installation-wide activity collection, overview, detail, and retention.                             | Make the retained signals useful before adding event families or segmentation.     | Marketing automation, general BI, warehouse, or multi-tenant analytics.          |
| Automation   | Reports       | Controlled browser-filling targets, templates, runs, steps, live frames, artifacts, cancellation, and recovery. | Strengthen authoring, validation, credential boundaries, visibility, and recovery. | Unrestricted scripts, general RPA, document editor, or open-ended browser agent. |

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
   acceptance evidence.
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

### Monitoring

Monitoring is defined by
[`features/monitoring/spec.md`](./features/monitoring/spec.md). It uses
the current Monitoring surface with a lightweight fixed-metric Agent and a
Controller-owned 30-day data, policy, incident, and report model. Agents report
CPU, memory, and per-mount disk usage every 30 seconds; they do not receive
configuration or execute configurable checks.

Monitoring alert policies and reports are authorized only within the Monitoring
central ownership and retention rules. Notification delivery configuration and
reports beyond the 30-day data window are not Monitoring capabilities; authorized
operators can read aggregate delivery health for existing Monitoring incidents
only when notification delivery is available.

The Monitor delivery-card Linux Chromium extension has a local, checkout-bound
closure: it passes only when
`target/rz/monitoring-ui-state/current/manifest.json` matches the current
checkout and has `status: "passed"`. That evidence validates 23 canonical
route runs, 18 owner and 26 viewer delivery steps, four screenshots, real
Monitor SQLite and authorized API receipts, permission behavior, and either
zero retries or a recorded single retry receipt. Runtime module disablement remains
covered by the complete Web application's module-state and authorization gates.

### Analytics

Retain the current single-project, instance-wide Analytics behavior.

Multi-project behavior changes event identity, permissions, navigation, and
data ownership. It requires one dedicated feature specification before
implementation.

### Reports and Automation

Retain the current target, flow, run, step, screenshot, artifact, cancellation,
and live-frame loop.

Each capability is a separate feature slice. The browser runtime stays
Reports-owned until another real module needs the same semantics; it must not
become a generic workflow engine in advance.

The bounded Reports automation slice is
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
- Confirmed: Dashboard includes account totals, module health, and the three
  key Admin-host resource percentages for operators who can view System
  Status; detailed resource and product telemetry stay on their owning pages.
- Confirmed: capability definitions cannot be added manually. Admin routes and
  module Manifests generate them, and role management assigns them.
- Confirmed: Dictionary management is removed from the current surface; its
  dormant SQLite table is not an upgrade-compatibility contract.
- Confirmed: general-purpose admin template positioning is primary; the
  complete operations application is its runnable reference and extension set.
- Confirmed: SQLite and one coherent signed release are the current supported
  operating boundary.
- Confirmed: Reports has selected the bounded daily/weekly schedule slice; it
  does not authorize credentials, datasets, expression/group DSL,
  notifications, or webhooks.
- Assumption: developers benefit from the current complete reference release;
  demand for a smaller template distribution still needs validation.
- Open: which retained journey currently causes the most user friction and
  should receive the next bounded feature specification.
- Open: which deferred capability becomes the first implementation slice.
- Open: whether Report Center will become an Admin projection, a Reports
  capability, or a separately approved module.
- Rejected: copying a former repository's Admin shell, authentication, RBAC,
  deployment, or directory layout into a module.
- Deferred: multi-project Analytics, Reports credentials/datasets/
  expression-group DSL/webhooks, Monitoring notification delivery
  configuration
  and reports beyond its 30-day window, Report Center, and a fifth process.
  Monitoring global/node thresholds and daily summaries are implemented within
  the current central-monitoring scope.

## Development horizon and success signals

Now, close the remaining approved core/full release gates while preserving the
current full default. Installer and current-source Linux native/browser fixture
acceptance are recorded in the OE ledger; the genuine preceding-full old-worker
row remains blocked on its exact signed artifact and trusted materials. Core
default selection remains gated by OE-5 and verified command wiring.

The remaining release work must preserve both composition contracts. Acceptance
must cover clean build/install/login, RBAC and audit, no dead module routes or
background calls, and update/recovery for each declared
composition. Selected-build code and current-source fixtures do not establish
historical-worker compatibility or overall release readiness. Independent module
releases and a plugin ecosystem remain outside this scope. Department scopes and
shared-instance multi-tenancy require separate specifications and isolation tests.

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

Ready for feature-specification slices.

Not ready for direct implementation of multi-project Analytics, expanded
Reports automation, Report Center, or a fifth module. Each remains deferred
until its named feature specification resolves behavior, permissions,
migration, failure states, and acceptance.
