# rust-template
<!-- init:template:start -->

> **This is a template.** Click **Use this template** on GitHub, clone your new repo, then:
>
> ```sh
> just init <name> [--lib] [--license dual|mit|apache|agpl]
> ```
>
> `--lib` drops the binary crate and binary releases. `--license agpl` also adds a CLA.
> `init` renames everything, strips this section, and deletes itself. Commit the result on a
> branch and open a PR: commits to `main` are blocked by a hook (and by the ruleset once
> `just gh-setup` ran). Then follow [Releasing](#releasing) once; until that setup is done the
> release-plz and docs workflow runs fail.
<!-- init:template:end -->

Short description of rust-template.
<!-- init:bin:start -->

## Install

```sh
# macOS / Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/mrbandler/rust-template/releases/latest/download/rust-template-installer.sh | sh
```

```powershell
# Windows
powershell -ExecutionPolicy Bypass -c "irm https://github.com/mrbandler/rust-template/releases/latest/download/rust-template-installer.ps1 | iex"
```

```sh
# Homebrew (macOS / Linux)
brew install mrbandler/tap/rust-template
```

Windows installer (MSI): [rust-template-x86_64-pc-windows-msvc.msi](https://github.com/mrbandler/rust-template/releases/latest/download/rust-template-x86_64-pc-windows-msvc.msi)

```sh
# Nix: try it without installing, or install it
nix run github:mrbandler/rust-template -- --help
nix profile install github:mrbandler/rust-template
```

```sh
# Prebuilt binary via cargo-binstall
cargo binstall --git https://github.com/mrbandler/rust-template rust-template
```
<!-- init:bin:end -->

## Development

With Nix: `direnv allow` (or `devenv shell`). Everything, including the git hooks, is set up.

Without Nix (macOS, Linux, Windows): install [rustup](https://rustup.rs),
[just](https://github.com/casey/just#installation) and
[cargo-binstall](https://github.com/cargo-bins/cargo-binstall#installation), then run `just setup`.

`just` lists all tasks; `just check` runs everything CI runs.

## Releasing

Releases are automated with [release-plz](https://release-plz.dev): merging feature PRs keeps a
"chore: release" PR up to date; merging that PR publishes to crates.io and tags the release.
<!-- init:bin:start -->
Tags of the binary crate trigger [dist](https://opensource.axo.dev/cargo-dist/), which builds
the binaries and installers.
<!-- init:bin:end -->

One-time setup per repository:

1. Create a GitHub App (once per account) with *Contents* and *Pull requests* read/write,
   install it on the repo, and add the secrets `APP_CLIENT_ID` (the App's Client ID) and
   `APP_PRIVATE_KEY`.
2. Create an environment named `release` (Settings → Environments).
3. Publish each crate once by hand: `cargo publish -p rust-template-core`.
4. On crates.io, add a trusted publisher for each crate: this repo, workflow `release-plz.yml`,
   environment `release`.
5. Run `just gh-setup` (ruleset for `main`, squash-only merges, GitHub Pages).
<!-- init:bin:start -->
6. Homebrew: create the public repo `mrbandler/homebrew-tap` (once per account, shared by all
   projects), create a fine-grained token with *Contents* read/write on it, and add it to this
   repo as the secret `HOMEBREW_TAP_TOKEN`. dist pushes the formula on every release.
<!-- init:bin:end -->

## License

Licensed under `MIT OR Apache-2.0`. See the `LICENSE*` files.
<!-- init:dual:start -->

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion
in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above,
without any additional terms or conditions.
<!-- init:dual:end -->
<!-- init:agpl:start -->

Contributions require signing the [Contributor License Agreement](CLA.md).
<!-- init:agpl:end -->
