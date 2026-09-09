# Admin maintenance task console UI

## Basis and composition

- Product basis: [Admin maintenance task console](../../product/features/admin-maintenance-tasks/spec.md).
- Shared visual and responsive authority: root [DESIGN.md](../../../DESIGN.md).
- Route owner: `apps/web/src/routes/manage/task.tsx`; request owner:
  `apps/web/src/api/manage/task/`.

`/manage/task` is a compact operational table. Each fixed built-in has name,
description, immutable cron, status, next run, last completion, error, records
and (when `manage:task:run` is granted) a manual-run action. Stable test IDs
identify the table, each status, run-record trigger and confirmation dialog.

The records modal is read-only and pages recent runs. It shows trigger,
running/success/failed/skipped state, schedule/start/finish times, and error.
The confirmation modal describes the selected built-in before the operator
submits it; it never offers a cron editor or enable switch.

## States and responsive behavior

Loading renders the page title and a loading state. A successful empty list is
an empty state. A request failure is an error state with Reload and must not
appear as empty. The task list refreshes only while any task runs; an open
records modal refreshes only while a visible run is active.

At narrow widths the table container owns horizontal scrolling. Rows keep their
status and action controls reachable, modals use the existing responsive width,
and the document must not gain horizontal overflow. List-only users retain the
records action and never receive the run action.

## Local acceptance

The `verify-task-console-linux` Chromium gate exercises owner and list-only
sessions on a fresh database: three rows, records modal, a harmless successful
run, list-only POST 403, empty/error state fixtures, and 1440x900 plus 390x844
screenshots. It publishes source, binary, browser and API receipts atomically.
It is local container evidence only; native-host and deployment validation are
`Not verified`.
