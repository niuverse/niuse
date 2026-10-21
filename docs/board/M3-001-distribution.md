+++
id = "M3-001"
title = "发布与依赖策略"
status = "backlog"
milestone = "M3"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-21"
depends_on = ["M0-003"]
docs = ["plan"]
+++

## 范围

- 决定 niucode 依赖：先发布 niucode crates、改为钉 rev 的 git 依赖，或明确只支持 `cargo install --git`。
- 补齐 `license`、`repository`、`description` 等发布元数据。
- `cargo package` / `cargo publish --dry-run` 可复现。
- 确认 crate 名可用性；必要时 crate 改名而保留二进制名 `niuse`。

## 验收

- [ ] `cargo package -p niuse` 通过（当前被 path 依赖卡住）。
- [ ] 安装路径在干净机器上可复现。
- [ ] 许可证与名称策略写进 README。

## 记录

当前 `cargo package` 的失败信息：`all dependencies must have a version requirement specified when packaging`，指向 `niucode-agent` 的 path 依赖。
