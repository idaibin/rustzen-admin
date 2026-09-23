# Frontend Rules

## Read

- `../../DESIGN.md`
- `docs/guides/frontend.md`
- `docs/guides/ai-coding-rules.md`

## Rules

- Keep auth routing and permission gates in `apps/web/src/routes/__root.tsx`.
- Keep request code inside `apps/web/src/api/`; pages use domain APIs from `@/api`.
- Do not edit `apps/web/src/routeTree.gen.ts` manually.
- Build the smallest implementation that solves the current need.
- When backend APIs change, update `apps/web/src/api/` first, then update page usage.
- Pages must not re-implement what a shared owner already provides. Extend the shared
  owner instead of adding a page-local copy; create a new shared module only when the
  existing owners leave an explicit gap.

## Shared Owners (unified maintenance)

- Formatting: `src/lib/format.ts` (`formatBytes`, `formatPercent`, `formatDuration`) and
  `src/lib/format-date-time.ts` (`formatDateTime`). No local formatter definitions in
  routes or page helpers.
- Status tags: `src/components/status-tag.tsx` (`StatusTag`) with maps from
  `src/constant/options.ts` (`getEnableStatusMeta`, `getUserStatusMeta`,
  `getMenuTypeMeta`). Status colors use Ant Design status presets
  (success/default/warning/error), not ad-hoc palette names.
- Confirm dialogs: `src/components/feedback/confirm-dialog.tsx` — `ConfirmDialog` for
  trigger-wrapped flows, `ConfirmModal` for controlled open state. No hand-rolled
  `Modal` with okText/okButtonProps or footer button pairs for confirm flows.
- Dialog footers: `src/components/feedback/dialog-footer.tsx` (`DialogFooter`) for
  cancel/submit button rows inside Modal/Drawer forms.
- Tables: `src/components/table/table-presets.tsx` (`displayTableProps`,
  `pagedTableProps`, `tablePagination`, `emptyTableLocale`) plus `action-column.ts`
  and `data-table-shell.tsx`; async feedback via `data-state.tsx` (`DataState`).
- Drawer pinned footers: `src/components/page/panel-layout.tsx` (`PanelBody`,
  `PanelFooter`).
- Selected portal builds: per-portal files under `src/distribution/` stay thin
  (route/permission tables, nav items, and required marker words) over the shared
  factories `portal-api.ts`, `auth-store.ts`, `portal-layout.tsx`,
  `menu-query-options.ts`. The selected-build module allowlist is owned by
  `scripts/distribution-web-inventory-policy.ts`; keep its fixtures and generated
  portal inputs aligned with that policy.
- After deduplication or refactor batches, re-grep for local copies and keep them at
  zero: `function format`, `statusMeta`, `okText=`, `showSizeChanger: false`,
  `justify-end gap-2`.

## Command Source

- Use root `justfile` as the command source of truth.
