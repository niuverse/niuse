+++
id = "M2-093"
title = "identity.md is the only name source"
status = "done"
milestone = "M2"
module = "prompt"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-022"]
docs = ["architecture"]
+++

## 问题

用户问「你是谁」，真实 REPL 答「我是 niuse」，被纠正后才改口。根因是 persona 的 `BASE` 硬编码了「你是 niuse」，而 `identity.md` 只是后面的一节；模型把产品名当成了自己的名字。

## 修复

- `BASE` 不再自称名字，只描述工作方式，并明确「名字、语气与边界由后面的『你是谁』一节定义；产品名不是你的名字」。
- identity 节增加「唯一真源」优先级：与本 prompt 其他段落（包括产品名）冲突时，以 identity 为准。
- 协议段把「niuse 会解析」改成「运行时会解析」，减少产品名被当作人格线索。
- 新增测试 `identity_is_the_only_name_source`：断言 `BASE` 不含硬编码名字，identity 的 `niubot` 与优先级文本都出现。

## 验收

- [x] `BASE` 不再包含「你是 niuse」。
- [x] identity 节声明唯一真源。
- [x] 真实模型在 identity 写 niubot 时直接答「我是 niubot」，无需纠正。
- [x] 现有 prompt 顺序与预算测试不变。

## 备注

产品名仍然出现在启动 banner、命令和文档里；它只是项目名，不是人格。identity 缺失时 persona 也不自报一个假名字。
