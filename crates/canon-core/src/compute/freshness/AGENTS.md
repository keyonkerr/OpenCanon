# freshness

相对当前实现的机器粗分。每个因素先算出 `value ∈ [0, 1]`，再声明角色。合成只认角色，不认因素名。不要改 `combine` 去特判某个 `id`。

无 `impl-path`（缺省或空白）由调用方 skip，不算分、不写盘。那是语料过滤，不是因素。

本 crate 零 IO：exists / `changed_at` / 文件文本由 CLI 注入。

## 角色

| 角色 | 作用 |
|------|------|
| Gate | 任一项 `value == 0` → 总分 0。多项门槛必须全过。 |
| Weighted | 门槛都过之后：`score = (1 - W) + Σ(weight_i * value_i)`。`W` 是总表所有 weight 之和，必须 `W ≤ 1`。 |
| Multiplier | 加权之后再 `score *= value`。有第一种实例再建 `multiplier/`。 |
| Observe | 进信封，不入总分。有第一种实例再建 `observe/`。 |

合成顺序：门槛 → 加权 → 乘数 → 四舍五入到两位并夹到 `[0, 1]`。

## 加因素

1. 在对应角色目录新建 `.rs`，导出同一形状：常量 `ID`（kebab-case）+ `value(...) -> f64`。因素文件只算 0–1，不写权重。
2. 该目录 `mod.rs` 加一行 `mod`，并收集进 `factors()`。
3. 若是加权：只在 `weighted/mod.rs` 的 `WEIGHTS` 总表加一行 `id → weight`。不要把权重写进因素文件，不要进 `config.yaml`。调某条有多重要、看地板 `1-W`、锁 `W ≤ 1`，都只打开总表。

不改 `combine.rs`。不用 `inventory`、启动扫盘、`build.rs` 生成。

权重数字只存在总表；本文不抄一份。
