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

Releases are automated with [release-plz](https://release-plz.dev):

1. Every PR is squash-merged, so each PR becomes one commit on `main` whose message is the PR
   title. Titles must be [conventional commits](https://www.conventionalcommits.org) (checked
   by the `PR Title` workflow): `fix:` bumps the patch version, `feat:` the minor, `feat!:` the
   major.
2. After every merge, release-plz updates a single open "chore: release" PR with the version
   bumps and the new `CHANGELOG.md` entries. Don't edit it; it collects changes until you are
   ready to release.
3. Merging that PR publishes the crates to crates.io and pushes `<crate>-v<version>` tags.
<!-- init:bin:start -->
4. The `rust-template-v<version>` tag triggers [dist](https://opensource.axo.dev/cargo-dist/),
   which builds the binaries and installers, creates the GitHub Release and updates the
   Homebrew formula. The binary crate itself is not published to crates.io.
<!-- init:bin:end -->

### One-time setup per repository

Until this is done, the release-plz and docs workflow runs fail.

1. **GitHub App** (create once per account, install on each repo). Actions run with the
   built-in `GITHUB_TOKEN`, and GitHub deliberately does not start other workflows from events
   that token causes. With it, CI would never run on the release PR (so the required checks
   never pass and it can't be merged) and the release tag would never trigger dist. A token
   from your own App does trigger workflows.
   - Settings → Developer settings → GitHub Apps → New GitHub App. No webhook needed.
     Repository permissions: *Contents* and *Pull requests* read/write.
   - Generate a private key, then install the App on this repo.
   - Add the secrets: `gh secret set APP_CLIENT_ID` (the App's Client ID) and
     `gh secret set APP_PRIVATE_KEY < key.pem`.

   A fine-grained personal access token works too, but it acts as you and expires; the App
   only has the permissions you gave it, on the repos it is installed on.
2. **First publish by hand**: `cargo publish -p rust-template-core`. crates.io only lets you
   configure a trusted publisher for a crate that already exists.
3. **Trusted publisher** on crates.io (crate → Settings → Trusted Publishing) for each crate:
   this repo, workflow `release-plz.yml`, environment `release`. The release job then gets a
   short-lived publish token through GitHub's OIDC, and no crates.io token is stored anywhere.
   The environment pins publishing to the one job that declares `environment: release`;
   GitHub creates it on the first run. Add protection rules to it (e.g. only `main` may deploy)
   under Settings → Environments if you want.
4. **Repository settings**: `just gh-setup` applies the ruleset for `main` (PR-only, squash,
   required checks), squash-only merges with the PR title as commit message, and GitHub Pages.
   GitHub does not copy these from the template.
<!-- init:bin:start -->
5. **Homebrew**: create the public repo `mrbandler/homebrew-tap` (once per account, shared by all
   projects), create a fine-grained token with *Contents* read/write on it, and add it as a
   secret: `gh secret set HOMEBREW_TAP_TOKEN`. dist pushes the formula on every release.
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
