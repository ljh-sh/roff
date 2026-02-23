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
    eprintln!("Usage: roff tojson [--pretty] <file>...");
    eprintln!("       roff tomd <file>...");
    eprintln!("       roff view [options] <file>");
    eprintln!("       roff bench [--count N] [--all]");
    eprintln!("       roff tojson --        # read from stdin");
    eprintln!("       roff tomd --          # read from stdin");
    eprintln!("");
    eprintln!("Commands:");
    eprintln!("  tojson  Convert man file(s) to JSON");
    eprintln!("  tomd    Convert man file(s) to Markdown");
    eprintln!("  view    Progressive disclosure view");
    eprintln!("  bench   Benchmark roff on manpath files");
    eprintln!("");
    eprintln!("View options:");
    eprintln!("  --description     Show NAME + description");
    eprintln!("  --synopsis       Show SYNOPSIS");
    eprintln!("  --options        Show OPTIONS");
    eprintln!("  --environment    Show ENVIRONMENT");
    eprintln!("  --files         Show FILES");
    eprintln!("  --exit-status    Show EXIT STATUS");
    eprintln!("  --see-also      Show SEE ALSO");
    eprintln!("  --examples       Show EXAMPLES");
    eprintln!("  --author        Show AUTHOR");
    eprintln!("  --outline        Show section titles not displayed");
    eprintln!("  --outline-head N Show outline + first N lines");
    eprintln!("  --meta           Shortcut: --description --synopsis --see-also --outline");
    eprintln!("  --all            Show all sections");
    eprintln!("");
    eprintln!("Options:");
    eprintln!("  --pretty  Pretty-print JSON output (tojson only)");
    eprintln!("  --count N Process first N files (bench only, default 10)");
    eprintln!("  --all     Process all files in manpath (bench only)");
    process::exit(1);
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

    if cmd == "bench" {
        let mut all = false;
        let mut count = 10usize;
        let mut i = 1;
        while i < args.len() {
            if args[i] == "--all" {
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
        let mut files = Vec::new();

        let mut i = 1;
        while i < args.len() {
            if args[i].starts_with("--outline-head=") {
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
            } else {
                files.push(args[i].clone());
            }
            i += 1;
        }

        let opts = roff::ViewOptions::from_args(&view_args);

        if files.is_empty() {
            let content = read_all_from_stdin().expect("failed to read stdin");
            let json = roff::parse_to_json(&content);
            println!("{}", roff::view(&json, &opts));
        } else {
            for f in files {
                let content = roff::read_to_string_lossy(&f).expect("failed to read file");
                let json = roff::parse_to_json(&content);
                println!("{}", roff::view(&json, &opts));
            }
        }
        return;
    }

    let mut pretty = false;
    let mut files = Vec::new();
    let mut use_stdin = false;
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
                let out = roff::parse_to_string(&content, pretty);
                if num_inputs > 1 {
                    outputs.push(format!("# File: {}\n{}", name, out));
                } else {
                    outputs.push(out);
                }
            }
            "tomd" => {
                let json = roff::parse_to_json(&content);
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
