---
layout: default
title: Install
---

# Install roff

## Cargo (recommended)

```bash
cargo install roff-cli
```

## Direct binary

```bash
curl -L https://github.com/ljh-sh/roff/releases/latest/download/roff-x86_64-unknown-linux-musl.tar.xz | tar xJ -
sudo mv roff-x86_64-unknown-linux-musl/bin/roff /usr/local/bin/
```

See the [releases page](https://github.com/ljh-sh/roff/releases) for all targets (Linux musl/glibc, Windows, macOS).

## eget

Via [x-cmd eget](https://x-cmd.com/install/roff):

```bash
x eget ljh-sh/roff        # download and install
x eget use ljh-sh/roff    # install to ~/.local/bin
```

## Build from source

Requires Rust 1.74+.

```bash
git clone https://github.com/ljh-sh/roff
cd roff
cargo build --release
```

The binary will be at `target/release/roff`.
