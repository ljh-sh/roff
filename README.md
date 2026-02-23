# man_parser

A Rust parser for roff (man page) files, supporting conversion to JSON and Markdown.

## Features

- **tojson**: Convert roff/man files to JSON
- **tomd**: Convert roff/man files to Markdown
- **bench**: Benchmark parser performance on manpath files

## Installation

```bash
cargo install --path .
```

## Usage

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

# Benchmark
roff bench                # process first 10 files
roff bench --count 100    # process first 100 files
roff bench --all          # process all manpath files
```

## Library Usage

```rust
use man_parser::{parse_roff, to_json, to_markdown};

let input = ".TH TEST 1\n.SH NAME\ntest \\- a test program";
let roff = parse_roff(input);

let json = to_json(&roff);
let md = to_markdown(&roff);
```

## License

Apache-2.0
