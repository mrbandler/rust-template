set windows-shell := ["pwsh", "-NoLogo", "-NoProfile", "-Command"]

# List recipes
default:
    @just --list

# Install dev tools without Nix (needs cargo-binstall) and the git hooks. Keep in sync with devenv.nix.
setup:
    cargo binstall -y prek cargo-deny cargo-shear cargo-nextest cargo-hack typos-cli zizmor mdbook
    prek install

# Format all code
fmt:
    cargo fmt --all

# Run all pre-commit hooks on every file
lint:
    prek run --all-files

# Run tests (nextest + doc tests)
test *ARGS:
    cargo nextest run --workspace --all-features {{ ARGS }}
    cargo test --doc --workspace --all-features

# Check that every feature compiles on its own
features:
    cargo hack check --workspace --each-feature --no-dev-deps

# Dependency policy and unused dependencies
deps:
    cargo deny check
    cargo shear

# Build API docs
doc:
    cargo doc --workspace --all-features --no-deps

# Serve the mdBook with live reload
book:
    mdbook serve docs --open

# Run everything CI runs
check: lint test features deps doc

# Apply GitHub repo settings: main ruleset, squash-only merges, Pages via Actions (needs gh).
# One-shot: re-running fails (ruleset name exists -> 422, Pages already enabled -> 409).
gh-setup:
    gh api --method POST "repos/{owner}/{repo}/rulesets" --input .github/rulesets/main.json
    gh api --method PATCH "repos/{owner}/{repo}" -F allow_squash_merge=true -F allow_merge_commit=false -F allow_rebase_merge=false -F delete_branch_on_merge=true -f squash_merge_commit_title=PR_TITLE -f squash_merge_commit_message=PR_BODY
    gh api --method POST "repos/{owner}/{repo}/pages" -f build_type=workflow
# init:template:start

# Arguments are passed through as-is (not via {{ }}), so quoted values with spaces survive.
# Turn this template into a new project (see README.md for the options)
[positional-arguments]
[unix]
init *ARGS:
    cargo xtask init "$@"

# `pwsh -Command` would join the arguments back into one string; a script gets them as `$args`.
[positional-arguments]
[windows]
[script("pwsh", "-NoLogo", "-NoProfile")]
[extension(".ps1")]
init *ARGS:
    cargo xtask init @args
    exit $LASTEXITCODE
# init:template:end
