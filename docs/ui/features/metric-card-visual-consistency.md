# Operational Metric Card Visual Consistency UI

Status: **Implemented and verified** for the retained 1920x1080 and 1440x900
runtime evidence. The remaining viewport-matrix entries are not re-asserted by
this document.

## Profile, authority, and selected source

- Profile: **Feature UI + shared component contract**.
- Product basis:
  `docs/product/features/metric-card-visual-consistency/spec.md`.
- Shared visual authority: root `DESIGN.md`, especially
  `#component-semantics`, `#themes-and-surfaces`, and `#status-semantics`.
- Selected source identity: 2026-07-31 user-provided reference image; private
  evidence, redistribution restricted.
- Revision: the image attached to the 2026-07-31 request asking whether the
  current implementation matches the selected UI.
- Source approval: approved by Daibin on 2026-07-31 as the implementation
  direction. Daibin confirmed the exact adaptation values and palette mapping
  on 2026-07-31 with the instruction to begin execution.
- Rights/use: user-supplied local reference, allowed for this repository's
  implementation; redistribution outside this task is not authorized.
- Use: four-card anatomy, hierarchy, neutral identity zone, tinted value zone,
  semantic per-card color, prominent number, icon/title pairing, and restrained
  border/radius treatment.
- Ignore: the image canvas size as a browser viewport, literal raster pixels as
  CSS tokens, any unverified font identity, image compression/blur, and shadows
  caused by raster scaling.
- Source canvas: 2556x520 at 100% image zoom; it is a component-group crop, not
  a verified page viewport.
- Selected state: populated, Simplified Chinese, light theme, four account
  metrics, no supporting copy, no hover/focus state.

The selected image is visually inspectable but has no design-tool metadata.
Source proportions are therefore `visually-inferred`; exact CSS values below
are the approved repository adaptation, not falsely relabeled source values.

## Shared component decision

`MetricCard` remains the single owner at
`apps/web/src/components/page/metric-card.tsx`. Dashboard, Monitoring, and
Analytics routes reuse it and own only grid columns, grid gaps, metric order,
localized title, value, icon selection, and semantic tone.

The shared component contract is:

- two stacked zones with an approximately 50/50 visual split;
- neutral identity zone with icon tile and title on one row;
- lightly tinted value zone separated by a semantic divider;
- number is the sole dominant element in the value zone;
- optional supporting copy is supported but absent from the current 16 metrics;
- component tones are `blue`, `green`, `violet`, `amber`, and `red` and use
  component-level palette tokens owned by the shared theme stylesheet,
  never literal route colors;
- no hover lift or click affordance because current cards are informational.

## Dashboard composition

The Dashboard keeps one page header followed by two panel-level Ant Design
`Card` containers for every user, plus one permission-gated panel:

1. account overview, containing its title, optional explanatory copy, query
   feedback, and the four shared `MetricCard` instances;
2. system running summary, visible only with `system:status:view`, containing
   CPU, memory, and disk `MetricCard` instances sourced from the existing
   System Status endpoint;
3. runtime modules, containing the three module connectivity rows.

The account-overview panel must appear before the system summary and runtime
modules. Runtime polling
continues every 15 seconds, but the interval is an implementation detail and
must not be repeated as visible panel copy.

## Target geometry and typography

The following values are the approved target adaptation for implementation and
runtime verification:

| Property | Target |
| --- | --- |
| Card height | `176px` desktop; content may grow for wrapping/localization |
| Card radius | `12px` |
| Card border | `1px`, shared border anchor |
| Card shadow | restrained panel shadow only; no glow |
| Zone split | `88px` identity / minimum `88px` value at desktop |
| Horizontal padding | `24px` in both zones |
| Icon tile | `48px` square, `12px` radius |
| Icon glyph | `26px`, centered |
| Icon/title gap | `16px` |
| Title | `18px`, weight `600`, shared foreground |
| Value | `56px`, weight `700`, line-height `1`, tabular numerals |
| Grid gap | route-owned `16px`; no gap embedded in the card |

The target intentionally increases hierarchy from the current 130px card
without copying the source crop's apparent 1.52 aspect ratio into every grid.
This keeps the five-column Monitoring layout operational at desktop widths while
making the value materially more prominent.

## Semantic tone and icon mapping

Color is scoped within each card: icon glyph, icon-tile tint, divider, numeric
value, and value-zone tint share one semantic tone. The rest of the card stays
on the neutral panel surface. Tints must remain subtle in both themes; do not
tint the whole card.

`apps/web/src/styles/theme.css` owns each `MetricCard` tone's foreground, soft
surface, and divider values for light and dark themes. The final composed hues
must remain visibly distinct from one another. A neutral/near-white dark-theme
value is not an acceptable substitute for the blue tone. Routes consume only
the tone enum and never define color values.

| Metric meaning | Tone | Icon rule |
| --- | --- | --- |
| totals, inventory, default volume | `blue` | closest existing group/inventory icon |
| active, healthy, completed | `green` | closest existing active/healthy icon |
| recent, visitor, informational activity | `violet` | closest existing clock/information icon |
| pending, offline, requests needing attention | `amber` | closest existing pending/attention icon |
| unhealthy, failed, critical | `red` | closest existing alert/error icon |

The current 16 metrics map as follows:

| Surface | Metric | Tone |
| --- | --- | --- |
| Dashboard | 用户总数 / Total users | `blue` |
| Dashboard | 活跃用户 / Active users | `green` |
| Dashboard | 今日登录 / Today's logins | `violet` |
| Dashboard | 待审核用户 / Pending users | `amber` |
| Dashboard | CPU | `blue` |
| Dashboard | 内存 / Memory | `green` |
| Dashboard | 磁盘 / Disk | `amber` |
| Monitoring | 已注册节点 / Registered nodes | `blue` |
| Monitoring | 在线节点 / Online nodes | `green` |
| Monitoring | 离线节点 / Offline nodes | `amber` |
| Monitoring | 异常检查 / Unhealthy checks | `red` |
| Monitoring | 活动事件 / Active incidents | `violet` |
| Analytics | 页面浏览量 / Page views | `blue` |
| Analytics | 独立访客 / Unique visitors | `violet` |
| Analytics | 全部事件 / Total events | `green` |
| Analytics | 接口请求 / API requests | `amber` |

Use existing `@ant-design/icons` assets already owned by the routes. Prefer a
filled icon only where the package provides a semantically equivalent asset;
otherwise retain the correct outlined asset and make it legible through the
48px tile and 26px glyph. Do not import a second icon system or substitute one
generic icon for every metric.

System resource cards display the percentage as the dominant value. CPU uses
core count as optional supporting copy; memory and disk use current used/total
capacity as optional supporting copy. They do not add progress bars or detailed
storage breakdowns.

## States, content, and accessibility

- Values use tabular numerals and remain text in the accessibility tree.
- Title and value must not rely on color to convey their meaning. Health facts
  that need status text remain outside this purely numeric component.
- One-line titles truncate only when the grid cell cannot accommodate the
  localized label; at narrow widths, prefer wrapping to hiding the value.
- Optional supporting copy, when present in a future approved consumer, sits
  below the title and must not reduce the value below the target hierarchy.
- The card is not interactive and receives no tab stop. Any future click action
  requires a separate product/UI slice with focus, hover, and target-size rules.
- Foreground/icon/value contrast must meet WCAG AA against their final composed
  surfaces. Semantic tint is never the only status indicator.
- Existing route `DataState` and refresh notices retain loading, error, empty,
  retry, and background-refresh ownership.

## Responsive contract and viewport matrix

Parent grids retain current ordering and reflow; `MetricCard` must not set grid
columns. No affected page may gain document-level horizontal overflow.

| Priority | Viewport | Theme / locale | Surface and required state | Acceptance |
| --- | --- | --- | --- | --- |
| Required | 1920x1080 CSS px @ 100% | light / zh-CN | Dashboard populated | four cards, equal height, 176px target, no clipping/overflow; viewport capture file must be exactly 1920x1080 raster px |
| Required | 1440x900 @ 100% | dark / zh-CN | Monitoring populated | five cards remain legible; semantic tones remain distinct |
| Required | 1440x900 @ 100% | light / en-US | Analytics populated | labels and values preserve hierarchy without overlap |
| Required | 390x844 @ 100% | light / zh-CN | Dashboard populated | one-column stacking, natural content growth, no horizontal overflow |
| Required | 390x844 @ 100% | dark / en-US | representative overview | wrapping/truncation and contrast remain acceptable |

### Dashboard screenshot artifact contract

The required Dashboard evidence is a viewport screenshot, not a full-page
capture. It must satisfy all of the following:

- set the browser CSS viewport to exactly `1920x1080` and keep page zoom at
  `100%`;
- capture only the current viewport; `fullPage` capture is prohibited for this
  acceptance item even when the page currently fits within one viewport;
- export a raster file whose intrinsic `pixelWidth` and `pixelHeight` are
  exactly `1920` and `1080`;
- do not crop, scale, stretch, resample, or otherwise post-process a different
  capture into the required dimensions;
- record both the runtime `window.innerWidth` / `window.innerHeight` values and
  the exported file's intrinsic pixel dimensions with the screenshot evidence.

If either the CSS viewport or intrinsic raster dimensions differ from
`1920x1080`, or if page zoom is not proven to be `100%`, the required Dashboard
viewport entry is `Not verified` and must not be reported as passed.

## Traceable delta table

| ID | Area | Selected source | Current runtime | Target contract | Priority | Owner and validation |
| --- | --- | --- | --- | --- | --- | --- |
| MC-001 | Anatomy | visually inferred neutral header plus separately tinted lower value region | browser-computed 130px card with 64px minimum zones | one shared 176px two-zone card, 88px/88px target | P1 | `MetricCard`; compare DOM rectangles at required desktop viewports |
| MC-002 | Numeric hierarchy | visually inferred large number as dominant content | browser-computed 44px value | 56px/700/1 tabular value, visually dominant over 18px title | P1 | `MetricCard`; computed type and side-by-side review |
| MC-003 | Icon/title | visually inferred prominent semantic tile paired with title | browser-computed 40px tile, 20px glyph, 14px title | 48px tile, 26px glyph, 18px/600 title, 16px gap | P1 | shared component plus route icon mapping; computed geometry and asset review |
| MC-004 | Per-card color | distinct blue, green, violet, and amber meanings; tint is local to icon/value areas | current light Dashboard has four distinct hue families, but proportion, saturation, and lightness differ; other themes/consumers are not verified | shared theme owns five component-level foreground/surface/divider values per theme; each of the 16 metrics follows the approved mapping above; neutral identity surface and local tint only | P1 | `DESIGN.md`, `theme.css`, `MetricCard`, route data; final composed color and dark-theme review |
| MC-005 | Reuse | repeated four-card grammar | current shared component has Dashboard, Monitoring, Analytics consumers | all 16 metrics retain one component; no route-local clone or palette | P1 | source scan plus affected-route runtime review |
| MC-006 | Responsive/accessibility | source crop proves desktop four-card direction only | narrow, dark, English, contrast and focus behavior not proven by source crop | pass full viewport matrix, AA contrast, no overflow, non-interactive cards excluded from tab order | P1 | browser evidence at every required viewport/state |

## Implementation and evidence gates

Before editing source, `dev-frontend` must map every `MC-*` item to the shared
component, route consumer, theme/asset owner, and verification method in the
`frontend-visual-evidence/v1` artifact. Implementation order is anatomy,
typography, icon mapping, color composition, then polish.

After editing, completion requires two same-viewport/state comparison passes,
computed geometry and final composed colors, light/dark and Chinese/English
coverage, the required viewport matrix, and no unresolved P0/P1 finding. Static
checks or a screenshot alone do not prove completion.

For the required Dashboard capture, each comparison pass must use the viewport
screenshot contract above and retain the unmodified `1920x1080` raster artifact.
File dimensions alone do not prove the viewport requirement; the matching
runtime viewport and `100%` zoom evidence are required in the same pass.

## Ready for dev-frontend metric-card-visual-consistency

Selected source identity, approval, allowed use, evidence limits, shared-system
ownership, target adaptation, semantic mapping, responsive/accessibility rules,
delta IDs, and acceptance gates are fixed and approved. Blockers: **None**.
Runtime fidelity remains a downstream implementation and two-pass evidence
obligation.
