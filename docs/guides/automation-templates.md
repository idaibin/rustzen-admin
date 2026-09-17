# Rustzen Reports browser automation task guide and templates

This guide describes the current `apps/reports` automation implementation. Browser automation is executed by Reports; external calls enter through the Admin Gateway's protected API.

## Execution and evidence boundary

Each run uses an isolated Chromium profile, and step results are written to the run's step records. During a run one PNG live frame can be refreshed; explicit screenshots and failure snapshots are also retained as PNG artifacts. The current implementation has no WebM, MP4, frame-sequence recording, video composition, or video playback artifact.

Template selectors, URLs, and inputs must come from the controlled target system and the current page contract. Run inputs can be used for the supported `{{input.name}}` substitution; do not submit fields in the form of passwords, tokens, or other credentials.

## FlowStep contract

The following 17 `action`s are the complete current set of `FlowStep` in `apps/reports/src/features/automation/types.rs`. Unknown actions or fields are rejected.

| `action` | Fields | Purpose |
| --- | --- | --- |
| `goto` | `url` | Navigate to a relative or absolute URL; the resolved target must be same-origin with the target system's `baseUrl`. |
| `fill` | `selector`, `value` | Fill an element value. |
| `click` | `selector` | Click an element. |
| `waitFor` | `selector` | Only wait for a matching element to appear in the DOM; visibility, text matching, or page readiness are not guaranteed. |
| `assertText` | `selector`, `text` | Assert the element text. |
| `assertValue` | `selector`, `value` | Assert the element's native value. |
| `assertAbsent` | `selector` | Assert the selector has no match. |
| `screenshot` | `name` (optional) | Save a full-page PNG screenshot artifact. |
| `screenshotViewport` | `name` (optional) | Save the current viewport as a PNG screenshot artifact. |
| `setViewport` | `width`, `height` | Set the browser viewport; only `1440x900` or `390x844` are allowed. |
| `setUiPreferences` | `theme`, `locale` | Set theme and language preferences; `theme` is only `light` or `dark`, and `locale` is only `zh-CN` or `en-US`. |
| `assertNoHorizontalOverflow` | none | Assert the document produces no horizontal overflow. |
| `assertElementLayout` | CSS `selector`, optional `elementCount`, `visibleCount`, `maxHeight`, `withinViewportRight`, `withinViewport` | Verify element count, visibility, height, and viewport bounds; at least one condition must be provided. |
| `assertFocus` | `selector` | Assert the element matched by the selector receives focus. |
| `guardExists` | `selector`, optional `onMissing` | When the element does not exist, handle it as `continue`, `skipNext`, `stop`, `fail`, or `error`; `error` is an accepted alias of the failure strategy. |
| `pressKey` | `key` | Send a keyboard key. |
| `pause` | `durationMs` | Pause for 0 to 30000 (inclusive) milliseconds; the stored value executes for the same duration, and `0` means no additional wait. |

## Template example

### Search result screenshot

```json
[
  { "action": "goto", "url": "/" },
  { "action": "waitFor", "selector": "#kw" },
  { "action": "fill", "selector": "#kw", "value": "{{input.keyword}}" },
  { "action": "click", "selector": "#su" },
  { "action": "waitFor", "selector": "#content_left" },
  { "action": "screenshot", "name": "search-result" }
]
```

The target system's `baseUrl` is, for example, `https://www.baidu.com`; a run input is, for example:

```json
{ "keyword": "Rustzen" }
```

## Admin Gateway API example

The Admin Gateway listens on `http://127.0.0.1:9801` by default. The following examples require the Bearer token of a user with the corresponding Reports capability.

```bash
export RUSTZEN_ADMIN_TOKEN='<admin bearer token>'
base=http://127.0.0.1:9801

# Register a target system
curl -fsS -X POST "$base/api/reports/systems" \
  -H "Authorization: Bearer $RUSTZEN_ADMIN_TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"name":"Baidu search","baseUrl":"https://www.baidu.com","enabled":true}'

# Create a flow
curl -fsS -X POST "$base/api/reports/flows" \
  -H "Authorization: Bearer $RUSTZEN_ADMIN_TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"systemId":"<SYSTEM_ID>","name":"Keyword search","steps":[{"action":"goto","url":"/"},{"action":"waitFor","selector":"#kw"},{"action":"screenshot","name":"result"}]}'

# Create a run and read the run and its artifacts
curl -fsS -X POST "$base/api/reports/runs" \
  -H "Authorization: Bearer $RUSTZEN_ADMIN_TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"flowId":"<FLOW_ID>","input":{"keyword":"Rustzen"}}'
curl -fsS -H "Authorization: Bearer $RUSTZEN_ADMIN_TOKEN" "$base/api/reports/runs/<RUN_ID>"
curl -fsS -H "Authorization: Bearer $RUSTZEN_ADMIN_TOKEN" "$base/api/reports/runs/<RUN_ID>/artifacts"
```
