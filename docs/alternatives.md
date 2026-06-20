---
layout: default
title: Alternatives
---

# Alternatives

## `man`

- **Pros**: installed everywhere, authoritative rendering, great for interactive reading.
- **Cons**: terminal-only, no structured output, hard to automate.
- **Use when**: you are a human reading a man page at a terminal.

## `groff -Thtml`

- **Pros**: produces HTML from roff source, no extra tools.
- **Cons**: output is presentation-oriented, not structured; hard to query programmatically.
- **Use when**: you need a web-rendered copy of a man page.

## `mandoc -Tmarkdown`

- **Pros**: high-quality Markdown output from `mdoc` pages.
- **Cons**: focused on human-readable Markdown, not a structured intermediate; no JSON.
- **Use when**: you want a Markdown rendering and can install `mandoc`.

## Pandoc

- **Pros**: can convert many formats, including roff.
- **Cons**: heavy dependency, not purpose-built for man pages, may not preserve roff/mdoc semantics.
- **Use when**: you need a general-purpose document converter.

## Web APIs

- **Pros**: can summarize or reformat pages.
- **Cons**: requires network, API keys, uploading documentation, latency, and cost.
- **Use when**: you need language translation or summarization beyond conversion.

## When to choose roff

Choose roff when you want:

- A small, self-contained binary with no runtime dependencies.
- Structured JSON output for scripts and agents.
- Clean Markdown for documentation pipelines.
- Progressive disclosure for LLM context windows.
- Local, offline conversion without API keys.
