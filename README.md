# rust-template
<!-- init:template:start -->

> **This is a template.** Follow [Using this template](#using-this-template) below to turn it
> into your own project. That section is removed by `just init`.

## Using this template

### What you get

- A Cargo workspace with a library crate (`<name>`) and, optionally, a CLI crate (`<name>-cli`,
  installing the command `<name>`) with `clap`, `miette` error reports and `tracing` logs. The
  library owns the crates.io name, so it can be embedded with `cargo add <name>`. Add more crates
  (e.g. `<name>-syntax`) under `crates/` as the project grows.
- Dev environment via [devenv](https://devenv.sh) + direnv, or plain `just setup` without Nix.
- Git hooks ([prek](https://github.com/j178/prek)): fmt, clippy (pedantic), typos, zizmor, …
- CI on Linux, macOS and Windows, MSRV check, cargo-deny, docs site on GitHub Pages.
- Automated releases: [release-plz](https://release-plz.dev) for versions, changelog and
  crates.io; [dist](https://opensource.axo.dev/cargo-dist/) for binaries, installers, MSI and
  Homebrew; a Nix flake.

### 1. Prerequisites

- A GitHub account and the [GitHub CLI](https://cli.github.com) (`gh auth login`).
- Either [Nix](https://nixos.org/download) with [direnv](https://direnv.net), or
  [rustup](https://rustup.rs), [just](https://github.com/casey/just#installation) and
  [cargo-binstall](https://github.com/cargo-bins/cargo-binstall#installation).
- A [crates.io](https://crates.io) account if you want to publish (log in with `cargo login`).

### 2. Create your repository

Click **Use this template** on GitHub, or:

```sh
gh repo create <you>/<name> --template mrbandler/rust-template --public --clone
cd <name>
```

`<name>` becomes your crate names on crates.io, which are permanent once published: lowercase
letters, digits and single dashes, starting with a letter. Check that it's free first:
`cargo search <name>` must not list that exact name. `init` checks this too and refuses taken
names.

### 3. Apply the repository settings

GitHub copies only the files from a template, not its settings. Run once:

```sh
just gh-setup
```

This adds a ruleset for `main` (changes only through squash-merged PRs with passing `CI Success`
and `PR Title` checks), makes squash merging with the PR title the only merge method, and
enables GitHub Pages for the docs site. It fails if run a second time; that's expected.

### 4. Set up the dev environment

```sh
direnv allow      # with Nix: installs all tools and the git hooks
just setup        # without Nix: installs the tools via cargo-binstall and the git hooks
```

### 5. Initialize the project

Commits to `main` are blocked, so work on a branch:

```sh
git switch -c chore/init
just init <name> [--lib] [--license dual|mit|apache|agpl] [--owner <github-user>] [--author-name <name>] [--author-email <email>] [--skip-name-check]
```

| Option | Effect |
| --- | --- |
| *(none)* | Library `<name>` + CLI crate `<name>-cli` (installs the command `<name>`), binary releases via dist, MIT OR Apache-2.0. |
| `--lib` | Library only. Drops the CLI crate, dist, installers, Homebrew and the Nix flake. |
| `--license mit` / `apache` | Single license instead of the dual MIT OR Apache-2.0. |
| `--license agpl` | AGPL-3.0-or-later, plus a CLA (`CLA.md`) and a workflow that asks contributors to sign it. |
| `--owner <github-user>` | GitHub user or org for repo URLs, install commands and the Homebrew tap. Default: read from the `origin` remote. |
| `--author-name <name>` | Your name for `authors`, the licenses, the code of conduct, the docs and the MSI. Default: `git config user.name`. |
| `--author-email <email>` | Your email for `authors`, `SECURITY.md` and the code of conduct. Default: `git config user.email`. |
| `--skip-name-check` | Skip the check that the library crate name `<name>` is still free on crates.io. Only for names you already own. |

`init` replaces `rust-template` everywhere (crate names, docs, workflows) and the template
author's name, email and GitHub user with yours, removes the parts you didn't choose, sets up the license files, generates fresh MSI GUIDs, deletes this section and
the `xtask` crate itself, and runs `cargo check`. It only works once: undo with
`git checkout . && git clean -fd` if you want to run it again with other options.

### 6. Make it yours

Write real descriptions: the line under the title here and `description` in each
`crates/*/Cargo.toml`. Check that nothing of the template author is left:
`git grep -n -e mrbandler -e "Michael Baudler"` should print nothing.

### 7. Check and open the PR

```sh
just check
git add -A
git commit -m "chore: initialize from template"
git push -u origin chore/init
gh pr create --fill
```

PR titles must be [conventional commits](https://www.conventionalcommits.org) (`feat:`, `fix:`,
`chore:`, …); they become the commit message on `main` and drive the version bumps. Merge once
the checks pass.

### 8. Set up releases

Follow the one-time setup in [Releasing](#releasing). Until that is done, the release-plz
workflow fails on every push to `main`. After that, each merge updates a "chore: release" PR;
merge it to ship your first release.
<!-- init:template:end -->

Short description of rust-template.
<!-- init:bin:start -->

## Install

```sh
# macOS / Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/mrbandler/rust-template/releases/latest/download/rust-template-cli-installer.sh | sh
```

```powershell
# Windows
powershell -ExecutionPolicy Bypass -c "irm https://github.com/mrbandler/rust-template/releases/latest/download/rust-template-cli-installer.ps1 | iex"
```

```sh
# Homebrew (macOS / Linux)
brew install mrbandler/tap/rust-template
```

Windows installer (MSI): [rust-template-cli-x86_64-pc-windows-msvc.msi](https://github.com/mrbandler/rust-template/releases/latest/download/rust-template-cli-x86_64-pc-windows-msvc.msi)

```sh
# Nix: try it without installing, or install it
nix run github:mrbandler/rust-template -- --help
nix profile install github:mrbandler/rust-template
```

```sh
# Prebuilt binary via cargo-binstall
cargo binstall --git https://github.com/mrbandler/rust-template rust-template-cli
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
4. The `rust-template-cli-v<version>` tag triggers [dist](https://opensource.axo.dev/cargo-dist/),
   which builds the `rust-template` command, the installers and the GitHub Release, and updates the
   Homebrew formula. The CLI crate (`rust-template-cli`) itself is not published to crates.io.
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
2. **First publish by hand**: `cargo publish -p rust-template`. crates.io only lets you
   configure a trusted publisher for a crate that already exists.
3. **Trusted publisher** on crates.io (crate → Settings → Trusted Publishing) for each crate:
   this repo, workflow `release-plz.yml`, environment `release`. The release job then gets a
   short-lived publish token through GitHub's OIDC, and no crates.io token is stored anywhere.
   The environment pins publishing to the one job that declares `environment: release`;
   GitHub creates it on the first run. Add protection rules to it (e.g. only `main` may deploy)
   under Settings → Environments if you want.
4. **Repository settings**: `just gh-setup` applies the ruleset for `main` (PR-only, squash,
   required checks), squash-only merges with the PR title as commit message, and GitHub Pages.
   GitHub does not copy these from the template. Skip it if you already ran it.
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
