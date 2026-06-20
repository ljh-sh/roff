---
layout: default
title: FAQ
---

# FAQ

## Supported formats

### What roff dialects are supported?

BSD `mdoc` and traditional `man` macros are the primary targets. The parser handles the most common macros used in system man pages (`.Sh`, `.Ss`, `.Nm`, `.Nd`, `.It`, `.Bl`, `.El`, `.Xr`, `.Ev`, font escapes, etc.).

## `.so` expansion

### Does it handle `.so` includes?

Yes, with `--source-expand`. By default `.so` file names are recorded in the `source` array but not expanded.

## Finding man pages

### How does `view` search the manpath?

`view` calls `manpath` to discover directories, then falls back to `/usr/share/man` and `/usr/local/share/man`. It walks `man1` through `man9` looking for `<name>.<section>`. If you give a section number (e.g., `roff view ls 1`), it only searches that section. Queries containing `/` or `.` are treated as direct file paths.

## Performance

### How fast is the parser?

Fast enough to process thousands of man pages in seconds. Run `roff bench --all` on your system to see exact throughput.

### How big is the binary?

The stripped Linux x86_64 musl binary is roughly 1.5–2 MB (Rust + serde_json). No runtime dependencies.

## Stdin and piping

### Can I pipe a man page into roff?

Yes. Use `--` to tell a command to read from stdin:

```bash
roff tojson -- < file.1
roff tomd -- < file.1
cat file.1 | roff view -
```

## Stability

### Is the output stable?

The JSON schema is stable for already-supported fields. New fields may be added as the parser learns more macros.

## Platforms

### Can I use it on non-Linux systems?

Yes. Prebuilt binaries are provided for Linux (musl/glibc), Windows, and macOS (Intel and Apple Silicon). It also builds anywhere Rust compiles.

## Reporting issues

### Where do I report bugs or request features?

Open an issue on [GitHub](https://github.com/ljh-sh/roff/issues). For security issues, please see [SECURITY.md](https://github.com/ljh-sh/roff/blob/main/SECURITY.md).
