# OpenCanon — 技术架构

## 0. 定位

**OpenCanon（CLI：`opencanon`）是确定性的文档原子库。** agent 是驱动方与 LLM 调用方；真源只存在于 `opencanon/atoms/` 里 `status: active` 的文件，且只能经 `opencanon` 写入。

| 角色 | 职责 | 不碰 |
|------|------|------|
| `opencanon`（Rust） | 原子的校验、流转、落盘、确定性计算 | LLM、流程编排、源文档 |
| agent | 读 skill、调 LLM、按顺序调命令 | 直接写 `opencanon/atoms/`、改源文档 |
| 人 | 指定源文档、审 draft、决定转正 | — |

---

## 1. 两棵目录树

源码树和被治理的数据树不是同一棵。混在一起会把「工具怎么长」和「真源怎么长」写成同一套规则。

```
产品树（本仓，随 git 版本走）                真源树（被治理项目 cwd，随业务走）
────────────────────────────────────     ────────────────────────────────
crates/     确定性实现                     opencanon/atoms/    全部原子
skills/     流程编排（agent 读）            opencanon/topics/   主题拼接清单
docs/       人/agent 读的说明（kebab-case）
```

| | 名字 |
|--|------|
| 产品 / 仓库 | OpenCanon / `opencanon` |
| CLI 二进制与 crate | `opencanon`、`canon-core`、`canon-store` |
| 使用方数据命名空间 | `<cwd>/opencanon/`（可见目录；给人审、给 git 看，不用点目录） |

- **产品树**回答：工具如何实现、如何扩展命令与流程。
- **真源树**回答：事实存在哪、谁可以写、什么算真源。全部托管数据收在 `opencanon/` 下，避免 `atoms/`、`topics/` 直接铺在使用方仓库根上。
- 真源树不进产品仓。产品仓忽略根上的 `/opencanon/`，避免在工具仓里跑命令得到套娃路径 `opencanon/opencanon/atoms/`。
- `opencanon` 打开的 `root` 永远是进程当前工作目录，没有 `--root`。原子路径由 store 拼为 `root/opencanon/atoms/<id>.md`。
- `skills/` 只在产品树：随 CLI 发布，不复制进使用方 `opencanon/`。
- 源文档（被拆的旧 md）在两棵树之外：命令面不读、不写；agent 用自身读文件能力取全文。
- `docs/` 文件一律 kebab-case，不加 `doc-` 前缀（已在目录里）。当前：架构 `architecture.md`，痛点与思路 `why.md`；决策需要独立演进时再落到 `adr/<nnnn>-<title>.md`。命令与信封以 `crates/opencanon/AGENTS.md` 与 serde 类型为准，不另开契约副本。

---

## 2. 单一性

解耦不是多几个 crate，而是让下面五条各自只有一处可以发生。目录拆分是这五条的物理形式。

| # | 单一性 | 唯一发生地 | 挡住第二处 |
|---|--------|------------|------------|
| 1 | 唯一写入口 | `canon-store` 写 `opencanon/atoms/*.md` | CLI 不拼 md；skill 禁止 agent 改 `opencanon/atoms/`；core 零 IO |
| 2 | 唯一真源 | `opencanon/atoms/` 中 `status == active` | 消费默认走 `ops` 的 Active 过滤；draft / deprecated 同目录但不是真源 |
| 3 | 唯一领域规则 | `canon-core`（`model` + `lifecycle` + `ops`） | store 只翻译；cli 只编排 IO；skill 只写步骤，不写校验 |
| 4 | 唯一编排 | `skills/*.md` | Rust 无 pipeline、无 LLM、不读 skill 文件 |
| 5 | 唯一契约源 | `opencanon` crate 里的 serde 返回类型 | 不维护独立 JSON Schema；命令级测试锁结构 |

若某条规则在 CLI、store、skill 里再写一遍，改一处必漏。发现重复时，删副本、留 `canon-core`。

---

## 3. 分层与依赖

```
opencanon/                      # 产品仓根（仓库名与二进制均为 opencanon）
├── Cargo.toml                  # workspace：三个 crate
├── crates/
│   ├── canon-core/             # 领域：模型 + 规则 + 纯计算
│   ├── canon-store/            # 存储：Atom ↔ 文件，唯一 IO
│   └── opencanon/              # 接口：clap + 信封 + 编排；二进制名 opencanon
├── skills/                     # 流程：agent 的执行规格（不进真源树）
├── docs/                       # kebab-case；见下表
│   ├── architecture.md         # 分层、落点、状态机、扩展通道
│   ├── why.md                  # 痛点与解题思路；不写命令/字段
│   └── adr/                    # 独立演进的决策；需要时再建
└── .gitignore                  # `/opencanon/`（真源命名空间）、target/
```

依赖单向、菱形：

```
opencanon (cli) ──► canon-store ──► canon-core
     │                                ▲
     └────────────────────────────────┘
```

- `canon-core` 不知道文件、不知道 clap。
- `canon-store` 不知道命令名、不知道信封；只认识 `Atom`。
- `opencanon`（cli）认识命令，但不算领域结果：它把 stdin/flag 交给 `ops`，把 `Atom` 交给 store，把结果收进信封。

不为假想的第二存储或第二协议预抽 trait。出现第二个实现再抽接缝。

---

## 4. 产品树模块

### 4.1 `canon-core` — 规则与计算的唯一位置

零 IO。不读文件系统、不读时钟、不读环境变量。需要时间戳的函数由调用方注入。

```
crates/canon-core/src/
├── model/                 # 值：Atom 的形状与结构不变量
├── lifecycle.rs           # 状态机流转表
├── ops/                   # 命令语义：有输入值、无文件
└── compute/               # 确定性计算：切块、指纹、查重召回、拼接、查询、新鲜度信号
```

| 子目录 | 作用 | 扩展时怎么动 |
|--------|------|--------------|
| `model/` | 原子的形状与字段不变量 | 新字段只加这里，并改 store 序列化键序 |
| `lifecycle` | 合法 `(from, to)` 表 | 新状态只改这一张表 |
| `ops/` | 一条命令对应一个纯函数（或一对 validate + apply） | 新命令先在这里长出函数，CLI 再接线 |
| `compute/` | 确定性算法 | 新算法一个模块、一个对外函数；不在这里读 `opencanon/atoms/` |

`ops` 把领域动作藏在函数后面（强制 draft、字段合并、转正戳记、按状态过滤）。CLI 只看到输入值与结果值。

`compute` 与 `ops` 的分界：`ops` 改变或筛选原子；`compute` 从已有值算出信号或派生文档，不改原子身份与状态。

### 4.2 `canon-store` — 唯一碰磁盘的地方

只做翻译：`Atom` ↔ `opencanon/atoms/<id>.md`。不判断 status 该不该变，不合并 freshness，不分配 id。

```
crates/canon-store/src/
├── layout.rs              # 路径约定；日后 topics 路径也只加这里
├── serialize.rs           # frontmatter 键序、kebab-case、缺省
├── io.rs                  # 读 / 写（tmp+rename）/ 删 / 列
└── error.rs
```

公开能力保持浅：打开一个 `root`，对原子做写、读、删、列。列在目录不存在时视为空。读路径永不创建目录；写在需要时创建 `opencanon/atoms/`（必要时先建 `opencanon/`）。

写入约定：同目录临时文件 → fsync → rename。目标已存在时先移走再替换，保证覆盖可移植。

整批 `add` / `edit` 的「先校验再写」不是文件系统事务：`ops` 把整批变成一组 `Atom`，CLI 再逐条写入。崩溃导致部分落盘可接受；不为此引入 journal。需要跨文件事务时再加独立机制。

| 文件 | 作用 | 扩展时怎么动 |
|------|------|--------------|
| `layout` | 路径约定的唯一处 | `opencanon/topics/` 在此加路径 |
| `serialize` | md 模板的唯一处 | 新 frontmatter 键只改这里的键序 |
| `io` | 原子写与目录扫描 | 新实体（topic 文件）加对称的 read/write |

### 4.3 `opencanon` crate — 唯一的进程入口

薄层。允许：解析 argv / stdin、注入 cwd 与本地时钟、调 `ops` / `compute` 与 `Store`、渲染信封、设退出码。不允许：重写 freshness 合并、手写 status 流转、拼 YAML。

```
crates/opencanon/src/
├── main.rs                # clap 树；cwd；dispatch；退出码
├── envelope.rs            # { ok, command, data | error }
├── stdin.rs               # 读 JSON 数组
├── map_error.rs           # core/store 错误 → error.code
└── commands/              # 一命令一文件
```

一条写命令的固定节奏（所有写命令共用）：

1. 解析输入（flag 或 stdin JSON）。
2. 读：需要旧值则 `store` 读单条或列全量。`add` 总是列已有原子（供 slug 占用检查）。
3. 算：`canon-core` 的 `ops` / `compute`（注入 `now`）。任一条失败 → 整批不写。
4. 写：`store` 写或删。
5. 渲染 `data`，退出 0。

| 文件 | 作用 | 扩展时怎么动 |
|------|------|--------------|
| `envelope` | JSON 信封的唯一处 | 信封字段变更只改这里 |
| `commands/*` | 一命令一文件 | 新命令：先 `ops` 或 `compute`，再新文件，再挂 clap |
| `help` / `version` | 非 JSON 出口 | 成功时无信封；clap 用法错误仍走退出码 2 / stderr |

退出码：`0` 成功（有信封，`ok: true`）／ `1` 业务失败（信封含 `error.code`）／ `2` clap 用法错误（无信封）。

命令按职责分组，不按落地批次分组：

| 组 | 命令 | 职责 |
|----|------|------|
| 元 | `--version` `help` | 进程身份与用法 |
| 命名空间 | `init` | 显式创建使用方 `opencanon/`（含 `atoms/`、`topics/`） |
| CRUD | `add` `get` `list` `edit` `delete` | 原子存取 |
| 生命周期 | `active` `deprecate` | 状态流转 |
| 计算 | `chunk` `fingerprint` `dup-candidates` `compose` `query` `freshness-signals` | 确定性派生 |

### 4.4 `skills/` — 唯一的流程位置

agent 的执行规格。Rust 不读取本目录。改流程不改 crate；改校验不改 skill。

```
skills/
├── opencanon-atomize/SKILL.md  # 原子化：读源 → LLM 拆 → query 召回 → LLM 判同 → 提问 → add/edit → 真实的再 active
├── dedup.md               # 查重：召回 → LLM 判同 → deprecate
├── compose.md             # 拼接：查询/按主题 → compose
└── freshness.md           # 新鲜度：信号 → LLM 对照实现 → edit
```

skill 是编排的单一源。命令长什么样以 serde 类型为准；skill 只写步骤、卡点、何时调哪条命令，不缓存字段表、不发明错误码、不让 agent 直接写 `opencanon/atoms/`。

---

## 5. 真源树

真源树在被治理项目的 cwd，不是本仓子目录。`Store` 以进程当前工作目录为 `root`。全部托管数据在 `opencanon/` 命名空间下。

```
<root>/                            # 进程 cwd（被治理项目根）
└── opencanon/                     # OpenCanon 唯一托管区
    ├── atoms/
    │   └── <id>.md                # 一原子一文件；文件名 = id
    └── topics/                    # 主题 → 原子 id 列表，供 compose
```

| 目录 | 作用 | 谁写 |
|------|------|------|
| `opencanon/` | 使用方侧本工具的全部数据 | `canon-store`（及 topic 命令） |
| `opencanon/atoms/` | 全部原子，含 draft / active / deprecated | 仅 `canon-store` |
| `opencanon/atoms/*.md` | frontmatter + 正文 | 同上 |
| `opencanon/topics/` | 拼接用的主题清单 | store 经 topic 相关命令 |

`opencanon/atoms/` 不存在时，读命令当空或未找到，不创建目录。第一次成功写入才创建。`init` 可预先建出命名空间（含 `topics/`），但不把 `skills/` 写进使用方数据目录。

文件内容 = YAML frontmatter + 正文。正文是 `body`，不进 frontmatter。键 kebab-case，顺序固定：`id` → `status` → `title` → `tags` → `freshness`。`freshness` 始终出现；其子键有则写、无则省略；三个都没有时为空对象。

源文档路径、旧 wiki、实现代码不是真源树的一部分。`freshness.impl-path` 只是对照指针，不把代码拷进 `opencanon/atoms/`。

---

## 6. 领域模型与状态机

原子是唯一领域实体。字段语义：

| 字段 | 含义 |
|------|------|
| `id` | 等于 `add` 时的 `slug`；之后永不变（改 `title` 也不改） |
| `status` | `draft` / `active` / `deprecated`；只有 `active` 是真源 |
| `title` | 一句话概括该事实；与 `body` 均非空 |
| `tags` | 分类；`query` 不扫 tags（按 tag 收窄留给 `list --tag`） |
| `freshness` | `last-verified`（上次确认仍成立的本地时间）、`impl-path`（对照实现相对路径）、`score`（机器粗分）；皆可缺省 |
| `body` | 单事实正文，自包含 |

`slug` 只出现在 `add` 入参里，用来作为 id，**不是**独立持久字段，不另进 frontmatter。

### 6.1 原子 ID

`id` 是句柄，不是 title，也不是内容指纹。无 `ATOM-` 前缀。文件名等于 `id`。形状就是 slug：

```
<slug>
```

例：`durability_daily_restore` → `opencanon/atoms/durability_daily_restore.md`

`slug` 由 agent 在 `add` JSON 里传入（默认小写英文词，词间 `_`；工具不再从 title 计算）。校验：非空；1–32 个 Unicode scalar；不含 `<>:"/\|?*`；首尾不是空白、`.` 或 `_`。`_` 允许，作词分隔。不合法 → `VALIDATION_FAILED`，不静默改写。

占用（已占用集合由 CLI 从 store 列出的 `id → status` 注入；`canon-core` 不读盘）：

1. id = slug。draft / active / deprecated 都占这个名字。
2. 本批内部重复、或与磁盘已有 id 相同 → `SLUG_CONFLICT`，整批不写；一次带全量冲突。
3. 改 title 不改 id。`edit` 不改 slug。

是否同一事实由查重流程判断；`add` 只保证 slug/id 唯一。

`add` 忽略输入中的 `id` / `status`，必填 `slug` / `title` / `body`，强制 `draft`。`edit` 对 `tags` 整键替换、对 `freshness` 子键合并，且不可改 `status` 与 `id`。`active` 在流转之外写入 `last-verified` 与 `score = 1`，保留已有 `impl-path`。

状态机：

```
        add                  active                 deprecate
   〇 ───────► Draft ──────────► Active ──────────────► Deprecated
                 │                                        ▲
                 └──────── 审不通过：delete ───────────────┘
```

合法流转仅 `Draft → Active`、`Active → Deprecated`。其余由 `lifecycle` 判为非法流转。`Deprecated` 回真源：重新 `add` 走审，不提供回流。

消费类能力（拼接、查询、查重召回、新鲜度信号）默认只作用于 `active`。`list` 与 `query` 共用状态过滤：省略 = active；`--status draft|active|deprecated`；`--all`。`query` 对 `body` 与 `id` 做子串召回，命中返回完整原子。

---

## 7. 流程编排

Rust 只提供原子能力；流程在 `skills/`，由 agent 按文档执行。四条主流程共用同一套命令面，互不把步骤写进 crate。

```
人 ──指定源/批复──► agent ──读 skill──► 按序调用
                      │
                      ├─ LLM：拆分 / 判是否同一事实 / 对照现状
                      └─ opencanon：校验、落盘、流转、确定性计算
```

### 7.1 原子化（`skills/opencanon-atomize/SKILL.md`）

把一篇多事实源文档变成多条单事实原子。人审在落盘前，不能省。

1. 人指定源文档；agent 自读全文（源文件只读，不经 opencanon）。
2. agent 调 LLM 拆成候选单事实（先不落盘）。
3. `query --all` 宽召回（body 抽词 ∪ 候选 slug）；LLM 判是否同一事实，不准才问人。同则复用，不新建。
4. 提问工具对将要新建的候选判定：真实 / 不确定 / 非真实。非真实不创建。复用且现有正文缺细节时再问是否 `edit`。
5. 新建的按 skill 模板 `add` 为 draft；真实的再 `active`。复用的不 `add`；确认补充则 `edit`。工具不记录与源文档的血缘。

### 7.2 查重（`skills/dedup.md`）

机器宽召回，人/agent 精判。误报成本低，漏报成本高。

1. `dup-candidates` 对 active 原子做字面相似度召回（只召回，不判定）。
2. agent 对每对 `get` 全文，调 LLM 判是否同一事实。
3. 判定为同：`deprecate` 下线一方；判定为不同：跳过。

### 7.3 拼接（`skills/compose.md`）

按主题把多个 active 原子拼成可读文档。纯确定性：不改写、不生成新表述、无需审核。

1. `query` 或按 tag 列出相关原子。
2. `compose` 按 `topics/` 中的清单做引用聚合，产出文档；拼接结果不写回原子正文。

### 7.4 新鲜度（`skills/freshness.md`）

新鲜度无法从文档自身算出，必须对照当前实现。

1. `freshness-signals` 算元数据信号与粗分（距上次修改、实现路径是否仍在、版本控制时间等）。
2. 对低于阈值者，agent 取原子内容与 `impl-path` 指向的实现，调 LLM 确认是否仍符合现状。
3. 仍符合：`edit` 更新 `last-verified`；已过时：人改内容后再 `edit`。

---

## 8. 规则落点

「这件事该改哪个模块」只允许有一行答案。

| 规则 | 落点 |
|------|------|
| title/body/slug 非空、id 与 slug 形状 | `canon-core` `model/` |
| 原子 id = slug；占用则 `SlugConflict` | `ops/id` + `ops` 的 add |
| 强制 draft、忽略输入 id/status | `ops` 的 add |
| tags 替换 vs freshness 子键合并 | `ops` 的 edit |
| `status` 不可经 edit 改 | `ops` 的 edit |
| Draft→Active + 戳 last-verified/score | `ops` 的 activate（内部调 `lifecycle`） |
| 合法流转表 | `lifecycle` |
| list / query 默认只 active；`--status` / `--all` | `ops` 的 `ListFilter` |
| md 键序与 kebab-case | store 序列化 |
| 原子 rename 写盘 | store IO |
| 信封形状、退出码 | CLI `envelope` + 进程入口 |
| 流程顺序与人审卡点 | `skills/*.md` |
| 命令载荷与错误码 | serde 类型 + 命令级测试 |

CLI 编排整批原子性（先 `ops` 全部成功，再写入）算胶水，不算第二条领域规则。

---

## 9. 扩展

每一种变化只走一条通道。新增能力时先问：这是规则、存储、接口，还是编排？

| 要做的事 | 动哪些目录 | 不动 |
|----------|------------|------|
| 新命令（已有数据形状） | `ops/` 或 `compute/` 新函数 → `commands/` 新文件 → clap → 命令级测试 | skill（除非新流程要用它） |
| 新流程（已有命令） | 只加 `skills/<name>/SKILL.md` | 任何 crate |
| 新字段 | `model/` → store 序列化键序 | 在 CLI 特判该字段 |
| 新状态 | 只改 `lifecycle`，再决定是否加命令 | 在多个 `commands/` 里写流转 |
| 确定性算法 | `compute/` 新模块 + 对应命令 | 把算法写进 store |
| 下线 | `deprecate` 命令调已有 `transition`；skill `dedup.md` | 改 activate 兼做下线 |
| 命名空间引导 | `init` + store 建 `opencanon/topics/` | 让 `add` 去建 `topics/`；不要把 `skills/` 写进使用方数据目录 |
| 主题拼接 | layout 加 `topics/`；`compute` 的 compose；store 读写 topic 文件 | 把拼接结果写回原子正文 |
| MCP / HTTP | 新 crate，依赖 core + store | 把 clap 逻辑拷一份 |
| 第二存储 | 此时才抽仓储 trait；文件系统与另一实现两个适配器 | 预先加空端口层 |
| 语义查重 | agent 调 LLM；opencanon 最多存/取向量字段 | Rust 引 embedding 运行时 |

扩展检查：

1. 新规则有没有第二处副本？
2. 新流程有没有在 Rust 里写死步骤？
3. 有没有为假想的第二存储/第二 LLM 抽 trait？

---

## 10. 维护与可测试性

### 10.1 改哪里

| 现象 | 打开 |
|------|------|
| 某条命令 JSON 变了 | CLI 的 serde 类型 + 命令级测试 |
| 转正时 freshness 写错 | `canon-core` 的 activate |
| 磁盘上键序/缺省不对 | store 序列化 |
| 拆分步骤、人审卡点不对 | `skills/opencanon-atomize/SKILL.md` |
| 状态不能从 A 到 B | `lifecycle` 一张表 |
| clap 用法、退出码 2 | CLI 进程入口 |
| agent 解析失败 | 先看出错 `error.code` 是否稳定；禁止让 agent 解析 `message` |

对调用方（agent）稳定的是：子命令名、stdin JSON、stdout 信封、`error.code`、原子文件形状。

对维护者可换的是：`ops` 内部算法、store 的临时文件名、clap 写法。换这些不得改变上一句。id **等于 slug** 且全状态唯一，是契约，不可暗换。

防腐：

- **输出即契约。** 不另维护 Schema 文件。命令级测试断言信封与 `data` 形状。
- **skill 不缓存 schema。** 字段以命令面类型为准。两者开始漂移时再给版本通道，而不是在每条 skill 里抄一份字段表。
- **依赖面零 LLM / http。** 新依赖先过这一刀。

### 10.2 测试对着模块接口写

测试不穿过接缝去断言对方的私有格式。

| 层 | 测什么 | 依赖 |
|----|--------|------|
| `canon-core` | 流转组合；add 强制 draft；id = slug 且占用报冲突；slug 校验；edit 合并；activate 戳记；list/query 过滤；compute 纯函数 | 无 IO |
| `canon-store` | 往返后键序与缺省稳定；tmp+rename；覆盖写；缺目录 list 为空 | 沙盒目录 |
| CLI | 退出码 + 信封 + `data`；每个 `error.code` | 临时 cwd 跑二进制 |
| 流程级 | skill 规定的命令序列（不经 LLM） | 夹具 |

core 失败 = 规则坏了。store 失败 = 文件形状坏了。cli 失败 = 契约坏了。不要用 cli 测试去覆盖 core 已测过的合并表。

---

## 11. 架构决策

需要独立演进（争议、选项、后果）时再落到 `docs/adr/<nnnn>-<short-title>.md`。当前有效：

1. **Rust 不调 LLM。** 语义判定在 agent。CLI 保持确定性、零 LLM 依赖。
2. **流程在 `skills/`，命令在 `opencanon`。** 编排与原子能力分开扩展。
3. **原文件只读；`canon-store` 是 `opencanon/atoms/` 唯一写入口。**
4. **原子全在 `opencanon/atoms/`，用 `status` 区分真源。** 不设 `pending/`。
5. **三 crate，存储单实现，不预置端口 trait。** 第二实现出现再抽接缝。
6. **契约真源 = serde 类型 + 命令级测试。** 不单独维护 JSON Schema。
7. **不追踪血缘。** `Atom` 无 `source`；`keywords` 并入 `tags`；无 `manifest`。`impl-path` 留在 freshness，指向活实现。
8. **命令语义在 `canon-core::ops`，不在 CLI。** CLI 只注入 cwd/时钟并渲染信封。
9. **产品名、CLI 二进制、使用方数据目录统一为 `opencanon`。** 内部 crate 仍为 `canon-core` / `canon-store`；CLI crate 为 `crates/opencanon/`。`skills/` 只在产品树。
10. **原子 id = `slug`。** `slug` 由 agent 传入；文件名等于 id；全状态占用则 `SLUG_CONFLICT`。见 §6.1。
