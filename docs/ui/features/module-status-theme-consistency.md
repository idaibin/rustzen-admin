# Module status theme consistency

## Basis and scope

- Preserve the existing module table, labels, state precedence and diagnostic tooltips.
- Source basis: `c35af40741184430e00d86110a61e5129c4425bf`.
- Existing authorities: root `DESIGN.md` Colors; `apps/web/AGENTS.md` status-tag rules;
  `apps/web/src/routes/system/module.tsx` owns module health presentation.
- Shared status meaning, geometry, behavior and assets stay unchanged. The light warning
  value is minimally darkened from #A86108 to #A76008 to clear 4.5:1 against its
  existing 5% surface; DESIGN frontmatter and runtime CSS stay synchronized.
- The exact DESIGN hash has current user approval recorded in
  `../approvals/theme-palette-20261003.md`. This approval does not supply runtime
  evidence, and any later DESIGN byte change requires fresh approval.

## Acceptance

- M01: disabled module health stays neutral even if stale availability is present.
- M02: available health uses Ant Design `success`; compatible but unavailable health
  uses `warning`; incompatible and not-ready health use `error`.
- M03: enabled/disabled tags use `success`/`default` consistently with existing status
  owners. State copy, tooltip focusability and precedence remain unchanged.
- M04: both themes consume the existing shared semantic text and surface values, with
  readable text (4.5:1 minimum contrast) and retained diagnostic focus behavior.
- M05: regression checks cover all five health branches and both theme palette mappings.

## Runtime boundary

Use the real module component with synthetic, read-only state fixtures, independently
from backend conformance. Inspect light/dark at 1920x1080 and 1440x900 (root DESIGN
baseline), theme toggle/repeat, diagnostic keyboard focus, and overflow. If a browser
cannot reach or configure the target, record those entries Not verified rather than
substituting source/tests for runtime evidence.
