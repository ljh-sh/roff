# Release process

## Reproducible builds

Release artifacts are byte-stable: the same source + `Cargo.lock` + toolchain
produce identical sha256 on every build. Three things make this work:

1. **`rust-toolchain.toml`** pins rustc to an immutable dated stable channel
   (`stable-2024-11-28`), so every CI/release run resolves the exact same
   compiler.
2. **`--locked`** on every cargo invocation forbids re-resolving `Cargo.lock`.
3. **`SOURCE_DATE_EPOCH` + `--remap-path-prefix`** in `release.yml` strip
   timestamps and absolute build paths from the binaries.

## Verification

[`.github/workflows/reproducible.yml`](../.github/workflows/reproducible.yml)
builds each target **twice** (into separate `--target-dir`s) and diffs the
sha256. It fails the check if the two builds disagree.

- **Tag push / `workflow_dispatch`**: all 7 release targets.
- **Pull request**: a single `x86_64-unknown-linux-musl` target (cheap drift
  check); the full matrix runs at release time.

Per-target sha256 pairs are uploaded as `reproducible-<target>` artifacts.

If a target ever comes back non-reproducible, the failing check names the
target and the two mismatched hashes — investigate newly-added
non-determinism (e.g. a dependency embedding `env!`/build paths, a new macro
that captures `file!`/`line!`, or a toolchain drift from `rust-toolchain.toml`).

## Cutting a release

```bash
git checkout main && git pull origin main
# bump version in Cargo.toml + write changelog/vX.Y.Z.md
# amend the version bump to include Cargo.lock (cargo publish rejects a dirty lock)
git tag vX.Y.Z && git push origin vX.Y.Z   # triggers release.yml (7 targets + cosign + crates.io)
```

The release workflow signs artifacts with cosign and publishes to crates.io.
