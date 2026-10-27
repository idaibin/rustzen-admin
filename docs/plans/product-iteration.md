# Admin 迭代方案

## 方案边界

复用现有 Web shell、`crates/auth`、`crates/ipc`、模块 Manifest、原生 API 和各应用 migrations。保持 Admin 与模块的身份、故障、存储和权限 owner，不因本轮审查抽取通用工作流、报表中心或新进程。

| 工作包 | 最小影响面 | 核验方式 |
| --- | --- | --- |
| 当前文档收口 | product/product、index、architecture、composable-distribution 的当前说明 | 以已接受 spec/OE 回执为依据，当前说明只保留终版；历史证据不改写 |
| 隔离启动审查 | `crates/config/src/shared.rs`、`admin.rs`、`scripts/local-acceptance-launch.sh` 与调用方 | 冻结现有 dirty，追踪 dotenv→配置→实际路径→首次写入；先静态，后获准纯合成测试 |
| 验证入口 | justfile、现有无构建子测试、版本声明 | 列明确 build/install/service/db/secret 副作用，再提出开发检查与发布检查分离方案 |
| 权限旅程收口 | 原角色定义/用户指派 feature、对应 UI/API/服务 | 先核对旧证据能否复用，再补具体缺口；不重写 auth 或扩权 |
| 模板采用验证 | 安装说明与现有最小旅程 | 观察真实使用，记录求助/恢复，不用代码体量代替价值 |

## OE 继续路径

原唯一状态入口是 [OE tasks](../product/features/composable-distribution/tasks.md)，执行证据在 implementation，验收定义在 validation。OE-T12 本地安装器范围已按固定源码与历史正负例完成独立接受，Linux 门仍开放；OE-T09 的剩余包范围与 OE-T10 真旧 worker/包的升级回滚相互关联，按原协议推进；OE-T11 单独完成实际 DOM。

不要重新派发 OE-0–2 已接受内容。`new verifier accepts old` 不证明 `old worker accepts new`；macOS 上 stub-systemctl 不证明 Linux 服务生命周期；wire 回执和 seam 测试不证明浏览器效果。新的源码/安装身份变化只重开受影响验收。

## 无构建开发检查方案

`justfile` 保留唯一命令来源。日常检查按下表直接选择已核对的子检查，完整门禁保留其原覆盖；不将完整门简单改名为无构建门。

| 入口 / 调用链 | 实际副作用与验收层 | 当前使用方式 |
| --- | --- | --- |
| `bash scripts/test-verify-services.sh` | shell 语法与源码接线断言；不执行被检查的服务脚本 | 可用于服务接线静态检查；不能证明服务实际运行 |
| `bash -n scripts/local-acceptance-launch.sh`、精确路径 `git diff --check` | 语法、空白，无服务/数据库写入 | 文档与隔离脚本检查；实际隔离另按 A-02 |
| `just check` | frozen Bun install（含 prepare）、fmt/lint/typecheck/tests、Web build、composition build、Cargo check/clippy/test | 完整工程门；本轮不执行；Cargo 检查/测试也会产生编译缓存，不称纯只读 |
| `just verify-web-compositions` | 四个 distribution Web 测试调用 producer；verify-web 测试先构建 full；随后 `build:core` | 明确属于产物门；不能因命令名带 test/verify 当作无构建检查 |
| `just contract-verify` → `verify-contract.sh` | cargo run 生成 OpenAPI 临时文件，再调用 contract-client 生成客户端并格式化，随后测试/typecheck | 合同生成/比较门；不列入只读检查；直接 baseline cmp 仅比较两个已有文件，不证明源码同步 |
| `just verify-services` / `verify-modules-mvp` | 固定 Bun dlx、Cargo build、真实服务/Agent、临时数据及生命周期脚本 | 需要精确运行目标和环境授权 |
| `just verify-cli` | Cargo tests/clippy/build，CLI help/doctor/version；doctor 实际端点依配置 | 不能整条当作无构建 CLI 冒烟 |
| `just verify-local-acceptance-harness-unit` | 既有 Python mock/临时 SQLite 用例 | 选择前读取当次匹配测试；不证明原生服务或新 launcher 路径 |
| `just verify-local-fixture-security-boundaries` | 启动 loopback HTTP/TLS fixture | 有监听副作用，当前禁止起服时不执行 |
| `just test-real-legacy` | 精确选择 ignored 用例后读取真实旧包/信任材料；普通 workspace tests 不覆盖它 | 沿用 OE-T10 专门窗口和输入，不用合成包替代 |
| `just build` → build-config / build-release | 生成配置、从签名私钥派生公钥、Docker 编译、打包、签名；不是只读获取 public-key | 仅已授权产物/发版阶段；本轮未读取私钥 |
| `reset-db` / `clean` | 删除数据库或构建目录 | 不作为失败后自动恢复步骤 |

### A-03 工具链收口步骤

项目 Web 的 `devEngines.runtime` 声明 Node 24.15.0、Bun 1.3.14，`packageManager` 为 bun@1.3.14；依赖锁为 `apps/web/bun.lock`，Rust 声明为 1.95.0。根 package.json 只拥有 cloud preview，不能把它当作 Web 工具链权威。

1. 保留这些声明和锁文件。无构建 Bun 子检查统一显式使用现有固定入口 `pnpm dlx bun@1.3.14 <已核对的子命令>`，在 `apps/web` cwd 执行 Web 检查，在仓库根执行 root 脚本。回读实际版本/cwd/可执行文件；禁止把全局 Bun 版本反写成项目版本来消除差异。`pnpm exec bun` 不是本项目的入口，因为 `packageManager` 选择 Bun。
2. 执行前先核对 `pnpm dlx bun@1.3.14 --version` 与 `node --version`；子命令失败必须传播非零退出码。该入口可能准备 pnpm runtime 缓存，不等于安装项目依赖；不得借它隐式启动服务、执行项目 prepare/install 或 Web producer。版本无法解析时保留失败，不能回退至全局 Bun。
3. 当前复用直接子命令足以完成最小检查，不新增包装脚本。以后确需 `check-dev` 时，只在原 justfile 中组合已证明无构建/无监听的子检查；不照搬全量 `bun test`，先检查每个测试的传递调用。现有 `check` 与 OE 发布门保持不变，其中裸 `bun` 仍依赖调用环境，执行完整门前须重新核对版本。
4. 记录实际 cwd/argv/版本/hash、直接 rc 与覆盖层；仅复测改变的调用方。当前观测和恢复条件见 A-03 任务回执。

## A-02 隔离审查与修复接续

审查表面固定为 launcher、config 的 dotenv 开关/合成测试及其直接调用链。用户已明确授权修复 P2；当前协调者实施 launcher 最小修复和纯合成回归，不修改既有 config 或业务源码。结论不等于可启动真实实例。

路径链：launcher 参数校验 → `mkdir` 新根及子目录 → 写 launch receipt → `exec env -i` → Admin command parse → `load_dotenv_if_present`（非空 skip 则跳过）→ `AdminConfig::load` 合并环境并 validate → logger 创建日志目录 → database pool 创建/打开数据库 → migrations。四个数据库最终路径通过 runtime root 解析；绝对 override 会越出 root，因此 launcher 拒绝继承的四个 override 和 runtime-root 变量。其他服务也调用共享 dotenv 入口，后续修改须覆盖其默认不跳过语义。

| 审查项 | 精确修复/验收方案 | 依赖边界 |
| --- | --- | --- |
| A-02/P2-01：临时目录隔离依赖已有 target/rz | 只解析已存在的候选 base，选中后将物理父路径绑定到该 base；另一 base 缺失不阻断合法路径，不为检查创建目录。保留 symlink、dot-dot、已有根拒绝 | 已实现；自有临时 repo/temp 正例均通过 |
| 失败/取消生命周期 | exec 后 launcher 不再持有子进程生命周期，child 非零后目录与 receipt 保留；receipt 是启动输入记录，不是完成收据。失败/取消保留本次创建的根，由调用方确认子进程结束及目录归属后清理；不提供 supervisor 或后代进程清理保证。覆盖取消、创建中断和同路径并发，不加入自动删除 trap | 合成生命周期已通过；真实服务生命周期仍独立待验 |
| dotenv 回归选择与测试隔离 | 现有回归由 `admin && modules` 条件编译；使用 `cargo test -p rustzen-config --features admin,modules dotenv_reinjection_of_sqlite_paths_reaches_final_paths_unless_skipped -- --test-threads=1` 时必须读回非零实际用例数。现有测试改变进程 env/cwd，仅成功尾部恢复；运行前固定纯合成环境，补 panic/取消后归属验证或独立子进程隔离 | 原 config 源码保持不变；独立 env -i 进程实际执行 1 个用例通过，四库最终配置路径有运行证据；未启动真实数据库服务 |

边界验收至少覆盖：干净 checkout+合法 temp root、repo root、空值、绝对/相对路径、继承 override、dot-dot、symlink、已有根、非法端口、子进程失败、取消、并发。拒绝项检查目标未新增/原数据未改；正例读回实际 child env 和最终配置路径，不只比较 receipt 文本。测试只用自有临时文件和无服务子进程；真实页面验收继续按原固定地址与身份执行。

### 可复跑的隔离检查

仓库根执行 `pnpm dlx bun@1.3.14 test scripts/local-acceptance-launch.test.mjs`。六组测试使用自有临时目录及无监听子进程，检查真实 child 环境、拒绝前无新增、失败保留、SIGTERM、创建中断和原子创建竞争；harness 只清理自己的 fixture。Rust 配置检查使用独立 `env -i`、固定 Rust 1.95.0、`--locked --offline` 和 `--test-threads=1`，详见任务回执。receipt 只记录启动输入；实际配置路径与真实服务写入分别验收。

## A-05 Developer-template adoption sample

This section is the self-contained sampling plan for the current iteration.
It does not require a document outside this repository. Day 1 starts with the
first real sample. The primary participant is a developer
building a business admin application, not an operator evaluating a monitoring
product. The outcome is to run Admin, complete a protected role/user journey and
explain how to adapt that feature.

### Participants and journey

Record whether each participant is the author and their Rust/admin experience.
Start with three attempts; by day 30 include at least three target developers,
of whom at least two complete the core journey without the author operating it.
Author self-tests are separate and repeated attempts do not become new people.

Use a disposable, participant-owned environment. Bind its source/composition,
existing data and allowed side effects before running commands. A supported
full development path can validate Admin behavior but does not establish the
Admin-only release gate. An existing running instance is not a clean install.

| Step | Observable result | Failure / recovery evidence |
| --- | --- | --- |
| Understand and prepare | Participant can identify Admin capabilities, example modules and unsupported data scopes | Record confusing claims, prerequisites and help required |
| First local startup and login | Web assets prepared before Cargo; expected page and fresh local account work | Record missing assets, wrong process/port, configuration or schema failures; preserve existing data |
| One protected feature | Create a custom role, reload it, assign a disposable user, prove an allowed and denied operation | Capture actual service results; hidden navigation alone is insufficient; retain original inputs on recoverable failures |
| Understand adaptation | Participant can locate the route, capability, handler, service, SQL and frontend state owner and describe the change needed for their business | Record where the author must explain or operate; do not invent a new business domain to fill the sample |
| Cleanup and subsequent use | Remove only the owned test assignments/records and observe an independent return to development | Separate prompted follow-up from voluntary reuse; keep unknown outcomes unverified |

Record anonymous participant identity, role/experience, basis, fresh versus existing
instance, attempt number, step times, human work, system wait, help, author
interventions, success/failure/cancellation, cleanup and evidence references. Never
record passwords or tokens. Current independent developer sample count is zero.

First useful outcome is successful readback of the owned role assignment and an
observed access-control result; code adaptation is a separate outcome. Count all
started attempts, including failures, in the completion denominator. Keep blocked
preparations separate, report n/N and time ranges, and do not use stars, clones
or page views as the trial denominator.

Set improvement thresholds from real samples. At the day-30 checkpoint, report
the participant and independent-completion outcomes defined above; longer-term
thresholds remain open until they can be grounded in those results. This plan does not authorize publishing,
recruitment messages, builds, service starts or writes to shared instances.
The existing Monitoring, Analytics, Reports and core/full feature acceptance
commitments remain with their original ledgers.

## 验收和收尾

隔离测试先使用纯合成目录/配置，验证 empty、覆盖、路径逃逸和失败清理；不得读真实 .env/密钥或启动现有服务来试验。需要真实环境的部分只访问项目已声明且匹配身份的服务；无目标时记恢复条件。

当前状态以任务账主表、最新执行回执和原 OE 账为准。V2 实例与较早的 DOM/API 记录保留为历史证据，不能替代之后的同源候选验收。

A-04 最新回执已记录剩余权限、编辑和恢复矩阵通过并获独立接受。A-06 对应当前源码 fixture 的 Linux 原生范围、18 个中断点和 23 项浏览器门已于 2026-10-24 在原 OE 账记录接受；真正上一版 full 0.5.1 原始签名包及旧 worker 兼容仍未验证。新源码或信任材料变化只重开受影响项，不复跑无变化且证据仍有效的阶段。

权限 UI 按 feature 合同覆盖两种语言、loading/empty/error/retry、原输入/选择保留、重复提交、角色撤回和服务端拒绝。各层分开记录，授权/数据副作用不由截图推断。

最终在原 OE/交互等账本回写其自身状态；本轮新增账只管理非重复的文档、入口、证据盘点和采用任务。收口后输出有限的剩余队列，不因缺 Linux 验收扩写其他模块。构建、私钥使用、安装、Git 与部署仅按精确已有授权执行。
