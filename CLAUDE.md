# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 开发规范

- **不要主动提交代码**。只有在用户明确要求时才执行 git commit 操作。

## 项目概述

`zotero-rdf` 是一个专注于解析 Zotero 导出 RDF/XML 文件的 Rust 库。它不追求通用 RDF 解析能力，而是针对 Zotero 的数据结构特点提供高效、强类型的解析接口。

**范围：**
- **输入：** Zotero 导出的 RDF/XML 文件（`.rdf`）
- **输出：** Rust 结构体表示的文献元数据，或原始 `oxrdf::Graph`
- **不支持：** Turtle、N-Triples 格式；RDF 写入/序列化功能

## 构建命令

```bash
# 构建库
cargo build

# 运行测试
cargo test

# 运行单个测试
cargo test test_name

# 检查编译错误（比 build 更快）
cargo check

# 生成文档
cargo doc --open
```

## 架构设计

采用精简的分层架构：

```
src/
├── lib.rs       # 库入口，导出公共 API
├── error.rs     # ZoteroRdfError 错误定义
├── vocab.rs     # 命名空间 URI 常量（Zotero、Dublin Core、FOAF、BIBO）
├── parser.rs    # 核心解析逻辑（封装 oxrdfxml）
└── extractor.rs # 高级数据提取（Graph → 结构体）
```

**数据流：** RDF/XML 文件 → `oxrdfxml` 解析器 → `oxrdf::Graph` → `ZoteroItem` 结构体

## 核心命名空间

解析时需基于 URI 而非 XML 前缀匹配：

| 命名空间 | URI | 用途 |
|----------|-----|------|
| Zotero Export | `http://www.zotero.org/namespaces/export#` | Zotero 特有字段（`z:itemType`、`z:key`）|
| Dublin Core | `http://purl.org/dc/elements/1.1/` | 标准元数据（`dc:title`、`dc:date`）|
| FOAF | `http://xmlns.com/foaf/0.1/` | 作者信息（`foaf:surname`、`foaf:givenName`）|
| BIBO | `http://purl.org/ontology/bibo/` | 引文详情（`bibo:doi`、`bibo:pages`）|

## 依赖项

- `oxrdf` - RDF 核心数据模型（Graph、Triple、Literal）
- `oxrdfxml` - 基于 quick-xml 的 RDF/XML 解析器
- `thiserror` - 错误类型派生
- `once_cell` - 命名空间常量的惰性初始化

## 测试数据

样例 RDF 文件位于 `rdfs/` 目录：
- `rdfs/gear-measure-without-attachments.rdf`
- `rdfs/simulation-with-attachments/` - 包含 RDF 文件及关联的 PDF 附件

## 技术文档

详细设计规范和实现示例见 `docs/technology-document.md`。
