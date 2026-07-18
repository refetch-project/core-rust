# Refetch 路线图

> 状态：维护者工作文档（非规范）
>
> 最近更新：2026-07-18
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

Adapter、Analyzer、Feed Lab、Flutter/PiliPlus Host 和其他语言实现必须独立演进。它们不能提前侵入 Foundation Core。

## 3. 当前基线

| 项目 | 当前值 |
| --- | --- |
| Concept spec version | `v0.1` |
| Locked Concept commit | `a49e51bbfd04462398bbb7ea613f003b2c417544` |
| Foundation revision | `v0.1.2` |
| Rust workspace version | `0.1.0` |
| `origin/main` baseline | `3621484e2d14be090bf0fcc6782a0479de40001f` |
| 当前工作分支 | `fix/v012-conformance-runner` |

版本含义必须分开：

- `specVersion: v0.1` 是跨语言 JSON 契约版本。
- Foundation `v0.1.2` 是当前锁定规范修订。
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

### 当前工作分支，待人工审查

- [~] 修复 invalid fixture wrapper 被整体反序列化导致的静默跳过
- [~] 按路径排序、发现并执行恰好 15 个 invalid fixtures
- [~] 精确匹配 `expectedError` 与实际 `RankError` 或声明的 slate 差异
- [~] 拒绝未知 JSON 字段
- [~] 补齐主要 Schema 输入约束与结构化错误路径
- [~] 修复 `Fixed6` 六位小数和科学计数法边界
- [~] 硬化 snapshot verifier、sync script 和 CI locked commands

最近一次本地验证结果（未提交工作树）：

```text
invalid fixtures discovered: 15
invalid fixtures executed: 15
Rust tests: 28 passed, 0 failed
snapshot verification: passed
fmt: passed
clippy -D warnings: passed
workspace release build: passed
```

这些结果只描述当前本地工作树，不代表已经合并或发布。

## 5. Now：Foundation v0.1.2 conformance 收口

当前唯一主线是让 Rust Core 与锁定 Concept 契约可信一致，不扩张产品功能。

### 5.1 当前修改人工审查

- [ ] 检查 conformance runner 是否真实执行每个 fixture 一次
- [ ] 检查新增验证是否完全来自锁定 Schema/RFC，而非 Rust 自创语义
- [ ] 检查 `Fixed6` 是否保持跨语言十进制行为
- [ ] 检查依赖增加是否与标准解析需求相称
- [ ] 决定当前大 diff 是否拆成独立提交切片

完成门槛：审查者能够逐项解释每个行为对应的规范来源，并确认没有修改 snapshot、expected output 或排序公式。

### 5.2 JSON 与错误边界闭环

- [ ] 为 CLI 增加端到端成功和失败测试
- [ ] 覆盖顶层与嵌套未知字段
- [ ] 覆盖 ID、token、version、date-time、URI、范围、非空和唯一性
- [ ] 覆盖 `Fixed6` 正负边界、六位精度、指数形式、溢出和 round-trip
- [ ] 确认所有失败都包含可定位路径和实际错误

完成门槛：合法输入不会被拒绝，非法输入不会静默通过，同一错误在重复执行时稳定一致。

### 5.3 Snapshot 与 CI 闭环

- [ ] 使用干净、处于锁定 commit 的真实 Concept checkout 验证一次完整 snapshot sync
- [ ] 确认同步前后 manifest 和文件集合完全一致
- [ ] 决定是否固定 Rust toolchain 版本
- [ ] 决定是否将 GitHub Actions 固定到 commit SHA
- [ ] 保持所有 Cargo 验收命令使用 `--locked`

完成门槛：本地与 CI 执行相同验收链，snapshot 更新只能通过显式、可审查流程发生。

### 5.4 Foundation release readiness

- [ ] 明确 Rust crate release version 与 Foundation revision 的对应关系
- [ ] 准备 release checklist
- [ ] 确认没有未解决的规范空缺
- [ ] 清理已合并的远端 Codex 临时分支
- [ ] 在人工确认后创建 Foundation 对应 tag/release

完成门槛：可以从干净 checkout 离线重现所有 expected outputs，并明确说明已验证和未验证内容。

## 6. Next：冻结数据产品验证准备

只有 Foundation conformance 收口后，才进入本阶段。

### 6.1 样本集

- [ ] 至少 20 条 GitHub 候选
- [ ] 至少 20 条 RSS 候选
- [ ] 每个主要字段都有真实使用案例
- [ ] 每条 Signal 都能回溯到 Evidence
- [ ] 样本冻结，可重复执行，不依赖实时网络

真实样本不得写入锁定的 `tests/spec/v0.1/`。开始前必须决定独立数据目录或独立仓库的所有权和更新规则。

### 6.2 Lens 与人工预期

- [ ] 定义 3 个任务差异明显的 Lens
- [ ] 每个 Lens 准备人工预期 Top 10
- [ ] 记录排序理由和争议项
- [ ] 验证 Lens 切换改变实际筛选结果，而不只是改变文案

### 6.3 Feed Lab 最小实验

Feed Lab 是第一个产品实验，只消费冻结输入并展示 Core 输出。它不是实时爬虫、AI Demo 或通用 UI 框架。

需要验证：

- 用户找到有价值内容的时间是否下降
- 重复和低价值内容是否减少
- Lens 切换是否产生有意义的排序变化
- 用户能否理解、预测和调整结果
- 不使用 AI 时，规则系统是否仍有价值

完成门槛：得到可比较的数据和用户反馈，而不是只有视觉演示。

## 7. Later：验证后才允许进入的方向

以下内容必须建立在 Feed Lab 结果上，不能与 Foundation 并行堆叠：

1. Adapter 契约与 GitHub/RSS Adapter
2. 规则型 Analyzer
3. 可选 AI Analyzer
4. 更多 Lens 编辑与调试工具
5. PiliPlus 作为第二个 Host 验证
6. 其他语言实现或 SDK

App Semantic Contract、MCP、AG-UI、A2UI、AppFunctions 和 App Intents 属于上层长期研究，不进入当前 Core 路线图的交付主线。

## 8. 当前非目标

除非任务明确授权，否则不实现：

- 实时爬虫、平台登录或平台专属 Core 分支
- 模型调用和 Prompt 框架
- 云同步、账户、数据库或遥测
- Flutter/PiliPlus 集成
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

开始修改前必须记录：

1. 起始 HEAD、目标分支和工作树状态
2. 对应的规范来源
3. 本轮假设
4. 允许修改的路径
5. 明确非目标

完成后必须记录：

1. 修改文件
2. fixture 发现数与执行数
3. Rust 实际运行测试数
4. 每条验收命令的成功或失败
5. 未解决问题
6. 完整 diff stat

遇到规范未定义的问题时停止并报告，不得为了 Rust API 便利自行发明跨语言语义。

## 11. 下一项具体工作

在继续增加代码前，先人工审查 `fix/v012-conformance-runner` 当前未提交 diff，并决定拆分方式。推荐审查顺序：

1. conformance runner 与 15 个 invalid fixtures
2. `Fixed6` 十进制行为
3. Schema validation 与 `RankError`
4. 新增标准解析依赖
5. snapshot scripts 与 CI
6. README 版本说明

审查完成前，不开始 Feed Lab、Adapter、Analyzer、PiliPlus 或 Concept v0.1.3 同步。
