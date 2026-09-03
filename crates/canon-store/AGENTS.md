# canon-store

`Atom` ↔ `opencanon/atoms/<id>.md` 与 `ComposedDoc` ↔ `opencanon/docs/<id>.md` 的唯一翻译层，也是磁盘上这些文件的唯一写入口。只认识这些值，不认识命令名或信封。

status 该不该变、freshness 怎么合并、id 怎么分配、compose 引用是否合法，都在 `canon-core`。这里只读写已经算完的值。

## 落点

| 要做的 | 写这里 |
|--------|--------|
| `opencanon/atoms/`、`opencanon/docs/` 的路径 | 只改 `layout.rs` |
| 原子 frontmatter 键序、kebab-case、缺省（`tags: []`、`freshness: {}`、子键有则写无则省） | 只改 `serialize.rs`。手写键序，不靠 serde_yaml 默认顺序 |
| 派生文档 frontmatter 键序（`id` → `title` → `atoms`） | 只改 `serialize_doc.rs`。不要复用原子序列化 |
| 读 / 写（tmp+rename）/ 删 / 列 | `io.rs`。文档用 `write_doc` / `read_doc` |

读路径在目录不存在时当作空或不存在，不创建目录。第一次成功写入原子才创建 `opencanon/atoms/`（必要时先建 `opencanon/`）。第一次成功写入派生文档才创建 `opencanon/docs/`。`add` 不建 `docs/`；`compose` 不建 `atoms/`；`skills/` 不写进使用方数据目录。

整批 `add` / `edit` 不是文件系统事务：CLI 先让 `ops` 整批成功，再逐条 `write`。崩溃导致部分落盘可接受，不为此加 journal。

派生文档不写回原子正文。

## 测试

沙盒目录。断言往返后键序与缺省稳定、覆盖写、缺目录 list 为空。不断言信封或 `ops` 的合并表。store 失败 = 文件形状坏了。
