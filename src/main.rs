use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process::{self, Command};
use std::time::Instant;

fn read_all_from_stdin() -> io::Result<String> {
    let mut buf = String::new();
    let mut stdin = io::stdin();
    stdin.read_to_string(&mut buf)?;
    Ok(buf)
}

fn usage() -> ! {
    eprintln!("roff - Skillful man page to JSON/Markdown converter");
    eprintln!("");
    eprintln!("Usage: roff <command> [options] [arguments]");
    eprintln!("");
    eprintln!("Commands:");
    eprintln!("  tojson          Convert man file(s) to JSON");
    eprintln!("  tomd            Convert man file(s) to Markdown");
    eprintln!("  view            Progressive disclosure view");
    eprintln!("  bench           Benchmark parser performance");
    eprintln!("");
    eprintln!("Examples:");
    eprintln!("  roff tojson file.1           # Convert to JSON");
    eprintln!("  roff tomd file.1             # Convert to Markdown");
    eprintln!("  roff view --synopsis file.1  # View synopsis only");
    eprintln!("  roff bench --all              # Benchmark all man files");
    eprintln!("  roff tojson -- < file.1      # Read from stdin");
    eprintln!("");
    eprintln!("Run 'roff <command> --help' for more details on a command.");
    eprintln!("");
    eprintln!("For full documentation, see: https://github.com/ljh-sh/roff");
    process::exit(1);
}

fn cmd_tojson_help() -> ! {
    eprintln!("roff-tojson - Convert man pages to JSON");
    eprintln!("");
    eprintln!("Usage: roff tojson [options] <file>...");
    eprintln!("       roff tojson -- < file.1");
    eprintln!("");
    eprintln!("Options:");
    eprintln!("  --pretty    Pretty-print JSON output");
    eprintln!("  -h, --help  Show this help message");
    eprintln!("");
    eprintln!("Examples:");
    eprintln!("  roff tojson file.1");
    eprintln!("  roff tojson --pretty file.1");
    eprintln!("  roff tojson --pretty file.1 file.2");
    eprintln!("  cat file.1 | roff tojson --");
    process::exit(0);
}

fn cmd_tomd_help() -> ! {
    eprintln!("roff-tomd - Convert man pages to Markdown");
    eprintln!("");
    eprintln!("Usage: roff tomd <file>...");
    eprintln!("       roff tomd -- < file.1");
    eprintln!("");
    eprintln!("Options:");
    eprintln!("  -h, --help  Show this help message");
    eprintln!("");
    eprintln!("Examples:");
    eprintln!("  roff tomd file.1");
    eprintln!("  roff tomd file.1 file.2");
    eprintln!("  cat file.1 | roff tomd --");
    process::exit(0);
}

fn cmd_view_help() -> ! {
    eprintln!("roff-view - Progressive disclosure view for man pages");
    eprintln!("");
    eprintln!("Usage: roff view [options] <query>...");
    eprintln!("       roff view [options] <file>...");
    eprintln!("       roff view [options] -            # read from stdin");
    eprintln!("");
    eprintln!("Query formats:");
    eprintln!("  roff view ls                   # Search 'ls' in manpath");
    eprintln!("  roff view ls 1                 # Search 'ls' in section 1");
    eprintln!("  roff view git ls                # Search multiple in manpath");
    eprintln!("  roff view /path/to/file.1      # Direct file path");
    eprintln!("  cat file.1 | roff view -        # Read from stdin");
    eprintln!("");
    eprintln!("Options:");
    eprintln!("  --description      Show NAME + description");
    eprintln!("  --synopsis         Show SYNOPSIS section");
    eprintln!("  --options          Show OPTIONS section");
    eprintln!("  --environment      Show ENVIRONMENT section");
    eprintln!("  --files            Show FILES section");
    eprintln!("  --exit-status      Show EXIT STATUS section");
    eprintln!("  --see-also         Show SEE ALSO section");
    eprintln!("  --examples         Show EXAMPLES section");
    eprintln!("  --author           Show AUTHOR section");
    eprintln!("  --outline          Show section titles not displayed");
    eprintln!("  --outline-head N   Show outline + first N lines of each section");
    eprintln!("  --meta             Shortcut: --description --synopsis --see-also --outline");
    eprintln!("  --all              Show all sections");
    eprintln!("  -h, --help         Show this help message");
    eprintln!("");
    eprintln!("Examples:");
    eprintln!("  roff view ls                       # Search 'ls' in manpath");
    eprintln!("  roff view ls 1                     # Search 'ls' in section 1");
    eprintln!("  roff view git ls                   # Search multiple: 'git' and 'ls'");
    eprintln!("  roff view --synopsis ls            # Show synopsis of 'ls'");
    eprintln!("  roff view --meta ls                # Show meta of 'ls'");
    eprintln!("  roff view --outline ls             # Show outline of 'ls'");
    eprintln!("  roff view --outline-head 3 ls      # Show outline + 3 lines");
    eprintln!("  roff view /path/to/file.1         # View specific file");
    eprintln!("  cat file.1 | roff view -           # Read from stdin");
    process::exit(0);
}

fn cmd_bench_help() -> ! {
    eprintln!("roff-bench - Benchmark parser performance on man files");
    eprintln!("");
    eprintln!("Usage: roff bench [options]");
    eprintln!("");
    eprintln!("Options:");
    eprintln!("  --count N    Process first N files (default: 10)");
    eprintln!("  --all       Process all files in manpath");
    eprintln!("  -h, --help  Show this help message");
    eprintln!("");
    eprintln!("Examples:");
    eprintln!("  roff bench                 # Benchmark first 10 files");
    eprintln!("  roff bench --count 100     # Benchmark first 100 files");
    eprintln!("  roff bench --all           # Benchmark all man files");
    process::exit(0);
}

fn get_manpath() -> Vec<String> {
    let output = Command::new("manpath")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok());

    match output {
        Some(s) => s.trim().split(':').map(|s| s.to_string()).collect(),
        None => vec![
            "/usr/share/man".to_string(),
            "/usr/local/share/man".to_string(),
        ],
    }
}

fn collect_man_files(manpaths: &[String]) -> Vec<String> {
    let mut files = Vec::new();

    for base in manpaths {
        let base_path = Path::new(base);
        if !base_path.exists() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        for sub_entry in sub_entries.flatten() {
                            let sub_path = sub_entry.path();
                            if let Some(ext) = sub_path.extension() {
                                let ext_str = ext.to_string_lossy();
                                if let Some(c) = ext_str.chars().next() {
                                    if c.is_ascii_digit() {
                                        files.push(sub_path.to_string_lossy().to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    files.sort();
    files
}

fn cmd_bench(all: bool, count: usize) {
    let manpaths = get_manpath();
    eprintln!("Manpath: {}", manpaths.join(":"));

    let files = collect_man_files(&manpaths);
    let total = files.len();
    let limit = if all { total } else { count.min(total) };

    eprintln!("Found {} man files, processing {}...", total, limit);

    let mut success = 0usize;
    let mut failed = 0usize;
    let mut errors = Vec::new();

    let start = Instant::now();

    for (i, file) in files.iter().take(limit).enumerate() {
        match roff::read_to_string_lossy(file) {
            Ok(content) => {
                let _ = roff::parse_to_json(&content);
                let _ = roff::to_markdown(&roff::parse_to_json(&content));
                success += 1;
            }
            Err(e) => {
                failed += 1;
                errors.push((file.clone(), e.to_string()));
            }
        }
        if (i + 1) % 100 == 0 {
            eprintln!("  Progress: {}/{}", i + 1, limit);
        }
    }

    let elapsed = start.elapsed();
    let ms = elapsed.as_millis();
    let secs = elapsed.as_secs_f64();

    eprintln!("");
    eprintln!("=== Benchmark Results ===");
    eprintln!("Files processed: {}", success);
    eprintln!("Files failed:    {}", failed);
    eprintln!("Total time:      {} ms ({:.2} s)", ms, secs);
    if success > 0 && secs > 0.0 {
        eprintln!("Avg time/file:   {:.2} ms", ms as f64 / success as f64);
        eprintln!("Files/second:    {:.2}", success as f64 / secs);
    }

    if !errors.is_empty() {
        eprintln!("");
        eprintln!("Errors (first 5):");
        for (file, err) in errors.iter().take(5) {
            eprintln!("  {}: {}", file, err);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        usage();
    }

    let cmd = &args[0];

    if cmd == "-h" || cmd == "--help" {
        usage();
    }

    if cmd == "help" {
        if args.len() > 1 {
            match args[1].as_str() {
                "tojson" => cmd_tojson_help(),
                "tomd" => cmd_tomd_help(),
                "view" => cmd_view_help(),
                "bench" => cmd_bench_help(),
                _ => {
                    eprintln!("Unknown command: {}", args[1]);
                    usage();
                }
            }
        } else {
            usage();
        }
    }

    if cmd == "bench" {
        let mut all = false;
        let mut count = 10usize;
        let mut i = 1;
        while i < args.len() {
            if args[i] == "-h" || args[i] == "--help" {
                cmd_bench_help();
            } else if args[i] == "--all" {
                all = true;
            } else if args[i] == "--count" {
                i += 1;
                if i < args.len() {
                    count = args[i].parse().unwrap_or(10);
                }
            } else if args[i].starts_with('-') {
                eprintln!("Unknown option: {}", args[i]);
                usage();
            }
            i += 1;
        }
        cmd_bench(all, count);
        return;
    }

    if cmd == "view" {
        let mut view_args = Vec::new();
        let mut queries = Vec::new();
        let mut section: Option<String> = None;
        let mut use_stdin = false;

        let mut i = 1;
        while i < args.len() {
            if args[i] == "-h" || args[i] == "--help" {
                cmd_view_help();
            } else if args[i] == "--" {
                use_stdin = true;
            } else if args[i].starts_with("--outline-head=") {
                view_args.push(args[i].clone());
            } else if args[i] == "--outline-head" {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    view_args.push(format!("--outline-head={}", args[i + 1]));
                    i += 1;
                } else {
                    view_args.push(args[i].clone());
                }
            } else if args[i].starts_with('-') {
                view_args.push(args[i].clone());
            } else if args[i].chars().all(|c| c.is_ascii_digit()) {
                section = Some(args[i].clone());
            } else {
                queries.push(args[i].clone());
            }
            i += 1;
        }

        let opts = roff::ViewOptions::from_args(&view_args);

        if use_stdin || queries.is_empty() {
            let content = read_all_from_stdin().expect("failed to read stdin");
            let json = roff::parse_to_json_with_opts(&content, opts.source_expand, None);
            println!("{}", roff::view(&json, &opts));
            return;
        }

        for query in queries {
            if query.contains('/') || query.contains('.') {
                let content = roff::read_to_string_lossy(&query).expect("failed to read file");
                let json =
                    roff::parse_to_json_with_opts(&content, opts.source_expand, Some(&query));
                println!("{}", roff::view(&json, &opts));
                continue;
            }

            let manpaths = get_manpath();
            let mut found_file = String::new();

            if let Some(sec) = &section {
                for manpath in &manpaths {
                    let path = format!("{}/man{}/{}.{}", manpath, sec, query, sec);
                    if std::path::Path::new(&path).exists() {
                        found_file = path;
                        break;
                    }
                }
            } else {
                for sec in 1..=9 {
                    for manpath in &manpaths {
                        let path = format!("{}/man{}/{}.{}", manpath, sec, query, sec);
                        if std::path::Path::new(&path).exists() {
                            found_file = path;
                            break;
                        }
                    }
                    if !found_file.is_empty() {
                        break;
                    }
                }
            }

            if found_file.is_empty() {
                eprintln!("Error: Could not find man page for '{}' in manpath", query);
                continue;
            }

            let content = roff::read_to_string_lossy(&found_file).expect("failed to read file");
            let json =
                roff::parse_to_json_with_opts(&content, opts.source_expand, Some(&found_file));
            println!("{}", roff::view(&json, &opts));
        }
        return;
    }

    let mut pretty = false;
    let mut files = Vec::new();
    let mut use_stdin = false;
    let mut source_expand = false;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "-h" || args[i] == "--help" {
            match cmd.as_str() {
                "tojson" => cmd_tojson_help(),
                "tomd" => cmd_tomd_help(),
                _ => usage(),
            }
        } else if args[i] == "--pretty" {
            pretty = true;
        } else if args[i] == "--source-expand" {
            source_expand = true;
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
                let content = roff::read_to_string_lossy(f).expect("failed to read file");
                (f.clone(), content)
            })
            .collect()
    };

    let num_inputs = inputs.len();
    let mut outputs = Vec::new();
    for (name, content) in inputs {
        match cmd.as_str() {
            "tojson" => {
                let base_path = if name != "stdin" {
                    Some(name.as_str())
                } else {
                    None
                };
                let json = roff::parse_to_json_with_opts(&content, source_expand, base_path);
                let out = if pretty {
                    serde_json::to_string_pretty(&json).unwrap()
                } else {
                    serde_json::to_string(&json).unwrap()
                };
                if num_inputs > 1 {
                    outputs.push(format!("# File: {}\n{}", name, out));
                } else {
                    outputs.push(out);
                }
            }
            "tomd" => {
                let base_path = if name != "stdin" {
                    Some(name.as_str())
                } else {
                    None
                };
                let json = roff::parse_to_json_with_opts(&content, source_expand, base_path);
                let out = roff::to_markdown(&json);
                if num_inputs > 1 {
                    outputs.push(format!("# File: {}\n{}", name, out));
                } else {
                    outputs.push(out);
                }
            }
            _ => usage(),
        }
    }

    println!("{}", outputs.join("\n\n"));
}
