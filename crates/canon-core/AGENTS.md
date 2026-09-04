# canon-core

领域规则与纯计算的唯一位置。时间戳由调用方注入。本 crate 不读文件系统、时钟、环境变量。

## 落点

| 要做的 | 写这里 |
|--------|--------|
| 原子形状、title/body/slug 不变量、新字段 | `model/`（YAML 键序在 store，不在这里） |
| 合法 `(from, to)` | 只改 `lifecycle` 一张表 |
| 命令语义（强制 draft、id 分配、edit 合并、转正戳记、按状态过滤、compose 校验派生文档） | `ops/` 先长纯函数，CLI 再接线 |
| 切块、指纹、查重召回、查询 | `compute/`（目录可新建）。算信号，不改原子身份与状态 |
| 新鲜度因素、合成、权重总表 | [`compute/freshness/AGENTS.md`](src/compute/freshness/AGENTS.md) |

`ops` 改变或筛选原子，或校验一条命令的写入值（含 `compose`）；`compute` 只从已有值计算。同一条规则若在 store / CLI / skill 里再写一遍，删副本、留这里。`compose` 不进 `compute/`。

## 代码读不出来的契约

- `slug` 只用于 `add` 作为 id，不是独立持久字段，不另进 frontmatter。`id` 等于 slug；文件名等于 id。改 title 不改 id。
- slug 形状见 `model/slug.rs`：非空；1–32；允许 `_` 作词分隔；首尾不可空白、`.`、`_`；不含 `<>:"/\|?*`。英文蛇形是调用面默认，写在 skill，core 不强制 ASCII-only。
- 全状态占用同一 slug 则 `Error::SlugConflict`（一次带全量冲突）。无 `ATOM-` 前缀。
- `list` 与 `query` 共用 `ListFilter`：默认 / `Active` = 只真源；`Status(x)` = 只该状态；`All` = 三种都在。`query` 不再自写语料过滤。
- `query` 命中：keyword 对 `id` / `title` / `tags` / `body` 做大小写折叠子串。任一场命中即入选。
- 只有 `status == active` 是真源。draft / deprecated 与真源同目录，靠 `ops` 过滤，不靠分子目录。
- Atom 无 `source`、无独立 `keywords`（并入 `tags`）、无 manifest。`freshness.impl-path` 指向活实现（一条或多条路径），不把源文档或代码拷进原子。
- `Deprecated` 回真源：删除占用该 slug 的文件后重新 `add` 走审。不提供回流。

## 测试

对着 `model` / `lifecycle` / `ops` / `compute` 的公开函数写。无 IO。不断言磁盘 md 键序，不断言 CLI 信封。core 失败 = 规则坏了。
