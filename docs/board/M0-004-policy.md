+++
id = "M0-004"
title = "软过期、冲突与密钥扫描"
status = "done"
milestone = "M0"
module = "policy"
size = "M"
priority = "P0"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-001"]
docs = ["adr-0001"]
+++

## 范围

- 30 天内正常注入；31–90 天标 unverified；90 天以上不再自动注入。
- 冲突用字符 bigram Jaccard：≥ 0.5 视为相似；同标签时阈值降到 0.35。
- 密钥模式：`sk-`、`ghp_`、`github_pat_`、`AKIA`、私钥块、Slack、Google、长 hex。
- 拒绝是默认；CLI 需要 `--force` 才能覆盖。

## 验收

- [x] 常见 key 形态命中，普通中文与短 hex 不误报。
- [x] 相似/不相似/同标签三组单测。
- [x] 真实输入 `sk-...` 被拒绝，退出码 1。
- [x] 过期记忆在 prompt 中被排除。

## 记录

扫描是兜底，不是保证。未公开项目、客户名、内部地址仍靠确认时的人判断。
