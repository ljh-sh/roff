# roff-cli

简洁的 man page 转 JSON/Markdown 转换器 - 人可读、AI 友好

## 功能

- **tojson**: 将 roff/man 文件转换为 JSON
- **tomd**: 将 roff/man 文件转换为 Markdown  
- **view**: 渐进式查看 - 分部分查看 man 页面
- **bench**: 基准测试解析器性能

## 安装

```bash
cargo install roff-cli
# 或
cargo install --path .
```

## 使用方法

### 转换为 JSON/Markdown

```bash
# 转换为 JSON
roff tojson file.1

# 格式化输出 JSON
roff tojson --pretty file.1

# 转换为 Markdown
roff tomd file.1

# 从 stdin 读取
roff tojson -- < file.1
roff tomd -- < file.1
```

### 渐进式查看 (view)

分部分查看 man 页面 - 非常适合 AI 代理和快速预览。支持多个选项组合使用。

```bash
# 查看指定章节 (可以组合多个)
roff view --description file.1      # 名称 + 描述
roff view --synopsis file.1          # 概要章节
roff view --options file.1           # 选项章节
roff view --see-also file.1         # 相关章节

# 组合多个选项
roff view --description --synopsis file.1   # 显示描述 + 概要
roff view --options --examples file.1        # 显示选项 + 示例
roff view --description --synopsis --see-also --environment file.1  # 显示多个

# 组合视图 (最常用)
roff view --meta file.1              # 描述 + 概要 + 相关 + 大纲
roff view file.1                     # 与 --meta 相同

# 大纲模式 - 显示章节标题
roff view --outline file.1           # 显示所有章节名称

# 带预览的大纲 - 显示标题 + 前 N 行
roff view --outline-head 3 file.1   # 显示标题 + 每章前 3 行
```

#### View 选项

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
| `--meta` | 快捷: --description --synopsis --see-also --outline |
| `--all` | 显示所有章节 |

### 基准测试

```bash
roff bench                # 处理前 10 个文件
roff bench --count 100    # 处理前 100 个文件
roff bench --all          # 处理所有 manpath 文件
```

## 库使用

```rust
use roff::{parse_roff, to_json, to_markdown};

let input = ".TH TEST 1\n.SH NAME\ntest \\- a test program";
let roff = parse_roff(input);

let json = to_json(&roff);
let md = to_markdown(&roff);
```

## 协议

Apache-2.0
