+++
id = "M0-002"
title = "Prompt 组装与带内协议"
status = "done"
milestone = "M0"
module = "prompt"
size = "L"
priority = "P0"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-001"]
docs = ["architecture", "plan"]
+++

## 范围

- 稳定前缀：persona → identity → user → memory → protocol。
- 易变段后置：today → loops → journal。
- 分段预算：identity / user 各 600，memory 2000，loops 300，journal 600。
- 带内协议：`<<niuse:memory>>`、`<<niuse:loop>>`、`<<niuse:journal>>`。
- 流式剥离需要尾部缓冲，未闭合块按普通文本显示。

## 验收

- [x] 段顺序有单测，过期记忆不注入、30 天以上带 unverified。
- [x] 超长 identity 会截断并标注。
- [x] 协议块跨 chunk、部分前缀、伪标记、未闭合都有测试。
- [x] 真实调用中候选块被剥离，正文完整。

## 记录

原计划只有 memory / loop 两种块。为了让 journal 不额外调模型，增加 journal 块并由 niuse 自动落盘。
