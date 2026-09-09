# Rustzen Reports 浏览器自动化任务指南与通用模板

本文档介绍 `rustzen-admin` 中 `apps/reports` 模块的浏览器自动化执行引擎体系，并提供适用于公网主流网站（百度、必应、知乎）的标准任务模板与使用方式。

---

## 1. 架构与设计原则

`rustzen-admin/apps/reports` 遵循终版精简架构原则：
1. **沙箱隔离**：每次任务运行（Run）均在系统临时目录下动态开辟独立的 Chromium 用户沙箱（User Data Profile），运行结束后立即回收，杜绝多任务状态互锁与 Cookie 串扰。
2. **原子动作模型**：任务由受校验的浏览器动作组成；元素布局断言使用 CSS selector，常规定位动作支持 CSS Selector 与 XPath。
3. **安全模板替换**：表单字段与 URL 支持 `{{input.variable}}` 运行时变量插值，并进行严格的边界与非法字符阻断。
4. **过程留存与存证**：支持步骤级审计耗时（`automation_run_steps`）、实时画面快照（Live Frame）与成果截图（Artifacts）。

---

## 2. 原子动作规范 (FlowStep)

| 动作 (`action`) | 必填字段 | 说明 | 示例 |
| :--- | :--- | :--- | :--- |
| `goto` | `url: string` | 页面跳转，支持相对路径（基于系统的 `baseUrl`）或绝对路径，支持 `{{input.param}}` | `{"action": "goto", "url": "/"}` |
| `waitFor` | `selector: string` | 等待目标 DOM 节点出现（默认最长等待 30 秒），支持 CSS 与 XPath | `{"action": "waitFor", "selector": "#kw"}` 或 `{"action": "waitFor", "selector": "//input[@id='kw']"}` |
| `fill` | `selector: string`, `value: string` | 表单输入，自动触发 `input` 与 `change` 事件 | `{"action": "fill", "selector": "#kw", "value": "{{input.keyword}}"}` |
| `click` | `selector: string` | 元素点击，支持 CSS 与 XPath | `{"action": "click", "selector": "#su"}` |
| `assertText` | `selector: string`, `text: string` | 目标区域文本断言包含校验 | `{"action": "assertText", "selector": "#received", "text": "OK"}` |
| `assertElementLayout` | `selector: string`，以及 `elementCount?`、`visibleCount?`、`maxHeight?`、`withinViewportRight?`、`withinViewport?` 中至少一项 | 对 CSS selector 的匹配数量、CSS 可见数量、可见元素最大高度、视口右边界或完整视口边界进行断言；`withinViewport` 要求每个可见元素的上下左右边界都位于当前截图视口内 | `{"action":"assertElementLayout","selector":"thead th","elementCount":9,"visibleCount":3,"maxHeight":64,"withinViewport":true}` |
| `screenshot` | `name?: string` | 现场存证截图保存（若步骤执行失败，系统亦会自动保存 failure 快照） | `{"action": "screenshot", "name": "result"}` |
| `guardExists` | `selector: string`, `onMissing?: "continue" \| "skipNext" \| "stop" \| "fail"` | 条件保护与元素存在性检测，支持未命中时继续、跳过下一步、提前成功结束或报错 | `{"action": "guardExists", "selector": "#modal-close", "onMissing": "skipNext"}` |
| `pressKey` | `key: string` | 触发键盘按键（如 Enter、Escape 等），支持输入框回车提交 | `{"action": "pressKey", "key": "Enter"}` |
| `pause` | `durationMs: number` | 定长延时休眠（毫秒），用于等待特定 SPA 动画或平滑渲染 | `{"action": "pause", "durationMs": 500}` |

---

## 3. 通用公共任务模板

### 3.1 百度全网检索与结果存证 (Baidu Search)

- **目标系统配置 (System)**：
  - 名称：`百度搜索服务`
  - Base URL：`https://www.baidu.com`
- **任务步骤 (Steps)**：
```json
[
  { "action": "goto", "url": "/" },
  { "action": "waitFor", "selector": "#kw" },
  { "action": "fill", "selector": "#kw", "value": "{{input.keyword}}" },
  { "action": "click", "selector": "#su" },
  { "action": "waitFor", "selector": "#content_left" },
  { "action": "screenshot", "name": "baidu_search_result" }
]
```
- **执行输入参数 (Input)**：
```json
{
  "keyword": "Rustzen"
}
```

---

### 3.2 必应多源检索与资讯探查 (Bing Search)

- **目标系统配置 (System)**：
  - 名称：`微软必应搜索`
  - Base URL：`https://cn.bing.com`
- **任务步骤 (Steps)**：
```json
[
  { "action": "goto", "url": "/" },
  { "action": "waitFor", "selector": "#sb_form_q" },
  { "action": "fill", "selector": "#sb_form_q", "value": "{{input.keyword}}" },
  { "action": "click", "selector": "#search_icon" },
  { "action": "waitFor", "selector": "#b_results" },
  { "action": "screenshot", "name": "bing_search_result" }
]
```
- **执行输入参数 (Input)**：
```json
{
  "keyword": "Rustzen Monorepo"
}
```

---

### 3.3 知乎全网实时热榜巡检 (Zhihu Billboard)

- **目标系统配置 (System)**：
  - 名称：`知乎平台`
  - Base URL：`https://www.zhihu.com`
- **任务步骤 (Steps)**：
```json
[
  { "action": "goto", "url": "/billboard" },
  { "action": "waitFor", "selector": ".HotList-list, [class*='HotList']" },
  { "action": "screenshot", "name": "zhihu_hot_billboard" }
]
```
- **执行输入参数 (Input)**：
```json
{}
```

---

## 4. API 调度与触发示例

系统提供标准的 REST/IPC 接口进行流模板创建与触发：

```bash
# 1. 注册目标系统 (POST /api/reports/systems)
curl -X POST http://127.0.0.1:19804/api/reports/systems \
  -H "Content-Type: application/json" \
  -d '{"name": "百度搜索", "baseUrl": "https://www.baidu.com"}'

# 2. 注册任务流 (POST /api/reports/flows)
curl -X POST http://127.0.0.1:19804/api/reports/flows \
  -H "Content-Type: application/json" \
  -d '{
    "systemId": "<SYSTEM_ID>",
    "name": "百度关键词检索与截图",
    "steps": [
      { "action": "goto", "url": "/" },
      { "action": "waitFor", "selector": "#kw" },
      { "action": "fill", "selector": "#kw", "value": "{{input.keyword}}" },
      { "action": "click", "selector": "#su" },
      { "action": "waitFor", "selector": "#content_left" },
      { "action": "screenshot", "name": "search_result" }
    ]
  }'

# 3. 触发运行任务 (POST /api/reports/runs)
curl -X POST http://127.0.0.1:19804/api/reports/runs \
  -H "Content-Type: application/json" \
  -d '{
    "flowId": "<FLOW_ID>",
    "input": { "keyword": "Rustzen" }
  }'

# 4. 获取运行状态与留存成果 (GET /api/reports/runs/:id)
curl http://127.0.0.1:19804/api/reports/runs/<RUN_ID>
```

---

## 5. 录屏操作与画面存证方案

对于任务执行过程的动态录制，系统支持不同层次的存证方案：

1. **现有实时帧录制（Live Frame）**：
   - 每次任务执行期间，系统在每个关键步骤均自动捕获当前视口快照，通过 `/api/reports/runs/:id/live-frame` 实时向前端或监控端推送最新画面（无需任务显式配置）。
2. **高频 Screencast 帧序列与 WebM/MP4 视频合成（可扩展特性）**：
   - **底层机制**：基于 Chromium CDP 原生 `Page.startScreencast` 接口，在任务开始时启动逐帧流式推流（每秒 10-30 帧，格式为 JPEG/PNG）。
   - **合成落地**：任务结束阶段，服务端后台将帧序列通过轻量编码器（如 ffmpeg 或 Rust webm muxer）合成为 `<run_id>.webm` 或 `.mp4` 录像文件，登记至 `automation_artifacts`，前端直接以内置 `<video controls>` 进行回放与时间轴定位。
   - **存储与性能平衡**：因录像文件体积远大于静态截图（单次任务约 2MB~20MB），建议通过 `automation_settings` 中的保留周期（Retention Policy）对视频产物进行独立生命周期管理。
