# Refetch Core 项目护栏与验证记录

> 维护者文档。本文不参与首页叙事，用于约束范围、记录反对意见，并在投入扩大之前判断项目是否值得继续。

## 1. 当前工作假说

Refetch Core 的最小工作假说是：

> 对相同候选集，使用用户明确选择的任务 Lens，并提供可追溯的排序理由，可以比单一的时间、热度或平台排序更快地帮助用户找到当前有价值的信息。

## 2. 必须保持的架构边界

- `concept` 是语言无关 Schema 与 fixtures 的规范源；Rust reference implementation 只绑定并执行该契约。
- Core 中不应出现 `if source == github`、`if source == bilibili` 等平台名称分支。平台差异由 Adapter 转换，确实无法通用的字段放入带命名空间的扩展区。
- 推荐理由必须引用排序中实际使用的特征、规则或证据。
- Lens 是用户可查看、可修改、可切换的任务视角，不是隐式人格画像。
- 多样性属于列表，由最终选择过程计算；FeedSlate v0.1 不包含未定义的 exploration 数据。
- `AnalysisRecord` 记录 analyzer 身份，`FeedSlate` 记录 request、Lens 与共享 algorithm id。Rust crate 版本等实现专属信息不得改变跨语言 expected output。

## 3. 基础阶段暂不实现的内容

视频下载、平台登录、推荐模型训练、云同步、插件市场、多语言完整行为实现、MCP、AG-UI、A2UI、完整 App Semantic Contract、大规模 UI 组件库和面向所有内容类型的统一质量真值，都不进入 Foundation v0.1。

## 4. 第一阶段范围预算

GitHub 与 RSS 是 fixtures 和早期验证来源，不是基础 Schema 枚举边界。PiliPlus/Bilibili 等平台必须先通过 Adapter 输出同一契约对象，不能推动 Core 出现平台名称分支。

## 5. Feed Lab / 产品验证前门槛

在进入 frozen-data Feed Lab 或产品价值验证前，至少准备 20 个 GitHub 与 20 个 RSS 候选样本、三个 Lens 的手工预期 Top 10、以及每个主要字段的真实使用案例说明。该门槛不是 Rust binding 开始前的条件。

## 6. 停止、收缩或转向条件

如果 Lens 切换只能改变文案、结构化理由无法回溯、平台特殊字段侵入排序层、固定样本无法重放、或模型费用/远程上传成为体验成立的必要条件，应暂停扩展来源和宿主，先判断是否收缩项目。

## Foundation v0.1.2 guardrail

`concept` is the normative source for language-neutral schema and fixtures. Reference implementations must not add hidden semantics that are absent from RFC 0001 and the v0.1 schemas.
