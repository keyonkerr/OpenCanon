# 召回

步骤 2 归属并通写之后的是会话里的**候选**（还不是库中原子）。本步顺序固定，不写盘：

1. 从每条候选的 `body` 抽领域词
2. 与本批候选 `slug` 去重合并成 `keywords`
3. `query --all` 召回库里已有的相关原子（含 draft 与 deprecated）
4. 用两边的 `body` 判是否同一件事

## 抽词

只从候选 `body` 抽（主张正文）。只纳入能把该主张和库里其它主张分开的词：

- 领域专名、机制名、对象名、可区分的约束（条件、范围、动作）
- 本篇源里已出现的别名（中英、缩写、产品内名称）

## 组成 keywords

body 抽词 ∪ 本批每条候选的 `slug`，去重，得到一份 `keywords`。按这份列表调用，不以候选为粒度。把 slug 放进去是为了占名召回：id 等于 slug 的已有原子一定出现在命中里。

## 调用 query

每个 keyword 是一个位置参数，空格分开。多词 **OR**：任一词作为子串出现在某原子 `body` 或 `id`，即命中。没有 `--keyword` 旗标，不走 stdin。

```
opencanon query --all durability restore durability_daily_restore
```

- 词写在 `query` 之后，与 `--all` 同为 argv。
- 词组内部有空格时用引号包成**一个** argv；否则 shell 会拆开。
- 每个词单独传。不要写成一个参数（`"durability restore"` 会去匹配这整串；逗号/顿号/JSON 数组同理，都不会按词拆开）。
- 至少一个 keyword。
- argv 过长则把列表切成多批，每批一次 `query`，命中按 `id` 去重合并。

`--all`：语料含 draft、active、deprecated，避免与未审或已下线的同名文件漏判。默认 `query`（不加旗标）只扫 active，本步不要用默认。

`data.atoms[]` 每项是库中已有原子的完整内容（含 `body`），留在会话。

## 判同

比对的是**正文是否同一件事**：该候选的 `body`（本篇刚拆出的主张）对 `query` 命中的已有原子 `body`。只用这两边的 `body`。slug 与某 hit 的 `id` 相同仍要判同，禁止「同 slug 就自动合并」。

无命中或目录为空：每条候选标 `different`。

有命中：只对与该候选同一主题的 hit 标号；每条候选至多一个 `same`：

- `same <id>`：两边描述同一件事
- `different`：不是同一件事。同一对象上互相矛盾的约束也标 `different`
- `unsure`：措辞不同但可能指向同一件事，无法确定

漏标 `same` 会建成重复原子。判定只留在会话里。

完成：每条候选都是 `same <id>`、`different` 或 `unsure`。
