# roff-cli

[![CII Best Practices](https://bestpractices.coreinfrastructure.org/projects/0/badge)](https://bestpractices.coreinfrastructure.org/projects/0)
[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/ljh-sh/roff/badge)](https://scorecard.dev/)
[![CI](https://github.com/ljh-sh/roff/actions/workflows/ci.yml/badge.svg)](https://github.com/ljh-sh/roff/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/roff-cli.svg)](https://crates.io/crates/roff-cli)
[![Docs](https://img.shields.io/badge/Docs-website-blue.svg)](https://roff.ljh.sh)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

> Skillful man page to JSON/Markdown converter — human readable, AI-friendly.

**roff-cli** turns Unix man pages (`roff`/`mdoc`) into structured JSON or readable Markdown. Use it for documentation pipelines, LLM context windows, or quick inspection of man pages without a pager.

Mirrors: [github.com/ljh-sh/roff](https://github.com/ljh-sh/roff) · [codeberg.org/ljh-sh/roff](https://codeberg.org/ljh-sh/roff)

Docs: [roff.ljh.sh](https://roff.ljh.sh)

## For AI agents

Minimal context with maximum flexibility — paste this one-line prompt into Claude Code, Cursor, or any agent's system prompt:

```md
Use `roff` to convert man pages to JSON/Markdown. Install if missing: `cargo install roff-cli`. Check `roff --help` for subcommands.
```

## Install

### Cargo (recommended)

```bash
cargo install roff-cli
```

### Direct binary

```bash
curl -L https://github.com/ljh-sh/roff/releases/latest/download/roff-x86_64-unknown-linux-musl.tar.xz | tar xJ -
sudo mv roff-x86_64-unknown-linux-musl/bin/roff /usr/local/bin/
```

See the [releases page](https://github.com/ljh-sh/roff/releases) for all targets (Linux musl/glibc, Windows, macOS).

### eget

Via [x-cmd eget](https://x-cmd.com/install/roff):

```bash
x eget ljh-sh/roff        # download and install
x eget use ljh-sh/roff    # install to ~/.local/bin
```

### Build from source

Requires Rust 1.74+.

```bash
git clone https://github.com/ljh-sh/roff
cd roff
cargo build --release
```

The binary will be at `target/release/roff`.

## At a glance

```bash
roff tojson file.1          # structured JSON
roff tomd file.1            # readable Markdown
roff view --meta ls         # progressive disclosure view
roff bench --count 100      # benchmark parser on man pages
```

---

## Usage

### Convert to JSON

```bash
roff tojson file.1
roff tojson --indent 2 file.1           # pretty-print with 2-space indent
roff tojson --indent 4 file.1           # pretty-print with 4-space indent
roff tojson --source-expand file.1      # expand .so includes
roff tojson -- < file.1                 # read from stdin
```

Example output:

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

### Convert to Markdown

```bash
roff tomd file.1
roff tomd --source-expand file.1
```

Output includes YAML front matter and clean Markdown sections.

### Progressive disclosure view

View man pages in parts — great for AI agents and quick inspection. Multiple options can be combined.

```bash
roff view --description file.1      # NAME + description
roff view --synopsis file.1         # SYNOPSIS section
roff view --options file.1          # OPTIONS section
roff view --see-also file.1         # SEE ALSO section
roff view --meta file.1             # description + synopsis + see-also + outline
roff view --outline file.1          # all section titles
roff view --outline-head 3 file.1   # titles + first 3 lines of each
```

| Option | Description |
|--------|-------------|
| `--description` | NAME + description |
| `--synopsis` | SYNOPSIS section |
| `--options` | OPTIONS section |
| `--environment` | ENVIRONMENT section |
| `--files` | FILES section |
| `--exit-status` | EXIT STATUS section |
| `--see-also` | SEE ALSO section |
| `--examples` | EXAMPLES section |
| `--author` | AUTHOR section |
| `--outline` | Show all section titles |
| `--outline-head N` | Show titles + first N lines |
| `--meta` | Shortcut: `--description --synopsis --see-also --outline` |
| `--all` | Show all sections |

### Benchmark

```bash
roff bench                # process first 10 files
roff bench --count 100    # process first 100 files
roff bench --all          # process all manpath files
```

## Library usage

```rust
use roff::{parse_to_json, parse_to_string, to_markdown};

let input = ".TH TEST 1\n.SH NAME\ntest \\- a test program";
let json = parse_to_json(input);
let md = to_markdown(&json);
let s = parse_to_string(input, true);  // pretty JSON (2-space indent)
```

## Design

roff-cli is intentionally **dumb** in the best way: it parses roff/mdoc macros and returns structured data. It does not try to render a terminal page, apply locale formatting, or synthesize summaries. Decisions belong to the caller:

```bash
roff tojson ls | jq '.sections[] | select(.title=="OPTIONS") | .items[]'
roff tomd git-push.1 | grep -A5 "## OPTIONS"
```

This keeps `roff --help` short and the tool cheap to use as an LLM tool.

## FAQ

See [docs/faq.md](docs/faq.md) or the [published FAQ](https://roff.ljh.sh/faq) for answers about supported formats, `.so` expansion, manpath search, performance, stdin handling, and more.

### What roff dialects are supported?

BSD `mdoc` and traditional `man` macros are the primary targets. The parser handles the most common macros used in system man pages (`.Sh`, `.Ss`, `.Nm`, `.Nd`, `.It`, `.Bl`, `.El`, `.Xr`, `.Ev`, font escapes, etc.).

### Does it handle `.so` includes?

Yes, with `--source-expand`. By default `.so` file names are recorded in the `source` array but not expanded.

### Is the output stable?

The JSON schema is stable for already-supported fields. New fields may be added as the parser learns more macros.

### Can I use it on non-Linux systems?

Yes. Prebuilt binaries are provided for Linux (musl/glibc), Windows, and macOS (Intel and Apple Silicon). It also builds anywhere Rust compiles.

### How big is the binary?

The stripped Linux x86_64 musl binary is roughly 1.5–2 MB (Rust + serde_json). No runtime dependencies.

### Why not just use `man --html` or `mandoc -Tmarkdown`?

Those tools are great for human reading. roff-cli targets structured data: JSON for scripts, Markdown for documentation pipelines, and progressive views for LLM context windows.

---

## Roadmap

See [ROADMAP.md](ROADMAP.md) for planned milestones.

## Changelog

See [`changelog/`](changelog/) for versioned release notes.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Issues and PRs are welcome.

## Security

See [SECURITY.md](SECURITY.md). For vulnerabilities, please email [lijunhao@x-cmd.com](mailto:lijunhao@x-cmd.com) instead of opening a public issue.

## Code of Conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## License

Apache 2.0 — see [LICENSE](LICENSE).
