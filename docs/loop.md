# OpenCanon — 闭环

本文写**工作怎么转完一圈**。痛点与思路见 [`why.md`](why.md)。步骤顺序与人审卡点以 `skills/` 为准；命令载荷与 `error.code` 以 [`crates/opencanon/AGENTS.md`](../crates/opencanon/AGENTS.md) 为准。

---

## 0. 闭环是什么

人指定源文档或提出问题；agent 读 skill、调 LLM、按序调用 `opencanon`。真源只存在于被治理项目 cwd 下 `opencanon/atoms/` 里 `status: active` 的文件，且只经命令写入。派生可读文档只经 `compose` 写入 `opencanon/docs/`。

确定性计算归 CLI，语义判断归 LLM，流程编排归 skill。人只在 LLM 无法核对时批复。

```
                    ┌─ init（每个被治理项目一次）
                    │     建 opencanon/、写 locales、安装 skill
                    │
人 ──指定源文档──► atomize ──► active 原子（真源）
                    │              ▲
                    │              │ 同则 edit / 复用，不新建
                    │              │
人 ──要派生文档──► compose ──► 会话成文；按需写入 opencanon/docs/
                    │              （派生，不是真源）
                    │
人 ──问现状/对照──► explore ──► 会话作答（只读；query --all）
```

atomize 与 compose 共用打分召回内核：抽词 → `query` → 对命中里的 active 调 `freshness` → 按落盘分终审。explore 只 `query --all`，不打分、不写盘。新鲜度没有独立 skill、没有独立触发阶段。（[真源：新鲜度须对照代码打分，0.60 再由 LLM 终审且不另开 skill](../opencanon/atoms/freshness_vs_code.md) · [真源：探索：先召回再对照是否仍是现状且不写盘](../opencanon/atoms/explore_read_only.md)）

| 痛点 | 合在哪一刀 |
|------|------------|
| 重复 | atomize：入库前 `query --all`，LLM 判是否同一事实；同则复用或 `edit` |
| 查找（成文） | compose：按问题 `query`（默认 active），LLM 整理成文 |
| 查找（对照） | explore：先召回再对照是否仍是现状；非 active / 低分标明后由人采信 |
| 新鲜度 | atomize / compose 召回时打分并终审；分数闭环见 [`freshness.md`](freshness.md) |

---

## 1. 角色

| 角色 | 在闭环里做什么 |
|------|----------------|
| 人 | 指定要迁的源文档；提问；仅在无法对照实现时批复（是否同一事实、是否仍真实、是否写入现有原子） |
| agent | 读 skill、自读源文件与实现、调 LLM、按序调命令；原子化结束后在源主张段末加真源链接 |
| `opencanon` | 校验、落盘、流转、子串召回、新鲜度粗分 |

agent 把结构化 JSON 交给命令。原子的增删改查只经 `opencanon`；`opencanon/atoms/` 与 `opencanon/docs/` 不手读、不手改。源文档（被拆的旧 md）在命令面之外：CLI 不读、不写。

---

## 2. 一次引导：`init`

每个被治理项目做一次，发生在产品 skill 之前。

人在 TTY 上多选文档语言。命令无参数。成功则：

- 建 `opencanon/`（含 `atoms/` 与 `config.yaml`）
- `locales` 为勾选的 BCP-47（勾了英语则含 `en`）
- 把产品 skill 按同名覆盖安装到 cwd 的 `.agents/skills/`，其它 skill 保留

之后 atomize / compose / explore 都读这份 `locales` 扩词。英语在抽词里默认不能少，与 yaml 里是否勾了 `en` 独立。

无 TTY 则退出码 2，不写信封。

---

## 3. 写入：原子化

触发：人指定一篇源文档（迁 wiki、拆 markdown、写入原子库）。编排：[`skills/opencanon-atomize/SKILL.md`](../skills/opencanon-atomize/SKILL.md)。

把一篇多事实文档变成多条单事实原子。先不落盘；闸门过了再写。工具不记录与源文档的血缘。

### 3.1 拆与合稿（会话内）

按原文抽出领域主张，跨节合并、通写成自包含 `body`。机制 / 规则 / 命令类查找实现，写入 `freshness.impl-path`（相对项目根，可多条）。无代码或配表可对的才省略路径。

完成时：源里每一处主张归恰好一条候选；每条有合法 `slug`（即将来的 `id`）、`title`、`body`、`tags`（英语查询面补集，可为 `[]`）。

### 3.2 先查后写

种子是本批候选的 `title` / `body`、源里的别名、本批 `slug` 与 `tags`。抽词后 `query --all`（含 draft，避免与未审占用漏判）。对命中里的 active 走 §6 的打分与终审。

然后 LLM 用两边的 **body** 判是否同一事实（不以 CLI 粗分为准）：

- `same <id>`：复用该原子，不 `add`
- `different`：按新原子走
- `unsure`：问人

`freshness.score == 0.00` 的 hit 不当现行真源去合并。无命中则本批视为新事实。

这一刀挡住的是「本批候选 vs 本次召回看到的原子」。`add` 只保证 slug/id 不撞，不判断语义是否同一事实。

### 3.3 对照实现

打开候选（及将要合并的 hit）的 `impl-path`，按 body 分块核对文件：

- 一致 → 真实性 `true`（已有原子可 `auto_edit`）
- 不一致 → `false`：不入库、不把错误细节写进真源
- 无路径或对不上 → 问人

### 3.4 只问剩余

全部相关题目有选择之前，不调 `add` / `edit` / `active`。不问已经自动判定的项。

| 问什么 | 何时 |
|--------|------|
| 是否同一事实 | 仅 `unsure` |
| 真实性 `true` / `uncertain` / `false` | 将要新建或复用 draft、且无法自动对照 |
| 是否 `edit` 现有原子 | 同一事实且有新细节、无法自动决定 |

没有判定为 `true` 的（LLM 或人）不转正。`false` 不创建。`uncertain` 可 `add` 为 draft，不加真源链接。

### 3.5 写入

| 判定 | 命令 |
|------|------|
| 新事实且 `true` / `uncertain` | `add`（强制 draft）；`true` 再 `active` |
| 同一事实、有新细节 | `edit`（不改 `id` / `status`） |
| 同一事实、命中是 draft、真实性 `true` | 对该已有 `id` `active` |
| 同一事实、命中已是 active、无新细节 | 不写 |

`SLUG_CONFLICT` 时整批未写：同事实则复用占用方；不同事实则改 slug 再查再试。占用方若需释放文件名，用 `delete` 后重试 `add`。

`active` 写入 `last-verified` 与 `score = 1.00`，保留已有 `impl-path`。

### 3.6 源文档段末链接

主张正文不动。每个含领域主张的段落末尾追加已是 **active** 的原子链接。draft 与未创建不加链。相对路径从源文件所在目录指向 `opencanon/atoms/<id>.md`。

迁完后真源是新权威；旧文只作入口。

---

## 4. 读出：组合

触发：人要一篇可读文档，或要把该文写入 `opencanon/docs/`。编排：[`skills/opencanon-compose/SKILL.md`](../skills/opencanon-compose/SKILL.md)。无需人审（不改真源）。问库里有什么、还成不成立、结合代码看项目现状，走 §5，不走本节。（[真源：拼接：要派生可读文档时按问题召回原子并由 LLM 组合成文，派生文不是真源](../opencanon/atoms/compose_from_atoms.md)）

### 4.1 召回与取材

种子是用户问题全文（问题里出现的原子 id 并进 keywords）。`query` 默认只扫 active。对命中走 §6。

零命中可再扩一轮同义词；仍零则告诉用户库中没有相关真源，不编文。

丢掉不回答该问的命中，以及 `score == 0.00`。成文只用落盘 `score > 0.80`。分数闭环见 [`freshness.md`](freshness.md)。

### 4.2 成文与落盘

LLM 只依据这些原子的 `body`：可调语序、写摘要，不得引入原子里没有的事实，不得反转主张。正文只写散文；`atoms` 声明用到的 id。落盘时命令按该列表在文末生成依据表。

| 人要什么 | 做什么 |
|----------|--------|
| 只要会话里看到 | 展示 `body`，不调 `compose` |
| 要落盘，或要把该文放到别处 | `compose` 校验原子并写入 `opencanon/docs/<id>.md` |
| 写到 `opencanon/docs/` 以外 | 先 `compose`，再在目标文件放链接，不复制正文 |

派生文档无 `status`，不是真源，不写回原子正文。（[真源：拼接：要派生可读文档时按问题召回原子并由 LLM 组合成文，派生文不是真源](../opencanon/atoms/compose_from_atoms.md)）

---

## 5. 读出：探索

触发：人要查库里现有内容、对照代码看主张还成不成立，或结合实现问项目现状。不论问记录还是问对照，都是先召回再对照是否仍是现状。编排：[`skills/opencanon-explore/SKILL.md`](../skills/opencanon-explore/SKILL.md)。零写盘；不调 `freshness` / `edit` / `compose`。要派生文档走 §4。（[真源：探索：先召回再对照是否仍是现状且不写盘](../opencanon/atoms/explore_read_only.md)）

### 5.1 召回

种子是用户问题全文（问题里出现的原子 id 并进 keywords）。一律 `query --all`（draft / active / deprecated）。不按 `status`、不按落盘 `score` 过滤。零命中可再扩一轮同义词；仍零则带着空列表对照代码，不停止。不得跳过召回直接搜代码。（[真源：探索：先召回再对照是否仍是现状且不写盘](../opencanon/atoms/explore_read_only.md)）

专题问题走 `query --all`；库存清单才 `list --all`。不手列 `opencanon/atoms/`。

### 5.2 对照与作答

相关集合由问题决定：专题题只对照回答该问的命中；清单题相关集合可以是全库。对留下的命中打开 `impl-path`，按 body 分块核文件，标一致 / 不一致 / 无法对照（含低分与非 active）。落盘 `score` 只当上次记录。扩圈读到的邻文件是观察，不改三分。（[真源：探索：先召回再对照是否仍是现状且不写盘](../opencanon/atoms/explore_read_only.md)）

会话里先写三层依据（库里记了什么；是否仍是现状；缺口），再写结论直接回答原问；发给用户时结论置顶。现行结论只用 `active` 且 `score > 0.80` 且对照一致的主张；不一致写成已不符。每条用到的原子带上 `id`、`status`、落盘 `score`。`status` 不是 `active`，或无分 / `score <= 0.80`，必须点名并写明未当作现行真源、是否采信由人决定。不因这些标记丢掉该条，不阻塞提问表单。（[真源：探索：先召回再对照是否仍是现状且不写盘](../opencanon/atoms/explore_read_only.md)）

观察只进「是否仍是现状」，不写成真源，也不写入 `opencanon/docs/`。（[真源：探索：先召回再对照是否仍是现状且不写盘](../opencanon/atoms/explore_read_only.md)）

---

## 6. 召回内核（atomize 与 compose 共用）

规格：atomize / compose 各目录下同文的 `references/query.md`。改这一段时两份一起改。种子与是否 `--all` 写在各自 `SKILL.md`。explore 的抽词与调用 `query` 在 `references/recall.md`，与上述三节同文，不含打分与终审。（[真源：新鲜度须对照代码打分，0.60 再由 LLM 终审且不另开 skill](../opencanon/atoms/freshness_vs_code.md)）

不另开新鲜度 skill。打分只传本次命中里 active 的 id；省略 id 会打全库。

1. 读 `opencanon/config.yaml` 的 `locales`；英语 ∪ locales ∪ 种子里的别名，组成 `keywords`。
2. `opencanon query`（atomize 加 `--all`，compose 不加）。命中是完整原子，留在会话。
3. 对命中里的 active：`opencanon freshness <id...>`。CLI 对照 `impl-path` 给粗分并写回 `score`，**只降不升**。无已有分则写入合成值。无 `impl-path` 则 `skipped`，不算分、不写盘。
4. 按信封把分铺回会话命中，再按**落盘分**终审（细则见 [`freshness.md`](freshness.md)）：

| 落盘分 | 含义 | 之后 |
|--------|------|------|
| `> 0.80` | 本轮可用 | 不问 LLM |
| `0.00` | 不可用 | 不当现行真源，不再问 |
| `(0.00, 0.80]` | 尚未终审 | LLM 对照 `impl-path`；判断不了则问人 |
| `skipped` | 无对照路径 | 问人 |

真实：`edit` 写入 `last-verified` 与 `score = 1.00`。不真实：`edit` `score = 0.00`。升分不靠再跑 `freshness`。终审不改 `body`。

`(0.00, 0.80]` 不得带入后续判同或取材。compose 成文用 `score > 0.80`。

因素与合成见 [`crates/canon-core/src/compute/freshness/AGENTS.md`](../crates/canon-core/src/compute/freshness/AGENTS.md)。

---

## 7. 真源树在闭环里长什么样

全部托管数据在被治理项目 cwd 的 `opencanon/` 下。`opencanon` 打开的 root 永远是进程 cwd。

```
<root>/
├── .agents/skills/             # init 安装的 opencanon-atomize / opencanon-compose / opencanon-explore
└── opencanon/
    ├── config.yaml             # locales
    ├── atoms/<id>.md           # 真源候选与真源（文件名 = id）
    └── docs/<id>.md            # 第一次 compose 才出现
```

闭环用到的状态是 **draft → active**（`add` 强制 draft；证据足再 `active`）。审不通过则不创建，或对已落盘的 draft `delete`。compose 默认只读 active；explore 显式 `--all`，非 active 只标明不丢弃。

---

## 8. 闭环用到的命令

按触发顺序列。载荷与信封形状不在本文抄写。

| 阶段 | 命令 |
|------|------|
| 引导 | `init` |
| 召回 | `query`、`freshness`、必要时 `get`（explore 只 `query --all`，不调 `freshness`） |
| 写入原子 | `add`、`edit`、`active`；释放文件名时 `delete` |
| 派生文档 | `compose` |
| 查看 | `list`、`get`（compose / atomize 主路径不依赖 `list`；explore 清单题才 `list --all`） |

`help` / `--version` 是进程身份，不进闭环。

---

## 9. 圈在何处合上

- **入库是防重的发生地。** 新内容对照已有主张（含未转正）判是否同一事实；已有则复用或补细节，没有才新建。
- **提问是查找的发生地。** 不做全局索引；要派生文档则 compose 成文；问记录或问对照则 explore 作答（先召回再对照是否仍是现状，结论回答原问；`query --all`，非 active / 低分标明后由人采信）。（[真源：拼接：要派生可读文档时按问题召回原子并由 LLM 组合成文，派生文不是真源](../opencanon/atoms/compose_from_atoms.md) · [真源：探索：先召回再对照是否仍是现状且不写盘](../opencanon/atoms/explore_read_only.md)）
- **召回是新鲜度的发生地。** atomize / compose 对照当前实现打粗分并终审；过时的原子 `score = 0.00`，不当现行真源，正文仍留在文件里等人决定怎么改。explore 不打分。分数闭环见 [`freshness.md`](freshness.md)。（[真源：新鲜度须对照代码打分，0.60 再由 LLM 终审且不另开 skill](../opencanon/atoms/freshness_vs_code.md)）
- **同一主题再迁一篇源** 会再次走 atomize 的判同与 `edit`，把新细节并进已有真源，并在新源上加链接。

没有「扫一遍库」的常规步骤。第三条 skill 是按问题只读探索，不是全库索引。步骤写在 `opencanon-atomize`、`opencanon-compose` 与 `opencanon-explore` 三条 skill 里。（[真源：探索：先召回再对照是否仍是现状且不写盘](../opencanon/atoms/explore_read_only.md)）
