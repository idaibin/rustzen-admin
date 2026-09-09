# Rustzen Reports 浏览器自动化任务指南与模板

本指南描述当前 `apps/reports` 自动化实现。浏览器自动化由 Reports 执行；外部调用通过 Admin Gateway 的受保护 API 进入。

## 执行与存证边界

每个运行使用独立 Chromium profile，步骤结果写入运行步骤记录。运行期间可刷新一张 PNG live frame；显式截图和失败快照也以 PNG artifact 留存。当前实现没有 WebM、MP4、帧序列录制、视频合成或视频回放 artifact。

模板的 selector、URL 与输入均应来自受控的目标系统和当前页面契约。运行输入可用于已支持的 `{{input.name}}` 替换；不要提交密码、令牌或其他凭据形态的字段。

## FlowStep 契约

以下 17 个 `action` 是 `apps/reports/src/features/automation/types.rs` 中 `FlowStep` 的完整当前集合。未知动作或字段会被拒绝。

| `action` | 字段 | 作用 |
| --- | --- | --- |
| `goto` | `url` | 跳转到相对或绝对 URL；解析后的目标必须与目标系统 `baseUrl` 同源。 |
| `fill` | `selector`, `value` | 填写元素值。 |
| `click` | `selector` | 点击元素。 |
| `waitFor` | `selector` | 只等待 DOM 中出现匹配元素；不保证可见、文本匹配或页面就绪。 |
| `assertText` | `selector`, `text` | 断言元素文本。 |
| `assertValue` | `selector`, `value` | 断言元素原生值。 |
| `assertAbsent` | `selector` | 断言 selector 未命中。 |
| `screenshot` | `name`（可选） | 保存全页 PNG screenshot artifact。 |
| `screenshotViewport` | `name`（可选） | 保存当前视口 PNG screenshot artifact。 |
| `setViewport` | `width`, `height` | 设置浏览器视口，仅允许 `1440x900` 或 `390x844`。 |
| `setUiPreferences` | `theme`, `locale` | 设置主题和语言偏好；`theme` 仅为 `light` 或 `dark`，`locale` 仅为 `zh-CN` 或 `en-US`。 |
| `assertNoHorizontalOverflow` | 无 | 断言文档不产生水平溢出。 |
| `assertElementLayout` | CSS `selector`，可选 `elementCount`、`visibleCount`、`maxHeight`、`withinViewportRight`、`withinViewport` | 验证元素数量、可见性、高度和视口边界；至少必须提供一个条件。 |
| `assertFocus` | `selector` | 断言 selector 匹配元素获得焦点。 |
| `guardExists` | `selector`，可选 `onMissing` | 元素不存在时按 `continue`、`skipNext`、`stop`、`fail` 或 `error` 处理；`error` 是接受的失败策略别名。 |
| `pressKey` | `key` | 发送键盘按键。 |
| `pause` | `durationMs` | 暂停 `0` 至 `30000`（含）毫秒；保存的值按同一时长执行，`0` 表示不额外等待。 |

## 模板示例

### 搜索结果截图

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

目标系统的 `baseUrl` 例如 `https://www.baidu.com`；运行输入例如：

```json
{ "keyword": "Rustzen" }
```

## Admin Gateway API 示例

Admin Gateway 默认监听 `http://127.0.0.1:9801`。以下示例要求具备相应 Reports capability 的用户 Bearer token。

```bash
export RUSTZEN_ADMIN_TOKEN='<admin bearer token>'
base=http://127.0.0.1:9801

# 注册目标系统
curl -fsS -X POST "$base/api/reports/systems" \
  -H "Authorization: Bearer $RUSTZEN_ADMIN_TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"name":"百度搜索","baseUrl":"https://www.baidu.com","enabled":true}'

# 创建 Flow
curl -fsS -X POST "$base/api/reports/flows" \
  -H "Authorization: Bearer $RUSTZEN_ADMIN_TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"systemId":"<SYSTEM_ID>","name":"关键词检索","steps":[{"action":"goto","url":"/"},{"action":"waitFor","selector":"#kw"},{"action":"screenshot","name":"result"}]}'

# 创建运行并读取运行与 artifacts
curl -fsS -X POST "$base/api/reports/runs" \
  -H "Authorization: Bearer $RUSTZEN_ADMIN_TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"flowId":"<FLOW_ID>","input":{"keyword":"Rustzen"}}'
curl -fsS -H "Authorization: Bearer $RUSTZEN_ADMIN_TOKEN" "$base/api/reports/runs/<RUN_ID>"
curl -fsS -H "Authorization: Bearer $RUSTZEN_ADMIN_TOKEN" "$base/api/reports/runs/<RUN_ID>/artifacts"
```
