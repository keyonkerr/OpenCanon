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

装好后跑 `opencanon help` 确认已在 PATH。日常用法见下一节。

## 怎么用

人指定源文档或提问；agent 读 skill、调命令。不要手改 `opencanon/atoms/` 或 `opencanon/docs/`。不要在本仓（产品树）里 init。

### 1. 在被治理项目里初始化（每个项目一次）

到**要治理文档的项目根**：

```bash
cd /path/to/your-project
opencanon init
```

终端里多选文档语言。成功后会建 `opencanon/`（含 `atoms/` 与 `config.yaml`），并把 `opencanon-atomize`、`opencanon-compose` 装进 `.agents/skills/`。需要交互终端；无 TTY 会失败。

如果你的 agent **不读** `.agents/skills/`（例如只用 Cursor / Claude Code / Codex 自己的 skill 目录），把这两个文件夹复制到它会加载的 skills 路径即可，整目录拷、保持目录名。源在项目里的 `.agents/skills/`，或本仓的 [`skills/`](skills/)。

### 2. 写入：把文档拆进原子库

在该项目里对 agent 指定一篇源文档，例如：

- 「把 `docs/xxx.md` 原子化」
- 「把这篇 wiki 迁进 OpenCanon」

agent 跑 `opencanon-atomize`：按原文抽出单事实主张，对照库里已有原子（同一事实则复用或补细节），能对照代码则转正，对不上才问你。迁完后源文档主张正文不动，段末追加指向 `opencanon/atoms/<id>.md` 的真源链接。只有 `status: active` 的原子才是真源。

### 3. 读出：按问题组合成文

有原子之后，对 agent 提问，例如：

- 「根据原子库回答：……」
- 「合成一篇可读文档：……」

agent 跑 `opencanon-compose`：按问题召回 `active` 原子，整理成文，不编造原子里没有的事实。只要会话里看就展示；要落盘或要放到别处时，写入 `opencanon/docs/`，别处只放链接、不复制正文。

闭环步骤见 [`docs/loop.md`](docs/loop.md)。新鲜度分数见 [`docs/freshness.md`](docs/freshness.md)。

## 它解决什么

文档越写越重复、不知道以哪篇为准、和当前代码脱节。迁出的事实落在被治理项目的 `opencanon/atoms/` 里；`status: active` 的才是真源。源文档主张不动，段末加链接。

## 谁做什么

| 角色 | 做什么 | 不做什么 |
|------|--------|----------|
| 人 | 指定源文档、提问；LLM 无法对照实现时批复 | — |
| agent | 读 skill、自读源文件与实现、调命令 | 手改 `opencanon/atoms/`、`opencanon/docs/` |
| `opencanon` | 校验、落盘、流转、子串召回、新鲜度粗分 | LLM、编排、读写源文档 |
