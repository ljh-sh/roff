---
layout: default
title: Why roff?
---

# Why roff?

## Why convert man pages?

Man pages are the authoritative documentation for Unix tools, but they are designed for terminal rendering. Converting them to JSON or Markdown makes them searchable, queryable, and composable.

## Why JSON?

JSON is the universal input format for scripts, dashboards, and LLM tool calls. With roff's output you can:

- Build a documentation search index
- Feed precise man-page context into an LLM
- Diff versions of a page programmatically
- Extract just the OPTIONS or SEE ALSO sections

## Why Markdown?

Markdown is the native format for READMEs, wikis, static sites, and most documentation pipelines. `roff tomd` gives you a clean Markdown file with YAML front matter, ready to drop into a docs repo.

## Why not just `man -P`?

`man -P` runs a pager. It is great for humans reading one page at a time, but it does not give you structured data. roff is for automation, not interactive reading.

## Why not `groff -Thtml`?

`groff -Thtml` renders a man page as HTML. The output is meant for browsers, not for scripts or LLM context windows. roff returns structured data that you can transform further.

## Why not a web converter?

Web converters require a network round-trip, may have usage limits, and often produce presentation-oriented output. roff runs locally, offline, with no API keys.

## AI context windows

LLMs work best with focused, structured context. `roff view --meta ls` gives an agent exactly the name, synopsis, see-also, and outline — no noise, no terminal markup, no extra prose.
