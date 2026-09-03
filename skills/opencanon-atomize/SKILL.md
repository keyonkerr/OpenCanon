---
name: opencanon-atomize
description: >-
  Use when the user wants to atomize, split a document, migrate a wiki or
  markdown into canon atoms, ingest a single source of truth, write into
  opencanon, or run opencanon-atomize — even if they only say "split this into
  the atom library". Do not use for dedup, compose, or freshness.
compatibility: Requires the `opencanon` CLI and a structured multiple-choice ask tool.
---

# opencanon-atomize

把指定的一篇多事实文档拆成单事实原子。对照库中已有原子：同一事实则合并进该原子（有新细节则 `edit`）；否则作为新原子，经人审后 `add`。

## 1. 按原文抽出候选

只在本次会话里拆。先不组命令 JSON，也不把候选、关键字、命中或判定写成文件。本步允许暂时过细；跨节合并在步骤 2。只使用本篇已覆盖的片段；人指定的若是摘要，真源也只含摘要宽度。更细文档以后原子化时走判同 + `edit`，不要本篇发明细节。

一条**候选**是一处要单独维护、单独引用、单独判新鲜度的**领域主张**（机制、规则、命令契约、设计决策）。主张是一个设计问题的完整回答，含必要落地，不是可单独赋真值的子句，也不是表的一行。人审问的是陈述是否成立，不是是否值得入库、是否过细；大纲复述字面上为真，子句碎片也会被标真实。所以这一步要把骨架和主张分开。

章节标题、编号、目录、命令演示、JSON/磁盘样例是骨架：给其下主张补主体和范围，本身不成候选。抽出对照见 [references/split.md](references/split.md)。

按原文顺序走完，覆盖源里每一处领域主张：只切分，不改含义。补上脱离原文也能读懂的最小上下文（主体、范围、条件）。主语是领域对象（产品、机制、命令），不是「该源文档」。本步 `body` 可以粗，成稿在步骤 2 通写。

每条候选需要：

- `title`：一句话点出该事实
- `slug`：从 `title` 精炼成小写英文蛇形，作为原子 id。规则见 [references/add-stdin.md](references/add-stdin.md) 的 `slug` 节。本步读该节、写进候选并自检合法；不要留到组 stdin 再编。
- `body`：这一处主张的自包含正文
- `tags`：一条一个主 tag，对应该主张被查时的问题类型（如 `查重`、`SSOT`）。本步可暂定，步骤 2 通写后按问题类型定稿。
- 类型：机制 / 规则 / 命令，或问题 / 市场 / 调研。`impl-path` 在步骤 2 合稿后查找，本步不搜仓库。

完成：源里每一处主张片段至少被一条候选覆盖；每条都有 `title`、`slug`、`body`、`tags`，且 `slug` 已按 add-stdin 自检合法。

## 2. 跨节合稿

只在会话里做，不写盘。先**归属**再**通写**。对照 [references/split.md](references/split.md)。

### 归属

对步骤 1 的列表做源回指：源里每一处主张片段指向恰好一条候选。

- 后文复述前文明示决策：并入先出现的那条。
- 结论与支撑它的因为所以（可跨小节）合成一条。
- 同一调研快照（几个近邻对象 + 共同结论，含 star 等捆绑观测）合成一条。时点必须可核对（源里的日期或版本；源没有则用本次原子化日期）；观测写成「截至 {时点}」。「非穷举」留在这条里。
- 同一表格行（理念+结论+理由，或角色+职责+不做）合成一条：一行不得再拆。多行若是同一决策的原则、落地形态、操作手续，并入先出现的那条。表行是下限，不是「不同行必须不同原子」。
- 同一主语的职责或能力清单合成一条。职责清单与「会被分开来问的机制」同时出现时：清单那条只写分工原则和互斥边界（谁做哪一类事、明确不做对方的事）。各机制的动词、约束、未决归机制自己那条。角色表仍合成一条，不按角色拆开。
- 落地形态或操作门禁离开那条原则只剩半句：并入先出现的原则。形态以后可能改，仍合。
- **自立**：每条候选单独拿给读者，仍是一个完整问题的回答。只剩邻条的时间表、标记手续、门禁半句：并入先出现的那条。
- 未决项、开放问题、待定公式或阈值：并入它修饰的那条机制。
- 仍拿不准：合。能指出读者会当成两个不同问题来查时才分；分了之后同一陈述只归一条。连接句与收束句（定位、差异化、因此应当…）归拥有该结论的那条；邻条写完本问即停。

本段只定哪些片段属于哪条，不定稿。

### 通写

归属冻结后，为每条候选重写 `title` 与 `body`（`slug` 随新 title 再精炼，仍按 add-stdin 自检合法）。

通写是一篇主张：可调整语序、合并句子、去掉骨架，使脱离源文档也能当一个问题的完整回答来读。主语是领域对象（产品、机制、命令）。只使用该候选已覆盖的源片段，因果只回答本条的问题。步骤 1 的「只切分、不改含义」约束的是主张，不是原文措辞。

通写后做**删句**：任一句子删去后，本条必须少一个独立约束或因果环节；否则该句是复述，删掉。同一理由只写一次。痛点句只作为本条机制的因为所以，通写落到「故…」后的约束。同一编号痛点的不同症状可以分给不同机制，每条只带支撑自己结论的那一截。

主 tag 取读者会用来提问的中心词（如 `查重`、`拼接`、`迁移`、`SSOT`），一条一个。该词（或源里已有的同义说法）必须出现在 `title` 或 `body` 中，语言随 `opencanon/config.yaml` 的 `locales`（英语中心词始终可出现在 slug / 英文别名里）。

然后按类型找实现：机制 / 规则 / 命令类在被治理项目里查找。写入 `freshness.impl-path` 当且仅当该路径的实现覆盖 `body` 里每一条机制约束（相对项目根，指向代码或配表，不指向源文档）。覆盖不全：先收窄 `body` 使与现状同宽，或省略 `impl-path`、保持为决策主张。无实现：省略 `impl-path`。问题 / 市场 / 调研类不填。

完成（须同时成立才进入召回）：

1. 源回指一对一；没有两条覆盖同一决策，也没有「原则一条、其形态或门禁又一条」。每条自立：单独成篇仍回答一个完整问题。
2. 未决项已在它所修饰的机制里；同一陈述只出现在一条。职责清单条只含原则与互斥边界；各机制的动词、约束、未决在机制条。
3. 每条 `body` 通过删句（仍能指出覆盖的源位置）；痛点已落到「故」后的约束；`title` 与之一致；`slug` 合法且本批互不相同。收束句没有两条同写。调研快照含可核对时点。
4. 机制 / 规则 / 命令类：已查找实现。`impl-path` 仅在实现覆盖 `body` 全部机制约束时填写；覆盖不全则已收窄 `body` 或已省略 path；无实现则省略。问题 / 市场 / 调研类无 `impl-path`。
5. 每条一个主 tag，取该主张被查时的中心词；该词或源里同义说法已出现在 `title` 或 `body`。
6. 已套过 split.md 的合稿例与通写例（含清单与机制、删句、调研时点、收束句、迁移手续并入、痛点落到故）。

## 3. 召回并判是否同一事实

按 [references/query.md](references/query.md) 读 `locales`、扩英语与 locale 关键词、把候选 slug 并进 keywords、调用 `query --all`、判同。命中按 id 去重。slug 与某 hit 的 id 相同仍要判同。本步不写盘。

完成：每条候选都是 `same <id>`、`different` 或 `unsure`。

## 4. 提问

用 agent 的提问工具。一次表单可以放多道题。全部相关题目都有选择之前，不组 stdin，不调 `add` / `edit` / `active`。没有答 `true` 的，不转正。

**是否同一事实**只问 `unsure`。题干列出拿不准的 hit（`id`、`title`、`body`）。选项：每条 hit 一个 `same_<id>`，外加 `different`。已是 `same` / `different` 的不再问。用户选 `different` 的改标为新建；若该条还没有真实性选择，再问真实性。

**真实性**只问将要新建的候选（无命中、`different`、用户选了 `different`），以及已判 `same` 且命中是 **draft** 的（转正权在人）。题干给该条 `title`、`body`，并固定一行实现状态，三选一：

- `实现：无`
- `实现：有且与 body 同宽（impl-path: <path>）`
- `实现：有但窄于 body，未挂 impl-path`

不得在窄于 `body` 的实现上填写 `impl-path`。选项固定：

| id | 标签 | 之后 |
|----|------|------|
| `true` | 真实 | 新建则写入再 `active`；复用 draft 则对该已有 `id` `active` |
| `uncertain` | 不确定 | 新建则写入保持 `draft`；复用 draft 则保持 `draft` |
| `false` | 非真实 | 不创建、不 `active` |

已判 `same`、命中是 **active**、且现有 `body` 已覆盖本篇主张：不问，不新建。

同一事实题与当时已确定要问的真实性题可以放在同一表单。

完成：每条新建候选有且仅有一个真实性选项；每条曾为 `unsure` 的候选有且仅有一个同一事实选项；进入本步的 draft 复用有且仅有一个真实性选项。

## 5. 缺细节才问是否写入现有原子

仅当已确定与某 `id` 同一事实（`same <id>` 或用户选了 `same_<id>`），并且本篇候选里有现有 `body` 没有的领域细节：再问一次。选项：`edit` 写进现有原子 / `keep` 现有正文不动。

`edit` 的 `body` 是通写后的整篇主张：把现有原子没有的细节写进去后重写全文。

判同且无新细节：不问，现有原子一字不动。

完成：每条「判同且有新细节」的候选都有 `edit` 或 `keep`。

## 6. 写入

`add` 数组只含要新建的条目，下标只与这批新建对齐。

新建且 `false`：不创建。过滤后既无新建也无 `edit`：跳过本步写入，进入步骤 7。

新建且 `true` / `uncertain`：读 [references/add-stdin.md](references/add-stdin.md) 组 stdin（`slug` / `title` / `body` / `tags` / `freshness` 从候选抄入），再：

```
opencanon add
```

stdin 用 UTF-8 文件重定向，例如 `cmd /c "opencanon add < add.json"`。临时 JSON 不要放进 `opencanon/atoms/`。用完即删。

成功时 `data.atoms[]` 与输入按下标一一对应，每项是 `{ "id", "title" }`（`id` 等于该条 `slug`）。`VALIDATION_FAILED` 时整批未写：按 `error.details.index` 改那条，重试**全数组**。

`SLUG_CONFLICT` 时整批未写。对 `error.details.conflicts` 里每一条候选：

1. `get <slug>`（或用步骤 3 已有全文）走判同
2. **同一事实**且占用方是 active 或 draft：不 `add` 该条；按步骤 5 决定 `edit` / `keep`；draft 且真实性为 `true` 则对该 id `active`
3. **同一事实**且占用方是 deprecated：先 `opencanon delete <slug>` 释放文件名，该条留在 `add` 数组
4. **不同事实**：改本条 slug（仍按 add-stdin 合法），对新 slug 再 `query --all`，重试全数组
5. 本批互撞（conflict 无 `status`）：先在候选之间合并或改 slug，再重试全数组

对 `true` 的新建 id 逐个：

```
opencanon active <id>
```

已确定同一事实：不 `add`。步骤 5 选了 `edit` 的：读 [references/edit-stdin.md](references/edit-stdin.md) 组 stdin，再：

```
opencanon edit
```

`VALIDATION_FAILED` 时同样按 `error.details.index` 改那条，重试全数组。

同一事实、命中是 **draft**、真实性为 `true`：对该已有 `id` 调 `opencanon active`。命中已是 **active**：不 `active`。

完成：本批该 `add` / `edit` / `active` 的已调用。进入步骤 7。

## 7. 源文档段末回写真源链接

opencanon 不读、不写源文件。agent 用自身写文件能力改源 md。依据步骤 2 的源回指，在每个含领域主张的段落末尾追加该段所属、且已是 **active** 的原子链接。

- 主张语句保持原文；只追加链接后缀。一段多原子则并列；纯标题、目录、命令演示、代码块不加。
- `uncertain`（draft）与 `false`（未创建）不加「真源」链。复用的 active 原子仍链。
- 相对路径从**源文件所在目录**指到被治理项目 cwd 下 `opencanon/atoms/<id>.md`，链接内一律 `/`。后缀形态：

```markdown
（[真源：OpenCanon 一处事实只记录一次](../opencanon/atoms/ssot_one_place.md)）
```

多条用 ` · ` 连接。段末若已有一组 `（[真源：…](…)）`，整组替换，不叠床架屋。

完成：每个主张段落后有且仅有一组对应该段 active 原子的链接；draft 与未创建无链；主张原文未动。
