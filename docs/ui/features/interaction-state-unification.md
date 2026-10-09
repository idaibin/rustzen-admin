# Interaction state verification contract

## Basis

Preserve root `DESIGN.md`, approved theme geometry, Ant Design component variants,
route ownership and the [product acceptance](../../product/features/interaction-state-unification/spec.md).
Baseline source: `42d6cb104a68a484685f6e6d9372b1d8e8af9b97`.
The user requested consistent appropriate interaction feedback; this is an implementation
and verification slice, not a new shared design system or a replacement visual source.

## State and input matrix

| Family | Applicable states | Inputs / observable outcome |
| --- | --- | --- |
| Primary/default/text/link/danger/icon buttons | rest, hover, held press, focus-visible, disabled, pending | pointer, Enter, Space; exactly one appropriate dispatch; variant identity retained |
| Links / shell brand | rest, hover, press, focus-visible, navigation | pointer/Enter; real destination; no artificial Space activation |
| Inputs / select / checkbox / switch / tree | focus, selected/checked, expanded, invalid, disabled/pending | Tab/ShiftTab, typing/IME, arrows, Enter/Space/Escape as supported; accessible labels |
| Menu / account / row actions | closed/open, hover, focus, selected/disabled | Enter/Space/ArrowDown opens trigger; arrows navigate; Enter selects; Escape restores; the opening key must not activate a new dialog button |
| Search | closed/open, input, active result, empty | Ctrl/Meta+K, arrows, Enter, Escape; active descendant exists and remains visible; IME Enter does not navigate |
| Modal / Drawer | opening, active, pending, failure, closing | tab containment, dismissal policy, surviving-trigger return and removed-trigger fallback |
| Table / pagination | row action, selection, current page, overflow | keyboard library behavior; current page distinct from focus; overflowing region remains keyboard-reachable |
| Notifications | unread/read, open detail, retry, related destination | real Ant Design row button feedback, focus-to-detail and return, pending writes shared across alternate paths |

Selection is persistent state; hover, held press and keyboard focus are independent
feedback. Do not make all variants look identical or apply page-wide pseudo-element
control styling. Ant Design tokens own library states; the shell owns its brand link.
Text in default/hover/active states must meet 4.5:1 against its actual surface. Focus
boundaries use a separate non-text target of 3:1 against adjacent surfaces. Disabled
text is recorded separately and is not treated as enabled text contrast evidence.

## Runtime frames and evidence

- 1920×1080 canonical and 1440×900 compatibility, light and dark.
- 390×844 narrow-screen check, both themes; controls must remain reachable without
  document-level horizontal overflow.
- Capture actual rendered state and computed foreground/background/outline values,
  including held pointer press and keyboard-generated focus-visible, not just screenshots
  of rest state or source declarations.
- Use a frozen source snapshot for each browser run. Record mock boundaries, actual
  dispatched callback/request counts, failures, retries, source hashes and screenshots.
- Shared-owner passes establish inherited control behavior only. A route is not marked
  fully tested merely because its Button/Modal owner passed an isolated fixture.
- Native file pickers, backend persistence, report automation, live service restart,
  actual exports and destructive live effects remain separate acceptance layers.

The finite route/control evidence ledger is maintained in
[interaction-state tasks](../../plans/interaction-state-tasks.md).
