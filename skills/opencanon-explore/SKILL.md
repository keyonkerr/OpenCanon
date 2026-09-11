---
name: opencanon-explore
description: >-
  Use when the user wants to know what canon atoms record and whether
  those claims still match the project's current implementation — even
  if they only say "库里有什么", "对照代码看还成不成立", or "结合代码看现状".
  Always recall, compare, then answer the question in session; never
  write files. Do not use for atomize, compose, or freshness.
compatibility: Requires the `opencanon` CLI.
---

# opencanon-explore

按问题召回库中原子，再对照每条主张是否仍是项目现状，在会话里回答该问。问记录或问对照都走这一条：先召回，再对照，再作答。零写盘：只调用 `query --all`、必要时 `get`，清单题才 `list --all`。只读打开实现文件。不调 `freshness` / `edit` / `add` / `active` / `delete` / `compose` / `init`。

原子正文以命令信封为准，不手读、不手列 `opencanon/atoms/`。

## 1. 召回

种子是用户问题全文；问题里出现的原子 id 并进 keywords。过滤：**一律**加 `--all`（draft / active / deprecated 全扫）。不按 `status`、不按落盘 `score` 丢掉命中。

专题问题按 [references/recall.md](references/recall.md) 抽词、调用 `query --all`。禁止 `freshness`。用户问的是库存清单（库里有哪些原子）才 `opencanon list --all`，仍不手列 `opencanon/atoms/`。

零命中：`recall.md` 已扩一轮同义词后仍零，带着空列表进入步骤 2，不停止。

完成：一份命中列表（可空），每条都有信封里的 `id`、`title`、`body`、`status`、`impl-path`、落盘 `score`（分可缺）。没有这份列表不得进入步骤 2。

## 2. 对照

只保留与问题相关的命中。丢掉不回答该问的原子。相关与否只看语义；`status` 与 `score` 不是丢掉的理由。清单题的相关集合可以是全库。

对留下的每一条按 [references/compare.md](references/compare.md) 对照，包括 draft / deprecated、`score == 0.00`、无分。对照回答该主张是否仍是项目现状。落盘 `score` 只当上次记录，不当成本轮结论。

零命中：用问题关键词只读搜代码，搜到的全部是观察。

完成：每个相关命中是 `一致`、`不一致` 或 `无法对照`；零命中则有观察，或明确代码里也没找到。

## 3. 作答

只在会话里写。不成文、不调 `compose`。先完成依据三段，再写**结论**；发给用户的正文以结论开头，依据随后。「库里记了什么」只来自步骤 1 的相关命中，观察只进「是否仍是现状」。每条用到的原子带上 `id`、`status`、落盘 `score`（无分写「无」）。

1. **结论**：几句话直接回答用户原问，单独拿出来就能当答案。可合并、摘要。现行陈述（当作现在成立的）只由「库里记了什么」里 `active`、落盘 `score > 0.80`、且「是否仍是现状」为 `一致` 的主张推出。「是否仍是现状」为 `不一致` 的，写成库记与代码已不符。`无法对照`、非 `active`、或分数偏低的只点名为未采信材料，不写进现行陈述。零命中：只基于「是否仍是现状」里已标明非真源的观察来答，或写库中没有、代码也没找到。缺口写进「还缺什么」。每一句都能指回下面某段已写的内容。
2. **库里记了什么**：相关命中的主张。无命中则写库中没有相关真源。`status` 不是 `active`（draft / deprecated），或分数偏低（无分、或落盘 `score <= 0.80`）时必须点名，并写明未当作现行真源、是否采信由你决定。不因这些标记删掉该条，也不替用户选。
3. **是否仍是现状**：对照结论（一致 / 不一致 / 无法对照），附 `impl-path`。扩圈或零命中搜到的标为观察。无命中时本段只有观察或「代码里也没找到」。
4. **缺口**：问了但库无，或代码有但未入库。

完成：四段都有；结论能单独回答原问；结论里每条现行陈述都能指回一条 `active`、落盘 `score > 0.80`、对照为 `一致` 的原子；引用过的原子 id 都出现在步骤 1 的命中里；每条非 `active` 或低分/无分都已在依据里标明且未抹掉；没有引入未标明来源的事实。

## 4. 收束

停在会话。用户要派生文档时改走 compose，本 skill 不代跑 `compose`。用户要改真源时改走 atomize。本步不调任何写命令。
