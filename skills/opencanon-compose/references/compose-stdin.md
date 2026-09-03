# `opencanon compose` stdin

步骤 4 要落盘、或 `compose` 报 `VALIDATION_FAILED` 时读。一次一篇文档。

stdin 必须是 JSON **对象**（不能是数组）。键 kebab-case。

```json
{
  "slug": "how_ssot_works",
  "title": "OpenCanon 如何保证一处事实只记一次",
  "atoms": ["ssot_one_place", "compose_by_topic"],
  "body": "# OpenCanon 如何保证一处事实只记一次\n\n摘要。 [ssot_one_place](../atoms/ssot_one_place.md)\n"
}
```

### 必填

- `slug`：非空字符串，作为 `opencanon/docs/<slug>.md` 的文件名
- `title`：非空字符串
- `atoms`：字符串数组，本篇实际引用的原子 id，须非空、无重复，且当前都是 `active`
- `body`：可读 markdown，引用规则见 [citations.md](citations.md)

### 不要传

- `id`（等于 `slug`，由 `compose` 写入）
- `status`（派生文档没有状态机）

### `slug`

从 `title` 精炼成**小写英文单词或短语，词与词之间用 `_`**。与原子 id **分目录**，允许同名。

- 1–32 个 Unicode 字符
- `_` 允许，作词分隔；首尾不能是 `_`、空白或 `.`
- 不含 `<>:"/\|?*`

形状不合法 → `VALIDATION_FAILED`，`error.details.field` 为 `slug`。不要静默改写。同 slug **覆盖**已有派生文档，不是 `SLUG_CONFLICT`。

`atoms` 必须等于 `body` 里全部引用的 id 集合。缺引用、段末无链、或 id 不是 active → `VALIDATION_FAILED`。`atoms` 里的 id 在磁盘上不存在 → `ATOM_NOT_FOUND`。

### 调用

```
opencanon compose
```

PowerShell 把 JSON 经管道交给 stdin。失败则不写盘。

成功 `data`：

```json
{
  "id": "<slug>",
  "title": "<title>",
  "path": "opencanon/docs/<slug>.md"
}
```
