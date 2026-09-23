# System Status module storage visibility

## Goal and implementation slice

The System Status storage card answers "which module's database is how large".
The four services keep separate SQLite databases and no module reads another
module's database, so per-module visibility is self-reported: monitor,
insights, and reports each expose `GET /internal/v1/storage` beside their
Manifest, returning point-in-time file metadata (main/-wal/-shm byte sizes and
a collection timestamp) without reading any database content. Like the
Manifest endpoint it has no middleware and relies on the internal-host
binding.

Admin aggregates the self-reports through the existing module synchronizer
(10-second cycle): each sync joins the storage fetch with the manifest and
health probes and stores the last successful report on the module runtime. The
`GET /api/system/status` response adds a `modules` array; an available module
exposes its current database sizes, and an unavailable module hides stale
bytes while keeping the last successful collection time. The page never
fabricates values for an offline module.

## Page composition (v2)

- Row 1: Admin's own SQLite total with the main/WAL/SHM breakdown (unchanged).
- Row 2: module database panel — one row per module sorted by total bytes
  descending, showing status dot, module name, `module.db`, total, and WAL; an
  unavailable module shows 不可用 and its last collection time instead of bytes.
- Row 3: directory usage collapsed to one summary line
  (data · logs · web · bin); the four comparison bars are gone.
- Local resources remain the right-hand equal-height column.

## Permissions and non-goals

- The whole surface stays behind `system:status:view`; the module endpoints
  are internal and add no user-facing capability.
- No cross-module management: cleanup, retention, and compaction remain each
  module's own product surface (Monitor 30-day retention, Insights retention
  policy, Reports run retention). Admin only aggregates visibility.
- No module reads another module's database, and no database content leaves
  the owning service — only file sizes.

## Acceptance

- Each module's `/internal/v1/storage` self-report returns its own database
  sizes; the shared wire type lives in `rustzen-ipc` (`ModuleStorageReport`).
- The status overview lists every fixed module for the composition; available
  modules carry bytes, unavailable modules carry only the last collection
  time, and none invents values.
- The OpenAPI contract (`SystemStatusOverview.modules` +
  `ModuleDatabaseStatus`), the pinned baseline, and the generated client stay
  in lockstep.
- Verified locally: module endpoint tests in all three services, aggregation
  and offline-degradation tests in Admin, the full admin suite, and the live
  page showing the three module rows plus the directory summary line.
