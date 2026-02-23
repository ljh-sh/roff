# Man Parser 开发进度与问题

## 当前状态

### ✅ 已完成
1. 创建了 compact Rust man parser (在 roff 目录下)
2. 实现了 JSON 转换功能
3. 实现了 Markdown 转换功能（包含 YAML header）
4. CLI 命令：`roff tojson <file>` 和 `roff tomd <file>`
5. 支持多文件处理
6. 支持 stdin 输入
7. 大部分 roff macros 已实现
8. 基本的列表解析（`.Bl` / `.El`）
9. 修复了列表内 macro 行处理（`.It` 后面的 `.Pq` 等）

---

## 测试结果

### ✅ mac 目录 - 全部通过
- `man1`: 所有 .1 文件解析成功
- `man3`: 所有 .3 文件解析成功
- 其他 section: 全部成功

### ✅ kernel 目录 - 全部通过
- 所有 .1 - .8 文件解析成功

---

## 代码位置

- 主解析逻辑：`roff/src/lib.rs`
- CLI 入口：`roff/src/main.rs`
- 测试文件：`roff/tests/basic.rs`

---

## 使用方法

```bash
# JSON 转换
./target/release/roff tojson <manfile>

# Markdown 转换（带 YAML header）
./target/release/roff tomd <manfile>

# 多文件
./target/release/roff tojson file1.1 file2.1

# stdin
cat file.1 | ./target/release/roff tojson -
```
