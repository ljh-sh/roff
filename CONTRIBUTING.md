# Contributing to roff-cli

Thanks for your interest! roff-cli is a small, focused tool. Please read this short guide before opening an issue or PR.

## Reporting issues

Open a [GitHub issue](../../issues) and include:

- Operating system and version
- roff-cli version (`roff --version`)
- Installation method (cargo / binary / eget / source)
- The exact command you ran and the input file (or a minimal sample)
- Expected vs actual output

## Feature requests

roff-cli deliberately stays small. We add features that make man pages more useful as structured data. If your idea fits, open an issue and explain the use case.

## Building from source

Requires Rust 1.74+.

```sh
git clone https://github.com/ljh-sh/roff
cd roff
cargo build --release
```

The binary will be at `target/release/roff`.

## Running tests

```sh
cargo test
```

For a quick parser smoke test on your local man pages:

```sh
roff bench --count 100
```

## Pull requests

- Keep the change minimal and focused.
- Follow the existing Rust style.
- Update README examples if your change affects CLI behavior.
- Do not add heavy dependencies.

## License

By contributing, you agree that your contributions will be licensed under the Apache 2.0 License.
