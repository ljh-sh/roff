# roff-cli

[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/ljh-sh/roff/badge)](https://scorecard.dev/)
[![CI](https://github.com/ljh-sh/roff/actions/workflows/ci.yml/badge.svg)](https://github.com/ljh-sh/roff/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/roff-cli.svg)](https://crates.io/crates/roff-cli)
[![Docs](https://img.shields.io/badge/Docs-website-blue.svg)](https://roff.ljh.sh)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

> 精巧的 man page 转 JSON/Markdown 工具 —— 人可读、AI 友好。

**roff-cli** 将 Unix man 页面（`roff`/`mdoc`）转换为结构化的 JSON 或易读的 Markdown。适用于文档流水线、LLM 上下文窗口，或无需分页器快速查看 man 页面。

镜像: [github.com/ljh-sh/roff](https://github.com/ljh-sh/roff) · [codeberg.org/ljh-sh/roff](https://codeberg.org/ljh-sh/roff)

文档: [roff.ljh.sh](https://roff.ljh.sh)

## 给 AI 代理

最小上下文、最大灵活性 —— 将下面这一行粘贴到 Claude Code、Cursor 或任何代理的系统提示中：

```md
使用 `roff` 将 man 页面转换为 JSON/Markdown。如未安装：`cargo install roff-cli`。运行 `roff --help` 查看子命令。
```

## 安装

### Cargo（推荐）

```bash
cargo install roff-cli
```

### 直接下载二进制

```bash
curl -L https://github.com/ljh-sh/roff/releases/latest/download/roff-x86_64-unknown-linux-musl.tar.xz | tar xJ -
sudo mv roff-x86_64-unknown-linux-musl/bin/roff /usr/local/bin/
```

所有目标（Linux musl/glibc、Windows、macOS）请查看 [releases 页面](https://github.com/ljh-sh/roff/releases)。

### eget

通过 [x-cmd eget](https://x-cmd.com/install/roff)：

```bash
x eget ljh-sh/roff        # 下载并安装
x eget use ljh-sh/roff    # 安装到 ~/.local/bin
```

### 从源码构建

需要 Rust 1.74+。

```bash
git clone https://github.com/ljh-sh/roff
cd roff
cargo build --release
```

二进制位于 `target/release/roff`。

## 速览

```bash
roff tojson file.1          # 结构化 JSON
roff tomd file.1            # 可读 Markdown
roff view --meta ls         # 渐进式查看
roff bench --count 100      # 在 man 页面上的解析器基准测试
```

---

## 使用

### 转换为 JSON

```bash
roff tojson file.1
roff tojson --indent 2 file.1           # 2 空格缩进美化输出
roff tojson --indent 4 file.1           # 4 空格缩进美化输出
roff tojson --source-expand file.1      # 展开 .so 包含
roff tojson -- < file.1                 # 从 stdin 读取
```

示例输出：

```json
{
  "title": "LS",
  "section": "1",
  "name": "ls",
  "description": "list directory contents",
  "sections": [
    {"title": "NAME", "text": "ls — list directory contents"},
    {"title": "SYNOPSIS", "text": "ls [options] [file ...]"},
    {"title": "OPTIONS", "items": ["-a: do not ignore entries starting with .", "-l: use a long listing format"]}
  ]
}
```

### 转换为 Markdown

```bash
roff tomd file.1
roff tomd --source-expand file.1
```

输出包含 YAML front matter 和清晰的 Markdown 章节。

### 渐进式查看

分部分查看 man 页面 —— 非常适合 AI 代理和快速检查。多个选项可组合使用。

```bash
roff view --description file.1      # 名称 + 描述
roff view --synopsis file.1         # 概要章节
roff view --options file.1          # 选项章节
roff view --see-also file.1         # 相关章节
roff view --meta file.1             # 描述 + 概要 + 相关 + 大纲
roff view --outline file.1          # 所有章节标题
roff view --outline-head 3 file.1   # 标题 + 每章前 3 行
```

| 选项 | 描述 |
|------|------|
| `--description` | 名称 + 描述 |
| `--synopsis` | 概要章节 |
| `--options` | 选项章节 |
| `--environment` | 环境变量章节 |
| `--files` | 文件章节 |
| `--exit-status` | 退出状态章节 |
| `--see-also` | 相关章节 |
| `--examples` | 示例章节 |
| `--author` | 作者章节 |
| `--outline` | 显示所有章节标题 |
| `--outline-head N` | 显示标题 + 每章前 N 行 |
| `--meta` | 快捷: `--description --synopsis --see-also --outline` |
| `--all` | 显示所有章节 |

### 基准测试

```bash
roff bench                # 处理前 10 个文件
roff bench --count 100    # 处理前 100 个文件
roff bench --all          # 处理所有 manpath 文件
```

## 库使用

```rust
use roff::{parse_to_json, parse_to_string, to_markdown};

let input = ".TH TEST 1\n.SH NAME\ntest \\- a test program";
let json = parse_to_json(input);
let md = to_markdown(&json);
let s = parse_to_string(input, true);  // 格式化 JSON
```

## 设计

roff-cli 故意保持**简单**：它解析 roff/mdoc 宏并返回结构化数据。不渲染终端页面、不应用本地化格式、不合成摘要。决策权交给调用方：

```bash
roff tojson ls | jq '.sections[] | select(.title=="OPTIONS") | .items[]'
roff tomd git-push.1 | grep -A5 "## OPTIONS"
```

这样 `roff --help` 保持简短，作为 LLM 工具使用成本很低。

## 常见问题

详见 [docs/faq.md](docs/faq.md) 或 [线上常见问题](https://roff.ljh.sh/faq)，了解支持的格式、`.so` 展开、manpath 搜索、性能、stdin 处理等。

### 支持哪些 roff 方言？

主要支持 BSD `mdoc` 和传统 `man` 宏。解析器处理系统 man 页面中最常用的宏（`.Sh`、`.Ss`、`.Nm`、`.Nd`、`.It`、`.Bl`、`.El`、`.Xr`、`.Ev`、字体转义等）。

### 是否处理 `.so` 包含？

是的，使用 `--source-expand`。默认情况下 `.so` 文件名会记录在 `source` 数组中，但不展开。

### 输出是否稳定？

已支持字段的 JSON schema 是稳定的。随着解析器学习更多宏，可能会添加新字段。

### 能否在非 Linux 系统使用？

可以。为 Linux（musl/glibc）、Windows 和 macOS（Intel 和 Apple Silicon）提供预编译二进制。任何 Rust 可编译的地方都能构建。

### 二进制有多大？

裁剪后的 Linux x86_64 musl 二进制大约 1.5–2 MB（Rust + serde_json）。无运行时依赖。

### 为什么不直接用 `man --html` 或 `mandoc -Tmarkdown`？

这些工具非常适合人类阅读。roff-cli 面向结构化数据：JSON 给脚本用，Markdown 给文档流水线用，渐进视图给 LLM 上下文窗口用。

---

## 路线图

详见 [ROADMAP.md](ROADMAP.md)。

## 更新日志

详见 [`changelog/`](changelog/)。

## 贡献

详见 [CONTRIBUTING.md](CONTRIBUTING.md)。欢迎提交 issue 和 PR。

## 安全

详见 [SECURITY.md](SECURITY.md)。漏洞请邮件联系 [lijunhao@x-cmd.com](mailto:lijunhao@x-cmd.com)，不要公开提交 issue。

## 行为准则

详见 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)。

## 协议

Apache 2.0 —— 详见 [LICENSE](LICENSE)。
