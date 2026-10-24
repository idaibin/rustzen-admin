# Admin 产品迭代需求索引

## 产品目标与现行权威

面向开发者的通用管理模板是主定位，Admin、Monitoring、Analytics、Automation 的完整应用是可运行参考。产品基础继续由 [product.md](product.md) 持有，各业务规则由 [feature index](index.md) 路由。本索引只定义本轮审查后需要收口的用户结果，不复制 feature spec 或改动共享 DESIGN。

| 用户 | 工作与痛点 | 目标结果 |
| --- | --- | --- |
| 模板开发者 | 需要理解启动、权限和扩展边界，避免被示例模块强绑定 | 可按已确认 core/full 合同选择，知道何者已被真实验收 |
| 实例 owner | 安装、配置、诊断、更新与恢复 | 所装服务/数据库/签名身份一致，故障可恢复 |
| admin/viewer/custom role | 完成授权的运维或只读任务 | 权限拒绝明确，模块故障不拖垮 Admin，失败不伪装为空 |
| 接手 AI | 原规格、阶段账与历史运行记录混杂 | 单次任务能定位当前合同、证据缺口和唯一 owner |

## 产品切片与迭代需求

| ID / 用户结果 | 原产品 owner | 本轮验收要求 | 就绪度 |
| --- | --- | --- | --- |
| A-R01 清楚选择 core/full 及其支持边界 | [composable-distribution](features/composable-distribution/spec.md) | A-AC01：产品入口、架构、命令和 OE 任务描述相同的目标/当前状态；既定旧 full 兼容例外清楚，默认切换有原门禁 | 文档核对 Ready |
| A-R02 可信安装、更新和恢复 | 原 [OE validation](features/composable-distribution/validation.md) | A-AC02：原 OE-A01–12 各自有证据；真 Linux/旧 worker/签名包、DOM 不被模拟层替代 | 沿用 OE-T09/T10/T11/T12 的依赖 |
| A-R03 验收不加载意外数据路径 | 当前隔离启动源码/脚本与原验证协议 | A-AC03：完整配置解析后所有实际数据库/目录仍在允许边界；拒绝先于写入；失败/取消不误动原实例 | 先固定当前 dirty 改动和授权，再评审 |
| A-R04 权限编辑可理解和恢复 | [role definition](features/role-definition-management/spec.md)、[user role readiness](features/user-role-assignment-readiness/spec.md) | A-AC04：loading/empty/error/retry 不混淆，输入/选择按原合同保留；空角色、owner-only、已分配角色删除均被服务端拒绝 | 原行为 Ready；先回读已有证据避免重复 |
| A-R05 开发者能重复执行所需验证 | 项目 justfile 与原验证指南 | A-AC05：开发检查与产物/部署检查副作用明确；声明版本可核对；执行不暗含未授权 build/起服 | 方案 Ready；命令实现另按任务授权 |
| A-R06 产品价值有真实基线 | 既定模板定位及开发者首个权限功能旅程 | A-AC06：记录首次有用结果耗时、求助、失败恢复与重复使用；结论带样本分母 | 采样方案 Ready；真实试用需参与者和环境 |

## 核心旅程与失败预期

实例 owner 从所选 composition 的可信产物开始，安装、登录、配置角色、完成一个已安装模块的有用任务，再诊断故障并恢复。core 的模块集合为空时不显示虚构离线模块；full 中已启用但故障的模块仍按既有权限保留可发现导航。数据属于各模块，Admin 不跨库读业务表。

用户/角色编辑依现行 feature spec 保留输入和选择，失败允许重试；后端仍为权限和删除事务的最终裁决者。部署失败或中断按原更新/恢复合同读回状态与实际 installed identity，不把进程退出或 HTTP 超时直接当作可重试部署的授权。

本轮没有新的页面布局、颜色或动效要求。视觉验收沿用根 DESIGN 和目标 UI feature；不能把 Dashboard 的源码字符串检查作为 DOM 结果。

## 质量与成功标准

既定功能收尾优先于模块扩展。必需证据按源码、API/SQLite、Web、产物、Linux、部署分层；权限与数据拒绝必须验证无越界副作用。已有未变化的有效证据可复用，但需要记录与当前输入的对应关系。

可宣称的成功是：开发者能理解并启动所支持组合，用户在权限内完成有用旅程，故障诊断与恢复有实证。具体效率阈值从真实基线设定；不把参考实现自动当成已有客户需求。

## 后续决定

沿用已接受 core/full 范围；独立模块发布、多租户、多项目 Analytics、凭据存储/扩展自动化等仍由各自原生 feature 决定。原 OE 的真实旧 full 兼容例外不能被“初始化模板”总原则静默删除，也不能扩为任意旧版本兼容。

下一步见 [方案](../plans/product-iteration.md) 和 [收口任务账](../plans/product-iteration-tasks.md)。原 OE、主题、交互账继续拥有其任务终态。


## Developer-facing iteration order

The confirmed primary audience is developers adapting a Rust + React admin
template. Preserve the A-* and OE-* contracts above and deliver one observable
user outcome at a time.

| Order | Developer outcome | Existing owner / dependency | Completion boundary |
| --- | --- | --- | --- |
| D-01 | Understand the template, run the documented development path and trace one feature | README pair, getting-started guide, existing role feature | Clear support matrix, exact prerequisites and source links; clean-environment runtime proof stays separate |
| D-02 | Create a custom role, assign it and verify allowed/denied operations | A-04, role-definition and user-role-readiness specs | Complete one real journey including selection recovery and a restricted identity; preserve backend denial and input recovery |
| D-03 | Start from the supported Admin-only composition | A-06 and OE-T09/T10/T11/T12 | Existing core installation, identity, update/recovery and command gates pass before changing defaults |
| D-04 | Adapt an existing end-to-end feature to a business need | Existing role feature and backend/frontend guides | Developer can locate route, capability, handler, service, SQL and UI states and complete an independent adaptation |
| D-05 | Assess department data-scope demand | Public issue #2 and real developer feedback | Confirm actors, organization membership, default scope and every read/write/export boundary before implementation |
| D-06 | Assess multi-tenant demand | Public issue #3 and independent use cases | Confirm tenant identity, membership, isolation and administration model before implementation |

D-05/D-06 are discovery items, not accepted implementation scope. Public issue
counts do not establish market demand. Growth work follows a working, explainable
feature: publish only an accepted candidate and measure real trials separately
from repository traffic.
