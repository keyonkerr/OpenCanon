# `opencanon edit` stdin

步骤 4 标了 `auto_edit`、步骤 6 选了 `edit`、组复用补充，或 `edit` 报 `VALIDATION_FAILED` 时读。只含确认写入现有原子的项。

stdin 必须是 JSON **数组**。元素必填 `id`；出现的可变字段覆盖，省略则保持。补全后的 `body` 是单一主张。

```json
[
  {
    "id": "durability_daily_restore",
    "body": "禁军突围中，装备耐久按日恢复。",
    "freshness": { "impl-path": "gamesvr/DurabilityManager.java" }
  }
]
```

不要传与当前不同的 `status`。`freshness` 按子键合并：只传 `impl-path` 时保留已有的 `last-verified` / `score`。可选 `tags`（整键替换）。

调用：`opencanon edit`（PowerShell 管道与 `add` 相同）。`VALIDATION_FAILED` 时按 `error.details.index` 改那条，重试全数组。
