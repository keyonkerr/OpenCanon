# OpenCanon

[English](README.md) | [中文](README.zh-CN.md)

确定性的文档原子库：把技术文档拆成单事实原子，**一处记录，别处只引用**。

CLI 二进制名是 `opencanon`。agent 读 skill、调 LLM、按序调用命令；本仓 Rust 只做校验、流转、落盘和确定性计算，不含 LLM 与 HTTP 依赖。

## Quickstart

macOS / Linux：

```bash
curl -fsSL https://raw.githubusercontent.com/keyonkerr/OpenCanon/master/scripts/install.sh | sh
```

Windows：

```powershell
irm https://raw.githubusercontent.com/keyonkerr/OpenCanon/master/scripts/install.ps1 | iex
```

然后到**要治理文档的项目根**运行 `opencanon init`（不要在本仓里 init）。之后指定源文档让 agent 跑 `opencanon-atomize` 写入；提问时跑 `opencanon-compose` 读出。

## 它解决什么

文档越写越重复、不知道以哪篇为准、和当前代码脱节。迁出的事实落在被治理项目的 `opencanon/atoms/` 里；`status: active` 的才是真源。源文档主张不动，段末加链接。

## 谁做什么

| 角色 | 做什么 | 不做什么 |
|------|--------|----------|
| 人 | 指定源文档、提问；LLM 无法对照实现时批复 | — |
| agent | 读 skill、自读源文件与实现、调命令 | 手改 `opencanon/atoms/`、`opencanon/docs/` |
| `opencanon` | 校验、落盘、流转、子串召回、新鲜度粗分 | LLM、编排、读写源文档 |
