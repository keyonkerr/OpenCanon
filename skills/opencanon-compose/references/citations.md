# 成文引用格式

`body` 必须让 `opencanon compose` 能确定性校验。相对链接从 `opencanon/docs/` 出发。

```markdown
# {title}

{摘要段} [id](../atoms/id.md)

{正文段} [id](../atoms/id.md) [id2](../atoms/id2.md)
```

- 链接文字必须等于原子 `id`。href 必须是 `../atoms/<id>.md`。
- `#` 标题行不需引用。每个非空段落（含摘要）结尾必须有至少一条上述引用。一段可以引用多个原子。
- `atoms` 字段的集合必须等于正文里出现的全部这类引用：多一个或少一个都会 `VALIDATION_FAILED`。
- 只用步骤 2 选出的原子。不要引用未召回的 id。
