// 引入 serde_json 用于 JSON 序列化，std::path 用于文件路径处理
use serde_json::{Map, Value};
use std::path::Path;

/// 表示 man 文档中的一个 section（章节）
/// 例如：NAME, SYNOPSIS, DESCRIPTION 等
#[derive(Default)]
struct Section {
    title: String,      // 章节标题，如 "NAME", "DESCRIPTION"
    text: String,       // 章节内的普通文本内容
    items: Vec<String>, // 章节内的列表项（如 OPTIONS 下的 -a, -l 等）
    in_list: bool,      // 标记当前是否在列表中（.Bl/.El 之间）
}

/// 表示整个 man 文档的结构
#[derive(Default)]
struct Doc {
    title: Option<String>,      // 文档标题，如 "LS"
    section: Option<String>,    // 手册 section，如 "1"
    date: Option<String>,       // 文档日期
    name: Option<String>,       // 命令/函数名称
    desc: Option<String>,       // 简短描述
    envs: Vec<String>,          // 环境变量列表
    xrefs: Vec<String>,         // 交叉引用列表 (Xr)
    sections: Vec<Section>,    // 所有章节的列表
}

/// 去除 macro 参数的首尾空白和双引号
/// 例如: `"  hello  "` -> `hello`
fn trim_macro_arg(s: &str) -> String {
    s.trim().trim_matches('"').to_string()
}

/// 读取文件内容，使用 lossy 方式处理 UTF-8（用于处理 mac 目录的 ISO-8859 编码文件）
pub fn read_to_string_lossy<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

/// 将文本推送到当前 section 的 text 字段
/// 如果 text 非空，会先添加一个空格再追加内容
/// 会对文本进行转义序列处理
fn push_text(sec: &mut Section, line: &str) {
    if !sec.text.is_empty() {
        sec.text.push(' ');
    }
    sec.text.push_str(&format_inline_macros(line));
}

/// 处理行内的转义字符和 & 引用
/// 处理 \\& -> (移除), \\e -> \, &. -> . 等
/// 处理 \fB -> **, \fR -> 关闭粗体, \fI -> *, \fP -> 关闭斜体, \f(CW -> `, \f4 -> `
fn format_inline_macros(arg: &str) -> String {
    let mut result = String::new();
    let mut i = 0;
    let chars: Vec<char> = arg.chars().collect();
    let mut bold_open = false;
    let mut italic_open = false;

    while i < chars.len() {
        // 处理反斜杠转义
        if chars[i] == '\\' {
            i += 1;
            if i < chars.len() {
                // 处理 \fX 字体转义序列
                if chars[i] == 'f' && i + 1 < chars.len() {
                    let font = chars[i + 1];
                    match font {
                        'B' | 'b' | '3' => {
                            // 开启粗体
                            if italic_open {
                                result.push('*');
                                italic_open = false;
                            }
                            if bold_open {
                                result.push_str("**");
                            } else {
                                result.push_str("**");
                                bold_open = true;
                            }
                            i += 2;
                            continue;
                        }
                        'R' | 'r' | '1' => {
                            // 关闭粗体，回到普通字体
                            if bold_open {
                                result.push_str("**");
                                bold_open = false;
                            }
                            i += 2;
                            continue;
                        }
                        'I' | 'i' | '2' => {
                            // 开启斜体
                            if bold_open {
                                result.push_str("**");
                                bold_open = false;
                            }
                            if !italic_open {
                                result.push('*');
                                italic_open = true;
                            }
                            i += 2;
                            continue;
                        }
                        'P' | 'p' => {
                            // 关闭当前字体（斜体或粗体），回到普通字体
                            if italic_open {
                                result.push('*');
                                italic_open = false;
                            }
                            if bold_open {
                                result.push_str("**");
                                bold_open = false;
                            }
                            i += 2;
                            continue;
                        }
                        '(' => {
                            // \f(CW - 等宽字体
                            if bold_open {
                                result.push_str("**");
                                bold_open = false;
                            }
                            if italic_open {
                                result.push('*');
                                italic_open = false;
                            }
                            if i + 3 < chars.len() {
                                let cw: String = chars[i+2..i+4].iter().collect();
                                if cw == "CW" || cw == "cw" {
                                    result.push('`');
                                    i += 4;
                                    continue;
                                }
                            }
                            result.push('\\');
                            i += 1;
                            continue;
                        }
                        '4' => {
                            // \f4 - 等宽字体 (VT100)
                            if bold_open {
                                result.push_str("**");
                                bold_open = false;
                            }
                            if italic_open {
                                result.push('*');
                                italic_open = false;
                            }
                            result.push('`');
                            i += 2;
                            continue;
                        }
                        _ => {
                            result.push('\\');
                            i += 1;
                            continue;
                        }
                    }
                }
                if chars[i] == '&' {
                    // \\& 表示输出 & 后的字符（如 &. 输出 .）
                    i += 1;
                    if i < chars.len() {
                        result.push(chars[i]);
                    }
                } else if chars[i] == 'e' {
                    // \\e 转义为 \
                    result.push('\\');
                } else {
                    result.push(chars[i]);
                }
            }
            i += 1;
            continue;
        }

        // 处理 & 引用（&后跟标点符号时输出该符号）
        if chars[i] == '&' {
            i += 1;
            if i < chars.len() {
                let next = chars[i];
                if next == '.' || next == ',' || next == ';' || next == ':' {
                    result.push(next);
                } else {
                    result.push('&');
                    result.push(next);
                }
            }
            i += 1;
            continue;
        }

        result.push(chars[i]);
        i += 1;
    }

    // 关闭未闭合的字体标记
    if bold_open {
        result.push_str("**");
    }
    if italic_open {
        result.push('*');
    }

    result
}

/// 根据 macro 名称格式化参数
/// 将 roff macro 转换为 Markdown 格式
/// 例如: .Fl a -> -a, .Ar file -> _file_, .Pq x -> (x)
fn format_macro(macro_name: &str, arg: &str) -> String {
    match macro_name {
        "Op" => format!("[{}]", format_nested_macros(arg)),      // [可选参数]
        "Ar" => format!("_{}_", format_nested_macros(arg)),       // _参数_
        "Fl" => format!("-{}", format_nested_macros(arg).trim_start_matches('-')), // -选项
        "Pa" => format_nested_macros(arg),                        // 文件路径（保持原样）
        "Xr" => {                                                  // 交叉引用，如 ls(1)
            let parts: Vec<&str> = arg.trim().split_whitespace().collect();
            if parts.len() >= 2 {
                format!("**{}**({})", parts[0], parts[1])
            } else if !parts.is_empty() {
                format!("**{}**", parts[0])
            } else {
                String::new()
            }
        }
        "Li" => format!("`{}`", format_nested_macros(arg)),       // `代码`
        "Va" => format!("_{}_", format_nested_macros(arg)),        // _变量_
        "Ev" => format!("_{}_", format_nested_macros(arg)),        // _环境变量_
        "Cm" => format!("**{}**", format_nested_macros(arg)),      // **命令**
        "Tn" => format_nested_macros(arg),                         // 技术术语（保持原样）
        "Sq" => format!("'{}'", format_nested_macros(arg)),       // '单引号'
        "Ql" => format!("`{}`", format_nested_macros(arg)),       // `原义`
        "Dq" => format!("\"{}\"", format_nested_macros(arg)),      // "双引号"
        "Em" => format!("_{}_", format_nested_macros(arg)),        // _强调_
        "Sy" => format!("**{}**", format_nested_macros(arg)),     // **粗体**
        "Pq" => format!("({})", format_nested_macros(arg)),       // (圆括号)
        "Nm" => format!("**{}**", format_nested_macros(arg)),     // **名称**
        "St" => String::new(),                                     // 标准（不输出）
        _ => format_nested_macros(arg),                            // 其他 macro 递归处理
    }
}

/// 处理嵌套的 inline macro
/// 例如: .Pq Sq Pa \&. -> (. ')
/// 先检查第一个词是否是 macro，如果是则递归处理
fn format_nested_macros(arg: &str) -> String {
    let trimmed = arg.trim();
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    if words.is_empty() {
        return format_inline_macros(arg);
    }

    let first = words[0];
    if is_inline_macro(first) {
        let rest = words[1..].join(" ");
        format_macro(first, &rest)
    } else {
        format_inline_macros(arg)
    }
}

/// 检查是否是 inline macro（不需要换行的行内 macro）
fn is_inline_macro(name: &str) -> bool {
    matches!(name, "Fl" | "Ar" | "Nm" | "Pa" | "Cm" | "Va" | "Ev" | "Li" | "Sy" | "Em" | "Sq" | "Ql" | "Dq" | "Tn" | "Xr" | "Op" | "Pq")
}

/// 将 man 文档内容解析为 JSON Value
/// 这是核心解析函数，逐行处理 roff macro
pub fn parse_to_json(input: &str) -> Value {
    let mut doc = Doc::default();      // 整个文档
    let mut current = Section::default(); // 当前正在处理的章节
    let mut have_section = bool::default(); // 是否已经有章节
    let mut found_header = false;      // 是否已经找到文档标题 (.Dt/.TH)

    // 逐行解析输入
    for raw in input.lines() {
        let line = raw.trim_end();

        // 跳过注释行 .\" 开头的行
        if line.starts_with(".\"") {
            continue;
        }

        // .Dt TITLE SECTION - 文档标题和 section
        // .TH TITLE SECTION - BSD/macOS 风格，等同于 .Dt
        if line.starts_with(".Dt ") || line.starts_with(".TH ") {
            // 找到标题后，清除之前解析的所有内容（版权声明等）
            doc = Doc::default();
            current = Section::default();
            have_section = false;
            found_header = true;

            let rest = line[4..].trim();
            let mut parts = rest.split_whitespace();
            let t = parts.next().map(|s| s.to_string());
            let sec = parts.next().map(|s| s.to_string());
            doc.title = t;
            doc.section = sec;
            continue;
        }

        // 如果还没有找到标题，先跳过所有内容
        if !found_header {
            continue;
        }

        // .Dd DATE - 文档日期
        if line.starts_with(".Dd ") {
            doc.date = Some(trim_macro_arg(&line[4..]));
            continue;
        }

        // .Os - 操作系统（跳过）
        if line.starts_with(".Os") {
            continue;
        }

        // .St - 标准（跳过，不输出）
        if line.starts_with(".St") {
            continue;
        }

        // 忽略格式化指令：.ad .na .hy .br .sp .nr 等
        if line == ".ad" || line == ".na" || line.starts_with(".hy") ||
           line == ".br" || line == ".sp" || line.starts_with(".nr") ||
           line == ".ns" || line == ".rs" || line.starts_with(".ll") ||
           line.starts_with(".ta") || line == ".fi" || line == ".nf" {
            continue;
        }

        // .Sh TITLE - 开始新章节 (支持 .Sh 和 .SH)
        let line_upper = line.to_uppercase();
        if line.starts_with(".Sh ") || line_upper.starts_with(".SH ") {
            if have_section {
                doc.sections.push(current);
                current = Section::default();
            } else {
                have_section = true;
            }
            current.title = trim_macro_arg(&line[4..]);
            continue;
        }
        // .Nm NAME - 命令/函数名称
        if line.starts_with(".Nm") {
            let arg = line.get(3..).unwrap_or("").trim();
            if !arg.is_empty() {
                if doc.name.is_none() {
                    doc.name = Some(trim_macro_arg(arg));
                } else {
                    push_text(&mut current, &format!("**{}**", trim_macro_arg(arg)));
                }
            } else if let Some(ref n) = doc.name {
                push_text(&mut current, &format!("**{}**", n));
            }
            continue;
        }

        // .Nd DESCRIPTION - 简短描述
        if line.starts_with(".Nd ") {
            doc.desc = Some(trim_macro_arg(&line[4..]));
            continue;
        }

        // .Ev ENV_VAR - 环境变量（收集到列表中）
        if line.starts_with(".Ev ") {
            let env = trim_macro_arg(&line[4..]);
            if !env.is_empty() && !doc.envs.contains(&env) {
                doc.envs.push(env);
            }
            continue;
        }

        // .Xr NAME SECTION - 交叉引用（收集到列表中）
        if line.starts_with(".Xr ") {
            let xref = trim_macro_arg(&line[4..]);
            if !xref.is_empty() && !doc.xrefs.contains(&xref) {
                doc.xrefs.push(xref);
            }
            continue;
        }

        // .Bl - 开始列表（tagged list, enum 等）
        if line.starts_with(".Bl") || (line.len() >= 3 && line.starts_with(".") && &line[1..3] == "Bl") {
            current.in_list = true;
            continue;
        }

        // .El - 结束列表
        if line.starts_with(".El") || (line.len() >= 3 && line.starts_with(".") && &line[1..3] == "El") {
            current.in_list = false;
            continue;
        }

        // .It - 列表项
        if line.starts_with(".It") || (line.len() >= 3 && line.starts_with(".") && &line[1..3] == "It") {
            let arg = line.get(3..).unwrap_or("").trim();
            if current.in_list {
                if !arg.is_empty() {
                    let formatted = format_nested_macros(arg);
                    current.items.push(formatted);
                } else {
                    current.items.push(String::new());
                }
            } else {
                if !arg.is_empty() {
                    push_text(&mut current, arg.trim());
                }
            }
            continue;
        }
        // .Pp - 段落分隔
        if line.starts_with(".Pp") {
            if !current.text.is_empty() && !current.text.ends_with('\n') {
                current.text.push_str("\n\n");
            }
            continue;
        }

        // 在列表内处理 macro 行（重要：添加到 items 而不是 text）
        if current.in_list && line.starts_with('.') && line.len() > 2 {
            let macro_part = &line[1..3];
            let rest = if line.len() > 3 {
                line[3..].trim()
            } else {
                ""
            };
            let formatted = format_macro(macro_part, rest);
            if !formatted.is_empty() {
                if let Some(last) = current.items.last_mut() {
                    if !last.is_empty() {
                        last.push(' ');
                    }
                    last.push_str(&formatted);
                }
                continue;
            }
        }
        if line.starts_with('.') && line.len() > 2 {
            let macro_part = &line[1..3];
            let rest = if line.len() > 3 {
                line[3..].trim()
            } else {
                ""
            };
            let formatted = format_macro(macro_part, rest);
            if !formatted.is_empty() {
                push_text(&mut current, &formatted);
                continue;
            }
        }
        if line.starts_with('.') {
            continue;
        }
        if current.in_list {
            if let Some(last) = current.items.last_mut() {
                let trimmed = line.trim();
                if trimmed.starts_with('.') && trimmed.len() > 2 {
                    let macro_part = &trimmed[1..3];
                    let rest = if trimmed.len() > 3 {
                        &trimmed[3..]
                    } else {
                        ""
                    };
                    let formatted = format_macro(macro_part, rest.trim());
                    if !formatted.is_empty() {
                        if !last.is_empty() {
                            last.push(' ');
                        }
                        last.push_str(&formatted);
                    }
                } else if !trimmed.is_empty() {
                    let formatted = format_inline_macros(trimmed);
                    if !last.is_empty() {
                        last.push(' ');
                    }
                    last.push_str(&formatted);
                }
            } else {
                let trimmed = line.trim();
                let formatted = format_inline_macros(trimmed);
                current.items.push(formatted);
            }
        } else {
            if !line.trim().is_empty() {
                push_text(&mut current, line.trim());
            }
        }
    }
    if have_section {
        doc.sections.push(current);
    }

    let mut sections_json = Vec::new();
    for s in doc.sections {
        let mut o = Map::new();
        o.insert("title".to_string(), Value::String(s.title));
        if !s.text.trim().is_empty() {
            o.insert("text".to_string(), Value::String(s.text.trim().to_string()));
        }
        if !s.items.is_empty() {
            let arr = s
                .items
                .into_iter()
                .map(|v| Value::String(v.trim().to_string()))
                .collect::<Vec<_>>();
            o.insert("items".to_string(), Value::Array(arr));
        }
        sections_json.push(Value::Object(o));
    }

    let mut root = Map::new();
    if let Some(t) = doc.title {
        root.insert("title".to_string(), Value::String(t));
    }
    if let Some(s) = doc.section {
        root.insert("section".to_string(), Value::String(s));
    }
    if let Some(d) = doc.date {
        root.insert("date".to_string(), Value::String(d));
    }
    if let Some(n) = doc.name {
        root.insert("name".to_string(), Value::String(n));
    }
    if let Some(d) = doc.desc {
        root.insert("description".to_string(), Value::String(d));
    }
    if !doc.envs.is_empty() {
        let arr = doc.envs.into_iter().map(Value::String).collect();
        root.insert("envs".to_string(), Value::Array(arr));
    }
    if !doc.xrefs.is_empty() {
        let arr = doc.xrefs.into_iter().map(Value::String).collect();
        root.insert("xrefs".to_string(), Value::Array(arr));
    }
    root.insert("sections".to_string(), Value::Array(sections_json));
    Value::Object(root)
}

pub fn parse_to_string(input: &str, pretty: bool) -> String {
    let v = parse_to_json(input);
    if pretty {
        serde_json::to_string_pretty(&v).unwrap()
    } else {
        serde_json::to_string(&v).unwrap()
    }
}

pub fn to_markdown(json: &Value) -> String {
    let mut out = String::new();

    out.push_str("---\n");
    if let Some(t) = json.get("title").and_then(|v| v.as_str()) {
        out.push_str("title: ");
        out.push_str(t);
        out.push('\n');
    }
    if let Some(s) = json.get("section").and_then(|v| v.as_str()) {
        out.push_str("section: ");
        out.push_str(s);
        out.push('\n');
    }
    if let Some(n) = json.get("name").and_then(|v| v.as_str()) {
        out.push_str("name: ");
        out.push_str(n);
        out.push('\n');
    }
    if let Some(d) = json.get("description").and_then(|v| v.as_str()) {
        out.push_str("description: ");
        out.push_str(d);
        out.push('\n');
    }
    if let Some(date) = json.get("date").and_then(|v| v.as_str()) {
        out.push_str("date: ");
        out.push_str(date);
        out.push('\n');
    }
    if let Some(envs) = json.get("envs").and_then(|v| v.as_array()) {
        if !envs.is_empty() {
            out.push_str("env:\n");
            for env in envs {
                if let Some(e) = env.as_str() {
                    out.push_str("  ");
                    out.push_str(e);
                    out.push_str(": true\n");
                }
            }
        }
    }
    if let Some(xrefs) = json.get("xrefs").and_then(|v| v.as_array()) {
        if !xrefs.is_empty() {
            out.push_str("xref:\n");
            for xref in xrefs {
                if let Some(x) = xref.as_str() {
                    out.push_str("  - ");
                    out.push_str(x);
                    out.push('\n');
                }
            }
        }
    }
    out.push_str("---\n\n");

    if let Some(t) = json.get("title").and_then(|v| v.as_str()) {
        out.push_str("# ");
        out.push_str(t);
        if let Some(s) = json.get("section").and_then(|v| v.as_str()) {
            out.push('(');
            out.push_str(s);
            out.push(')');
        }
        out.push('\n');
    }
    if let Some(n) = json.get("name").and_then(|v| v.as_str()) {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("\n**");
        out.push_str(n);
        out.push_str("**");
        if let Some(d) = json.get("description").and_then(|v| v.as_str()) {
            out.push_str(" - ");
            out.push_str(d);
        }
        out.push('\n');
    }
    if let Some(sections) = json.get("sections").and_then(|v| v.as_array()) {
        for sec in sections {
            if let Some(title) = sec.get("title").and_then(|v| v.as_str()) {
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str("\n## ");
                out.push_str(title);
                out.push('\n');
            }
            if let Some(text) = sec.get("text").and_then(|v| v.as_str()) {
                if !text.trim().is_empty() {
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                    for para in text.split('\n') {
                        let p = para.trim();
                        if !p.is_empty() {
                            out.push_str(p);
                            out.push_str("\n\n");
                        }
                    }
                }
            }
            if let Some(items) = sec.get("items").and_then(|v| v.as_array()) {
                for item in items {
                    if let Some(s) = item.as_str() {
                        if !s.trim().is_empty() {
                            out.push_str("- ");
                            out.push_str(s.trim());
                            out.push('\n');
                        }
                    }
                }
            }
        }
    }
    out.trim_end().to_string()
}
