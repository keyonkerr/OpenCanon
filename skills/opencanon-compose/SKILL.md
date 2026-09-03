---
name: opencanon-compose
description: >-
  Use when the user wants a readable document assembled from canon atoms,
  compose true-source atoms into prose that answers a question, write into
  opencanon/docs, or run opencanon-compose — even if they only say "合成一篇可读文档".
  Do not use for atomize, dedup, or freshness.
compatibility: Requires the `opencanon` CLI.
---

# opencanon-compose

用库中 `active` 原子回答用户的问题，整理成一篇可读文档。成员只来自本次 `query` 命中。成文可调语序、写摘要，不得改变原子语义。落盘只经 `opencanon compose` 写入 `opencanon/docs/`。

## 1. 召回

从用户问题抽会写在原子 `body` 里的词（专名、机制、约束）；问题里若出现原子 id，把 id 也并进 keywords。调用：

```
opencanon query <keyword>...
```

默认只扫 active。不要加 `--all`。命中按 `id` 去重。argv 过长则分批 `query`，再按 id 合并。

完成：有一份命中列表（可为空）。零命中则停止，告诉用户库中没有相关真源，不编文。

## 2. 取材

只保留与问题相关的命中。丢掉不回答该问的原子。需要全文时用命中里已有的 `body`，不够再 `opencanon get <id>`。

完成：一份相关原子列表，每条都有 `id`、`title`、`body`，且都出现在步骤 1 的命中里。

## 3. 成文

只依据这些原子的 `body`。可调语序、合并句子、在文首写摘要。不得引入原子里没有的事实，不得反转主张。

正文格式见 [references/citations.md](references/citations.md)。`slug` 从标题按 [references/compose-stdin.md](references/compose-stdin.md) 的规则写成，组 stdin 前自检合法。

完成：会话里有 `slug`、`title`、`atoms`（实际用到的 id，无重复）、`body`（每段末尾有引用；引用集合等于 `atoms`）。

## 4. 交付

- 用户要落盘，或要把该文放到 `opencanon/docs/` 以外的路径：按 [references/compose-stdin.md](references/compose-stdin.md) 调用 `opencanon compose`。成功后 `data.path` 是 `opencanon/docs/<id>.md`。
- 用户只要会话里看到：展示 `body`，不调 `compose`。
- 用户要求写到别处：先 `compose` 落盘，再在目标文件插入指向 `opencanon/docs/<id>.md` 的 markdown 链接，不把正文复制过去。

`opencanon/docs/` 与 `opencanon/atoms/` 只经命令写入。`VALIDATION_FAILED` 时按 `error.details.field` 改正文或 `atoms` 后重试同一对象。`ATOM_NOT_FOUND` 时回到步骤 2，只用当前命中里仍存在的 id。
