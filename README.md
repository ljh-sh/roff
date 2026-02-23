# roff-cli

Skillful man page to JSON/Markdown converter - human readable, AI-friendly

## Features

- **tojson**: Convert roff/man files to JSON
- **tomd**: Convert roff/man files to Markdown  
- **view**: Progressive disclosure - view man pages in parts
- **bench**: Benchmark parser performance on manpath files

## Installation

```bash
cargo install roff-cli
# or
cargo install --path .
```

## Usage

### Convert to JSON/Markdown

```bash
# Convert to JSON
roff tojson file.1

# Pretty-print JSON
roff tojson --pretty file.1

# Convert to Markdown
roff tomd file.1

# Read from stdin
roff tojson -- < file.1
roff tomd -- < file.1
```

### Progressive Disclosure (view)

View man pages in parts - great for AI agents and quick inspection. Multiple options can be combined.

```bash
# View specific sections (can combine multiple)
roff view --description file.1      # NAME + description
roff view --synopsis file.1          # SYNOPSIS section
roff view --options file.1           # OPTIONS section
roff view --see-also file.1         # SEE ALSO section

# Combine multiple options
roff view --description --synopsis file.1   # Show description + synopsis
roff view --options --examples file.1        # Show options + examples
roff view --description --synopsis --see-also --environment file.1  # Show multiple

# Combined views (most useful)
roff view --meta file.1              # description + synopsis + see-also + outline
roff view file.1                     # same as --meta

# Outline mode - show section titles NOT displayed
roff view --outline file.1           # show all section names

# Outline with preview - show section titles + first N lines
roff view --outline-head 3 file.1   # show titles + first 3 lines of each
```

#### View Options

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
| `--outline` | Show all section titles (not displayed sections) |
| `--outline-head N` | Show titles + first N lines of each section |
| `--meta` | Shortcut: --description --synopsis --see-also --outline |
| `--all` | Show all sections |

### Benchmark

```bash
roff bench                # process first 10 files
roff bench --count 100    # process first 100 files
roff bench --all          # process all manpath files
```

## Library Usage

```rust
use roff::{parse_roff, to_json, to_markdown};

let input = ".TH TEST 1\n.SH NAME\ntest \\- a test program";
let roff = parse_roff(input);

let json = to_json(&roff);
let md = to_markdown(&roff);
```

## License

MIT
