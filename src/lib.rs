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
    title: Option<String>,   // 文档标题，如 "LS"
    section: Option<String>, // 手册 section，如 "1"
    date: Option<String>,    // 文档日期
    name: Option<String>,    // 命令/函数名称
    desc: Option<String>,    // 简短描述
    envs: Vec<String>,       // 环境变量列表
    xrefs: Vec<String>,      // 交叉引用列表 (Xr)
    source: Vec<String>,     // .so 展开的文件列表
    sections: Vec<Section>,  // 所有章节的列表
}

/// 将 JSON Value 转换为 Section 结构体
fn json_value_to_section(v: &serde_json::Value) -> Option<Section> {
    let obj = v.as_object()?;
    let title = obj.get("title")?.as_str()?.to_string();

    // Skip sections with empty titles
    if title.is_empty() {
        return None;
    }

    let text = obj
        .get("text")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();
    let items: Vec<String> = obj
        .get("items")
        .and_then(|i| i.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Some(Section {
        title,
        text,
        items,
        in_list: false,
    })
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
                                let cw: String = chars[i + 2..i + 4].iter().collect();
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
/// 例如: .Fl a -> -a, .Ar file -> file, .Pq x -> (x)
fn format_macro(macro_name: &str, arg: &str) -> String {
    match macro_name {
        "Op" => format!("[{}]", format_nested_macros(arg)), // [可选参数]
        "Ar" => {
            if arg.is_empty() {
                "file ...".to_string() // default for .Ar with no argument
            } else {
                format_nested_macros(arg) // 参数直接显示，不加装饰
            }
        } // 参数（不加装饰）
        "Fl" => {
            let processed = format_nested_macros(arg);
            if processed.starts_with('-') {
                processed // already has - prefix
            } else {
                format!("-{}", processed)
            }
        } // -选项
        "Pa" => format_nested_macros(arg),                  // 文件路径（保持原样）
        "Xr" => {
            // 交叉引用，如 ls(1)
            let parts: Vec<&str> = arg.trim().split_whitespace().collect();
            if parts.len() >= 2 {
                format!("**{}**({})", parts[0], parts[1])
            } else if !parts.is_empty() {
                format!("**{}**", parts[0])
            } else {
                String::new()
            }
        }
        "Li" => format!("`{}`", format_nested_macros(arg)), // `代码`
        "Va" => format!("_{}_", format_nested_macros(arg)), // _变量_
        "Ev" => format!("_{}_", format_nested_macros(arg)), // _环境变量_
        "Cm" => format!("**{}**", format_nested_macros(arg)), // **命令**
        "Tn" => format_nested_macros(arg),                  // 技术术语（保持原样）
        "Sq" => format!("'{}'", format_nested_macros(arg)), // '单引号'
        "Ql" => format!("`{}`", format_nested_macros(arg)), // `原义`
        "Dq" => format!("\"{}\"", format_nested_macros(arg)), // "双引号"
        "Em" => format!("_{}_", format_nested_macros(arg)), // _强调_
        "Sy" => format!("**{}**", format_nested_macros(arg)), // **粗体**
        "Pq" => format!("({})", format_nested_macros(arg)), // (圆括号)
        "Nm" => format!("**{}**", format_nested_macros(arg)), // **名称**
        "St" => String::new(),                              // 标准（不输出）
        "Ns" => format_nested_macros(arg),                  // No-space: suppress preceding space
        "No" => format_nested_macros(arg),                  // Normal text: output as-is
        _ => format_nested_macros(arg),                     // 其他 macro 递归处理
    }
}

/// 处理嵌套的 inline macro
/// 例如: .Pq Sq Pa \&. -> (. ')
/// 处理所有词作为 potential macro，递归处理每个 macro
fn format_nested_macros(arg: &str) -> String {
    let trimmed = arg.trim();
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    if words.is_empty() {
        return format_inline_macros(arg);
    }

    let mut result = String::new();
    let mut i = 0;
    let mut suppress_space = false;

    while i < words.len() {
        let word = words[i];

        if word == "Ns" {
            suppress_space = true;
            i += 1;
            continue;
        }

        if word == "No" {
            i += 1;
            continue;
        }

        if is_inline_macro(word) {
            // Collect remaining words for this macro
            let mut j = i + 1;
            let mut arg_end = j;
            let mut depth = 0;

            // Find where this macro's argument ends
            // Simple heuristic: count words that belong to this macro
            while j < words.len() {
                let w = words[j];
                if is_inline_macro(w) || w == "Ns" || w == "No" {
                    break;
                }
                arg_end = j + 1;
                j += 1;
            }

            let arg_words: Vec<&str> = words[i + 1..arg_end].iter().copied().collect();
            let arg_str = arg_words.join(" ");

            let formatted = format_macro(word, &arg_str);

            // Add space before formatted macro if needed
            if !result.is_empty() && !suppress_space {
                let last_char = result.chars().last().unwrap_or(' ');
                if last_char != ' '
                    && last_char != '-'
                    && last_char != '('
                    && last_char != '['
                    && last_char != '='
                {
                    result.push(' ');
                }
            }

            if suppress_space && !result.is_empty() && result.ends_with(' ') {
                result.pop(); // Remove trailing space
            }
            result.push_str(&formatted);
            suppress_space = false;

            i = arg_end;
            continue;
        } else {
            if !result.is_empty() && !suppress_space {
                let last_char = result.chars().last().unwrap_or(' ');
                if last_char != ' '
                    && last_char != '-'
                    && last_char != '('
                    && last_char != '['
                    && last_char != '='
                {
                    result.push(' ');
                }
            }
            result.push_str(word);
            suppress_space = false;
            i += 1;
        }
    }

    result
}

/// 检查是否是 inline macro（不需要换行的行内 macro）
fn is_inline_macro(name: &str) -> bool {
    matches!(
        name,
        "Fl" | "Ar"
            | "Nm"
            | "Pa"
            | "Cm"
            | "Va"
            | "Ev"
            | "Li"
            | "Sy"
            | "Em"
            | "Sq"
            | "Ql"
            | "Dq"
            | "Tn"
            | "Xr"
            | "Op"
            | "Pq"
            | "Ns"  // No-space - suppresses preceding space
            | "No" // Normal text - stays as is
    )
}

/// 将 man 文档内容解析为 JSON Value
/// 这是核心解析函数，逐行处理 roff macro
/// source_expand: 是否展开 .so 包含的文件
pub fn parse_to_json(input: &str) -> Value {
    parse_to_json_with_opts(input, false, None)
}

/// 解析并可选展开 .so 文件
/// base_path: 用于解析 .so 文件的相对路径
pub fn parse_to_json_with_opts(input: &str, source_expand: bool, base_path: Option<&str>) -> Value {
    let mut doc = Doc::default(); // 整个文档
    let mut current = Section::default(); // 当前正在处理的章节
    let mut have_section = bool::default(); // 是否已经有章节
    let mut found_header = false; // 是否已经找到文档标题 (.Dt/.TH)

    // 逐行解析输入
    for raw in input.lines() {
        let line = raw.trim_end();

        // 跳过注释行 .\" 开头的行 (dot, backslash, quote)
        // In Rust: .\\" means dot + backslash + quote
        if line.starts_with(".\\\"") {
            continue;
        }

        // .Dt TITLE SECTION - 文档标题和 section
        // .TH TITLE SECTION - BSD/macOS 风格，等同于 .Dt
        // 注意：某些文件（如 zshall）会在中间包含第二个 .TH，这时不应该重置文档
        if line.starts_with(".Dt ") || line.starts_with(".TH ") {
            if !found_header {
                // 找到标题后，清除之前解析的所有内容（版权声明等）
                doc = Doc::default();
                current = Section::default();
                have_section = false;
                found_header = true;

                let rest = line[4..].trim();
                let mut parts = rest.split_whitespace();
                let t = parts.next().map(|s| trim_macro_arg(s));
                let sec = parts.next().map(|s| trim_macro_arg(s));
                doc.title = t;
                doc.section = sec;
            }
            // Skip this line - either process first .TH or ignore subsequent .TH
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
        if line == ".ad"
            || line == ".na"
            || line.starts_with(".hy")
            || line == ".br"
            || line == ".sp"
            || line.starts_with(".nr")
            || line == ".ns"
            || line == ".rs"
            || line.starts_with(".ll")
            || line.starts_with(".ta")
            || line == ".fi"
            || line == ".nf"
        {
            continue;
        }

        // .Sh TITLE - 开始新章节 (支持 .Sh 和 .SH)
        let line_upper = line.to_uppercase();
        if line.starts_with(".Sh ") || line_upper.starts_with(".SH ") {
            // Only push current section if it has content
            if have_section
                && (!current.title.is_empty()
                    || !current.text.is_empty()
                    || !current.items.is_empty())
            {
                doc.sections.push(current);
                current = Section::default();
            }
            have_section = true;
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
            let mut xref = trim_macro_arg(&line[4..]);
            xref = xref.trim_end_matches(',').trim().to_string();
            if !xref.is_empty() && !doc.xrefs.contains(&xref) {
                doc.xrefs.push(xref);
            }
            continue;
        }

        // .Bl - 开始列表（tagged list, enum 等）
        if line.starts_with(".Bl")
            || (line.len() >= 3 && line.starts_with(".") && &line[1..3] == "Bl")
        {
            current.in_list = true;
            continue;
        }

        // .El - 结束列表
        if line.starts_with(".El")
            || (line.len() >= 3 && line.starts_with(".") && &line[1..3] == "El")
        {
            current.in_list = false;
            continue;
        }

        // .It - 列表项
        if line.starts_with(".It")
            || (line.len() >= 3 && line.starts_with(".") && &line[1..3] == "It")
        {
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

        // .IP - indented paragraph (like .It but for .IP "tag")
        // Creates a new list item with the tag
        if line.starts_with(".IP")
            || (line.len() >= 3 && line.starts_with(".") && &line[1..3] == "IP")
        {
            // Push previous item if exists
            if current.in_list && !current.items.is_empty() {
                // Keep current item, just add new content
            }
            current.in_list = true;

            let arg = line.get(3..).unwrap_or("").trim();
            if !arg.is_empty() {
                let formatted = format_nested_macros(arg);
                current.items.push(formatted);
            } else {
                current.items.push(String::new());
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

        // .P - new paragraph (same as .Pp)
        if line.starts_with(".P ") || line == ".P" {
            if !current.text.is_empty() && !current.text.ends_with('\n') {
                current.text.push_str("\n\n");
            }
            continue;
        }

        // .sp - vertical space
        if line.starts_with(".sp") {
            if !current.text.is_empty() && !current.text.ends_with('\n') {
                current.text.push_str("\n\n");
            }
            continue;
        }

        // .br - line break (within paragraph)
        if line.starts_with(".br") {
            if !current.text.is_empty() && !current.text.ends_with('\n') {
                current.text.push(' ');
            }
            continue;
        }

        // .nf - no-fill mode (preserve line breaks)
        if line.starts_with(".nf") {
            // Start a new line in no-fill mode - just add newline
            if !current.text.is_empty() && !current.text.ends_with('\n') {
                current.text.push('\n');
            }
            continue;
        }

        // .fi - fill mode (wrap text)
        if line.starts_with(".fi") {
            // Just continue, we're already handling text wrapping
            continue;
        }

        // .RS - relative start (indent)
        if line.starts_with(".RS") || line.starts_with(".rs") {
            continue;
        }

        // .RE - relative end (outdent)
        if line.starts_with(".RE") || line.starts_with(".re") {
            continue;
        }

        // .TP - Tagged paragraph (hanging indent)
        // The next line is the tag (bold), following lines are body
        if line.starts_with(".TP")
            || (line.len() >= 3 && line.starts_with(".") && &line[1..3] == "TP")
        {
            // Start a new item in the list for the tag
            current.in_list = true;
            current.items.push(String::new()); // Empty tag marker for TP
            continue;
        }

        // .PD - Paragraph distance (set to 0 for compact lists)
        // Just skip it for now, don't close lists automatically
        if line.starts_with(".PD")
            || (line.len() >= 3 && line.starts_with(".") && &line[1..3] == "PD")
        {
            current.in_list = false; // Close list after PD
            continue;
        }

        // 在列表内处理 macro 行（重要：添加到 items 而不是 text）
        // 但跳过控制性 macro 如 .PD, .TP 等
        if current.in_list && line.starts_with('.') && line.len() > 2 {
            // Skip control macros that shouldn't be added to list items
            let macro_name = &line[1..3];
            if matches!(
                macro_name,
                "PD" | "TP"
                    | "Bl"
                    | "El"
                    | "It"
                    | "PP"
                    | "Sp"
                    | "Rs"
                    | "Re"
                    | "IP"
                    | "P"
                    | "br"
                    | "nf"
                    | "fi"
            ) {
                continue;
            }

            let rest = if line.len() > 3 { line[3..].trim() } else { "" };
            let formatted = format_macro(macro_name, rest);
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
        // .so FILE - source (include another file)
        if line.starts_with(".so ") {
            let filename = line[4..].trim();

            // Always record the source file
            doc.source.push(filename.to_string());

            // If source_expand is enabled and we have a base path, try to expand
            if source_expand {
                // First, push the current section if it has content
                if have_section
                    || !current.title.is_empty()
                    || !current.text.is_empty()
                    || !current.items.is_empty()
                {
                    doc.sections.push(current);
                    current = Section::default();
                    have_section = true;
                }

                if let Some(base) = base_path {
                    // Try to resolve the included file relative to base_path
                    let base_dir = std::path::Path::new(base)
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| std::path::Path::new(".").to_path_buf());
                    let included_path = base_dir.join(filename);

                    if let Ok(included_content) = std::fs::read_to_string(&included_path) {
                        // Parse the included content
                        let included_json = parse_to_json_with_opts(
                            &included_content,
                            true,
                            included_path.to_str(),
                        );

                        // Merge sections from included file into current document
                        if let Some(included_sections) =
                            included_json.get("sections").and_then(|v| v.as_array())
                        {
                            for sec_val in included_sections {
                                if let Some(sec) = json_value_to_section(sec_val) {
                                    doc.sections.push(sec);
                                }
                            }
                        }
                    }
                }
            }
            continue;
        }
        if line.starts_with('.') && line.len() > 2 {
            let macro_part = &line[1..3];
            let rest = if line.len() > 3 { line[3..].trim() } else { "" };
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
                    let rest = if trimmed.len() > 3 { &trimmed[3..] } else { "" };
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
    if !doc.source.is_empty() {
        let arr = doc.source.into_iter().map(Value::String).collect();
        root.insert("source".to_string(), Value::Array(arr));
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

#[derive(Default)]
pub struct ViewOptions {
    pub description: bool,
    pub synopsis: bool,
    pub options: bool,
    pub environment: bool,
    pub files: bool,
    pub exit_status: bool,
    pub see_also: bool,
    pub examples: bool,
    pub author: bool,
    pub outline: bool,
    pub outline_head: Option<usize>,
    pub meta: bool,
    pub all: bool,
    pub source_expand: bool, // 展开 .so 包含的文件
}

impl ViewOptions {
    pub fn from_args(args: &[String]) -> Self {
        let mut opts = ViewOptions::default();

        for arg in args {
            match arg.as_str() {
                "--description" => opts.description = true,
                "--synopsis" => opts.synopsis = true,
                "--options" => opts.options = true,
                "--environment" => opts.environment = true,
                "--files" => opts.files = true,
                "--exit-status" => opts.exit_status = true,
                "--see-also" | "--seealso" => opts.see_also = true,
                "--examples" => opts.examples = true,
                "--author" => opts.author = true,
                "--outline" => opts.outline = true,
                "--outline-head" => {}
                _ if arg.starts_with("--outline-head=") => {
                    if let Ok(n) = arg.trim_start_matches("--outline-head=").parse() {
                        opts.outline_head = Some(n);
                    }
                }
                "--meta" => opts.meta = true,
                "--all" => opts.all = true,
                "--source-expand" => opts.source_expand = true,
                _ => {}
            }
        }

        if opts.meta {
            opts.description = true;
            opts.synopsis = true;
            opts.see_also = true;
            opts.outline = true;
        }

        if opts.all {
            opts.description = true;
            opts.synopsis = true;
            opts.options = true;
            opts.environment = true;
            opts.files = true;
            opts.exit_status = true;
            opts.see_also = true;
            opts.examples = true;
            opts.author = true;
        }

        opts
    }

    pub fn is_empty(&self) -> bool {
        !self.description
            && !self.synopsis
            && !self.options
            && !self.environment
            && !self.files
            && !self.exit_status
            && !self.see_also
            && !self.examples
            && !self.author
            && !self.outline
            && self.outline_head.is_none()
            && !self.meta
            && !self.all
    }
}

fn section_match(title: &str, opts: &ViewOptions) -> bool {
    let t = title.to_uppercase();
    match t.as_str() {
        "NAME" => opts.description,
        "SYNOPSIS" => opts.synopsis,
        "OPTIONS" => opts.options,
        "ENVIRONMENT" | "ENV" => opts.environment,
        "FILES" => opts.files,
        "EXIT STATUS" | "EXITSTATUS" => opts.exit_status,
        "SEE ALSO" | "SEEALSO" => opts.see_also,
        "EXAMPLES" => opts.examples,
        "AUTHOR" | "AUTHORS" => opts.author,
        _ => {
            // For other sections, show them if --all is set
            opts.all
        }
    }
}

pub fn view(json: &serde_json::Value, opts: &ViewOptions) -> String {
    let mut out = String::new();

    if let Some(t) = json.get("title").and_then(|v| v.as_str()) {
        out.push_str("# ");
        out.push_str(t);
        if let Some(s) = json.get("section").and_then(|v| v.as_str()) {
            out.push('(');
            out.push_str(s);
            out.push(')');
        }
        out.push_str("\n\n");
    }

    if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
        out.push_str("**");
        out.push_str(name);
        out.push_str("**");
        if let Some(desc) = json.get("description").and_then(|v| v.as_str()) {
            out.push_str(" - ");
            out.push_str(desc);
        }
        out.push_str("\n\n");
    }

    let mut shown_sections = std::collections::HashSet::new();

    if let Some(sections) = json.get("sections").and_then(|v| v.as_array()) {
        for sec in sections {
            if let Some(title) = sec.get("title").and_then(|v| v.as_str()) {
                if section_match(title, opts) {
                    shown_sections.insert(title.to_string());
                    out.push_str("## ");
                    out.push_str(title);
                    out.push_str("\n\n");

                    if let Some(text) = sec.get("text").and_then(|v| v.as_str()) {
                        for para in text.split('\n') {
                            let p = para.trim();
                            if !p.is_empty() {
                                out.push_str(p);
                                out.push_str("\n\n");
                            }
                        }
                    }

                    if let Some(items) = sec.get("items").and_then(|v| v.as_array()) {
                        for item in items {
                            if let Some(s) = item.as_str() {
                                let trimmed = s.trim();
                                if !trimmed.is_empty() {
                                    out.push_str("- ");
                                    out.push_str(trimmed);
                                    out.push('\n');
                                }
                            }
                        }
                        out.push('\n');
                    }
                }
            }
        }
    }

    if opts.outline || opts.outline_head.is_some() {
        if let Some(sections) = json.get("sections").and_then(|v| v.as_array()) {
            let head_lines = opts.outline_head.unwrap_or(0);
            let show_heads = head_lines > 0;

            if opts.outline && !show_heads {
                out.push_str("## Outline\n\n");
            }

            for sec in sections {
                if let Some(title) = sec.get("title").and_then(|v| v.as_str()) {
                    if !shown_sections.contains(title) {
                        out.push_str("### ");
                        out.push_str(title);
                        out.push_str("\n\n");

                        if show_heads {
                            if let Some(text) = sec.get("text").and_then(|v| v.as_str()) {
                                let lines: Vec<&str> = text.lines().take(head_lines).collect();
                                for line in lines {
                                    let l = line.trim();
                                    if !l.is_empty() {
                                        out.push_str(l);
                                        out.push_str("\n");
                                    }
                                }
                                out.push('\n');
                            }
                        }
                    }
                }
            }
        }
    }

    out.trim_end().to_string()
}
