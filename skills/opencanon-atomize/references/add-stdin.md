# `opencanon add` stdin

拆候选定 `slug`、组新建数组、或 `add` 报 `VALIDATION_FAILED` / `SLUG_CONFLICT` 时读。只把要新建的 `true` 与 `uncertain` 放进数组（LLM 已判 `true` 的与人选 `true` 同等对待）；复用已有原子的不进 `add`。

stdin 必须是 JSON **数组**（不能是单个对象）。元素 kebab-case。

```json
[
  {
    "slug": "<slug>",
    "title": "<title>",
    "tags": ["<tag>"],
    "body": "<body>",
    "freshness": { "impl-path": ["<relative-path>", "<relative-path>"] }
  }
]
```

### 必填

- `slug`：非空字符串
- `title`：非空字符串
- `body`：非空字符串

### 可选

- `tags`：字符串数组。省略 = `[]`
- `freshness`：对象。省略 = `{}`。拆分时只填 `impl-path`：相对被治理项目根的代码/配表路径，一条字符串或字符串数组（一篇 body 所引用实现的总览）

### 不要传

- `id`（等于 `slug`，由 `add` 写入）
- `status`（强制 `draft`）
- `freshness.last-verified` / `freshness.score`（`active` 时才戳）

### `slug`

从 `title` 精炼成**小写英文单词或短语，词与词之间用 `_`**（如 `durability_daily_restore`）。这就是原子 id 与文件名。第 1 步写入候选；组 stdin 时从候选抄入，不要现编。title / body 仍可用中文。

- 1–32 个 Unicode 字符
- `_` 允许，作词分隔；首尾不能是 `_`、空白或 `.`
- 不含 `<>:"/\|?*`

形状不合法 → `VALIDATION_FAILED`，`error.details.field` 为 `slug`，整批不写。不要静默改写后再假装成功；改 slug 后重试全数组。

已有原子占用该 slug（任一 status）或本批重复 → `SLUG_CONFLICT`。`error.details.slugs` 是已占用 slug 列表；`error.details.conflicts[]` 每项 `{ index, slug, status? }`（磁盘占用带 `status`；本批互撞无 `status`）。整批不写。分支见 SKILL 步骤 7。

### 调用

整批先校验再写。任一条失败则一条都不落盘。

成功 `data`：

```json
{
  "atoms": [{ "id": "<slug>", "title": "<title>" }],
  "count": 1
}
```

`id` 与输入按下标对齐，等于该条 `slug`，用它去做后续的 `opencanon active <id>`。

**Example**

候选与判定：

1. 耐久按日恢复 → 与已有原子同一事实（不进 `add`）
2. 声称 `get` 只读 `active` → 打开实现后与代码不一致，`false`（不进 `add`）
3. 上限是否走配表 → 无实现可对照，人选 `uncertain`

组出的 stdin（一条）：

```json
[
  {
    "slug": "durability_cap_from_table",
    "title": "装备耐久从实现表读取上限",
    "tags": ["armybreak"],
    "body": "耐久上限以配表为准。"
  }
]
```

`opencanon add` 返回一个 id 后：该条是 `uncertain`，不调 `active`。
