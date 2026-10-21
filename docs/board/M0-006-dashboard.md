+++
id = "M0-006"
title = "架构看板"
status = "done"
milestone = "M0"
module = "dashboard"
size = "L"
priority = "P1"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-003"]
docs = ["architecture", "plan"]
+++

## 范围

- 复用 niucode 的 xtask 生成器与三栏壳，换成 niuse 的项目模型。
- 四个视图：Architecture（默认）、Work、Docs、Decisions。
- 数据源：`project.json` + `board/*.md` + `docs/**/*.md`。
- 单文件 `docs/index.html`，离线可开，无 CDN、无运行时 fetch。
- `cargo xtask dashboard --check` 在生成物过期时失败。

## 验收

- [x] `cargo xtask dashboard` 生成 `docs/index.html` 与 `model.json`。
- [x] 校验通过：模块路径存在、卡片引用有效、ADR 状态合法。
- [x] Architecture 展示模块、分层与依赖箭头。
- [x] Docs 能打开架构文档、实施计划与两份 ADR。
- [x] `--check` 通过。

## 记录

看板只读，状态变更走仓库，避免它变成第二个真相源。生成器复制自 niucode，只改品牌与项目模型。
