# skills/

agent 的执行规格，编排的唯一位置。改流程只改本目录；改校验、落盘、信封只改 crate。Rust 不读取这些文件。

本目录随产品仓发布，不复制进使用方 `opencanon/`。

## 写一条 skill

只写：步骤顺序、人审卡点、何时调哪条 `opencanon` 命令、该命令的 stdin 模板。字段与 `error.code` 以命令面 serde 类型为准（见 [`crates/opencanon/AGENTS.md`](../crates/opencanon/AGENTS.md)），不另抄一份领域字段表、不发明错误码。

命令还不存在时先在 `canon-core` 的 `ops/` 或 `compute/` 长函数，再接 CLI，最后才让 skill 调用——skill 不能发明 Rust 里没有的子命令。

已有命令、只要新流程：只加 `skills/<name>/SKILL.md`（需要时再加 `references/`），不动任何 crate。`add` / `edit` / `compose` 的 stdin 模板写在对应 skill 里，那是调用面，不是领域字段表。

产品 skill 的 `name` 用 `opencanon-` 前缀，后面跟能力的英文（如 `opencanon-atomize`）。

## 代码读不出来的卡点

- 源文档在命令面之外：opencanon 不读、不写源文件。agent 自读全文；原子化结束后在主张段末写入指向 `opencanon/atoms/<id>.md` 的真源链接，主张正文不动。
- `opencanon/atoms/` 只经 `add` / `edit` 写入；`opencanon/docs/` 只经 `compose` 写入。skill 把结构化 JSON 交给命令，不让 agent 拼 frontmatter 或直接改这些 md。
- 人审是质量闸门（原子化里的 `active`）。转正权在人，skill 把卡点写清楚。
- 语义判定（是否同一事实、是否仍符合实现）在 agent 调 LLM；opencanon 只召回或给信号。
- 真源纯度：消费类命令默认只作用于 `active`，除非流程显式纳入 draft。
