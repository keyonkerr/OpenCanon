# 召回

步骤 2 归属并通写之后的是会话里的**候选**（还不是库中原子）。本步顺序固定，不写盘：

1. 读 `opencanon/config.yaml` 的 `locales`（没有该文件或没有 `locales` 则视为未勾选额外语言）
2. 从候选扩关键词：英语（源码默认，不能少）∪ `locales` ∪ 源里已出现的词 ∪ 本批 `slug`
3. `query --all` 召回库里已有的相关原子（含 draft 与 deprecated）
4. 用两边的 `body` 判是否同一件事

## 抽词

先读被治理项目 cwd 下 `opencanon/config.yaml`。`locales` 是 init 勾选写入的 BCP-47 列表（勾了英语则含 `en`）。无文件或无该键：不补 locale 词，英语仍要有。

从每条候选的 `title` / `body` / `slug` 抽能把该主张和库里其它主张分开的词，再按语言补全：

- 英语永远要有：slug、机制英文名、问题或正文里的英文
- 再为 `locales` 里每种语言补读者会用来提问的中心词（与主 tag 同类）。yaml 里已有 `en` 时与默认重复即可，不要去掉英语
- 本篇源里已出现的别名（各语言、缩写、产品内名称）

不要只抽一种语言。

## 组成 keywords

英语词 ∪ locale 中心词 ∪ 源里已有别名 ∪ 本批每条候选的 `slug`，去重，得到一份 `keywords`。按这份列表调用，不以候选为粒度。把 slug 放进去是为了占名召回：id 等于 slug 的已有原子一定出现在命中里。

## 调用 query

每个 keyword 是一个位置参数，空格分开。多词 **OR**：任一词作为子串出现在某原子 `id`、`title`、`tags` 或 `body`，即命中。没有 `--keyword` 旗标，不走 stdin。

```
opencanon query --all durability restore 查重 durability_daily_restore
```

- 词写在 `query` 之后，与 `--all` 同为 argv。
- 词组内部有空格时用引号包成**一个** argv；否则 shell 会拆开。
- 每个词单独传。不要写成一个参数（`"durability restore"` 会去匹配这整串；逗号/顿号/JSON 数组同理，都不会按词拆开）。
- 至少一个 keyword。
- argv 过长则把列表切成多批，每批一次 `query`，命中按 `id` 去重合并。

`--all`：语料含 draft、active、deprecated，避免与未审或已下线的同名文件漏判。默认 `query`（不加旗标）只扫 active，本步不要用默认。

`data.atoms[]` 每项是库中已有原子的完整内容（含 `body`），留在会话。

零命中：按英语 + `locales` 再扩一轮同义词后重 query。仍零则每条候选标 `different`。不要改去 `list` 全库。

## 判同

比对的是**正文是否同一件事**：该候选的 `body`（本篇刚拆出的主张）对 `query` 命中的已有原子 `body`。只用这两边的 `body`。slug 与某 hit 的 `id` 相同仍要判同，禁止「同 slug 就自动合并」。

无命中或目录为空：每条候选标 `different`。

有命中：只对与该候选同一主题的 hit 标号；每条候选至多一个 `same`：

- `same <id>`：两边描述同一件事
- `different`：不是同一件事。同一对象上互相矛盾的约束也标 `different`
- `unsure`：措辞不同但可能指向同一件事，无法确定

漏标 `same` 会建成重复原子。判定只留在会话里。

完成：每条候选都是 `same <id>`、`different` 或 `unsure`。
