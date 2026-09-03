# OpenCanon

确定性文档原子库。agent 驱动并调用 LLM；本仓 Rust 只做校验、流转、落盘与确定性计算。

产品树（本仓）与真源树不是同一棵。真源是被治理项目 cwd 下 `opencanon/atoms/` 里 `status: active` 的文件。`skills/` 只随产品发布，不写入使用方 `opencanon/`；`init` 按同名覆盖安装到 `.agents/skills/`。

## 改哪里

读对应模块的 `AGENTS.md` 再动手。规则只允许有一处发生地；发现副本则删掉，留给 `canon-core`。

| 要动的 | 打开 |
|--------|------|
| 校验、状态机、id、字段合并、过滤、确定性算法 | [`crates/canon-core/AGENTS.md`](crates/canon-core/AGENTS.md) |
| 原子路径、md 键序、原子写盘 | [`crates/canon-store/AGENTS.md`](crates/canon-store/AGENTS.md) |
| 信封、退出码、clap、命令接线 | [`crates/opencanon/AGENTS.md`](crates/opencanon/AGENTS.md) |
| 流程步骤与人审卡点 | [`skills/AGENTS.md`](skills/AGENTS.md) |

对调用方稳定的是：子命令名、stdin JSON、stdout 信封、`error.code`、原子文件形状。`ops` 内部算法、临时文件名、clap 写法可换，不得改变上一句。

新依赖先过：零 LLM、零 http。第二存储出现之前不抽仓储 trait。MCP / HTTP 是新 crate，依赖 core + store，不复制 clap。
