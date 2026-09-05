# Module Log Diagnostics UI

## Profile, authority, and selected source

- Profile: **Feature UI**.
- Product basis: [Module Log Diagnostics, Backup, and Cleanup](../../product/features/module-log-diagnostics/spec.md).
- Shared visual authority: root `DESIGN.md`; this slice does not restate its
  theme, component, state, layout, or accessibility semantics.
- Selected source identity: accepted current System Status and Manage Log
  surfaces in `apps/web/src/routes/system/status.tsx` and
  `apps/web/src/routes/manage/log.tsx`, plus repository-owned `DESIGN.md`
  adopted root `DESIGN.md` baseline.
- Selection status: accepted existing product surfaces. Rights/use are
  repository-owned; legacy file browsers, glass/gradient references, and the
  operation-log surface as a process-log substitute are ignored.
- Target: an owner-only diagnostics section under System Status with
  module/file list, bounded reverse-cursor tail, bounded Blob archive backup
  action, cleanup preview, and confirmation in loading, populated, empty,
  error, processing, and partial states
  at 1920x1080, 1440x900, and 390x844 CSS px, 100% zoom, light/dark,
  zh-CN/en-US. The frozen disposable Linux Chromium contract uses owner `/system/status`, an explicit current-UTC `admin` fixture and one expired `monitor` fixture while retaining service-created current-day entries; it captures 1440x900 zh-CN and 390x844 en-US, with no page horizontal overflow. Runtime captures for the actual deployment remain `Not verified`.

Dates shown for daily files, retention cutoffs, and current-day protection are
UTC, matching the rolling logger; labels must make the UTC basis clear.

The selected source proves the current status page shell, storage/resource
cards, operation-log table, download transport, and confirmation semantics. It
does not authorize arbitrary filesystem browsing or a second log database.

## Surface and layout contract

- Keep one `PageHeader` for System Status. Preserve the source order of
  `StorageCard`, `ResourceCard`, then `ModuleLogDiagnostics`; do not rename or
  replace the separate `/manage/log` operation-log page.
- The diagnostics panel begins with a fixed module selector and file/date
  metadata table. Selecting a file opens a bounded reverse-cursor tail Drawer or detail region
  that owns its vertical scroll; the page owns no nested horizontal scroll.
- The module-log diagnostics route/menu and all three actions are owner-only;
  admin, viewer, and custom-role users cannot enter this System Status surface
  and direct endpoint requests are rejected. No diagnostics-local permission
  state is rendered for a user stopped at that boundary. Cleanup uses a preview
  list and existing `ConfirmDialog` pattern; a one-click destructive action is
  prohibited.
- Long lines wrap by default. If a bounded preformatted region is necessary,
  it is explicitly labeled and horizontally scrollable within the Drawer only.
- At narrow widths, module/file controls stack, metadata columns reduce to
  prefix/date/size/status/action essentials, and the detail Drawer retains a
  reachable close and confirmation action.

The module and date filters occupy the diagnostics heading upper right and apply
on change. Refresh, backup and cleanup remain explicit actions. Limits/help text sits
below the heading; it does not stretch the filter group. A short log-file list uses
natural height without an internal vertical scrollbar, while the page owns vertical
scrolling and the table bounds horizontal overflow. Filter changes clear selection.

## Component and data-owner mapping

| Responsibility | Current owner | Decision |
| --- | --- | --- |
| Page title and status summary | `PageHeader` and existing System Status cards | Reuse |
| Module/file metadata list | `DataTableShell` + route-local `ProTable` or existing table pattern | Reuse; columns remain local |
| Loading, empty, error, processing | `DataState` | Reuse; owner-only route boundary prevents a local permission state |
| Bounded reverse-cursor log tail | Ant Design `Drawer` + `Typography`/code region | Wrap route-local content; hard caps are 256 KiB, 2,000 lines, and 16 KiB per line; no shared file viewer |
| Backup download | Admin generated binary transport and download action semantics | Reuse; bounded 64 MiB Blob, manifest/hash copy, and fail-closed result are route-local. Require and validate `Content-Disposition`, `X-RustZen-Archive-SHA256`, and `X-RustZen-Archive-File-Count` before download; success feedback shows filename, file count, and hash summary. |
| Cleanup review/confirm | Existing `ConfirmDialog` and Ant Design list/table | Reuse |
| Partial result | Ant Design `Alert`/`Tag` with semantic status | Wrap route-local item outcomes |
| HTTP transport | Admin system API client and binary response metadata | Reuse; Admin route is authority and Web rejects a missing or invalid archive metadata header |
| Log content | Runtime files emitted by each service | Services own content; Admin owns allowlist/access/audit |

No new global file-browser, log database, table, or archive component is
introduced. `DataState` and semantic tokens remain the only shared feedback
owners.

## State and interaction contract

| State | Presentation | Interaction |
| --- | --- | --- |
| Loading | Compact `DataState` in the diagnostics panel or Drawer | Preserve selection; disable duplicate actions. |
| Populated | Metadata table or bounded tail with module/date labels | Keyboard-accessible row opens detail. |
| Empty | `DataState` distinguishes absent/empty file from no cleanup candidates | Explain scope/cutoff; no false error. |
| Error | Alert-semantic `DataState` with retry | Retry only the owning read/action; no success toast alone. |
| Processing | Inline progress/disabled action for Blob download, preview, or confirm | Keep selected module/file and prevent duplicates. |
| Partial | Item-level outcome list and summary | Failed files remain identified; successful cleanup is not repeated blindly. |
| Preview ready | Candidate list plus cutoff/current UTC-day warning | Confirmation is explicit, scoped, and short-lived. |

The content viewer does not expose shell commands, arbitrary paths, live tail,
or operation-log rows. Current UTC-day/active files and changed candidates remain
non-destructible even when a preview was previously shown.

## Accessibility and responsive behavior

- The diagnostics section has a descriptive heading and table headers. Module,
  date, file size, and status are available as text; color is supplementary.
- Tail, backup, preview, and confirm controls are real keyboard-focusable
  buttons with visible focus and bilingual accessible labels. Drawer focus is
  trapped/restored; confirmation states do not discard the selected scope.
- Loading uses status semantics, errors and destructive warnings use alert
  semantics. The global route/API denial does not reveal log content; no local
  permission state is rendered.
- Long lines may scroll only inside an explicitly labeled bounded region. The
  page, table, and modal do not acquire hidden horizontal overflow.
- Existing reduced-motion, light/dark, and typography tokens remain
  authoritative. No new animation, glow, gradient, or decorative file icon
  system is introduced.

## API and data ownership

The Web client consumes an Admin-owned system diagnostics contract after the
Admin `ContractRouter` route is registered, exported to OpenAPI, and reflected
in the Orval-generated Admin client. This is the only one of these four slices
that uses the Admin OpenAPI/Orval chain. The document intentionally does not
duplicate paths or file schemas. Admin owns the fixed allowlist, path safety,
64 MiB preflight/Blob archive, manifest/hash metadata, confirmation token,
capability checks, and audit metadata. `crates/runtime` and each service own
file naming/emission/retention; no module database is read by Admin.

## Traceable UI deltas

| ID | Selected source | Current runtime | Target contract | Priority | Owner and validation |
| --- | --- | --- | --- | --- | --- |
| ML-UI-001 | `source-extracted`: System Status `PageHeader` and PageCard shell | `source implemented`: System Status composes the diagnostics section as an h2 PageCard | Keep one bounded diagnostics panel without replacing resource/storage summary | P1 | Linux Chromium owner flow freezes desktop/narrow screenshots and no-overflow assertions; actual deployment remains `Not verified` |
| ML-UI-002 | `source-extracted`: Manage Log table/download semantics | `source implemented`: fixed module/date metadata table and bounded Tail Drawer are rendered | Keep process logs distinct from operation logs; fixed module/file/date metadata only | P1 | Linux Chromium owner flow freezes current-file Tail Drawer/marker/boundary evidence; full archive-byte/hash verification remains the Admin/client service gate |
| ML-UI-003 | `source-extracted`: existing ConfirmDialog/DataState | `source implemented`: preview, expiry, processing, confirmation, failure, and partial paths are rendered | Preview then short-lived confirm; loading/error/partial remain distinct, while non-owner access stops at the route/API boundary | P1 | Linux Chromium owner flow freezes expired-fixture-only preview, ConfirmDialog confirmation, and result evidence; fault/permission matrices remain separately covered |
| ML-UI-004 | `source-extracted`: generated binary transport and semantic status treatment | `source implemented`: metadata headers are required before download; success shows filename, file count, and hash summary; Tail Drawer renders bounds/truncation | Bounded Blob backup with manifest/hash metadata; preflight or mid-build change fails closed with no partial download; reverse-cursor tail caps at 256 KiB/2,000 lines/16 KiB per line and sets `truncated=true` whenever a cap is reached | P1 | Linux Chromium owner flow freezes selected-backup summary UI; adapter + service HTTP tests remain the archive-byte/hash authority |

The disposable Linux Chromium gate freezes the two specified geometry/localization states and success lifecycle. Actual service-account permissions, native systemd, production deployment, and broader failure/permission browser matrices remain `Not verified`. Source, OpenAPI/client, and disposable-service HTTP evidence do not replace those boundaries.

## Responsive and verification matrix

| Priority | Viewport | Theme/locale | Surface and state | Acceptance |
| --- | --- | --- | --- | --- |
| Required | 1920x1080 @ 100% | light / zh-CN | System Status populated | Diagnostics panel aligns with existing status content; module/file actions are reachable. |
| Required | 1440x900 @ 100% | dark / en-US | Tail Drawer and backup processing | Long lines, integrity copy, focus, and status contrast pass. |
| Required | 390x844 @ 100% | light / zh-CN | Empty/error | Controls stack; no content or action is clipped. |
| Required | 390x844 @ 100% | dark / en-US | Cleanup preview/partial confirmation | Candidate list wraps/scrolls in its owner; confirmation remains keyboard reachable. |

Static checks cover fixed module scope, owner-only route/menu/API boundaries,
absence of a diagnostics-local permission state, and separation from operation
logs. HTTP/browser validation must exercise
symlink/path safety, 64 MiB preflight, manifest/hash metadata, mid-build change
fail-closed behavior, current UTC-day protection, partial cleanup results, and two
same-viewport comparison passes after implementation.

## Shared-system changes and readiness

Shared-system changes: **None**. Reuse current System Status/operation-log
visual owners, `DataState`, table shell, Drawer, ConfirmDialog, download
transport, and semantic tokens.

## Ready for dev-frontend module log diagnostics

The selected source, layout ownership, component mapping, route/API authorization and action states, responsive/accessibility rules, and acceptance IDs are fixed. The disposable Linux Chromium verifier is limited to the explicit current `admin` and expired `monitor` fixtures while retaining service-created current-day entries, owner success lifecycle, 1440x900 zh-CN and 390x844 en-US screenshots, and no-overflow assertions. Runtime log availability (especially Insights), actual runtime permissions, native systemd, production deployment, and unlisted browser states remain `Not verified`.
