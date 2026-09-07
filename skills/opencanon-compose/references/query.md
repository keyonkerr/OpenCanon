# 抽词与 query

本文件写抽词、调用 `query`、以及对命中里的 active 打分。种子、是否 `--all`、判同/取材由本 skill 的 `SKILL.md` 本步给出。

## 抽词

先读被治理项目 cwd 下 `opencanon/config.yaml`。`locales` 是 init 勾选写入的 BCP-47 列表（勾了英语则含 `en`）。无文件或无该键：不补 locale 词，英语仍要有。

从 **SKILL 本步给出的种子文本** 抽能把该主张和库里其它主张分开的词，再按语言补全：

- 英语永远要有：机制英文名、问题或正文里的英文、种子里已有的英文标识
- 再为 `locales` 里每种语言补读者会用来提问的中心词（与主 tag 同类）。yaml 里已有 `en` 时与默认重复即可，不要去掉英语
- 种子里已出现的别名（各语言、缩写、产品内名称）

不要只抽一种语言。

## 组成 keywords

英语词 ∪ locale 中心词 ∪ 种子里已有的词，去重，得到一份 `keywords`。按这份列表调用，不以单条候选或单句为粒度。

若本步要求把已有 slug/id 并进 keywords，去重后并入。

## 调用 query

每个 keyword 是一个位置参数，空格分开。多词 **OR**：任一词作为子串出现在某原子 `id`、`title`、`tags` 或 `body`，即命中。没有 `--keyword` 旗标，不走 stdin。

```
opencanon query [--all] durability restore 查重 durability_daily_restore
```

- 词写在 `query` 之后；`--all` **仅当本步 SKILL 写明时**加上，与 keywords 同为 argv。
- 不加 `--all` 时默认只扫 active。
- 词组内部有空格时用引号包成**一个** argv；否则 shell 会拆开。
- 每个词单独传。不要写成一个参数（`"durability restore"` 会去匹配这整串；逗号/顿号/JSON 数组同理，都不会按词拆开）。
- 至少一个 keyword。
- argv 过长则把列表切成多批，每批一次 `query`，命中按 `id` 去重合并。

`data.atoms[]` 每项是库中已有原子的完整内容（含 `body`），留在会话。

零命中：按英语 + `locales` 再扩一轮同义词后重 query。仍零则回到 SKILL 本步写明的完成条件，不调 `freshness`。不要改去 `list` 全库。

## 对命中打分

`query` 与 `freshness` 各是一次独立进程：磁盘上的 `score` 不会自动覆盖会话里已留下的命中。必须按下面铺回，否则后续步骤仍拿着打分前的旧分。不要为了对齐分数再 `query` / `get` 一遍全文（body 未改）。不抄因素表、不写阈值、不 `edit`、不调 LLM。

1. 只收集命中里 `status == active` 的 `id`，去重、保持命中顺序。draft / deprecated 不准传入（指定非 active 会整批 `VALIDATION_FAILED`）。
2. 该列表为空：不调 `freshness`，会话命中保持 `query` 原样。
3. 调用（禁止省略 id，省略会打全库）：

```
opencanon freshness id1 id2 ...
```

argv 过长则分批，规则同 `query`。失败则本步失败，不带着过期分继续。

4. 按 `id` 把信封铺回会话中的命中：
   - `skipped: true`：不算分、不写盘；保留 `query` 带来的 freshness
   - `skipped: false`：用信封的 `score` 覆盖该 hit 的 `freshness.score`；`factors` 留在会话
