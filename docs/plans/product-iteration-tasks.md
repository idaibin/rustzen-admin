# Admin 产品收口任务账

定义权威：[迭代需求](../product/iteration.md) 与其引用的原 feature specs。方案：[product-iteration.md](product-iteration.md)。本账只拥有以下 A-*；原 OE-*、IN-*、主题任务保留原账和有效完成证据。任何相同验收已由原任务完成时关联证据，记有效 no-op，不新增同义实施任务。

当前新增任务：Ready 0、Blocked 1、Doing 0、Done 5。A-01 由当前 Codex 协调者于 2026-10-13 领取，basis 为 `d209eac830391d4f21f7c0bc47bb49e796273f95` 加当前文档工作区；仅修正文档事实，保留现有源码改动与 OE owner。A-02、A-03、A-05 由同一协调者于 2026-10-13 领取并完成各自审查/工具链入口/方案范围；本次用户进一步授权 A-02 修复及 A-04/A-06 可执行证据归并，已完成有界实现与检查；当前没有 Ready 项；2026-10-23 的用户继续授权及隔离窗口已解除 A-04 的目标环境依赖，原验收 owner 已完成剩余矩阵并经 ZCode 独立审查接受；A-06 仍等待原 OE 验收。领取前回读 HEAD/dirty 和现有 owner，写入 owner/claimed_at/basis/next_action 后再推进；不接管已有实施者。

| ID | Outcome / Owner / Scope | Inputs / Dependencies | Acceptance / Evidence | Status / Blocker / Next action |
| --- | --- | --- | --- | --- |
| A-01 | 当前产品/组合说明一致；Product 文档 owner；product/index/architecture/组合文档 | A-R01、现行 accepted spec、OE 回执；无前项 | A-AC01，目标与实现/验收分离，旧 full 精确例外和默认切换门一致；文档读回 | Done（A-AC01 文档层）；当前 full 默认、core/full 阶段证据、OE-5 切换门和旧 full 精确例外已对齐；运行验收继续由原 OE 账持有 |
| A-02 | 当前隔离改动可评审接续；当前协调者；启动脚本、合成测试及 config 只读验证 | A-R03，已有 dirty 与来源；无前项 | A-AC03，最终路径/首次副作用/失败清理链明确，合成测试选择与缺口清楚；固定 diff/源码证据 | Done（A-AC03 有界修复/验证）；P2-01 已修复，6 组 launcher 回归与 1 个 Rust 最终配置路径用例通过；真实服务写入/生命周期仍 Not verified |
| A-03 | 验证命令副作用和工具版本可控；工具链方案 owner；justfile、manifest、调用脚本 | A-R05、当前命令链；无前项 | A-AC05，列出无构建最小命令与完整发布门，覆盖不丢失；授权实施后精确子检查 | Done（A-AC05 有界入口）；副作用矩阵与版本已核对，固定 pnpm dlx bun@1.3.14 入口及失败传播通过；裸 Bun 漂移已隔离于本次检查路径，完整工程门独立待验 |
| A-04 | 用户/角色原旅程验收闭合；验收 owner；两个原 role feature | A-R04、A-02 的必要环境结论、既有验收证据 | A-AC04，已通过输入未变者复用；仅缺口补真实 UI/API，负例无数据副作用 | Done（A-AC04）；当前协调者与原单一实施 owner，claimed_at 2026-10-23，basis 为上述 HEAD 加当前工作文件。原旅程证据按源码哈希复用，剩余编辑、异常恢复、双语/窄屏与权限边界采用成对覆盖；真实与 Mock 分层明确。ZCode 固定快照审查 PASS，无 P0–P2 或必需验收缺口。临时数据、浏览器、服务和副本已清理。当前证据见 ignored `.codex/artifacts/role-user-acceptance/` 和 `.codex/reviews/admin-a04-acceptance-*`；next_action：D-03 仅沿 A-06 原 OE 门继续。 |
| A-05 | 可执行的模板价值采样方案；Product owner；安装→有用结果旅程 | A-R06、原定位与现有 90 天计划；无前项 | A-AC06，目标参与者/旅程/样本分母/计时/失败记录/退出条件明确；方案层 | Done（A-AC06 方案层）；开发者启动与权限功能旅程、参与者/分母/计时/失败恢复/退出条件齐备；样本 0，真实招募和运行另按授权 |
| A-06 | 当前产品剩余验收可关闭；验收协调 owner；相关原账的入口和总结 | A-01–04；原 OE-T09/T10/T11/T12 必要结果 | 原 Required 验收全部可读回，声明来源/产物/Linux/UI/部署边界；无隐性未决 | Blocked；当前隔离 Linux core/full 构建、安装、生命周期、更新/回滚与 18 个原生中断点已实测，最终独立接受待 ZCode；Linux 23 项页面门运行中。用户本轮跳过的真旧 0.5.1 worker/OE-A09 仍未验证；A-04 与此前本地 OE-T11 接受保持 |

## 原任务继续入口

- OE-T12：原账拥有 installer 独立验收；先核对最新固定候选与正负例证据。
- OE-T10：原账拥有真旧 worker、真实包、Linux 更新/回滚；执行前核对运行窗口及精确权限。
- OE-T11：原账拥有 core shell/Monitor card 的 DOM 验收；只能用同版真实目标关闭。
- OE-T09：原账拥有剩余 packaging 集成；不在本账另起打包任务。

## 回执要求

每项保留原 requirement/acceptance ID、实际 owner、当前 basis、变更路径、命令与直接 rc、证据引用/hash、真实/替代依赖、失败次数、阻塞恢复条件。Ready 不授权自动 build、起服、改数据库、使用密钥或 Git。A-03 的方案完成不能关闭其后续必要配置运行验收；需要实现时在同一 ID 下追加有界步骤和依赖。

## A-01 执行与验收

Owner：当前 Codex 协调者；claimed_at：2026-10-13（Asia/Shanghai）；basis：`d209eac830391d4f21f7c0bc47bb49e796273f95` 加领取时工作区。允许范围为产品/组合/架构入口文档；既有 config、dashboard、启动脚本及测试改动已作 SHA-256 前后对照，保持不变。

结果：EN/ZH README、product/index、architecture、组合 spec/实施入口和关联 UI 链接统一目标、源码阶段与真实验收边界；AGENTS 指向已批准的精确 preceding-full 例外。原 OE 任务状态与历史执行回执未改。现行 Cargo 默认 full、just build→full、packager 显式 core/full 和 core Web 入口已只读核对；CLI Module::All 仍查询四服务，文档明确留在 OE-A08 收口，不冒充 core CLI 已完成。

验收：10 份相关文档的 95 个相对链接/锚点可解析；EN/ZH bash/text 命令块完全一致；`git diff --check` rc 0。源码追踪涵盖 Admin features/runtime、Web producer/embed、packager、CLI statuses 和 deploy backup set。只读 Rust architecture/baseline profile，不涉及并发、性能、SQLite 或安全专项审计。

过程记录：一次临时文档编辑命令因 JavaScript 字符串语法失败，未写入文件；改用精确 patch 后回读通过。一次后续读取命令使用了错误仓库 cwd，未改变文件，后续按目标 Skills cwd 读取。未执行构建、起服、安装、数据库操作或真实 UI/Linux 验收；这些不属于 A-01 的文档验收。next_action：本次按用户指定转 S-03；Admin 其余 Ready 前沿为 A-02 → A-03 → A-05。

最终交叉回读：本次两仓 13 份相关文档的 117 个本地链接/锚点有效；A/S 主任务行状态计数一致；领取时无关 dirty 的 SHA-256 保持不变，S-03 私有明细 hash 与账本一致。

## A-02 / A-03 / A-05 执行回执

Owner：当前 Codex 协调者；claimed_at：2026-10-13（Asia/Shanghai）；basis：HEAD `d209eac830391d4f21f7c0bc47bb49e796273f95` + 领取时工作区。原源码归属未接手；只读核对相关既有改动，方案/证据写回本账与 product-iteration.md。原 OE/IN 账和历史回执未改。

### A-02：审查完成，源码修复及运行层仍有缺口

固定 SHA-256：shared.rs `96010f349022bfd24c3f84b03769694bb432151aec70bc8bb24c738a70b37048`；admin.rs `1d36e5191e18066bcedaae470b51665a1eb730307207a69b05e3428dd97fe21d`；launcher `a2294477c73a8e1e7c20144c347509138a8b0c1d3f20e5c0568f37698835b3b6`。

审查覆盖 launcher→共享 dotenv→AdminConfig→日志→pool/migrations；其他三服务调用共享 dotenv 的默认行为也在影响范围。检索 scripts、docs 与可读 .codex 中的 local-acceptance-launch/OE-ISOLATION-GUARD 引用，仅找到脚本和本轮方案入口，未据此推断其他宿主没有回执。review basis 不含本轮无关 dashboard 修改。

**P2-01 / validated，Standards + Spec：** launcher 第 110 行无条件 cd repo target/rz；干净合成 checkout 中，即使指定允许的 temp root，也 rc 1 且未执行 child。原脚本字节复制到自有临时 repo 布局，缺 target/rz 时失败，存在时同一 temp 路径方案成功。修复方案及原 owner 验收条件见 [隔离审查](product-iteration.md#a-02-隔离审查与修复接续)。这是可用性缺陷，不是已证明越界写入。

9 项纯合成探测：缺 repo base rc 1（问题复现）；合法自有 temp root rc 0；空 root、相对 root、dot-dot、继承 DB override、已有 root、symlink 组件六项均 rc 2；child exit 7 原样传播且新根/receipt 保留。负例没有新建被拒绝根（已有根用例原目录保留）。child 仅 true/exit，不启动服务、读取真实 .env、密钥或数据库。测试结束由探测 harness 清理其自有临时父目录，不是 launcher 已具备自动清理证明。

`bash -n` rc 0。Standards verdict：存在 P2-01；Spec verdict：最终配置路径由源码追踪支持，但 Rust 回归、取消/并发、实际 child 最终路径仍 Not verified。A-02 的审查/接续材料完成不代表隔离实现验收完成；仅阻断依赖该入口的真实执行，A-03/A-05 已继续。

### A-03：检查入口收口

已读 justfile/manifest/锁文件位置及直接调用脚本，确认 Web composition 四个测试均可触发 build；contract verify 会生成临时 OpenAPI/client；build-config 的 public-key 会访问签名私钥来源。只追踪代码，没有读取密钥内容。副作用分类和最小入口见 [无构建方案](product-iteration.md#无构建开发检查方案)。

实际 Node 24.15.0 与声明一致；全局 Bun 1.4.2 与项目 Bun 1.3.14 不同，pnpm 12.6.0。两个 manifest 和 Rust toolchain 只读，未更改版本。`pnpm exec bun --version` 在 Web cwd rc 1，ERR_PNPM_OTHER_PM_EXPECTED；该诊断明确指向 dlx/shim 路线。采用仓库已有的固定 dlx 路线：`pnpm dlx bun@1.3.14 --version` rc 0；Web cwd 实际 eval 回读 Bun 1.3.14 及 pnpm store 内可执行文件；仓库根 eval `process.exit(7)` 直接 rc 7，失败未被包装为成功。未改全局 Bun/shim，未执行项目依赖安装、prepare 或 build。

`bash scripts/test-verify-services.sh` rc 0（现有静态接线断言）；完整 just check、Rust toolchain 下载/编译、浏览器/Linux、私钥和发版路径本轮未运行。fixed dlx 的来源为本机已解析实际入口，未声称离线新机缓存下载层通过。先前全局版本漂移不再作为本次有界检查的阻塞，后续完整门执行前仍须按计划核对环境。

### A-05：方案完整性

以现行 product/Monitoring spec 和原 90 天价值计划为依据，已明确模板开发者/实例操作者、获准 full 环境、材料齐备→安装→登录→真实 report→诊断恢复→再次使用；分别计安装与已有实例旅程、准备阻塞与已开始失败、人工时间与等待、参与人数与重复尝试。验收回读覆盖参与者、旅程、身份、n/N、计时、失败/取消、求助与作者代操作、退出规则九类要素。样本为 0，仅完成方案；不宣称产品采用、收益或真实安装通过。

后续：A-04 先在原 IN/role 回执中归并同版证据并固定目标；隔离路径的 P2-01 修复与真实验收由原 owner 接续。A-06 等待 A-04 与 OE-T09/T10/T11/T12 必要证据，不新增替代任务账或重复已接受阶段。

最终回读：两份本次 Admin 文档的 6 个本地链接/锚点通过；6 项任务为 4 Done / 2 Blocked；`git diff --check` rc 0；领取时所有既有非本次文档改动的 SHA-256 均未改变。纯合成探测明细留于 ignored `.codex/reviews/admin-iteration-20261013.md`，SHA-256 `05e9d0191d5cd3698c1823f83e7ee43fdf3a5e2d1e779fd20e9573cca7fbe984`。

## A-02 修复与 A-04/A-06 接续回执

Owner：当前 Codex 协调者；2026-10-13；basis：HEAD `d209eac830391d4f21f7c0bc47bb49e796273f95` 加领取时 dirty。用户明确授权本次 P2 修复；保留上述历史审查回执，当前状态以本节和主表为准。实现仅修改 launcher 并新增 `scripts/local-acceptance-launch.test.mjs`，既有 config/dashboard/OE/IN 源码及账本不改。

### A-02 / A-AC03

P2-01 已修复：忽略缺失的无关 base，只将所选允许 base 的物理路径用于父路径检查；不创建 repo target 来绕过校验。失败目录保留，由调用方在子进程终止并核对归属后清理，启动 receipt 不代表成功。

- 红阶段：固定 Bun 1.3.14 执行新测试，rc 1，0 pass / 6 fail，均被缺失 base 提前阻断；不是六个独立产品缺陷。
- 绿阶段：`pnpm dlx bun@1.3.14 test scripts/local-acceptance-launch.test.mjs`，rc 0，6 pass / 0 fail / 50 assertions。覆盖干净 checkout 的 temp、缺失另一 base 的 repo、输入/环境拒绝、child exit 7、exec 127、创建中断 9、SIGTERM 取消和同根并发唯一成功；拒绝时无新增且原 sentinel 不变，失败根不可复用。测试未监听、未读取真实环境或数据库。
- 配置层：独立 `env -i PATH="$PATH" HOME="$HOME" TMPDIR="${TMPDIR:-/tmp}" LANG=C RUSTUP_TOOLCHAIN=1.95.0 cargo test --locked --offline -p rustzen-config --features admin,modules dotenv_reinjection_of_sqlite_paths_reaches_final_paths_unless_skipped -- --test-threads=1`，rc 0，1 pass / 0 fail / 26 filtered。合成 dotenv 对照和 skip 后四个最终数据库路径均通过；进程退出隔离 env/cwd，不修改测试源码。
- `bash -n` 和 `node --check` rc 0。launcher SHA-256：`01bd8240ee1d5fa631264208f8bd2f6661c4f87527ddd23e47d5b2e1da1c5139`；测试 SHA-256：`38c783160059f0ddc54e87ece2287564fc4f2c6d25fea53358a958388639db2c`。

上述证据关闭 P2 和合成隔离/生命周期缺口；不证明真实服务的实际写入、后代进程清理或页面运行。没有构建发行物、起服、重启、密钥访问、提交或推送。

### A-04 / A-AC04

核对 role-definition-management、user-role-assignment-readiness 原 spec，以及 IN-01–07 已接受记录（`b94909de11c4e09a50815eceec123dbe8c00a6f6`）。旧账记载真实用户/角色 CRUD 与标注故障注入；当前 checkout 的 `.codex/artifacts/interaction-unification/route-runtime/` 下 README.md、verification-summary.json、final-source-sha256.json 三个精确引用未找到，不能重建同版 UI 证明。检索结论仅限这三个路径，不推断其他宿主不存在回执。

原验收 SHA 到当前的 system routes、role/user backend、Web components/hooks 范围差异仅为 user repo/service/types 的 dashboard count 条件编译 full→core；不改指派逻辑。该比较不覆盖所有构建输入，不用于证明当前 bundle 身份。

| 验收项 | 当前直接证据 | 待补层 |
| --- | --- | --- |
| role/user composition 及删除错误派生状态 | `pnpm dlx bun@1.3.14 test apps/web/tests/role-composition.seam.test.mjs apps/web/tests/user-composition.seam.test.mjs apps/web/src/routes/system/-role-delete-state.test.ts`；rc 0，5 pass，23 assertions | DOM、双语、loading/error/retry、表单保留与重复提交 |
| 已指派角色拒删、列表 count/deletable | isolated cargo test filter `infra::app::tests::roles::`；rc 0，2 pass / 223 filtered，Axum oneshot + 合成 SQLite | 当前真实服务 API 与授权身份 |
| 过期 count 后删除、已删角色再指派 | filter `soft_delete_rechecks_assignments_after_stale_count`；rc 0，1 pass / 224 filtered，自有临时 SQLite、双 pool，结束关闭并清理 | 真实旅程及禁用角色权限撤回 |

两个 Admin Cargo 命令都沿用上述独立 env 与固定 toolchain，并使用 `cargo test --locked --offline -p rustzen-admin --bin rz-admin <filter> -- --test-threads=1`。恢复：原 owner 提供可回读的原回执，或固定已运行且与候选匹配的地址、账号权限和自有测试数据；只补受影响 UI/API 项。当前 README 演示地址不能绑定本地 dirty 版本，不据此访问其他实例写数据或自行起服。

### A-06 / 原 OE 归并

| 原项 | 本次已完成 | 剩余与恢复条件 |
| --- | --- | --- |
| OE-T12 | 读取 `target/rz/oet12-installer-20261010/PROVENANCE.md`，四份红/绿 harness 与 deploy-sign 日志 SHA-256 均符合记录；末尾分别为基线失败、16 groups、21 groups、7 pass。读取 `target/rz/oet12-negative-verification-20261010/`；SHA256SUMS 中 14/14 文件匹配，run.log 记录六例 exit 1、6/6 passed；deploy/setup-layout.sh 与 scripts/test-setup-layout.sh 相对 79f1206 无差异 | 当前回读支持历史合成正负例；不等于本次重跑或独立接受。原 owner 完成固定候选的独立验收，Linux 生命周期单独提供 |
| OE-T10 | 已读取 `target/rz/oet10-env-prep-20261010/ASSESSMENT.md`，仅为历史 x86_64 Colima 环境准备 | 真旧 0.5.1 worker 接受新包、更新/回滚仍缺；原 owner 固定 Linux 执行窗口、候选/旧包/可信材料与权限，不读取密钥或启动 VM补证 |
| OE-T11 | 原账仍需要 core shell / Monitor card 真实 DOM | 提供同版既有服务与页面身份后补验；seam 不替代 |
| OE-T09 | 沿用原 packaging 集成范围及依赖 | 待原候选产物/Linux 必要结果，当前不授权另行 build/package |

原 OE/IN 状态保持不变，A-06 仍依赖 A-04 及上述 Required 结果。剩余队列仅上述真实目标/独立接受缺口，不新增重复任务。任务计数维持 4 Done / 2 Blocked。

最终回读：方案/任务账本地链接目标可解析，主表 4 Done / 2 Blocked，`git diff --check` rc 0；领取时既有文件除本次 launcher 和两份计划文档外 SHA-256 均保持不变。

## A-04/A-06：ZCode 与既有环境续验

2026-10-13，当前 Codex 协调者；HEAD 仍为 d209eac830391d4f21f7c0bc47bb49e796273f95。只读 ZCode 原 Admin 会话和现存环境，不发送消息、不新建会话、不构建、不起服。

### 来源与同版边界

以只读 SQLite 连接检索 `/Users/daibin/.zcode/cli/db/db.sqlite` 中目录严格等于本仓的 session（最近 8 项），读取原 Admin 会话 `sess_22fb64e0-0e04-4e58-b72f-12a1e8a5e886` 的近期文字终态及 OE-UI-ENTRY-01/v2 指令。末条实质结果为 V6 candidate-004 的 10 个合成调用终态，属于隔离 harness，不是 OE-T10/11/12 完成证明。排除 provider 配置、凭据、无关会话和真实业务数据库；一次 sqlite3 JSON path 查询失败后改用标准库只读解析，没有重发业务操作。

本机 lsof 和进程 cwd/executable/start time 定位到既有 PID 3606（full，9801/9811）和 3608（core，9821），均始于 2026-10-12 18:08:34，本次保持运行。可执行文件为 `target/rz/oe-ui-entry-01-20261012/V2/bin/rz-admin-{full,core}`；打开的 SQLite shm 位于该 V2 各自 runtime，未打开数据库内容。二进制 SHA 分别 c21006e8…4e6e7e4 / caf0faf4…7a4b84f，首页实际 HTTP 200 字节 SHA 分别 4986ceb4…dacc79d / 80c3b3a4…8f2a47d，均完整匹配原 svc meta/asset 记录。prebuild-source-state 的两个 dashboard 源文件 hash 也与当前一致。

这证明既有 V2 产物身份；不证明当前工作区整体已构建。当前 config dotenv skip 与 launcher 修复未包含在原二进制中，不能用本次页面验收关闭 A-02 的新启动链运行层。原 attempt-1 真实根访问事故及其 UNKNOWN 结论保留，不追读真实数据补证。

### 本次直接浏览器验收

唯一任务自有 Codex IAB 标签页 1，1920×1080、亮色、原已声明地址 127.0.0.1:9821/9801。使用源码明确的本地合成 owner 登录；登录/退出会更新合成会话与审计数据，未创建用户、角色、权限或提交业务表单。

| 原项 | 直接结果 | 层次/边界 |
| --- | --- | --- |
| A-04 角色/用户就绪 | core 真实角色表 3 个内置角色，assigned=1/0/0，内置角色无编辑删除入口；创建角色权限目录加载成功，0 选择时 Create 禁用；用户角色选项 Owner/Viewer/Administrator 可见，空表单提交显示必选角色校验 | 原服务真实 GET 200；后端拒绝、非 owner 和成功创建不由此推断 |
| A-04 用户失败恢复 | 仅当前标签页阻断已观察的 roles/options GET；显示 Failed to load roles + Reload，Create 禁用；移除阻断后 Reload 恢复选项，实名 A04 Unsaved 保留 | 浏览器受控故障，不冒充真实服务故障。用户名/邮箱被工具脱敏；读取值/布尔比较不适合证明这两项保留，保持 Not verified；密码未填写，角色选择保留未覆盖 |
| A-04 权限失败恢复 | 同法阻断 menus/options；显示 Failed to load permissions + Reload，Create 禁用；解除后恢复权限树，角色名称、编码、描述逐项保留 | 草稿全部取消；未提交角色；成功空目录、403、重复提交仍未覆盖 |
| A-04 双语 | 角色列表英文→中文，内置角色、标题、表头及分页切换正确，再恢复英文 | 不等于中文首选新上下文和所有弹窗双语全矩阵 |
| OE-T11 部分 | core dashboard API code=0/data=[] 且无 Runtime modules 面板；full code=0/3 个 available:false，DOM 为 Monitor/Insights/Automation 三张 Unavailable 卡；两张同视口截图已回读 | 这是 Dashboard 模块面板，不是 Monitor delivery card；后者仍需已注册且可用的 Monitor 目标 |
| 登录边界 | core 未登录访问角色路由转 login；两实例退出后再次访问保护路由均转 login | 退出后页面拒绝已观察，未导出 token |

full 控制台当前采集 error/warn 为 0；core 的预期 ERR_BLOCKED_BY_CLIENT 独立保存在 network 证据中，不声明所有历史控制台无错。第一次用 Retry 定位时页面实际按钮为 Reload，定位超时、没有触发请求；回读后使用实际按钮一次恢复，保留该工具错误，不计产品失败。

证据目录：ignored `.codex/artifacts/admin-iteration-runtime/`，含 DOM、脱敏请求结果、模块响应、截图、身份及清理 manifest；manifest SHA-256 `d9f1715bad3bb396ecd61539aea78b2f3bef0479bf88aad857013ce1a7da8f14`。结束已恢复英文、移除请求阻断、退出两实例账号、关闭自有标签页并重置临时视口；原服务和用户标签页未停止/关闭。

### A-06 的剩余前沿

OE-T12 对 parent b6af0ee2b8f03244322741428cf6acb1a77114aa → 79f1206c227a8fccff782289dbbae11761159e5e 的本地安装器范围完成独立源码/证据接受，详见原 OE implementation 新回执。复用此前已核哈希的 21 组正例/负例与 6/6 定向负例；本轮仅 bash -n 两文件 rc 0，没有新安装或独立动态重跑。原 OE 账同步本地范围接受，不清 Linux 门。

`colima list` rc 0：default/aarch64、p8e-amd64/x86_64、rz-p8e-amd64/x86_64 均为 Stopped。OE-T09/T10 依赖的候选构建、真旧 worker 与升级/回滚窗口在当前“不构建、不起服”边界内不可执行。OE-T11 已有 core/full 页面证据，缺 Monitor 模块注册/服务及 delivery card 状态；不自行补服务。A-04 下一步是获准的可写独占测试数据/受限操作者身份，以及仍缺的 UI 状态证据；当前共享旧 V2 保持无业务 CRUD 写入。状态仍 4 Done / 2 Blocked，阻塞只作用于这些剩余项。

本轮收尾回读：4 份相关仓库文档本地链接目标有效，`git diff --check` rc 0，PID 3606/3608 及原监听仍在；A 主表计数保持 4 Done / 2 Blocked。没有修改源码、构建产物或启动配置。

## A-04/A-06：原 ZCode 客户端同包验收收口

Owner：ZCode 原 Admin 会话 `sess_22fb64e0-0e04-4e58-b72f-12a1e8a5e886` 唯一执行；当前 Codex 协调者最终验收/文档 owner。2026-10-13（Asia/Shanghai），basis HEAD `d209eac830391d4f21f7c0bc47bb49e796273f95` + 19 项 dirty 文件；任务 `a04-remaining` 回链 A-04/A-AC04 与 A-06，未新增任务。用户最终指定客户端原会话续行、不再使用 ZCode CLI，已按该边界执行。

### 交接与归属

原 CLI 第一次在提交前 rc 1：安装包 Built-in Provider Config 的相对路径解析不匹配。已检索安装资源及 bootstrap 调用链，实际安装文件存在；第二次只设置进程级路径，未改应用/全局配置。该轮浏览器控制不具备，原三次同因调用失败保留。用户改用客户端后，唯一自有 CLI PID 90550 安全中断并以 rc 130 结束；原会话无 running model request 后，客户端发送一次同包交接续行、一次有界断言补证反馈，没有重派库存、没有新会话或新实施者。

客户端原生终态：message `msg_mv6awsr1_343bcd7b-7fef-437c-b144-01b84ccfa6b8`，finish=stop，completed=1791873005439（2026-10-13 14:30:05 +08:00）；实际 model_usage 为 provider `account:bigmodel-individual-coding-plan`、model `GLM-5.3`。客户端自动采用个人套餐，未手工改模型或设置；报告自述 bigmodel/glm-5.3 只作候选描述，归属以上述原生记录为准。

目标为既有 V2 full/core 实例，PID 3606/3608、执行文件/cwd/启动时间/监听、二进制与实际服务资产 hash 均匹配前回执。19/19 既有文件在协调者文档更新前 hash 不变。现有用户总数 1（合成 owner），角色选项为三内置角色；无本次拥有的受限或可编辑账号，不创建/重置凭据或改变权限。普通登录/退出会改变隔离实例的会话/审计数据，不能声明数据库字节未变。

### 当前用例结果（按 ID 的唯一分母）

原 ZCode 报告将 backend leg 的 PASS 混进旅程总数；本账按八个 ID 归并为 **PASS 5 / PARTIAL 1 / BLOCKED 2**，合计 8。独立接口测试 1 PASS 另计，不提高旅程完成数。

| ID / 原验收 | 当前终态及直接证据 | 验收层 / 剩余缺口 |
| --- | --- | --- |
| A04-FORM-PRESERVE / A-AC04 | PASS；受控选项 403→Reload 恢复真实选项，用户名/邮箱/实名前后匹配；未提交合成密码草稿的前后 SHA-256 相同；非默认 Disabled 重试后保留；图 01/02/06 | Mock 故障 + 真实 UI/恢复 GET。密码最初仅报长度不足，后按同包反馈补哈希；字段只用于未提交草稿，没有生成/修改凭据 |
| A04-ROLES-EMPTY / A-AC04 | PASS；200 空数组显示 No roles available to assign，0 checkbox、无失败态 Reload，Create 禁用，取消后 users total=1；图 04 | Mock UI。截图密码未填写，不能称“其他字段全部有效”；提交禁用的 roleReady 条件由源码和 UI 共同支持，不冒充真实服务空数据 |
| A04-PERM-EMPTY / A-AC04 | PASS；200 空权限目录独立空态、0 selected、有效角色文本草稿、Create 禁用，取消后仍三内置选项；图 05 | Mock UI，不代表服务端权限目录真实为空 |
| A04-DENIED-RETRY / A-AC04 | PASS；任务页返回 403，权限失败文案/指引/Reload 可见，未就绪时禁止提交；撤销规则后单次 Reload 恢复真实三个选项；图 01 | 受控响应产生的 Mock UI，不能升级为真实受限身份获得服务端 403 |
| A04-SELECTED-RETRY / A-AC04 | PARTIAL；失败→恢复后实际勾选 Viewer/Administrator 可用；图 07。已经选择后再经历第二次失败→恢复仍 Not verified | 首次 focus 因 staleTime=Infinity 没发请求，旧“通过”断言已拒绝并保留。owner 行无编辑入口，现有实例未提供可编辑已指派行。角色/权限预选项的完整编辑重试仍需拥有的 fixture 和实际可达触发 |
| A04-DUPLICATE-CANCEL / A-AC04 | PASS（本次取样）；有效表单的 POST 在任务页挂起、未送达服务；Creating/loading、提交/取消/关闭入口禁用；独立 users 查询仍 1；图 03。另在空态草稿执行真实 Cancel 并读回零业务写入 | Mock 延迟提交 + 真实 UI/独立 GET；证明本次提交期间入口防重，不声称所有并发时序或真实成功创建通过 |
| A04-RESTRICTED / A-AC04 | BLOCKED（真实旅程）；只有 owner。最近既有 Axum + 内存 SQLite 回归实际 rc 0、1 pass / 224 filtered，viewer-token POST users→403 | 后端测试单列 PASS；真实 UI 的 owner 选项排除、受限动作与接口拒绝仍依赖拥有的受限账号 |
| A06-OE-T11 / A-06 | BLOCKED（成功数据/完整卡片）；实际 full /monitoring/incidents 显示 Delivery status failed、告警加载失败与 Reload；实际两个 GET 为 HTTP 503/code 40001，图 08 | 真实路由/失败 badge 已接受。不是 Dashboard 卡；通知投递成功、pending/quarantine/detail、注册后导航仍待同版可用 Monitor |

隔离接口命令：`cargo test --locked --offline -p rustzen-admin --bin rz-admin documented_nested_routes_match_their_final_public_paths -- --test-threads=1`，原生工具输出直接 `DIRECT_CARGO_RC=0`，1 pass/0 fail/224 filtered。用例采用 TestLoader + in-memory SQLite，不启动服务，不证明当前真实登录身份的授权配置。

### 原件与协调者核验

ignored 原件：`.codex/reviews/a04-remaining-{task,basis,invocation,client-receipt,events,progress,result,verification}` 及 `a04-remaining-evidence/`。结果 SHA-256 `3af5cc1c7fa567eb4e5aff83bd9f9862bb524ac1dcc3791851671cde632f8d8a`；证据 SHA256SUMS `2858bf8263217dc9af6d60110e0eb28fed084ae87a2bed611076cf3fd34ebbab`，11/11 证据匹配；八张截图均实际回读。API inventory 后缀 json，内容是分段文本/JSON 响应，按证据日志读，不当机器合同。

原始 statusText 选择器未读到 Ant Select 值，布尔断言为 false；不将该断言算通过。后续已用重试前实际 DOM 的 Disabled title 与图 06 的重试后 Disabled 回读支持非默认保留。密码原生 part `part_mv6aip3e_ce07bd14-d5e4-48fc-8d09-1de406530797` 明确记录 BEFORE/AFTER 同哈希。时间占位 `06:0xZ` 只在过程原件中保留，不作为精确时间证据；原生事件/文件终态负责定时。没有为更正文案重发执行或改写原结果。

清理：任务页拦截全部移除、真实 UI 退出后 login/token-null 元数据可读回、三个任务标签页关闭；用户的 ZCode 客户端和原 Admin 进程保留。没有源码/config/test/Git 修改，没有 build/bundle/package、依赖安装、服务/VM 启停。

### 唯一剩余恢复队列

1. A-04 已选项编辑重试与真实业务旅程：取得可回读、归属明确的可编辑测试用户/自定义角色及受限身份；固定其角色/权限、允许副作用和清理 owner，仅补未验断言。若需新建持久 fixture、凭据或授权，先明确该写入窗口，当前未执行。
2. A-06/OE-T11：取得同版既有 Monitor 注册/可用服务及投递数据，补卡片 loading/ready/permission/retry、pending/quarantine/details 与入口旅程；当前 503 不构成可用数据证明。
3. OE-T09/T10：当前 `colima list` 仍显示三个 VM Stopped；真实旧 worker、Linux 候选与升级/回滚依赖原构建/运行窗口，在本轮不构建/不起服边界内暂停。OE-T12 已接受的本地范围保持，不重复执行。

本轮可执行前沿耗尽；任务账保持 4 Done / 2 Blocked，所阻塞的只是上述依赖，不撤回已接受的文档、隔离修复、工具入口或当前有界用例。

最终收尾核验：四份本次仓库文档链接目标有效、`git diff --check` rc 0；19 项基线仅四份授权文档变化，其他既有文件 hash 全部保持；结果 hash 稳定，11/11 证据 hash 相符；原 PID 3606/3608 和 9801/9811/9821 监听保留。


## Developer adoption sequence

The D-* sequence belongs to the developer-facing outcomes in
[the iteration scope](../product/iteration.md#developer-facing-iteration-order).
It reuses A-/OE- acceptance instead of creating another implementation owner.
The A-* totals at the top of this ledger are unchanged.

| ID | Current work | Status / next action |
| --- | --- | --- |
| D-01 | README pair, source-grounded getting-started guide and safe startup recovery | Documentation and isolated startup verified: pinned dependency install, Web build, Rust build, fresh SQLite owner login and first custom role operation passed. The isolated source snapshot includes current uncommitted changes; this is not proof of a published clean clone. |
| D-02 | A-04 user/role journey | The real custom-role → assigned user → allowed user list / denied management journey is verified. Selected-option recovery, same-record edit refresh, cancel/reopen, successful-empty directories and duplicate-submit cases have bounded evidence below. The remaining A-04 required state matrix is now accepted on paired evidence; final ZCode review and cleanup are recorded in the A-04 closure receipt below. |
| D-03 | A-06/OE core delivery | Same-source isolated Monitor→Admin delivery, full card state/retry and narrow popover, plus current core-shell login/dashboard/navigation are verified. Synthetic gap rendering remains a separately labeled UI fixture. Local OE-T11 acceptance remains valid. Current-fixture Linux native evidence covers core/full packaging, installation, lifecycle, updates/rollback and 18 interruption points; the final 23-case browser gate and ZCode acceptance are pending. Genuine historical-worker OE-A09/OE-T10 remains unverified and was excluded by the user from this run. |
| D-04 | Existing role feature adaptation | Introductory source walkthrough included in D-01; independent developer adaptation has not been sampled. Do not add a new business domain merely to create an example. |
| D-05/D-06 | Department scope / tenant discovery | Deferred implementation; collect concrete use cases and resolve the product boundaries first. |

Each runtime slice starts only after its target, data owner and side effects are
fixed. Stop after the named acceptance closes; an unavailable environment does not
authorize unrelated feature expansion.

### Isolated onboarding and permission acceptance

2026-10-23, basis HEAD `d209eac830391d4f21f7c0bc47bb49e796273f95`
plus the current working files. The user explicitly authorized dependency installation,
Web/Rust builds, ports 9800/9801 and disposable user/role writes in a fresh temporary
SQLite. A task-owned copy excluded existing runtime data, dotenv files, dependencies
and build output. The launch receipt and process open-file inspection bound Admin to
that temporary database; no existing database was used.

Web dependency installation (425 packages), Web build and Rust Admin build passed.
The full-profile Admin process and Web development server supported owner login,
UI creation of `onboarding_reader` with three permissions, and UI assignment and
enablement of an API-created disposable user. The restricted user could read users,
had no create/edit buttons, and reached `/403` when directly opening menu management.
API readback confirmed the exact assignment and permissions; forbidden role creation
and menu listing returned HTTP 403. Deleting the assigned role returned HTTP 400 /
code 10002. Deleting the disposable user and then the role succeeded; readback left
one owner and the three built-in roles. The deleted user's page returned to login.

The browser ran at its observed 1280×720 viewport; the full specified viewport and
bilingual matrix was not exercised. Selected-option retry retention, successful empty
catalog and duplicate-submission cases remain unverified. This receipt does not close
all A-04 cases or the core/Linux delivery gates. Independent developer sampling remains
zero. The running login/dashboard still use operations-oriented copy, so aligning
those entry points with the accepted template positioning remains a product iteration.

Build output reported large Web chunks; the page console reported the existing Ant
Design Drawer `width` deprecation warning. Neither blocked this journey. Evidence is
in ignored `.codex/artifacts/template-adoption/`: source manifest, build logs, launch
receipt, API verification, screenshots and cleanup receipt. Task-owned services, browser
tab and temporary data/build copy were removed after verification; no commit, push or
deployment was performed. Original A-* totals remain unchanged.

### Permission refresh recovery and template entry copy

The current slice adds a visible background-refresh failure notice and Retry to
the existing permission and role pickers. Cached choices remain visible; primary
buttons and submit handlers reject submission while the directory request is
fetching or its refresh has failed. Retrying does not reset dialog fields, search,
or selection. Login and Dashboard now identify the Rust + React admin template in
both languages, including the visible narrow-login card; extension examples retain
their actual availability and existing permissions.

The implementation owner completed the two changes sequentially. The isolated Web
gate passed 178 tests / 985 assertions across 64 files, TypeScript, formatting and
lint (48 warnings in unchanged files, no errors). The final Web build passed with
the existing chunk-size warning. Unchanged Rust backend startup was reused; the
browser served current frontend source through Vite rather than claiming that the
earlier Admin binary embeds the final UI.

Real page verification covered four consecutive synthetic 503 responses for each
option directory, cached selections and disabled submission, followed by removal
of interception and actual GET 200 recovery. A user was created from the recovered
draft; API readback checked its fields, status and selected role, and login checked
the retained password before deletion. These two functional runs used the observed
1280×720 viewport. A separate 390×667 permission run also covered a failed explicit
retry, pending-submit blocking, draft retention, real recovery and no document
horizontal overflow. Login and Dashboard copy were checked in both languages at
1920×1080 and 390×844. Browser targets were closed and the viewport reset to the
observed 1280×720 default. Temporary services were stopped.

Evidence and the frozen source hashes live in ignored
`.codex/artifacts/permission-retry/`. Original harness failures are retained separately
from passing runs. Final independent ZCode review is tracked in
`.codex/reviews/admin-permission-template-*`. ZCode returned PASS with no material
actionable findings; native finish/idle and all 13 reviewed source hashes match the
final candidate. Its independent review did not rerun browser checks. This receipt does not close
the remaining successful-empty, duplicate-submit, complete edit/bilingual state
matrix or core/Linux delivery gates of the broader A-04/OE work.


### D-02 edit-session completion and connected Dashboard examples

Owner: current Codex coordinator with the existing single Luna implementation owner.
Authority: the user selected management-template positioning, requested continued
implementation/acceptance and ZCode review, then explicitly requested hiding examples
that have not been connected. The approved temporary-copy / 9800–9801 / fresh-SQLite
window was reused. Basis: HEAD `d209eac830391d4f21f7c0bc47bb49e796273f95` plus the
21-entry final working-file manifest in ignored `.codex/artifacts/edit-session/final-basis.json`.

The edit session now initializes once per open record identity. An actual changed
server record and list GET refetch preserved role draft fields, search and selected
permissions, and user draft fields and role selections. Cancel/reopen read the latest
record. Saving the user draft independently confirmed its retained email, real name
and custom-role assignment. Browser email values were privacy-redacted, so persistence
readback owns that assertion. Rapid click/Enter attempts produced one create POST
for each resource. The assigned user could list users, saw no create/edit controls,
reached `/403` for menu management, and received HTTP 403 for direct role-create/menu
requests. Assigned-role deletion returned HTTP 400/code 10002 without data loss.
The disposable user and role were removed; one owner and three built-in roles remained.

Successful-empty permission and role directories were injected as exact synthetic
HTTP 200 `[]` responses through Forgeway temporary interception. Both disabled
submission. After revocation, fresh page queries read real populated HTTP 200 results;
all controlled requests/timers settled and their task tabs closed. These are Mock UI
states, separate from the real role/user journey.

Dashboard now returns and renders only enabled examples with a current validated
Manifest or durable reconciled module-navigation registration. A previously connected
offline example keeps its unavailable state; never-connected and disabled examples
are omitted. An empty result hides the complete region and lets resources fill the
grid. Fresh final Admin/SQLite returned real `[]`; the actual page matched in English
and Chinese at 1920×1080 and at 390×844 without horizontal overflow. Partial connected,
offline, disabled and restart-registration assertions are Rust service + SSR evidence,
not live-module browser verification.

The final formatted frontend bytes passed 179 tests / 995 assertions / 65 files,
TypeScript, formatting and lint (48 existing warnings, none in changed files). Three
focused Rust tests, Rust formatting and Clippy passed. Final isolated Web/Admin builds
passed. Two formatter-only differences initially left in the isolated copy were
reconciled back to the canonical files; all final canonical/isolated/review hashes
match. Source-string assertions were replaced by rendering checks; the real Dashboard
GET and page acceptance remain separate route-wiring evidence.

ZCode review: the full frozen round returned PASS with no P0–P2 issue. Its stale owner
reference and initial-error wording observations were fixed; the suggestion for a
source-string wiring assertion remains nonblocking and was declined in favor of
behavioral evidence. Final same-session confirmation of the five formatting/document
deltas completed in the original native session with PASS (idle/stop, exit 0); all
21 final file hashes match. P3-1/P3-3 are closed; P3-2 remains a nonblocking coverage
observation, accepted without an extra source-string test requirement. Review reports and reconciliation are
held in ignored `.codex/reviews/admin-edit-connected-*`; runtime and cleanup receipts
are in `.codex/artifacts/edit-session/`.

This closes the named D-02 journey and current recovery/edit acceptance slice. The
original A-04 complete bilingual/edit/error matrix and A-06/OE core/Linux gates remain
with their original owners and recovery conditions; A-* totals remain 4 Done / 2 Blocked.
The next planned developer outcome remains D-03 under those existing gates. No commit,
push or deployment was performed.


### A-04 final acceptance closure

2026-10-23. The existing A-04/A-AC04 user/role outcome is Done after the original
single owner completed the remaining cases and the primary reconciled all 14
coverage areas with the two original Product/UI feature contracts. Paired coverage
combines real API/UI evidence, explicitly labeled synthetic 403/503/empty inputs,
and unchanged shared-source paths; it does not claim every state in every
mode/language/viewport. Both English and Chinese and the specified desktop/narrow
sizes are represented. Source/tests were unchanged during this acceptance pass;
only this ledger changed. The prior 179 Web tests / 995 assertions are reused on
verified unchanged inputs; fresh isolated Web/Admin builds also passed. Role-rule
unit-test source inspection is not reported as a newly executed test.

Real requests verified built-in/reserved protection, owner-only and covering-wildcard
create/update refusal, zero permissions/roles, assigned-role deletion protection,
and same-token capability removal after disabling a role while preserving its
assignments. UI cases verified edit draft/status/search/selection retention through
refresh failure and retry, initial failure versus successful empty, disabled submit,
record switching/create reset, bilingual deletion guidance and bounded submit/cancel
behavior. Synthetic recovery used captured real payloads and remains Mock evidence;
separate unmocked GETs and primary page consumption confirmed populated permission
and role options.

ZCode independently verified the 1286-file snapshot
`8d6d13c03fec9f2e8314375d3d12b36a7edf970a909301f011685cae800b857e` and
returned PASS with no P0–P2 or mandatory A-04 gap. Three nonblocking observations
remain recorded: old delta-only AX artifacts are not standalone full-state proof;
refresh was triggered through development Query Devtools; illegal status0 still
returns a pre-existing backend500 outside this acceptance. The primary read back
native session/model/idle/stop/exit evidence and inspected screenshots separately;
the reviewer did not rerun runtime cases or inspect image pixels.

Evidence and precise limitations: ignored `.codex/artifacts/role-user-acceptance/`
(final coverage, primary reconciliation, real/Mock receipts, hashes and cleanup),
plus `.codex/reviews/admin-a04-acceptance-result.md` and its verification. Test users
and roles were removed with one owner/three built-ins preserved; task browser
contexts closed, primary signed out, temporary viewport reset, ports9800/9801 freed
and the task copy/database/build directory removed. No existing database, commit,
push or deployment was changed. A-* totals are 5 Done / 1 Blocked. D-03 and A-06
retain their original core/Linux/OE owners and runtime gates; developer sampling
remains zero.


### D-03/A-06 current receipt — 2026-10-23

Basis: isolated copy of HEAD `d209eac830391d4f21f7c0bc47bb49e796273f95` plus
all 1,222 tracked and nonignored untracked source paths; source digest and
path inventory are in ignored `.codex/artifacts/core-delivery/source-manifest.json`.
No preexisting ignored key, database, runtime data, or build output was copied.
Bun frozen install, full Web build, and locked Rust Admin/Monitor build passed in
the isolated copy. Admin/API ran at `127.0.0.1:9801`, Monitor at
`127.0.0.1:9802`, and the Admin notification ingress at `127.0.0.1:9811`;
9800 was unused. All use fresh temporary SQLite and generated task-only
credentials, external delivery disabled. No existing database, VM, key, Git
commit, push, or deployment was touched.

Three synthetic agent reports were accepted by the real Monitor API and yielded
four Monitor incidents and four real locally stored Admin ingress receipts with
recipient counts. The real delivery endpoint reported zero pending/quarantine
and a non-null lastSuccess. Owner and restricted-viewer desktop DOM captured the
ready card/details. A separate aggregate-only synthetic fixture (no outbox
payloads or delivery rows) rendered 3 gaps, pending/quarantine values and first/
last-gap timestamps for owner and restricted viewer; it is UI-only evidence.
The temporary aggregate counters were restored to the original healthy values,
with the real lastSuccess preserved and API readback verified. The shared
delivery card popover was fixed for narrow viewports. Its final 390×844
controlled 403 capture measured x=8, width=358, right=366; body and Retry
remained readable. The final 1440×900 controlled 503 state and loading/error/
retry/recovery flows passed; each isolated headless context closed with no
pending requests or timers. Final IAB captures show owner 1440×900 English
gap/details and restricted-viewer 390×844 Chinese gap and healthy details on
the final full binary. Owner healthy recovery was separately confirmed by
native real-API readback and the headless restored-healthy trigger; no healthy
popover is claimed. The aggregate-only gap fixture remains synthetic UI
evidence.

The distinct core composition built its real `dist-core` Web bundle and an
Admin binary with `--no-default-features --features core`, then ran from a
separate fresh SQLite root. Owner login, English 1440×900 dashboard, Chinese
390×844 dashboard/navigation, role-list and audit reads passed. The dashboard
has no extension-module card; navigation lists only dashboard, system and
management. Captured network requests contained no Monitor, Insights, Reports,
or Notifications API path. The current core inventory includes the dashboard
helper and excludes the notification shell.

Core composition verification exposed and fixed two existing route-staging
contract gaps: core did not stage its dashboard helper; portal builds did not
stage `system/-dialog-session.ts` although role/user dialogs import it, and the
inventory verifier omitted that helper from expected routes. The exact source
delta and build/browser receipts are under ignored
`.codex/artifacts/core-delivery/`; initial failures and diagnostic stderr are
preserved separately. After the staging fixes, the original four Web
composition tests passed (4/4, 79 expectations). The core bundle and binary
also built successfully. ZCode independently reviewed the 1,323-file frozen
snapshot and returned PASS with no P0–P2. Three nonblocking documentation
traceability findings were closed in this final ledger transition; code stayed
unchanged. The report is in ignored `.codex/reviews/admin-d03-core-result.md`;
review basis and final documentation hashes are in `final-closure.json` under
the receipt directory above.

The initial Chrome viewport override could not be restored through the
authorized tab-scoped Browser Use capability because the task-bound tab was
no longer reachable; restoration remains Not verified. All task IAB tabs were
signed out/closed and viewport reset. OE-T11 local scope is independently accepted
(ZCode PASS); the Linux 23-case browser gate and OE-T09/OE-T10 real
old-worker, package, deployment and rollback gates remain blocked. Only the
listed source/spec/ledger paths in `d03-source-delta.json` were edited in the
shared checkout; runtime, temporary identities, SQLite databases, and build
outputs stayed inside the isolated task tree. The task runtime, identities,
database, build/source tree and Admin process were cleaned after the parent
copied review inputs; authorized local ports were confirmed free. Sanitized
ignored receipts remain in the repository.


### D-03/A-06 Linux current receipt — 2026-10-24

The authorized task-owned x86_64 Linux window was executed and the native
current-fixture campaign has completed all 18 frozen interruption points,
core/full packaging and fresh installation, lifecycle/reboot, normal/reverse
updates, failure rollback and cross-composition/occupied-root refusal.
The rollback start-limit defect is repaired and natively retested. Existing
user/role and local OE-T11 acceptance is preserved. Exact layer boundaries,
failed attempts, selective database restore oracles, child-identity inference
and source/version overlays are in the composable-distribution implementation
receipt and ignored `.codex/artifacts/core-delivery/linux/`.

The fixed-source Linux 23-case browser gate and local evidence publication
passed (outer exit 0); final independent ZCode review passed on 2026-10-24
(no P0–P2). The dedicated environment was cleaned after evidence retention. The missing genuine preceding-full 0.5.1 signed package and its
old-worker compatibility row were explicitly skipped by the user; they remain
Not verified. A-06 stays Blocked and A-* totals stay 5 Done / 1 Blocked for that
original gate. New fixture versions do not prove historical compatibility.
No commit, push, release, production access or goods VM operation is claimed.
