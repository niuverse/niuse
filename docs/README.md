+++
id = "docs-readme"
title = "看板维护指南"
kind = "guide"
status = "living"
order = 90
summary = "docs/ 的源文件、生成命令、四个视图，以及加模块/加卡/加文档的仪式。"
+++

## 这是什么

`docs/index.html` 是项目唯一的架构阅读面：Architecture / Work / Docs / Decisions 共享一个三栏壳。它由仓库里的源文件生成，离线可开，不依赖服务端。

```sh
cargo xtask dashboard          # 生成 docs/index.html 与 docs/dashboard/model.json
cargo xtask dashboard --check  # CI：生成物过期、引用断开时失败
```

生成是确定性的：相同输入生成相同 HTML。`receipt.json` 记录生成时间、git revision 与输入清单，但不参与过期判断。

## 四个视图

- **Architecture（默认）**：按层排列模块，显示依赖箭头、契约与源码。点模块会聚焦关系；状态只表示落地程度，不表示进度。
- **Work**：一卡一文件的 handoff 面，按里程碑筛选；不显示完成率、WIP 数字或进度条。
- **Docs**：Markdown 文档索引，右栏阅读，支持 `#/docs/adr-0001` 深链。
- **Decisions**：ADR 卡片 + `kind = "decision"` 的时间线；这里不放完成流水账。

## 源文件

```text
docs/
  index.html                # 生成物，提交，file:// 可开
  dashboard/
    project.json            # 项目、里程碑、层、模块、关系、决策事件
    template.html           # 单文件壳
    style.css               # 三栏与四个视图的样式
    app.js                  # 无框架渲染
    model.json              # 生成物
    receipt.json            # 生成信息，gitignore
  board/
    M0-001-five-stores.md   # 一张卡一个文件
  adr/
    0001-memory-layer.md
    0002-identity-layer.md
  architecture.md
  plan.md
```

## 加东西

**加模块**：在 `project.json` 的 `modules` 里加一项，`layer` 必须是已存在的层，`paths` 指向真实文件；非 `planned` 的模块至少要有一个路径存在。

**加工作卡**：在 `docs/board/` 新建 Markdown，TOML frontmatter 必须含 `id`、`title`、`status`、`milestone`、`size`、`updated`；`done` 必须有 `completed`，`blocked` 必须有 `blocked_reason`。

**加文档**：`docs/` 下任意 `.md`，开头用 `+++` 包住 TOML frontmatter，至少含 `id`、`title`、`kind`；ADR 的 `status` 只接受 `proposed` / `accepted` / `superseded` / `rejected` / `draft`。

**加决策**：写 ADR，或在 `project.json` 的 `events` 里加一条 `kind = "decision"` 的时间线；两条都做时用 `refs` 连起来。

## 纪律

看板只读，浏览器不提供拖拽回写。状态变更走仓库，避免它变成第二个真相源。提交前跑一次 `--check`。
