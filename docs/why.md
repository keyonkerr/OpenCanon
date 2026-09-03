# 文档单一真实源治理

本文件只记**痛点**和**解题思路**。命令、字段、目录、状态机以 [`architecture.md`](architecture.md) 与 [`../crates/opencanon/AGENTS.md`](../crates/opencanon/AGENTS.md) 为准。

---

## 痛点

技术文档随时间增长会出现三类顽疾：

1. **越写越重复** —— 同一事实被复制、改写进多篇文档，改一处漏多处，逐渐失同步。（[真源：OpenCanon 一处事实只记录一次](../opencanon/atoms/ssot_one_place.md)）
2. **查找困难** —— 不知道某件事「以哪篇为准」，找不全、找不准。（[真源：OpenCanon 不养全局索引、改为按需扫描](../opencanon/atoms/no_global_index_scan.md)）
3. **新鲜度失控** —— 文档与当前代码/配表脱节，读者无法判断「这话现在还成立吗」。（[真源：新鲜度须对照当前代码配表、不能只看时间](../opencanon/atoms/freshness_vs_impl.md)）

约束：**一处事实只在一处记录，其他地方只引用，不复制。**（[真源：OpenCanon 一处事实只记录一次](../opencanon/atoms/ssot_one_place.md)）

---

## 解题思路

把旧文档迁成原子真源，而不是把主张改写进原地。源里的主张正文不动；迁出的事实落独立真源目录。人确认真实后才成为真源。迁完后真源是新权威，旧文只作入口、用链接指向真源。（[真源：把旧文档迁成原子真源而非原地改写](../opencanon/atoms/migrate_to_atom_ssot.md) · [真源：转正权在人且在落盘前](../opencanon/atoms/human_promotes_before_write.md) · [真源：迁完后真源为唯一权威且不长期追踪血缘](../opencanon/atoms/ssot_authority_no_lineage.md)）

确定性计算归工具，语义判断归 LLM，流程编排归 Skill。（[真源：确定性归工具、语义归 LLM、编排归 Skill](../opencanon/atoms/tool_llm_skill_split.md)）

### 对付重复

- 其它文档用链接跳转，不复制正文。（[真源：OpenCanon 一处事实只记录一次](../opencanon/atoms/ssot_one_place.md)）
- 新内容先查后写：入库前对照已有主张（含未转正、已下线）判是否同一事实。已有则复用或补细节，没有才新建。（[真源：入库前对照已有主张判是否同一事实](../opencanon/atoms/ingest_dedup_same_fact.md)）
- 转正权在人，且在落盘前：确认真实才算真源；不确定可暂不转正；非真实不入库。（[真源：转正权在人且在落盘前](../opencanon/atoms/human_promotes_before_write.md)）
- 库内查重仍用字面相似度做宽阈值召回（宁宽勿漏），再交给 agent 判「是否同一事实」。误报成本低（判否即可），漏报成本高。字面相似度抓不住措辞差很大的同事实，这是已知边界，入库时的语义判同补这一刀。（[真源：入库前对照已有主张判是否同一事实](../opencanon/atoms/ingest_dedup_same_fact.md)）

### 对付查找

- 不做全局索引。索引是第二份事实，极易腐坏；按需扫描原子的元数据与正文。（[真源：OpenCanon 不养全局索引、改为按需扫描](../opencanon/atoms/no_global_index_scan.md)）
- 回答一个问题时，按问题召回相关原子，由 LLM 整理成可读文档：可调语序、写摘要，不得改变真源语义、不得引入原子里没有的事实。工具只校验引用闭合并写入派生目录。派生文不是真源；别处若要出现该文，只放链接。无需人审，因为不改真源。这是组合成文，不是去重合并，也不是按主题做确定性拼接。（[真源：按问题召回原子并由 LLM 组合成文](../opencanon/atoms/compose_from_atoms.md)）

### 对付新鲜度

- 新鲜度无法从文档自身算出，必须对照当前代码/配表。（[真源：新鲜度须对照当前代码配表、不能只看时间](../opencanon/atoms/freshness_vs_impl.md)）
- 工具只给粗分（距上次修改、引用符号是否还在、版本控制时间等）。低于阈值再让 LLM 把真源与实现比对。（[真源：新鲜度须对照当前代码配表、不能只看时间](../opencanon/atoms/freshness_vs_impl.md)）
- 很久没改但仍正确的文档不应被判不新鲜，所以不能只看时间。（[真源：新鲜度须对照当前代码配表、不能只看时间](../opencanon/atoms/freshness_vs_impl.md)）

### 迁移

旧文档不可能一次迁完。已迁的主张在源段落末尾加真源链接（正文不动），未迁的保持原样。血缘只在迁移过程中有用，迁完后真源即唯一权威，工具不长期追踪血缘。（[真源：把旧文档迁成原子真源而非原地改写](../opencanon/atoms/migrate_to_atom_ssot.md) · [真源：迁完后真源为唯一权威且不长期追踪血缘](../opencanon/atoms/ssot_authority_no_lineage.md)）
