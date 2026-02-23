// man-parser CLI 入口
// 支持将 man 文件转换为 JSON 或 Markdown
use std::env;
use std::io::{self, Read};
use std::process;

/// 从 stdin 读取所有内容
fn read_all_from_stdin() -> io::Result<String> {
    let mut buf = String::new();
    let mut stdin = io::stdin();
    stdin.read_to_string(&mut buf)?;
    Ok(buf)
}

/// 打印使用说明并退出
fn usage() -> ! {
    eprintln!("Usage: roff tojson [--pretty] <file>...");
    eprintln!("       roff tomd <file>...");
    eprintln!("       roff tojson --        # read from stdin");
    eprintln!("       roff tomd --          # read from stdin");
    eprintln!("");
    eprintln!("Commands:");
    eprintln!("  tojson  Convert man file(s) to JSON");
    eprintln!("  tomd    Convert man file(s) to Markdown");
    eprintln!("");
    eprintln!("Options:");
    eprintln!("  --pretty  Pretty-print JSON output (tojson only)");
    process::exit(1);
}

fn main() {
    // 解析命令行参数
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        usage();
    }

    let cmd = &args[0];  // tojson 或 tomd
    let mut pretty = false;  // 是否格式化 JSON
    let mut files = Vec::new();  // 要处理的文件列表
    let mut use_stdin = false;  // 是否从 stdin 读取
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--pretty" {
            pretty = true;
        } else if args[i] == "--" {
            use_stdin = true;
        } else if args[i].starts_with('-') {
            eprintln!("Unknown option: {}", args[i]);
            usage();
        } else {
            files.push(args[i].clone());
        }
        i += 1;
    }

    // 读取输入（文件或 stdin）
    let inputs: Vec<(String, String)> = if use_stdin {
        let content = read_all_from_stdin().expect("failed to read stdin");
        vec![("stdin".to_string(), content)]
    } else if files.is_empty() {
        let content = read_all_from_stdin().expect("failed to read stdin");
        vec![("stdin".to_string(), content)]
    } else {
        files
            .iter()
            .map(|f| {
                let content = man_parser::read_to_string_lossy(f).expect("failed to read file");
                (f.clone(), content)
            })
            .collect()
    };

    // 处理每个输入文件
    let num_inputs = inputs.len();
    let mut outputs = Vec::new();
    for (name, content) in inputs {
        match cmd.as_str() {
            "tojson" => {
                // 转换为 JSON
                let out = man_parser::parse_to_string(&content, pretty);
                if num_inputs > 1 {
                    outputs.push(format!("# File: {}\n{}", name, out));
                } else {
                    outputs.push(out);
                }
            }
            "tomd" => {
                // 转换为 Markdown
                let json = man_parser::parse_to_json(&content);
                let out = man_parser::to_markdown(&json);
                if num_inputs > 1 {
                    outputs.push(format!("# File: {}\n{}", name, out));
                } else {
                    outputs.push(out);
                }
            }
            _ => usage(),
        }
    }

    // 输出结果
    println!("{}", outputs.join("\n\n"));
}
