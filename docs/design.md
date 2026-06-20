---
layout: default
title: Design
---

# Design & principles

roff-cli is intentionally **dumb** in the best way: it parses `roff`/`mdoc` macros and returns structured data. It does not try to render a terminal page, apply locale formatting, or synthesize summaries.

## Native roff/mdoc parsing

The parser understands the macros you actually find in system man pages: `.TH`, `.SH`, `.SS`, `.Nm`, `.Nd`, `.It`, `.Bl`, `.El`, `.Xr`, `.Ev`, font escapes, and more. It does not shell out to `groff`, `mandoc`, or any other tool.

## No external dependencies

The release binary is a single stripped executable (roughly 1.5–2 MB for Linux x86_64 musl). It is built with Rust and `serde_json`; there are no runtime libraries to install.

## JSON-first

Every conversion produces JSON with a stable schema. The caller decides what to do next:

```bash
roff tojson ls | jq '.sections[] | select(.title=="OPTIONS") | .items[]'
roff tojson git-push.1 | jq '.description'
```

## Pipe-friendly

Read from files, from stdin with `--`, or use `view` to pull a specific section. Output goes to stdout so it composes with `jq`, `grep`, `bat`, or any other CLI tool.

## Progressive disclosure

`view` lets you request only the slices of a man page that matter for the task at hand. This is especially useful for AI agents working with limited context windows:

```bash
roff view --meta ls
```

## Decisions belong to the caller

roff-cli does not guess formatting or add synthetic summaries. What you see in the man page is what you get — structured and ready to pipe.
