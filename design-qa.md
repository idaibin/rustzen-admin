# Login Copy Layout Design QA

Status: historical task evidence only. This file does not define the current
shared design system, selected visual source, viewport contract, or acceptance
status. Root [`DESIGN.md`](./DESIGN.md) is the sole shared visual-semantic
authority; current runtime claims require a new task-local evidence package.

## Comparison Target

- Source visual truth: `/Users/daibin/.codex/generated_images/019f9718-f2a4-7950-bcaf-75a08fe2d80f/exec-222d0ff8-b676-42fe-9016-26abea69388b.png`
- Implementation screenshot: `.codex/artifacts/2026-07-26-login-copy-layout/implementation-refined-1920x1080.png`
- Full-view comparison: `.codex/artifacts/2026-07-26-login-copy-layout/comparison-final-full.png`
- Focused caption comparison: `.codex/artifacts/2026-07-26-login-copy-layout/comparison-final-caption.png`
- State: `/login`, Simplified Chinese, light theme, empty form, desktop browser.
- Source pixels: `1672 x 941`; normalized to `1920 x 1080` for comparison.
- Implementation pixels and CSS viewport: `1920 x 1080`; device density `1`.

## Required Fidelity Surfaces

- Fonts and typography: the implementation uses the repository font stack with a `40px/800` primary caption, `16px/600` keyword line, and `15px/400` supporting copy. The hierarchy and optical weight match the selected direction.
- Spacing and layout rhythm: the caption begins at the same left anchor as the source, uses a `48px` divider, and balances the two text groups with `40px` desktop gaps. The full-page scale remains repository-owned because the selected ImageGen mock drifted outside the requested caption-only scope.
- Colors and visual tokens: the existing navy and blue-gray palette is preserved; the divider uses the selected pale-blue treatment.
- Image quality and asset fidelity: the repository-owned login illustration is unchanged and rendered at native quality. No replacement or code-drawn asset was introduced.
- Copy and content: Chinese copy is `让运维，更从容`, `高效 · 可靠 · 智能`, and `统一运维管理平台，让管理更简单、更高效。`; matching English copy is available through the language switch.

## Comparison History

### Pass 1

- P2 typography and spacing: the initial `32px` headline and `16px` group gaps were visibly lighter and tighter than the selected visual.
- Fix: increased the headline to `40px`, divider to `48px`, desktop group gaps to `40px`, and supporting type to `16px/15px`; removed paragraph margin drift.
- Post-fix evidence: `comparison-final-caption.png` shows aligned anchors, hierarchy, divider height, and supporting-copy rhythm.

### Pass 2

- P1 responsive clipping: at `1024 x 768`, the legacy two-column breakpoint placed the card at `x=672..1092`, outside the viewport.
- Fix: enable the illustration/two-column composition at `1440px` and above, then center the card with grid alignment below that breakpoint.
- Post-fix evidence: at `1024 x 768`, the card is centered at `x=302..722`; at `390 x 844`, it fits at `x=28..362`. Neither viewport has horizontal or vertical overflow.
- P2 interaction: the language control changed its own label but the route copy did not subscribe to locale state.
- Fix: subscribe `LoginPage` to `useLocale()`.
- Post-fix evidence: all three marketing strings switch to English and back to Chinese; theme switching also works.

### Pass 3

- P1 vertical alignment: the in-flow header and standalone copyright footer changed the main content's available-height calculation, so the login card did not own the viewport center directly.
- Fix: remove the optional copyright footer, position the header independently, and center the content grid against the full `100svh` viewport. The outer desktop width was expanded from `1540px` to `1760px` so the illustration and login card use more of the horizontal canvas.
- Post-fix evidence: the card center differs from the viewport center by `0px` at `1920 x 1080`, `1024 x 768`, and `390 x 844`; copyright copy is absent and no checked viewport has horizontal overflow.

## Browser Verification

- Browser surface: Codex in-app Browser.
- Viewports checked: `1920 x 1080`, `1440 x 900`, `1024 x 768`, and `390 x 844`.
- Primary interactions: language switch and light/dark theme switch.
- Form surface: username field, password field, password visibility control, and submit button rendered; backend login submission was intentionally not executed.
- Console: no warnings or errors in the final checked state.
- Overflow: none at the checked viewports.

## Findings

No actionable P0, P1, or P2 differences remain within the selected caption-layout scope.

## Follow-up Polish

No P3 polish is required for this pass.

final result: passed

---

## Preserved Historical Dashboard QA

- Source reference: `docs/ui/reference-assets/figure-2-light-glass-dashboard.png`
- Implementation: `docs/ui/evidence/dashboard-1920x1080.png`
- Comparison: `docs/ui/evidence/dashboard-reference-comparison.png`
- Surface: authenticated web dashboard at 1920×1080

### Historical Findings

- The implementation carries the reference's pale blue/coral atmosphere, translucent panels, softened borders, and three-column dashboard rhythm into the existing product shell.
- Existing routes, modules, metrics, account state, and operational information remain factual; no reference-only legal or calendar content was copied.
- The app remains full-bleed instead of placing a mock desktop frame around the product. This is an intentional product-shell constraint, not a fidelity defect.
- No horizontal overflow or browser console errors were observed. The authenticated operations ledger also remains usable at 1600px width.

### Historical Issue History

- P1: Main dashboard content previously lacked the reference's clear primary-column/right-rail balance. Fixed by regrouping module, activity, metrics, account, and health content.
- P2: Opaque surfaces weakened the glass hierarchy. Fixed with restrained translucency, backdrop blur, pastel borders, and a generated abstract background asset.
- P2: Search was visually undersized for the reference's command-bar role. Fixed at desktop widths.

### Historical Final Result

passed
