# CLAUDE.md

rust-template — Rust workspace (edition 2024).

## Layout

- `crates/rust-template-core` — library: typed errors with `thiserror` + `miette::Diagnostic`.
<!-- init:bin:start -->
- `crates/rust-template` — binary: `clap`, renders errors with `miette` (fancy), logs via `tracing` (`RUST_LOG`).
<!-- init:bin:end -->

## Commands

```sh
just            # list tasks
just check      # everything CI runs
just test       # nextest + doc tests
just lint       # all pre-commit hooks
```

## Conventions

- Conventional commits; PR titles too (squash merges drive release-plz).
- Never commit to `main`; work in feature branches.
- Toolchain: `rust-toolchain.toml`. MSRV: `rust-version` in `Cargo.toml`. Hooks: `.pre-commit-config.yaml`.
- Clippy pedantic + nursery, warnings are errors in CI; `unsafe` is forbidden.
- New crates go in `crates/`; set `publish = false` for crates that must not reach crates.io.
<!-- init:template:start -->

## Template notes

This repo is a template. Optional content is wrapped in `init:<group>:start` / `init:<group>:end`
marker lines (groups: template, bin, dual, agpl) and the tokens `rust-template`, `rust_template`
and the dual license string are replaced by `cargo xtask init`. Keep new template content
expressed through these tokens and markers, and extend `xtask/src/main.rs` tests when adding groups.
<!-- init:template:end -->
