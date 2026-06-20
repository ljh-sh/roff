# Roadmap

This is the high-level plan for `roff-cli`. For already-released changes see [`changelog/`](changelog/).

Guiding principle: **roff-cli only parses and converts man pages — it does not render or synthesize content.**

---

## Shipped

- [x] Core roff/mdoc parser with JSON output
- [x] Markdown output with YAML front matter
- [x] Progressive disclosure `view` command
- [x] Local manpath search and benchmark command
- [x] Multi-target release pipeline with signed artifacts
- [x] OpenSSF Scorecard tracking

---

## v0.2.0 — Parser hardening

Goal: handle more real-world man pages without losing structure.

- [ ] Support nested `.Bl`/`.El` lists
- [ ] Better `.TP`/`.IP` tagged paragraph handling
- [ ] Preserve indentation in code/preformatted blocks
- [ ] Fuzz / property tests for parser robustness
- [ ] CI smoke test against a corpus of public man pages

Success criteria:

- `roff bench --all` succeeds on the reference Linux/BSD man page corpus.
- No panic on malformed input.

---

## v0.3.0 — Output formats

Goal: make the parsed structure useful in more pipelines.

- [ ] `roff tohtml` — minimal HTML output
- [ ] `roff toman` — round-trip sanity check (best-effort)
- [ ] `--format` flag on `tojson` for compact vs. pretty vs. ndjson
- [ ] Configurable Markdown YAML front matter fields

Success criteria:

- New formats are stable and documented in `--help`.

---

## Infrastructure & hardening

These run in parallel with feature milestones.

- [ ] OpenSSF Scorecard >= 8.5
  - keep `Signed-Releases`, `CI-Tests`, `Code-Review`, and dependency-update checks green
- [ ] Reproducible build verification in CI for every release
- [ ] Dependabot keeps GitHub Actions and Cargo dependencies up to date
- [ ] Publish to crates.io automatically on every tagged release

---

## Not planned

These are intentionally out of scope for `roff-cli`:

- **Full roff typesetting** — we only extract structure, not layout.
- **Terminal rendering** — use `man` or `mandoc` for that.
- **Built-in summarization** — leave that to `jq` / `awk` / LLMs downstream.
- **Editing man pages** — this is a read-only converter.

---

## How decisions are made

New candidates are evaluated with two questions:

1. Does it make man pages more useful as structured data?
2. Does it stay within the parser/converter scope?

If both are yes, it belongs in a future milestone.
