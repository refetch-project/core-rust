# Refetch Core 操作与收口协议

> 状态：维护者工作流程（非规范）
>
> 适用仓库：`refetch-project/core-rust`
>
> 规范优先级与产品方向见 [`ROADMAP.md`](ROADMAP.md)。本文件只定义如何安全地开始、执行、验证和交付一次任务。

本文用于防止两类问题：测试或文档看似完成但证据不足，以及任务在“继续完善”过程中无意扩张到新的规范、仓库或产品范围。

## 1. 权限与范围锁

每轮任务必须先写清楚以下七项，未写明的高风险动作一律视为未授权：

```text
目标：
起始 HEAD / 分支：
规范来源与锁定 revision：
本轮假设：
允许修改的仓库与路径：
明确非目标：
允许的 Git / 远端动作：
```

### 1.1 连续执行与统一报告

范围锁建立后，授权作用于该任务的完整范围，而不是只作用于下一次编辑或下一条命令。

- 已经属于允许路径和目标的本地实现、测试、缺陷修复及文档同步应连续完成，不在每个阶段重复请求审核。
- 如果任务已经明确授权一组本地提交，可以按事先约定的职责边界连续创建，不需要逐个提交再次确认。
- “待人工审查”等状态用于说明证据级别，不自动构成停工门槛；只有任务、release 流程或维护者明确把它设为门槛时才暂停。
- 完成后统一报告本轮完成内容、真实验证、查出的缺陷、尚未验证边界和推荐的下一步。

只有出现以下情况才中断连续执行并请求决定：

- Concept、RFC、Schema 或 fixture 存在无法从锁定规范消解的冲突；
- 完成目标必须新增仓库、路径、规范 revision 或产品层级；
- 需要删除、覆盖、stash、reset、rebase、merge 或其他可能改变既有历史和用户数据的动作；
- 需要 push、PR、tag、release、发送外部消息或产生其他远端影响；
- 起始基线或远端历史在执行期间发生变化，继续工作会混淆所有权；
- 同一阻塞已经无法通过安全的本地检查和替代方案消除。

授权按动作判断，不按“为了完成目标可能需要”推导：

| 已获得的授权 | 不自动包含 |
| --- | --- |
| 检查或审查 | 修改文件、提交、远端操作 |
| 修改一个仓库 | 修改相邻仓库或实验目录 |
| clone / fetch | 修改上游、切换产品边界 |
| “不要使用某目录” | 移动或删除该目录 |
| “继续 / 完善 / 优化” | 升级规范、扩大路径、增加功能 |
| 本地验证 | 人工确认、提交、推送、合并、发布 |
| 提交 | 推送、创建 PR、合并、tag、release |
| 推送 | 创建 PR、合并、删除分支 |

需要分别明确授权的动作包括：

- 修改 Concept、PiliNara、实验仓库或本仓库范围外的文件；
- 修改 `SPEC_LOCK.json`、`tests/spec/**`、expected outputs 或排序/选择语义；
- 对真实 index 执行 `git add`，以及 commit、push、PR、merge、tag、release；
- 删除、移动、stash、reset、clean、rebase 或其他可能隐藏/丢失现有修改的操作；
- 新增网络、AI、Adapter、WASM、Flutter/FFI、Feed Lab 或平台专属行为。

若用户要求的结果确实依赖其中一项，但没有授权，先完成所有安全的只读检查，然后报告阻塞点。

## 2. 启动门槛

### 2.1 基线检查

进入任务后先记录真实输出：

```bash
git status --short --branch
git remote -v
git rev-parse HEAD
git symbolic-ref --short -q HEAD
git rev-parse origin/main
git merge-base HEAD origin/main
cargo --version
rustc --version
```

任务依赖最新远端状态时才执行 `git fetch origin`，并记录 fetch 后的 `origin/main`。离线或禁止网络的任务必须明确写出没有刷新远端。

必须确认：

- `origin` 指向预期仓库；
- 当前不是 detached HEAD 或应当废弃的临时分支；
- 预期基线是 `origin/main` 或其可解释后代；
- 工作树是否干净，以及每个已有修改属于谁。

任务给出预期祖先时，必须运行 `git merge-base --is-ancestor <expected> origin/main`。返回非零表示远端历史不满足任务前提，应停止并报告，不能自行 rebase、替换锁或寻找“差不多”的 revision。

### 2.2 脏工作树

已有修改默认属于用户，不能视为可整理的临时文件。

- 若本轮必须基于这些修改工作，只改允许路径，并在结束时区分原有与本轮改动。
- 若本轮与其无关，优先建立基于明确基线的独立 worktree；不得自动 stash、reset、checkout 覆盖或 clean。
- 无法可靠区分所有权时停止并请求决定。

创建 worktree 只解决本地隔离，不授权提交、推送或删除旧 worktree。

## 3. 规范与快照门槛

以下文件是规范边界，不属于普通实现清理：

```text
SPEC_LOCK.json
tests/spec/**
crates/refetch-contract/src/lib.rs
expected FeedSlate
排序和选择公式
```

修改锁或快照前必须同时满足：

1. 用户明确指定并授权目标 Foundation revision 或 Concept commit；
2. Concept checkout 干净且 HEAD 恰好等于目标 commit；
3. 该 commit 与当前锁定来源的继承关系已经确认；
4. 使用仓库同步脚本，而不是手工挑改 snapshot；
5. 同步后验证文件集合和 SHA-256 manifest；
6. 规范快照、Rust binding、实现、测试与产品文档分开审查，后续若授权提交则按职责拆分提交。

Concept、RFC、Schema 和 fixture 出现冲突时，按 `ROADMAP.md` 的规范优先级定位空缺。不得由 Rust API 便利性决定新的跨语言语义。

## 4. Conformance 可信门槛

Conformance runner 必须让“发现”“尝试”和“完成验证”可区分：

- 按完整路径稳定排序 fixture；
- 锁定快照要求的 fixture 数量，零个、少于或多于预期都失败；
- 每个 fixture 恰好分派一次；
- `executed` 只在该 fixture 的声明验证真正运行完成后增加；若还需要表达已经分派但未完成，另用 `attempted`，不得提前增加 `executed`；
- wrapper fixture 先解析明确的 wrapper 类型，再解析其中的 `request`；
- `expectedError` 必须匹配具体 serde 失败类别、`RankError` 变体或声明的精确输出差异；
- `expectedScoreMismatch`、`coverageMismatch` 等输出差异 fixture 必须先成功完成 rank，再与 wrapper 的预期 slate 比较，并拒绝任何未声明的额外差异；
- 结果根据 fixture 内容判断，不得用文件名硬编码；
- 未知 `expectedError`、无法读取、wrapper 解析失败和意外有效输入都立即使测试失败；
- 失败信息包含完整路径、`expectedError`、实际错误、发现数以及截至失败点的执行数。

禁止以控制流方便为由使用会吞错的 `if let Ok`、`.ok()`、`filter_map`、`unwrap_or_default` 或 catch-all 后继续循环。

至少保留以下回归测试：

- wrapper 的 `request` 确实被解析；
- 空 fixture 目录失败；
- 未知 `expectedError` 失败；
- 意外有效的 invalid fixture 失败；
- fixture 数量不符失败；
- 精确差异 matcher 拒绝未声明的额外差异。

如果 fixture 只声明宽泛的 `schema`，实现可以验证为 serde/schema 拒绝，但不能声称已经匹配到某个规范未声明的具体字段错误。

## 5. 实现与产品边界

Core 只消费完整、已标准化的 `RankRequest`，并产生可重放的 `FeedSlate`。平台差异属于 Adapter，非结构化增强属于可选 Analyzer，导出、隐私同意、UI 和网络属于 Host。

因此：

- Core 的通过不等于 PiliNara、Flutter 或其他 Host 已可接入；
- 文件 CLI 可用不等于生产 FFI、实时导出或产品体验已完成；
- synthetic fixture 可用不等于真实数据、隐私约束或用户价值已验证；
- Lens 输出不同不等于用户效率已经提高；产品结论必须来自冻结样本和预先定义的评估协议。

平台实验应使用独立目录或仓库，不得把真实样本写入锁定的 `tests/spec/**`，也不得把 Bilibili、GitHub 或 RSS 条件分支放进 Core。

## 6. 文档与测试声明

文档中的强声明必须和可执行证据一致：

| 文档声明 | 最低证据 |
| --- | --- |
| “接受合法输入” | 至少一个正向测试 |
| “拒绝某类输入” | 对应负向测试及可定位错误 |
| “隐私字段已阻止” | 每类禁止字段的负向测试 |
| “与锁一致” | 实际比较 lock commit / version 的测试 |
| “可重放” | 同一冻结输入重复运行结果一致 |
| “全部 fixture 已执行” | 发现数、执行数、锁定数一致 |

只有 Schema 中有一个布尔字段、README 中写了要求或 validator 能正常退出，都不足以证明对应保证成立。暂时没有自动化证据时，应写“要求”“计划”或“待验证”，不能写“已保证”。

## 7. 验收与机械收口

### 7.1 最后一次修改后的验收

代码、配置、fixture 或脚本每次发生影响行为的最终修改后，重新运行适用的完整验收链。Rust Foundation 的默认链为：

```bash
python3 scripts/verify-spec-snapshot.py
cargo fmt --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --workspace --release --locked
git diff --check
git diff --stat
```

只改 Markdown 时，仍必须运行工作树/范围审计、`git diff --check` 和适用的文档检查；不需要用无关 Cargo 构建制造“验证很多”的印象。

测试报告必须给出每个 test binary 的数量或可复核总数。`cargo test` 退出码为 0 不能代替实际测试数量；过滤后运行的测试不能冒充 workspace 全量测试。

验收必须检查完整输出，而不只查看退出码或最后几行。被跳过的测试、fixture 数量、warning、panic 摘要和子命令失败都应保留在报告或可复核日志中。

### 7.2 当前工作树与真实 index

结束前至少运行：

```bash
git status --short --branch
git diff --name-status
git diff --cached --name-status
git ls-files --others --exclude-standard
git diff --check
git diff --stat
```

必须逐项确认：

- 所有修改都在允许路径中；
- 没有意外 staged 文件、未跟踪生成物、临时目录或凭证；
- 报告没有遗漏 untracked 文件；
- 没有把先前修改误写成本轮成果；
- 最终 diff 与最后一轮验收使用的是同一内容状态。

需要在不改变真实 index 的情况下审计包含 untracked 文件的候选 diff 时，可使用独立临时 index：

```bash
(
  audit_dir="$(mktemp -d)"
  audit_index="$audit_dir/index"
  trap 'if test -e "$audit_index"; then rm -- "$audit_index"; fi; rmdir -- "$audit_dir"' EXIT
  GIT_INDEX_FILE="$audit_index" git read-tree HEAD
  GIT_INDEX_FILE="$audit_index" git add -A -- .
  GIT_INDEX_FILE="$audit_index" git diff --cached --check
  GIT_INDEX_FILE="$audit_index" git diff --cached --stat
)
```

这只是只读式审计候选内容，不授权操作真实 index 或创建提交。

## 8. 完成状态词典

状态必须逐级报告，不能合并。这些词描述已经取得的证据，不要求在每一级停下来等待审核：

| 状态 | 准确定义 |
| --- | --- |
| 已检查 | 已读取并形成结论，未必修改 |
| 已实现 | 文件已修改，未必通过验收 |
| 本地验证通过 | 最终内容通过已列出的本地命令 |
| 待人工审查 | 尚无维护者明确确认 |
| 维护者已确认 | 维护者已明确接受指定 diff / revision |
| 已提交 | 本地 commit 已创建并报告 SHA |
| 已推送 | push 命令成功且远端分支已核对 |
| 已创建 PR | PR 已真实存在并报告链接 |
| 已合并 | 目标分支已包含该 commit |
| 已发布 | tag/release/artifact 已真实存在 |
| 产品验证完成 | 预先定义的真实样本与评估指标已完成 |

代理、CI 或自动测试不能把状态提升为“维护者已确认”；本地 commit 不能写成“已推送”；Foundation conformance 不能写成“产品验证完成”。

## 9. 提交拆分与远端动作

只有用户明确要求提交时才操作真实 index；一旦同一任务已授权一组本地提交，可按约定边界连续完成，不逐个请求审核。建议按责任拆分：

1. `SPEC_LOCK.json` 与规范 snapshot；
2. Rust contract / validation 实现；
3. conformance 与 Schema regression tests；
4. CLI 及其端到端测试；
5. 集成或操作文档；
6. 产品路线图。

不要为了得到“整洁提交”重写、删除或混入来源不明的已有改动。push、PR、merge、tag 和 release 仍需各自授权，并在执行后核对远端真实状态。

## 10. 最终报告模板

```text
状态级别：已检查 / 已实现 / 本地验证通过 / 待人工审查 / ...
起始 HEAD 与 origin/main：
工作分支 / worktree：
修改文件：
本轮完成：
允许范围审计：通过 / 失败
fixture：发现 N，尝试 N，执行 N（不适用则说明）
Rust 测试：按 test binary 列出并汇总（未运行则说明）
验收命令：逐条列出成功 / 失败 / 未运行及原因
查出的缺陷：
未解决问题与未验证边界：
下一步建议：
git status --short --branch：
完整 git diff --stat（另列 untracked）：
Git 状态：未提交 / 已提交 SHA / 已推送分支 / PR / 已合并
```

如果任何必需验收失败，状态只能是“已实现，验证失败”或“被阻塞”；不得以部分命令成功概括为完成。
