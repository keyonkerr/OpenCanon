# opencanon（CLI）

进程入口。允许：解析 argv / stdin、注入 cwd 与本地时钟、调 `ops` / `compute` 与 `Store`、渲染信封、设退出码。领域结果在 `canon-core` 算完再交给本层。

写命令的固定节奏：解析 → 读（`add` 必列已有原子以查 slug 占用；`compose` 按 `atoms` 读原子）→ `ops`/`compute`（任一条失败则整批不写）→ store 写或删 → 信封。整批原子性是胶水，不是第二条领域规则。`compose` 的 stdin 是单个 JSON 对象，不是数组。

## 落点

| 要做的 | 写这里 |
|--------|--------|
| 新命令（数据形状已在 core） | `ops/` 或 `compute/` 已有函数 → `commands/` 新文件 → clap → 命令级测试 |
| 信封 `{ ok, command, data \| error }` | 只改 `envelope.rs` |
| `error.code` 映射 | 只改 `map_error.rs`。agent 按 `code` 分支，`message` 不稳定 |
| `--version` / `help` | 成功时无信封；clap 用法错误退出码 2、stderr、无信封 |
| `list` / `query` 状态 argv | 省略 = active；`--status` 取 draft、active 或 deprecated；`--all`；二者互斥。不要 `--status all`，不要 `--include-draft` |
| `init` | 无 argv。无 TTY 时退出码 2、stderr、无信封。成功时走信封。 |
| `freshness [id...]` | 省略 = 全部 active（同 `list`）；指定 id 须存在且 active；argv 去重保序；不读 `OPENCANON_NOW`；CLI 对每个 `impl-path` 注入实现快照后调 `compute/freshness`，`ops::apply_score` 只改 `score` |

契约真源是本 crate 的 serde 返回类型加命令级测试，不另维护 JSON Schema。skill 不抄字段表。

`OPENCANON_NOW`（`YYYY-MM-DD HH:MM:SS`）只为测试注入时钟，不是公开旗标。

退出码：`0` 成功（有信封，`ok: true`）／ `1` 业务失败（信封含 `error.code`）／ `2` clap 用法错误（无信封）。

## 测试

临时 cwd 跑二进制。断言退出码 + 信封 + `data`，每个 `error.code` 至少一次。不穿过接缝去覆盖 core 已测的合并表，不手写 md 当成功路径的断言（文件形状由 store 测）。cli 失败 = 契约坏了。
