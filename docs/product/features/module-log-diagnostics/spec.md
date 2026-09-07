# Module Log Diagnostics, Backup, and Cleanup

## Goal and implementation slice

Give an owner a safe way to inspect the four local Rustzen service log files,
download a bounded verified external archive, and remove expired files without confusing
process logs with Admin operation logs. The fixed scope is `admin`, `monitor`,
`insights`, and `reports` under the configured runtime log directory.

The existing services use daily files and retention cleanup. This slice adds a
bounded Admin control-plane view and explicit backup/cleanup actions; it does
not expose arbitrary host logs or create a second log service.

Daily file dates and the current-day protection rule use UTC, matching the
runtime rolling logger; local timezone does not change eligibility.

## Users and scenarios

- An owner opens System Status, selects one allowed module and date, and reads a
  bounded tail of the corresponding process log.
- An owner requests a backup and receives a bounded Blob archive through the
  existing download transport, containing the selected files, a manifest, and
  a SHA-256 digest that can be checked outside the installation.
- An owner previews cleanup for a fixed retention cutoff, reviews the exact
  candidates, and confirms with a short-lived token.
- An admin, viewer, or custom-role user cannot enter the module-log diagnostics
  surface under System Status and cannot read, export, preview, or delete
  content through direct endpoints. All three module-log capabilities are
  owner-only in this slice; no local permission state is rendered for users who
  are stopped at the route/menu boundary.
- If one file disappears or cannot be read during metadata or cleanup, the
  result is visibly partial and identifies the affected module/file; a backup
  preflight or mid-build failure aborts the whole archive and returns no
  partial Blob.

## Confirmed decisions and rationale

| Decision | Rationale | Acceptance consequence |
| --- | --- | --- |
| The scope is the four local service prefixes only. | These files have a stable runtime owner and path contract. Admin, Monitor, and Insights emit directly under the runtime log root; Reports emits `reports.YYYY-MM-DD` under the fixed `logs/reports/` service-account directory. | No arbitrary path, managed-node OS log, or application log search is accepted. The Reports selector remains `reports` and archive basename remains `reports.YYYY-MM-DD`; the nested directory is never accepted from a request. |
| Admin owns authorization and audit; each service owns emitted content. | The Web console needs one control-plane boundary without merging databases. | File access is mediated by a fixed allowlist and actions are audited without copying content into Admin DB. |
| Backup is a bounded external Blob archive downloaded through the Admin binary transport. | A same-host copy is not an independent recovery artifact, while an unbounded archive complicates the current client boundary. | A preflight enforces a 64 MiB archive cap; Web requires `Content-Disposition`, `X-RustZen-Archive-SHA256`, and `X-RustZen-Archive-File-Count`, validates them before download, and shows the filename, file count, and hash summary after success; no local backup directory is invented. |
| Cleanup is preview plus short-lived confirmation. | Destructive file removal needs an explicit review seam. | Preview lists candidates; confirmation cannot be replayed after expiry. |
| The current UTC day is never deleted and is the only known-active marker. | A running process may still write it and operators need current evidence; cross-platform detection of an external process holding an older file is Not verified. | Cleanup rejects the current UTC-day file even when the cutoff is older; it makes no unsupported held-by-process claim for older files. |
| Operation logs remain a separate product surface. | They contain request audit semantics, not process stdout/stderr. | `/manage/log` is not relabeled as a module-log viewer. |

The archive cap is fixed at 64 MiB for the first release. An operator may
configure a lower limit but may not raise it above the hard cap. A server
preflight computes the selected file set and expected archive size before
starting a download; if the cap, path-safety, or readability check fails, the
request fails closed and no Blob is returned. The archive carries a manifest
entry and SHA-256 for every file, and the same digest is exposed in response
metadata/header. If any selected file changes, disappears, or fails hashing
while the archive is being created, the entire request fails and the client
receives no partial archive.

Cleanup uses a descriptor-relative atomic no-replace rename transaction where
the target provides it (Linux `renameat2` or macOS `renameatx_np`), then
rechecks identity and digest before unlinking the private entry. Targets
without that primitive fail closed as unsupported; this contract does not claim
an unprovable path-based compare-and-delete guarantee.

The tail contract is also fixed and bounded. Each request returns no more than
256 KiB or 2,000 lines, and no individual line exceeds 16 KiB. An opaque
reverse cursor starts at the newest end when omitted and continues toward older
content when supplied; arbitrary byte offsets are not accepted. If any byte,
line, or per-line cap is reached, the response sets `truncated=true` and may
return the next reverse cursor. Implementations may choose lower limits but may
not raise these hard caps or omit the truncation signal.

## Scope and non-goals

In scope:

- fixed module selector for `admin`, `monitor`, `insights`, and `reports`;
- file/date metadata: prefix, date, byte size, modified time, and readable
  status;
- bounded reverse-cursor tail viewing at no more than 256 KiB, 2,000 lines, and
  16 KiB per line, with `truncated=true` whenever any cap is reached;
- bounded Blob archive backup of selected files with manifest entries and
  SHA-256 digest;
- cleanup preview and short-lived confirmation for files older than a fixed
  policy cutoff, excluding the current UTC-day file (the executable known-active
  contract);
- clear partial results and safe retry for multi-file metadata/cleanup
  operations;
- localized warnings, route/API authorization errors, processing states, and
  audit records that include action metadata but never log content.

Non-goals:

- arbitrary filesystem browsing, path parameters, symlink traversal, or shell
  command execution;
- managed-node or third-party application logs;
- full-text indexing, search warehouse, live streaming tail, or log parsing;
- changing process log format, retention defaults, or service stdout behavior;
- backing up databases, uploads, releases, or operation logs through this action;
- deleting the current UTC day, unknown prefix, or a file outside the fixed runtime
  log directory;
- a generic file-manager component or shared log database.

## Main and failure flows

1. The owner enters the owner-only System Status diagnostics surface and the
   fixed module/file list loads. Non-owners are stopped by the route/menu gate
   and do not receive a local permission state.
2. Selecting an allowed file loads a bounded reverse-cursor tail. The viewer
   labels the module/date, reports `truncated=true` when a byte/line/per-line
   limit is reached, and uses the cursor to request older content.
3. Backup selection is validated server-side, preflighted against the 64 MiB
   archive cap, and downloaded as one bounded Blob. Before starting a browser
   download, Web validates `Content-Disposition`, `X-RustZen-Archive-SHA256`,
   and `X-RustZen-Archive-File-Count`; its success feedback shows the filename,
   file count, and hash summary. The manifest lists every included file and
   digest. A read, size, change, hashing, or invalid/missing metadata failure
   fails closed and returns no partial archive.
4. Cleanup preview computes candidates from the fixed prefixes and cutoff. It
   excludes the current UTC-day file, symlinks, unknown names, and path escapes.
5. The owner reviews the candidate list and confirms once. The token expires
   quickly, is bound to the preview, and cannot be reused. The result names
   removed, retained, and failed files and writes an audit entry.
6. A file changes or disappears between preview and confirmation. The service
   rechecks safety and returns a partial/failed item without deleting a new or
   current UTC-day file. Whether an external process still holds an older file is
   Not verified by this cross-platform contract.

## Business rules and permissions

- `system:module:log:view` gates listing and tail reads;
  `system:module:log:backup` gates Blob archive backup;
  `system:module:log:cleanup` gates preview and confirmation. All three are
  owner-only in this slice; admin, viewer, and custom roles cannot receive
  them through ordinary role assignment.
- The backend is the authorization and path-safety boundary. UI visibility is
  advisory. The route/menu boundary keeps non-owners out of the diagnostics
  surface; direct endpoint calls are rejected with the same owner-only
  authorization result, and no diagnostics-local permission state is shown.
- Only exact file names matching `<prefix>.YYYY-MM-DD` are eligible. Prefixes
  are the four fixed service IDs. `admin`, `monitor`, and `insights` resolve in
  the runtime log root; `reports` resolves only in its fixed `reports/`
  subdirectory. A missing Reports directory produces no list item, while a
  non-directory or symbolic-link Reports directory fails closed. Symlinks,
  directories, unknown suffixes, traversal, and arbitrary absolute paths are
  rejected.
- A backup manifest includes file name, size, modification time, and SHA-256.
  The bounded Blob archive is not stored as a local recovery copy by this
  slice, and any mid-build file change fails the entire download.
- Cleanup never removes the current UTC date, a candidate outside the fixed cutoff,
  or an item that fails a safety recheck. The contract does not claim to detect
  an external process holding an older file; that state is Not verified.
- Linux runtime acceptance follows the shipped units: Admin, Monitor, and
  Insights retain their current unit identity, while Reports runs as
  `rz-reports:rz-reports`. All four service units declare `UMask=0077`, so
  service-created daily logs follow the existing private-file contract at
  mode `0600`. `/opt/rz/logs` remains mode `0711` and
  `/opt/rz/logs/reports` remains owned by `rz-reports:rz-reports` at mode
  `0750`. This slice verifies that Admin can mediate the nested Reports file;
  it does not change service identities or broaden filesystem permissions.
- Partial applies to named metadata/cleanup items that were not read or
  removed. Backup is all-or-none: any archive item failure aborts the entire
  Blob and is not reported as a partial archive.
- Audit records include actor, action, selection, preview/confirmation IDs,
  result counts, and failure category, but never log lines or backup contents.

## UI states and evidence

The UI contract is [Module Log Diagnostics UI](../../../ui/features/module-log-diagnostics.md).
It places a diagnostics section under the existing System Status surface and
keeps operation logs on their existing page. Permission is enforced at the
route/menu/API boundary: non-owners do not enter this surface, so the panel does
not render a local permission state.

| State | User-visible meaning | Required behavior |
| --- | --- | --- |
| Loading | Metadata, tail, preview, or result is loading. | Keep selected module/date and prevent duplicate actions. |
| Populated | Allowed files or log lines are available. | Show bounded content with module/date and truncation context. |
| Empty | Query succeeded with no matching file/line/candidate. | Explain whether the file is absent, empty, or outside retention. |
| Error | Read, backup, or cleanup request failed. | Show localized reason and safe retry; no false success. |
| Processing | Bounded archive download or cleanup confirmation is active. | Disable duplicate actions and preserve selection. |
| Partial | Some files/items succeeded and others did not. | Keep the item-level result and require review before retry. |

## User-visible data effects

Tail viewing reads existing files only. Backup sends one transient bounded Blob
archive and manifest; it does not create a local backup record. Cleanup removes
only the explicitly confirmed eligible files and records metadata in the Admin
operation audit boundary. No log content is inserted into an Admin or module
database.

## Affected product surfaces and dependencies

- `crates/runtime` remains the owner of daily-file naming and retention
  mechanics; `apps/admin`, `apps/monitor`, and `apps/reports` continue to emit
  their own prefixes, while Insights must use the shared file logger before
  this slice is runtime-complete.
- Admin owns the control-plane route, fixed allowlist, preflight/Blob/manifest/hash
  implementation, confirmation token, and audit metadata.
- `apps/web` owns the System Status diagnostics composition and Admin API client.
- Existing `PageHeader`, `PageCard`, `DataState`, `DataTableShell`/route-local
  table, Ant Design `Drawer`/`Typography`, `ConfirmDialog`, and download
  transport are reused or wrapped locally. No shared file-viewer component is
  introduced.

## Acceptance criteria

- Owner can select only `admin`, `monitor`, `insights`, or `reports`; arbitrary
  path/module input is rejected at the backend.
- File metadata and bounded tail are readable without exposing symlink targets,
  traversal paths, or unbounded content; truncation is explicit.
- Viewer/admin/custom-role users without the owner-only capability cannot enter
  the diagnostics surface or read, backup, preview, or confirm cleanup; direct
  requests fail with the correct permission result and no local permission
  state is rendered.
- A tail response is never larger than 256 KiB, contains at most 2,000 lines,
  and caps each line at 16 KiB. The reverse cursor moves toward older content,
  and any cap reached sets `truncated=true`.
- Backup is one bounded external Blob archive (64 MiB hard cap) and contains a
  manifest plus SHA-256 digest for every included file. Web validates
  `Content-Disposition`, `X-RustZen-Archive-SHA256`, and
  `X-RustZen-Archive-File-Count` before download and shows their filename,
  file-count, and hash-summary evidence; preflight and any mid-build change
  fail closed with no partial download and no local same-host copy is treated
  as a backup.
- Cleanup requires a fresh preview and short-lived confirmation; current UTC-day,
  unknown, symlink, and changed-between-preview files are never deleted. An
  older file held by an external process is Not verified and is not represented
  as a reliable active-file guarantee.
- Partial cleanup results identify every failed item and never claim complete
  success; backup failures return no partial archive, and retry does not repeat
  successful destructive work.
- Operation audit metadata contains actor/action/result counts and no log
  content; operation logs remain distinct from process logs in navigation and
  copy.
- Fixed copy is localized in Simplified Chinese and English, while module IDs,
  file names, timestamps, and log lines remain unchanged.
- The linked UI matrix covers desktop/narrow, light/dark, loading/empty/error/
  processing/partial, keyboard focus, long lines, bounded tail truncation, and
  no overflow.

## Verification matrix

| Layer | Evidence | Acceptance |
| --- | --- | --- |
| Source/static | Implemented: fixed allowlist, symlink/path checks, owner-only capability, archive/hash, token, audit, System Status composition, and Web metadata validation | No arbitrary filesystem or content-to-DB path. |
| Automated | Implemented: file safety, preflight cap, Blob manifest/hash, preview-confirm, active-day, partial-result, tail caps/cursor, OpenAPI/client adapter, and service HTTP checks | Destructive boundaries, archive integrity, and bounded tail semantics pass. |
| HTTP | Disposable-service owner/non-owner requests through Admin | Owner-only route/API boundaries, direct denial, tail cursor/cap, archive headers, and cleanup token behavior are observable; no local permission state is needed for non-owners. |
| Browser | Disposable Linux Chromium owner flow at `/system/status` | The verifier creates only an explicit current-UTC `admin` fixture and expired `monitor` fixture while retaining service-created current-day entries, then proves the owner panel, current file tail Drawer/markers/boundary copy, selected-backup filename/file-count/SHA summary, preview-only expired fixture, explicit confirm, result state, desktop 1440x900 zh-CN and narrow 390x844 en-US screenshots, and no horizontal overflow. Archive bytes and full SHA verification remain covered by the service/client gate rather than simulated in Chromium. |
| Runtime/deployment | **Closed locally** by `just verify-module-log-runtime-linux` against current-provenance Linux binaries | The final `aarch64` Colima manifest at `target/rz/module-log-runtime/current/manifest.json` binds head `21ed7a8`, source tree `f0f4ede624e96600e894bf9b5a097c6df138a7dd0e2c7ce6386fe667fd4e91d4`, 25 receipts, five exact non-owner 403 envelopes, and an archive of 4 files and 9216 bytes with SHA-256 `de9330db80df01c6f4c30ce66921b61ae19295ac1653a51e4c939462dd1cf96a`. Reports process/directory/file identities agree at UID/GID `999:999`; `/opt/rz/logs` is `0711`, `/opt/rz/logs/reports` is `0750`; cleanup removed four old files with zero failures and preserved the current UTC-day files. The first jq-verifier failure remains under `failed-runs`, while `current` records the accepted final result. Independent review found no remaining P1/P2. Native systemd and production deployment remain `Not verified`. |

## Assumptions, open questions, rejected and deferred decisions

### Assumptions

- All four services write daily files under the configured runtime log layout:
  Admin, Monitor, and Insights at the log root, and Reports in its fixed
  service-account subdirectory.
- A bounded Blob archive can be consumed by the operator's external storage or
  download flow; long-term retention of that external copy is outside Admin.

### Open questions

- None block this slice. An implementation may choose lower tail limits, but it
  may not exceed 256 KiB, 2,000 lines, or 16 KiB per line and must preserve the
  reverse cursor and `truncated=true` semantics. The archive limit is fixed at
  64 MiB.

### Rejected

- Reusing the operation-log page as a process-log viewer.
- Accepting a user-provided path or shell command.
- Copying logs to the same host and calling that a disaster-recovery backup.
- Deleting files immediately from a single button without preview/recheck.

### Deferred

- Remote managed-node logs, live streaming, full-text search, compression
  policy selection, scheduled exports, and database/upload backups.

## Ready for module log diagnostics implementation

The fixed module scope, ownership, permission boundary, path safety, backup
integrity, cleanup confirmation, failure semantics, non-goals, and acceptance
are implemented in the Admin and Web source, including download metadata
validation. The browser acceptance contract is frozen for the disposable Linux verifier: it may create an explicit current UTC `admin` log fixture and one expired `monitor` fixture while retaining service-created current-day entries; all browser interaction uses the real owner `/system/status` UI, while archive-byte/hash proof remains service/client evidence. The dedicated Linux runtime gate is **Closed locally** at the published `current` manifest for source tree `f0f4ede624e96600e894bf9b5a097c6df138a7dd0e2c7ce6386fe667fd4e91d4`; its 25 receipts cover the four service-created prefixes, five exact 403 denials, Reports `999:999` ownership, archive SHA/bytes, and cleanup preservation. The retained first jq-verifier failure does not replace that final result. Native systemd and production deployment remain `Not verified` until exercised.
