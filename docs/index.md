---
layout: default
title: Home
---

<div class="hero">
  <h1>roff</h1>
  <p>Skillful man page to JSON/Markdown converter — human readable, AI-friendly.</p>
  <div class="cta">
    <a class="btn primary" href="{{ '/install' | relative_url }}">Install</a>
    <a class="btn secondary" href="{{ '/subcommands' | relative_url }}">Command reference</a>
    <a class="btn secondary" href="https://github.com/ljh-sh/roff" target="_blank" rel="noopener">GitHub</a>
  </div>
</div>

## What is roff?

**roff-cli** turns Unix man pages (`roff`/`mdoc`) into structured JSON or readable Markdown. Use it for documentation pipelines, LLM context windows, or quick inspection of man pages without a pager.

- *Convert a man page to JSON and query it with `jq`*
- *Render a clean Markdown copy for a docs site*
- *Progressively inspect just the synopsis, options, or outline of any page*

## At a glance

```bash
roff tojson file.1          # structured JSON
roff tomd file.1            # readable Markdown
roff view --meta ls         # progressive disclosure view
roff bench --count 100      # benchmark parser on man pages
```

## Output schema

`roff tojson` produces a stable JSON object:

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

## For AI agents

Paste this one-line prompt into Claude Code, Cursor, or any agent's system prompt:

```md
Use `roff` to convert man pages to JSON/Markdown. Install if missing: `cargo install roff-cli`. Check `roff --help` for subcommands.
```

## Where to go next

- [Install roff]({{ '/install' | relative_url }}) — Cargo, direct binary, eget, or build from source
- [Command reference]({{ '/subcommands' | relative_url }}) — every subcommand, option, and output example
- [Design & principles]({{ '/design' | relative_url }}) — why roff is shaped the way it is
- [Why roff?]({{ '/why' | relative_url }}) — why convert man pages to JSON/Markdown
- [FAQ]({{ '/faq' | relative_url }}) — supported formats, `.so` expansion, manpath search, and more
- [Alternatives]({{ '/alternatives' | relative_url }}) — how roff compares to `man`, `groff`, Pandoc, and web converters
