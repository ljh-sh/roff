---
layout: default
title: Command reference
---

# Command reference

roff has four commands: `tojson`, `tomd`, `view`, and `bench`. All read man pages in `roff`/`mdoc` format and write to stdout.

## `roff tojson`

Convert man page(s) to JSON.

```bash
roff tojson file.1
roff tojson --pretty file.1
roff tojson --source-expand file.1
roff tojson -- < file.1
```

### Options

| Option | Description |
|--------|-------------|
| `--pretty` | Pretty-print JSON output |
| `--source-expand` | Expand `.so` (source) includes |
| `-h, --help` | Show help message |

### Example output

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
  ],
  "source": []
}
```

When multiple files are given, each output is prefixed with `# File: <path>`.

---

## `roff tomd`

Convert man page(s) to Markdown with YAML front matter.

```bash
roff tomd file.1
roff tomd --source-expand file.1
roff tomd -- < file.1
```

### Options

| Option | Description |
|--------|-------------|
| `--source-expand` | Expand `.so` (source) includes |
| `-h, --help` | Show help message |

### Example output

```markdown
---
title: LS
section: "1"
name: ls
description: list directory contents
---

# NAME

ls — list directory contents

# SYNOPSIS

ls [options] [file ...]

# OPTIONS

- `-a`: do not ignore entries starting with .
- `-l`: use a long listing format
```

---

## `roff view`

Progressively disclose parts of a man page. Use it to grab just the synopsis, options, or outline for an LLM prompt.

```bash
roff view ls
roff view ls 1
roff view --synopsis ls
roff view --meta ls
roff view --outline-head 3 ls
roff view /path/to/file.1
cat file.1 | roff view -
```

### Query formats

| Query | Meaning |
|-------|---------|
| `ls` | Search `ls` in the manpath |
| `ls 1` | Search `ls` in section 1 |
| `git ls` | Search multiple names in the manpath |
| `/path/to/file.1` | Use a direct file path |
| `-` | Read from stdin |

### Options

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
| `--outline-head N` | Show titles + first N lines of each section |
| `--meta` | Shortcut for `--description --synopsis --see-also --outline` |
| `--all` | Show all sections |
| `-h, --help` | Show help message |

### How `view` finds man pages

`view` first checks if a query contains `/` or `.` and treats it as a file path. Otherwise it walks `manpath` output, falling back to `/usr/share/man` and `/usr/local/share/man`. It searches sections `1` through `9` unless a numeric section argument is given.

---

## `roff bench`

Benchmark parser performance over files in the manpath.

```bash
roff bench
roff bench --count 100
roff bench --all
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--count N` | `10` | Process first N files |
| `--all` | — | Process all files in the manpath |
| `-h, --help` | — | Show help message |

### Example output

```
Manpath: /usr/share/man:/usr/local/share/man
Found 2847 man files, processing 100...
  Progress: 100/100

=== Benchmark Results ===
Files processed: 100
Files failed: 0
Total time: 1234 ms (1.23 s)
Avg time/file: 12.34 ms
Files/second: 81.04
```

`bench` parses each file to JSON and then to Markdown, reporting throughput and the first few errors.
