# Role Definition Management UI

## Profile and selected source

- Profile: **Feature UI**.
- Selected source: the accepted current Admin role-management page and dialog
  in `apps/web/src/routes/system/role.tsx` on the current product source.
- Selection status: accepted existing product surface; repository-owned source.
- Use: current `PageCard`, table, dialog, form controls, `DataState`, semantic
  status styles, theme tokens, spacing, density, and bilingual copy pattern.
- Ignore: deleted legacy glass imagery and any visual alternative that is not
  part of the current standard light/dark Admin system.
- Product basis:
  `docs/product/features/role-definition-management/spec.md`.

This slice does not change shared tokens, component semantics, or the accepted
visual language. A design-system Manifest is skipped because no structured
automation consumer or shared-system revision is introduced.

## Layout and ownership

- Preserve the current role-list `PageCard`, toolbar, table, pagination, and
  action-column ownership.
- Preserve the current column order through **Permissions**, then add
  **Assigned users / 已分配用户**, followed by **Updated at** and the existing
  action column. The new column renders the backend `assignedUserCount` without
  changing the table's page size or pagination ownership.
- Preserve the existing role dialog width, field order, and footer.
- The permission region owns its bounded vertical overflow. The dialog remains
  the focus and outer scroll boundary; no page-level or horizontal overflow is
  introduced.
- Permission state feedback occupies the same bordered region as the populated
  permission catalog so loading, error, and empty transitions do not materially
  move the dialog footer.
- At narrow widths, use the existing dialog reflow and keep the permission tree
  inside its bounded scroll region without horizontal overflow. Do not add new
  breakpoints.

## Component and token mapping

| UI responsibility | Owner | Decision |
| --- | --- | --- |
| Page shell and actions | `PageCard`, existing role route | Reuse |
| List feedback and loading states | `DataState` in route scope | Reuse |
| Permission states | `DataState` in compact mode | Reuse locally |
| Dialog and footer | existing `Modal` primitives | Reuse |
| Permission selection | Ant Design `Input`, `Checkbox`, and `Tree` | Reuse |
| Status and feedback color | current semantic theme tokens | Reuse |

Do not add a store, shared hook, new component variant, token, or API module.
Keep role-specific state composition local until another real consumer proves
reuse.

## State and interaction contract

### Role list deletion state

| Role state | Assigned users column | Delete affordance |
| --- | --- | --- |
| Built-in (`owner`, `admin`, `viewer`) | Backend count is shown. | No edit or delete action is rendered. |
| Custom, `assignedUserCount = 0`, `deletable = true` | Shows `0`. | Existing delete confirmation remains available. |
| Custom, `assignedUserCount > 0` | Shows the positive count. | Delete button is disabled; hover, focus, `title`, and accessible label explain in Chinese and English that all assignments must be removed first. |
| Custom, backend `deletable = false` | Shows the returned count. | Delete button is disabled with a localized unavailable-state explanation. |

The list projection is advisory for the interaction. The backend delete
transaction rechecks `user_roles` and remains authoritative for races.

| Permission state | Presentation | Primary action |
| --- | --- | --- |
| Loading | Compact `Loading permissions` status with localized copy | Disabled |
| Error or permission denial | Destructive status, guidance, and localized `Reload` action | Disabled |
| Successful empty | Compact `No permissions available to assign` status with localized copy | Disabled |
| Populated, none selected | Search and permission choices | Disabled |
| Populated, selected | Search and selected count | Enabled when other fields are valid |
| Submitting | Current form retained; button shows creating or saving state | Disabled |

- Opening the dialog starts permission retrieval.
- Retry repeats only permission retrieval and preserves name, code, status,
  description, selected permission IDs, and the open dialog.
- Background refresh failure keeps the last successful permission data visible.
- Permission search filters visible choices without removing selected IDs.
- A no-match search result is distinct from an empty permission catalog.
- Closing and reopening the dialog restores values from the current role record
  through the existing form lifecycle.

## Accessibility and responsive behavior

- Loading and empty feedback use status semantics; retrieval failure uses alert
  semantics through `DataState`.
- Retry is a keyboard-reachable `type="button"` action and uses the existing
  visible focus treatment.
- Permission tree checkboxes retain Ant Design's label association and target size.
- The dialog owns focus trapping and restoration. Retry does not move focus out
  of the dialog or reset the form.
- Existing reduced-motion behavior remains authoritative; no new animation is
  introduced.
- Fixed copy uses the existing localization helper with Simplified Chinese and
  English variants defined in source.

## Evaluation gates

- The list layout keeps the new Assigned users column immediately after
  Permissions at the standard desktop and narrow table widths.
- The assigned-user state matrix reaches zero-count confirmation, positive-count
  disabled guidance, and built-in action omission from deterministic role rows.
- The disabled action exposes the bilingual reason to a keyboard-focusable
  wrapper and the control's accessible label/title; it never opens confirmation.
- List loading, error, empty, populated, and retry remain distinct.
- Permission loading, error, retry, empty, no-match, populated, and submitting
  states are reachable from deterministic query/form inputs.
- Retrieval failure never appears as an empty permission catalog.
- Retry preserves every role field and selected permission ID.
- Submit remains disabled until permission data is ready and at least one
  permission is selected.
- No new horizontal overflow appears at the repository-standard 1920x1080
  viewport or existing narrow-dialog breakpoint.
- Browser evidence covers loading, populated data, four consecutive forced 503
  responses, retry recovery with preserved fields, successful empty data,
  disabled submission, console/page errors, and 1920x1080 plus 390x667 overflow.

## Ready for dev-frontend role-definition-management

The selected source, layout ownership, component mapping, required states,
interaction, responsive/accessibility rules, copy, and UI acceptance checks are
fixed. Shared-system changes: **None**. Structured artifacts: **Skipped**.
