+++
id = "M0-001"
title = "五件套与原子存储"
status = "done"
milestone = "M0"
module = "stores"
size = "L"
priority = "P0"
updated = "2026-10-21"
completed = "2026-10-21"
docs = ["architecture", "adr-0001", "adr-0002"]
+++

## 范围

- `identity.md`、`user.md`、`memory/`、`journal/`、`loops.md` 五个持久化存储。
- 全部 Markdown，目录 `0700`、文件 `0600`，写临时文件再 rename。
- memory 按类型拆成 `preferences.md` / `lessons.md`；journal 按月一个文件、按天合并。
- 日期用自带的 civil-date 换算，不引入时区库。

## 验收

- [x] 五件套读写往返有单测。
- [x] journal 同一天追加会合并，不同日期按序。
- [x] 权限实测为 `0700` / `0600`。
- [x] 缺失 `identity.md` 时返回 `None`，不伪造空字符串。

## 记录

Markdown 是唯一格式：单用户、200 条以内，文件就是最好的查询引擎。写回会重排格式，手工加的注释可能丢失，这是接受的代价。
