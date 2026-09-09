# 文档单一真实源治理

## 痛点

技术文档随时间增长会出现三类顽疾：

1. **越写越重复** —— 同一事实被复制、改写进多篇文档，改一处漏多处，逐渐失同步。（[真源：一处事实只记录一次，其它文档只引用不复制](../opencanon/atoms/one_fact_one_record.md)）
2. **查找困难** —— 不知道某件事「以哪篇为准」，找不全、找不准。（[真源：不做全局索引，按需扫描原子](../opencanon/atoms/no_global_index.md)）
3. **新鲜度失控** —— 文档与当前代码/配表脱节，读者无法判断「这话现在还成立吗」。（[真源：新鲜度须对照代码打分，0.60 再由 LLM 终审且不另开 skill](../opencanon/atoms/freshness_vs_code.md)）

约束：**一处事实只在一处记录，其他地方只引用，不复制。**（[真源：一处事实只记录一次，其它文档只引用不复制](../opencanon/atoms/one_fact_one_record.md)）

---

## 解题思路

把旧文档迁成原子真源，而不是把主张改写进原地。源里的主张正文不动；迁出的事实落独立真源目录。body 与代码事实一致则可代行转正；无法对照才问人。迁完后真源是新权威，旧文只作入口、用链接指向真源。（[真源：把旧文档迁成原子真源，不原地改写](../opencanon/atoms/migrate_docs_to_atoms.md) · [真源：转正须有独立证据，源文档不是证据](../opencanon/atoms/activate_needs_evidence.md) · [真源：迁完后真源即唯一权威，旧文只作入口且不长期追踪血缘](../opencanon/atoms/true_source_authority.md)）

确定性计算归工具，语义判断归 LLM，流程编排归 Skill。工作怎么形成闭环见 [`loop.md`](loop.md)。（[真源：分工：确定性计算归工具，语义判断归 LLM，流程编排归 Skill](../opencanon/atoms/tool_llm_skill_roles.md)）

### 对付重复

- 其它文档用链接跳转，不复制正文。（[真源：一处事实只记录一次，其它文档只引用不复制](../opencanon/atoms/one_fact_one_record.md)）
- 新内容先查后写：入库前对照已有主张（含未转正）判是否同一事实。已有则复用或补细节，没有才新建。防重发生在这一刀；字面召回抓不住的措辞差异，由入库时的语义判同补上。（[真源：查重：入库前对照已有主张判是否同一事实，同则复用没有才新建](../opencanon/atoms/dedup_before_write.md)）
- 转正须有独立证据：body 与代码事实一致、且不与 active 真源冲突则可代行转正；与代码不一致则不入库；无法对照才问人，且问在落盘前。源文档不是证据。不确定由人选择保持 draft。（[真源：转正须有独立证据，源文档不是证据](../opencanon/atoms/activate_needs_evidence.md)）

### 对付查找

- 不做全局索引。索引是第二份事实，极易腐坏；按需扫描原子的元数据与正文。（[真源：不做全局索引，按需扫描原子](../opencanon/atoms/no_global_index.md)）
- 回答一个问题时，按问题召回相关原子，由 LLM 整理成可读文档：可调语序、写摘要，不得改变真源语义、不得引入原子里没有的事实。工具只校验引用闭合并写入派生目录。派生文不是真源；别处若要出现该文，只放链接。无需人审，因为不改真源。这是组合成文，不是去重合并，也不是按主题做确定性拼接。（[真源：拼接：按问题召回原子并由 LLM 组合成文，派生文不是真源](../opencanon/atoms/compose_from_atoms.md)）

### 对付新鲜度

- 新鲜度无法从文档自身算出，必须对照当前代码/配表。（[真源：新鲜度须对照代码打分，0.60 再由 LLM 终审且不另开 skill](../opencanon/atoms/freshness_vs_code.md)）
- 召回时工具给粗分并写回 `score`（只降不升）。`score` 表示主张是否仍真实：1 可用，0 不可用，0.60 尚未终审。不能只看时间。（[真源：新鲜度须对照代码打分，0.60 再由 LLM 终审且不另开 skill](../opencanon/atoms/freshness_vs_code.md)）
- 0.60 再让 LLM 对照实现（判断不了则问人）。仍符合则 `edit` 盖验证时间并写成 1；不真实则 `edit` 写成 0。不另开新鲜度 skill。（[真源：新鲜度须对照代码打分，0.60 再由 LLM 终审且不另开 skill](../opencanon/atoms/freshness_vs_code.md)）

### 迁移

旧文档不可能一次迁完。已迁的主张在源段落末尾加真源链接（正文不动），未迁的保持原样。血缘只在迁移过程中有用，迁完后真源即唯一权威，工具不长期追踪血缘。（[真源：把旧文档迁成原子真源，不原地改写](../opencanon/atoms/migrate_docs_to_atoms.md) · [真源：迁完后真源即唯一权威，旧文只作入口且不长期追踪血缘](../opencanon/atoms/true_source_authority.md)）
