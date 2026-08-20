# Refetch 路线图

> 状态：维护者工作文档（非规范）
>
> 最近更新：2026-08-20
>
> 适用仓库：`refetch-project/core-rust`

本文记录 Refetch Rust Core 的长期方向、当前进度、阶段门槛和下一步工作。它不定义跨语言语义，也不能覆盖已锁定的 Concept 契约。

发生冲突时，始终按以下优先级处理：

```text
已锁定的 Concept 契约
    >
RFC 与 JSON Schema
    >
valid / invalid fixtures 和 expected outputs
    >
跨语言一致性
    >
Rust API 便利性
    >
当前实现代码
```

## 1. 项目目标

Refetch 要建立一个开源、可审计、可替换、用户可控的信息筛选层。

核心体验是：

> 同一批内容，通过用户明确选择的不同 Lens，产生不同但可解释的信息视图。

Refetch 不以训练另一个黑箱推荐模型为基础。没有 AI、网络或云服务时，系统仍必须能够完成内容标准化、规则分类、去重、聚类、评分、排序和理由追溯。

目标数据流：

```text
平台或信息源
    ↓
Adapter：转换为统一内容对象
    ↓
可选 Analyzer：补充 Analysis、Signal、Evidence 和 Tag
    ↓
Lens：表达用户当前任务
    ↓
Core：验证、过滤、评分、排序、聚类限制和列表指标
    ↓
FeedSlate：可重放、可解释的结果
    ↓
Host：Feed Lab、桌面、Flutter 或其他客户端
```

## 2. 仓库边界

### Concept

[`refetch-project/concept`](https://github.com/refetch-project/concept) 是语言无关的规范源，负责：

- RFC 和核心术语
- JSON Schema
- valid / invalid fixtures
- expected outputs
- 项目护栏和已知限制

### Rust Core

[`refetch-project/core-rust`](https://github.com/refetch-project/core-rust) 是 Rust 参考实现，只负责：

- 契约类型和可靠 JSON 入口
- 输入与语义验证
- 确定性评分和 Lens 排序
- 聚类限制、Coverage 与 Diversity
- 可追溯 RankingReason
- 离线 conformance tests
- 最小 CLI

Rust 类型是 Concept 的 binding，不能反向定义规范。Core 中不得出现 GitHub、RSS、Bilibili 等平台名称条件分支。

### 后续模块

Adapter、Analyzer、Feed Lab、Flutter/PiliNara Host 和其他语言实现必须独立演进。它们不能提前侵入 Foundation Core。

## 3. 当前基线

| 项目 | 当前值 |
| --- | --- |
| Concept spec version | `v0.1` |
| Locked Concept commit | `823c5303246b467fe9425141c1dcbca92537db28` |
| Foundation revision | `v0.1.3` |
| Rust workspace version | `0.1.0` |
| `origin/main` baseline | `ddcd4f056f968a031c04833f24aff72538da119e` |
| 当前工作分支 | `main`（本地 release readiness） |

版本含义必须分开：

- `specVersion: v0.1` 是跨语言 JSON 契约版本。
- Foundation `v0.1.3` 是当前锁定规范修订。
- Rust crate `0.1.0` 是实现发布版本。

任何一个版本变化都不能隐式改变另外两个版本的行为。

## 4. 当前进度

### 已进入 `origin/main`

- [x] 三 crate workspace：`refetch-contract`、`refetch-core`、`refetch-cli`
- [x] 锁定 Concept snapshot 与 SHA-256 manifest
- [x] 三个 valid RankRequest 和三个 expected FeedSlate
- [x] 确定性 baseline ranking、tie-break、cluster limit、Coverage 与 Diversity
- [x] 最小离线 CLI
- [x] 基础 CI
- [x] v0.1.2 invalid runner 静默跳过修复与 15/15 fixture 计数
- [x] 未知字段、Schema 约束与 `Fixed6` 六位精度验证
- [x] snapshot verifier、同步脚本与 locked Cargo 验收链

### 已进入 `origin/main`：Foundation v0.1.3

- [x] 锁定并同步 Concept Foundation v0.1.3 snapshot
- [x] 按路径发现并执行恰好 36 个 invalid fixtures
- [x] 精确匹配新增 precision 与 Evidence `expectedError`
- [x] Analysis Signal 与 Cluster 支持 Candidate + Analysis Evidence 并集
- [x] Evidence ID 在整个 RankRequest 内全局唯一
- [x] 三个 valid fixture 生成完全一致的 expected FeedSlate
- [x] CLI 成功、确定性、Schema/语义失败和 malformed JSON 端到端测试
- [x] 明确 Rust API、CLI、Host、Adapter 与 Analyzer 的接入边界

最近一次完整验证结果：

```text
invalid fixtures discovered: 36
invalid fixtures executed: 36
Rust tests: 36 passed, 0 failed
Concept fixtures: 3 valid and 36 invalid passed
snapshot verification: passed
fmt: passed
clippy -D warnings: passed
workspace release build: passed
clean checkout acceptance: passed
```

这些结果描述已经进入 `origin/main` 的 v0.1.3 Core；操作规范与 release readiness 仍是本地提交，不代表已经打 tag、发布或完成产品验证。

## 5. Now：Foundation v0.1.3 conformance 收口

当前唯一主线是让 Rust Core 与锁定 Concept 契约可信一致，不扩张产品功能。

### 5.1 v0.1.3 snapshot 与 conformance 本地检查

- [x] 确认 `SPEC_LOCK` 指向经过审查的 Concept v0.1.3 commit
- [x] 确认 snapshot 与该 commit 的干净 Concept checkout 逐文件一致
- [x] 检查 conformance runner 是否真实执行 36 个 fixture，每个一次
- [x] 检查新增 `expectedError` 是否匹配实际 serde 路径或具体 `RankError`
- [x] 检查 Evidence 全局唯一性和 Candidate/Analysis 引用范围是否与 RFC 一致
- [x] 确认三个 expected FeedSlate 完全重现且排序公式未修改

完成门槛：审查者能够逐项解释每个行为对应的规范来源，并确认没有为了实现方便修改 Concept snapshot、expected output 或排序算法。

维护者已确认 v0.1.3 的 Evidence 范围、36 个 invalid fixtures、六位精度测试以及锁定 merge commit `823c5303246b467fe9425141c1dcbca92537db28`。Core 实现已进入 `origin/main`；这些结论仍不等同于 tag、release 或产品价值验证。

### 5.2 JSON 与错误边界闭环

- [x] 为 CLI 增加成功输出和重复执行端到端测试
- [x] 为 CLI 增加 malformed JSON、Schema 错误和语义错误端到端测试
- [x] 覆盖顶层与嵌套未知字段
- [x] 覆盖 ID、token、version、date-time、URI、范围、非空和唯一性
- [x] 覆盖 `Fixed6` 六位精度、指数形式、超精度和 round-trip
- [x] Core Schema 失败包含可定位路径和实际信息
- [ ] 决定未来是否需要机器可读 CLI error envelope；v0.1.3 不把 stderr 文本提升为跨语言契约

完成门槛：合法输入不会被拒绝，非法输入不会静默通过，同一错误在重复执行时稳定一致。

### 5.3 Snapshot 与 CI 闭环

- [x] 使用干净、处于锁定 commit 的真实 Concept checkout 完成 snapshot sync
- [x] 确认同步后 manifest 和文件集合校验通过
- [x] 使用 `rust-toolchain.toml` 固定 Rust `1.97.1`
- [x] 将 GitHub Actions 固定到经过官方仓库核对的完整 commit SHA
- [x] 保持所有 Cargo 验收命令使用 `--locked`

完成门槛：本地与 CI 执行相同验收链，snapshot 更新只能通过显式、可审查流程发生。

### 5.4 Foundation release readiness

- [x] 明确 Rust workspace `0.1.0` 对应 Foundation v0.1.3；本轮不宣称 crates.io 已发布
- [x] 准备 [`RELEASE_CHECKLIST.md`](RELEASE_CHECKLIST.md)
- [x] 当前锁定 RFC、fixtures、expected outputs 与实现之间没有已知未解决规范冲突
- [x] 从干净 checkout 复跑 Concept 与 Core 完整验收链
- [ ] 清理已合并的远端 Codex 临时分支
- [ ] 在人工确认后创建 Foundation 对应 tag/release

完成门槛：可以从干净 checkout 离线重现所有 expected outputs，并明确说明已验证和未验证内容。

## 6. Next：PiliNara 冻结实验与跨来源验证

只有 Foundation conformance 收口、实现版本锁定并从干净 checkout 通过验收后，才进入本阶段。

### 6.1 PiliNara 导出与样本集

- [ ] 为 `pilinara-export.v0` 明确独立实验目录或仓库的所有权
- [ ] 在 PiliNara 独立分支增加仅 Debug、用户显式触发的 allowlist exporter
- [ ] v0 只读取当前已加载的 App 或 Web 单源模型，不处理合并模式
- [ ] 至少采集 3 个批次，去重后不少于 40 条、目标 60 条候选
- [ ] 每个主要字段都有真实使用案例
- [ ] 每条 Signal 都能回溯到 Evidence
- [ ] 公开样本不含凭证、查看者标识或未授权的个性化字段
- [ ] 样本冻结后可重复执行，不再依赖实时网络

真实样本不得写入锁定的 `tests/spec/v0.1/`。开始前必须决定独立数据目录或独立仓库的所有权和更新规则。

### 6.2 Adapter、规则 Analyzer 与 Core 边界

- [ ] 在实验层将 PiliNara export 转换为完整 `RankRequest`
- [ ] 平台字段只进入 Adapter 或显式 `extensions`，不进入 Core 条件分支
- [ ] 规则 Analyzer 版本、配置和 Evidence 全部冻结
- [ ] 使用文件 CLI 生成 FeedSlate，暂不设计 Flutter/Rust FFI
- [ ] 保存 RankRequest、FeedSlate、Concept lock 和实现版本以供重放

### 6.3 Lens 与人工预期

- [ ] 定义 3 个任务差异明显的 Lens
- [ ] 每个 Lens 准备人工预期 Top 10
- [ ] 记录排序理由和争议项
- [ ] 验证 Lens 切换改变实际筛选结果，而不只是改变文案

### 6.4 Feed Lab 最小实验

Feed Lab 是第一个产品实验，只消费冻结输入并展示 Core 输出。它不是实时爬虫、AI Demo 或通用 UI 框架。

需要验证：

- 用户找到有价值内容的时间是否下降
- 重复和低价值内容是否减少
- Lens 切换是否产生有意义的排序变化
- 用户能否理解、预测和调整结果
- 不使用 AI 时，规则系统是否仍有价值

完成门槛：得到可比较的数据和用户反馈，而不是只有视觉演示。

### 6.5 锁定 Concept 的跨来源门槛

当前产品顺序选择 PiliNara/Bilibili 作为第一个 Host 实验，因为 GitHub Host 尚未准备完成。这一顺序只属于非规范产品路线，不改变 Core 契约。

锁定的 Concept 维护者护栏仍记录了至少 20 条 GitHub 与 20 条 RSS 候选的跨来源验证要求。PiliNara 实验不能冒充已经满足该门槛。在宣称 Refetch 已证明来源无关的产品价值前，必须二选一：

1. 补齐 GitHub 与 RSS 冻结样本验证；或
2. 在 Concept 中通过独立、明确审查的新修订调整该维护者门槛，再显式同步 snapshot。

本路线图不能自行修改或绕过这一约束。

## 7. Later：验证后才允许进入的方向

以下内容必须建立在 Feed Lab 结果上，不能与 Foundation 并行堆叠：

1. 通过冻结实验门槛后的 PiliNara 最小 Host 原型
2. GitHub/RSS Adapter 与跨来源验证
3. 可复用的 Adapter/规则 Analyzer 框架
4. 可选 AI Analyzer
5. 更多 Lens 编辑与调试工具
6. 其他语言实现或 SDK

App Semantic Contract、MCP、AG-UI、A2UI、AppFunctions 和 App Intents 属于上层长期研究，不进入当前 Core 路线图的交付主线。

## 8. 当前非目标

除非任务明确授权，否则不实现：

- 实时爬虫、平台登录或平台专属 Core 分支
- 模型调用和 Prompt 框架
- 云同步、账户、数据库或遥测
- 冻结实验通过前的 Flutter/PiliNara 生产集成
- WASM、多语言完整 SDK
- MCP、AG-UI、A2UI 或动态 UI
- 插件市场
- 隐式 Persona 或用户画像
- 未经规范定义的 exploration 算法
- 为测试通过而修改锁定 snapshot 或 expected outputs

## 9. 停止与转向条件

出现以下情况时，暂停扩张并报告：

- 没有 AI 就完全没有基础价值
- Lens 只改变文案，不改变筛选结果
- RankingReason 无法回溯到真实 Signal 和 Evidence
- Core 开始出现平台专属条件分支
- 相同输入不能稳定重放
- fixtures 显示通过但没有实际执行
- Schema 与实现相互迁就
- Feed Lab 退化成普通信息流加摘要
- 用户无法理解或调整结果
- 用户寻找信息的效率没有可测量改善

## 10. 每个任务的工作协议

任务的范围锁、权限矩阵、规范升级门槛、conformance 计数、验收证据、状态词典和最终报告格式统一定义在 [`OPERATIONS.md`](OPERATIONS.md)。

路线图只记录方向、进度和阶段门槛，不以勾选项代替操作协议。遇到规范未定义的问题时停止并报告，不得为了 Rust API 便利自行发明跨语言语义。

## 11. 下一项具体工作

Foundation v0.1.3 repository-only 候选已完成 release readiness 和干净 checkout 重放：Concept validator 通过 3 个 valid / 36 个 invalid fixtures，52 个锁定 snapshot 文件逐一一致，Rust workspace 36 个测试通过，定向 conformance 明确发现并执行 36/36 个 invalid fixtures。

这不代表 crates.io 发布已经就绪。当前 `refetch-core` 与 `refetch-cli` 的 path dependency 缺少发布所需 version requirement，三个 crate 也尚未补齐发布元数据；在决定发布 crate 之前，不为仓库 tag 候选扩大本轮范围。

下一项受控工程增量按以下顺序进行：

1. 将 `pilinara-refetch-lab` 纳入独立版本控制，明确样本所有权、隐私和维护规则。
2. 加固 `pilinara-export.v0` validator 与负向回归测试，保证空目录、锁不匹配、位置断裂、隐私矛盾和未知字段不会假绿。
3. 在 PiliNara 独立分支实现仅 Debug、用户显式触发的 allowlist exporter。
4. 使用冻结导出验证 Adapter 映射；仍不发起新请求、不调用 Core、不加入 FFI、不改变首页行为。

远端临时分支清理、tag、release、push 和 PR 仍需独立授权，不因本地验收通过而自动执行。
