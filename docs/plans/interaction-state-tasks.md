# Interaction-state unification tasks

Active ledger for the requested 2026-10-03 Rustzen Admin interaction pass.
Baseline: `42d6cb104a68a484685f6e6d9372b1d8e8af9b97` on
`feat/theme-palette-20261003`. No prior 19-route/theme evidence is counted here.
Product authority: [acceptance](../product/features/interaction-state-unification/spec.md).
UI authority: [state/input contract](../ui/features/interaction-state-unification.md).
Root DESIGN bytes remain unchanged at SHA-256
`c401f71cbb66206b4166e85a3339b027ff0183efd44e7e0ce74280b4425f93cd`.

## Execution ledger

| ID | Outcome / scope | Owner | Dependencies | Acceptance and evidence | Status / next action |
| --- | --- | --- | --- | --- | --- |
| IN-01 | Finite route/control inventory and interaction contract | product-spec / ui-spec | Baseline and user-requested scope | Existing route/function ownership, applicable state matrix, explicit non-goals | Complete; preserved below |
| IN-02 | Shared Button/link/focus tokens; ConfirmModal, search and menu behavior | dev-frontend | IN-01 | Variant states, disabled/pending/error and repeat handling, IME and menu-to-modal dispatch | Complete; pointer/keyboard/failure and final contrast checks passed |
| IN-03 | Forms, report actions, monitoring Drawers, notifications, query retries | dev-frontend | IN-01, IN-02 | Handler locks, accessible labels, correct dismissal ownership, retry recovery and visible query refresh | Complete within the finite action checklist; native limits below remain explicit |
| IN-04 | Browser acceptance with owned fixtures | ops-browser | IN-02, IN-03 | Frozen source hashes; actual states, keyboard, contrast and callback counts at contract viewports | Passed (scoped); real owned API/DB flows plus independently labeled fault fixtures |
| IN-05 | Independent review and final checks | repo-review | IN-03, IN-04 | Fixed diff, no unresolved P0–P2, tests/types/lint/build, explicit gaps | Reviewed; no unresolved actionable findings at manifest `74a170e2…`; final reconciliation below is documentation-only |
| IN-06 | Local implementation checkpoint | repo-delivery | IN-02, IN-03 source gates | Exact path staging; 164 tests, types/lint/build pass; verified remote baseline; `[skip ci]` | Local checkpoint `d3cbb8a9cbcb8d56b8aaad789665c6d56511d117`; no remote publication claimed |
| IN-07 | Final branch publication | repo-delivery | IN-04, IN-05 | Reviewed immutable basis and explicit remote readback; no main merge/deployment | Ready for branch-only delivery after this documented coverage; remote SHA must be verified separately |

Ready frontier: IN-07 only. Actual API/SQLite outcomes and synthetic fault fixtures
remain separately identified. A commit containing this ledger is not itself proof of
remote publication. Main merge and native host deployment remain excluded.

## Finite source inventory

Every row below was source-audited in this pass. “Inherited” means the route consumes
an actually exercised shared/library owner; it does not mean every page-specific event
or backend effect was executed. Final direct-action coverage is recorded below; other states remain shared-owner
inheritance rather than independent route-flow acceptance.

| Surface | Interactive families / actual route owners | Current evidence boundary |
| --- | --- | --- |
| `/` | dashboard error/background retry | Real responsive/theme render; shared Button/refresh inheritance |
| `/profile` | edit-profile and password Forms/Modals, avatar Upload | Real profile save/failure/retry; password component fixture; native picker/avatar persistence excluded |
| `/system/user` | search/status/pagination; create/edit/role choices; dropdown confirmations | Real create/edit/status/reset/delete, filters and keyboard dropdown; error guard inheritance |
| `/system/role` | three filters/pagination; role Form; permission search/tree/select-all; delete | Real create/edit, assigned-delete protection, page 2 keyboard, deletion/H1 restoration |
| `/system/menu` | three filters; module-menu Form | Real menu edit/native submit and API readback |
| `/system/module` | enable confirmation; focusable diagnostic Tags; retry | Real owned Insights off/on confirmation and API readback; diagnostic tooltip inheritance |
| `/system/status` | loading/error retry | Real responsive/theme render; shared Button inheritance |
| `/system/module-log` | module/date filters, row selection, backup, cleanup preview/confirm, tail Drawer/load older | Real owned log tail, selected archive download, cleanup preview/deletion; confirmed archive bytes |
| `/manage/log` | filters/pagination/export/retry | Real filters/render and audit CSV download; shared pagination inheritance |
| `/manage/task` | run-history Modal/pagination, manual-run confirmation/retry | Real history and safe owned retention run reaching terminal success |
| `/manage/deploy` | upload/expire forms, deploy/delete/cleanup confirmations/pagination/retry | Real empty/blocked route; source/shared form inheritance; upload picker, release application and host effects not verified |
| `/monitoring/overview` | retry/background refresh | Real responsive/theme render; shared control inheritance |
| `/monitoring/nodes` | refresh, onboarding/global Drawer, node details/policy save/reset/retry | Real node/global policy submit, repeated input, failure/retry/API persistence and settled opener restoration; onboarding intentionally unavailable |
| `/monitoring/incidents` | status/severity filters, pagination/details Drawer/retry | Real responsive/theme render; shared control inheritance |
| `/monitoring/summaries` | pagination/retry/background refresh | Real responsive/theme render; shared control inheritance |
| `/analytics/overview` | retry/background refresh; chart tooltip | Real responsive/theme render plus final API-created Analytics metrics/readback |
| `/analytics/details` | event/path filters, pagination/retry/background refresh | Real responsive/theme render plus final genuine event/path readback |
| `/reports/templates` | target/template forms, clone/delete, schedule form/switch/delete/link | Real target/template/schedule creation, clone, toggle/edit/delete, latest abort/reopen/retry and API persistence |
| `/reports/runs` | run form, view/retry/cancel, detail Modal/download/live retry/page | Real UI-created run, renderer success, cancellation/retry success, PNG download and latest failure feedback |
| `/login` | username/password Form, theme/language, submit | Real native login; invalid input/network feedback and retry; shared theme/language controls |
| `/403`, `/404` | history/home actions | Actual status titles/render; home/back Button inheritance |
| Shared shell | brand link, route Menu, page search, mobile Drawer, theme/language/account | Real desktop Menu, search navigation, account/profile, mobile Drawer/navigation, guarded logout; brand/theme shared-state coverage |
| Message center | bell, Drawer, filter, mark-read/all/retry, load-more, row/detail/related navigation | Real read/all/retry, detail/related navigation, bell restoration and both missing-record focus paths |

Redirect-only `/analytics`, `/monitoring`, `/reports` expose no independent controls.
Selected portal builds inherit touched theme/shared files; their separate portal shell
is source-reviewed but is not counted as full Admin-shell runtime coverage.

## Reproducible checks and evidence limits

Run Web checks from `apps/web`: `bun test`, `bun x tsc --noEmit`, `bun run vp lint`,
`bun run vp build`; inspect the matching root `justfile` targets first. Runtime fixtures
and frozen screenshots/results are task evidence under ignored
`.codex/artifacts/interaction-unification/`. Individual receipts distinguish real
API/DB outcomes from mocks; neither is proof of production deployment.
Each runtime receipt identifies a source snapshot, mocked API/callback boundaries,
frames and real assertions. Do not commit browser caches/screenshots or credentials.


## Accepted evidence and qualifications

- Independent review: baseline `42d6cb104a68a484685f6e6d9372b1d8e8af9b97`,
  manifest `74a170e2d16818ffd5b2f35d25f4871b0c8b73a16ac22f2d3b220fade4475e33`,
  binary diff `a2e17e9db7a567f782d248d9af828394948d203a7a703fed7527f285b4b2d84c`.
  All 68 paths were verified; no unresolved actionable findings remain. Final ledger
  and UI-index status reconciliation is a documentation-only delta after that review.
- Final Web gate: 167 tests pass, zero fail, 916 assertions; TypeScript, build,
  formatting and lint pass. Existing generated/test lint warnings and the existing
  large-chunk build warning remain; neither is represented as a new zero-warning gate.
- Four actual development services ran with fresh owned SQLite databases. Real API
  operations created users/roles, agent reports/incidents, Analytics events, report
  templates/schedules/runs and durable notifications. A real report renderer produced
  a downloaded PNG; module-log backup and CSV downloads were also checked.
- The 19-route/114-view matrix covers three viewports and both themes with zero
  document overflow. It is an earlier resting-build matrix, not 114 complete workflows
  or a full recapture of the final bundle. Later changed shared states were verified
  against HTTP-served bundle hashes in bounded final runs.
- Final corrected text measurements: outlined active text 7.73:1 light / 6.44:1 dark;
  selected-menu text 5.69:1 / 5.51:1. The owned table-scroll focus boundary measures
  4.64–4.97:1 light and 4.59–5.38:1 dark against adjacent header/body surfaces. Its
  outline is visible on the scrollable region; sticky actions may cover the right-hand
  continuation, so an independently verified complete rectangular perimeter is not claimed.
- Final table focus/keyboard checks: 13/13. Final genuine Analytics read/UI checks:
  16/16, including actual API-created events and populated metrics in both themes.
- The first `/tmp` Analytics fixture failed closed because its mount was not exposed
  by the application's disk-discovery dependency. Only new fixture databases moved
  to a recognized workspace mount; no storage guard or threshold changed. Final
  ingestion accepted three real events. Earlier synthetic Analytics views are retained
  as diagnostic evidence only; final genuine Analytics checks supersede them.
- One daily-summary row remains an explicit synthetic SQL fixture; the hourly summary
  generator was not accelerated or counted as verified. Native file pickers, signed
  release installation, host-service restart and production deployment remain unverified.
- Headless forward Tab can leave document focus for browser chrome. The same behavior
  reproduces with native HTML modal dialogs, and no background app control receives
  focus. Do not claim unrestricted modal containment or all assistive-technology behavior.
  An apparent node-Drawer restoration failure disappeared when the original source was
  tested after its close animation settled; its speculative source workaround was removed.
- Selected portal builds receive source/shared-owner review only, not separate full
  runtime acceptance. No all-combinations claim is made for every permission, locale,
  concurrent actor, browser or background refresh.

The consolidated task-local report and source/bundle receipts are in
`.codex/artifacts/interaction-unification/route-runtime/README.md`,
`verification-summary.json`, and `final-source-sha256.json`. Raw failed/partial runs
are retained without rewriting history; the consolidated checklist names the later
checks that supersede harness timing/selector diagnostics. All owned browser/service
processes were stopped. No fixture databases, screenshots, caches or runtime keys are
part of the source commits.
