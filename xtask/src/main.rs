//! Template maintenance tasks. `init` turns the template into a new project and deletes this crate.

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

const USAGE: &str = "usage: cargo xtask init <name> [--lib] [--license dual|mit|apache|agpl] [--owner <github-user>] [--author-name <name>] [--author-email <email>]";
const TOKEN_KEBAB: &str = "rust-template";
const TOKEN_SNAKE: &str = "rust_template";
const TOKEN_LICENSE: &str = "MIT OR Apache-2.0";
const TOKEN_CORE_KEBAB: &str = "rust-template-core";
const TOKEN_CORE_SNAKE: &str = "rust_template_core";
const TOKEN_OWNER: &str = "mrbandler";
const TOKEN_AUTHOR_NAME: &str = "Michael Baudler";
const TOKEN_AUTHOR_EMAIL: &str = "hello@mrbandler.dev";
/// Directories never rewritten by `init`.
const SKIP_DIRS: &[&str] = &[".git", "target", "xtask", ".devenv", ".direnv", ".jj", "book"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum License {
    Dual,
    Mit,
    Apache,
    Agpl,
}

impl License {
    fn parse(key: &str) -> Result<Self, String> {
        match key {
            "dual" => Ok(Self::Dual),
            "mit" => Ok(Self::Mit),
            "apache" => Ok(Self::Apache),
            "agpl" => Ok(Self::Agpl),
            other => Err(format!(
                "unknown license `{other}` (expected dual, mit, apache or agpl)"
            )),
        }
    }

    const fn spdx(self) -> &'static str {
        match self {
            Self::Dual => "MIT OR Apache-2.0",
            Self::Mit => "MIT",
            Self::Apache => "Apache-2.0",
            Self::Agpl => "AGPL-3.0-or-later",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Author {
    name: String,
    email: String,
}

impl Author {
    fn new(name: &str, email: &str) -> Result<Self, String> {
        let (name, email) = (name.trim(), email.trim());
        // Both end up inside TOML and YAML strings, and as `Name <email>` in `authors`.
        let clean = |part: &str| !part.is_empty() && !part.contains(['"', '\\', '<', '>', '\n']);
        if !clean(name) {
            return Err(format!("invalid author name `{name}`"));
        }
        if !clean(email) || !email.contains('@') {
            return Err(format!("invalid author email `{email}`"));
        }
        Ok(Self {
            name: name.to_owned(),
            email: email.to_owned(),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Options {
    name: String,
    lib: bool,
    license: License,
    owner: String,
    author: Author,
}

/// Parses `init` arguments. `git` runs a git command and returns its trimmed output; it supplies
/// the defaults for `--owner` (the `origin` remote), `--author-name` (`user.name`) and
/// `--author-email` (`user.email`).
fn parse_args(args: &[String], git: impl Fn(&[&str]) -> Option<String>) -> Result<Options, String> {
    let mut name = None;
    let mut lib = false;
    let mut license = License::Dual;
    let mut owner = None;
    let mut author_name = None;
    let mut author_email = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--lib" => lib = true,
            "--license" => license = License::parse(iter.next().ok_or("`--license` needs a value")?)?,
            "--owner" => owner = Some(iter.next().ok_or("`--owner` needs a value")?.clone()),
            "--author-name" => author_name = Some(iter.next().ok_or("`--author-name` needs a value")?.clone()),
            "--author-email" => author_email = Some(iter.next().ok_or("`--author-email` needs a value")?.clone()),
            flag if flag.starts_with('-') => return Err(format!("unknown flag `{flag}`\n{USAGE}")),
            value if name.is_none() => name = Some(value.to_owned()),
            value => return Err(format!("unexpected argument `{value}`\n{USAGE}")),
        }
    }
    let name = name.ok_or(USAGE)?;
    validate_name(&name)?;
    let owner = owner
        .or_else(|| owner_from_remote(&git(&["remote", "get-url", "origin"])?))
        .ok_or("can't detect your GitHub user from the `origin` remote: pass `--owner <github-user>`")?;
    validate_owner(&owner)?;
    let author_name = author_name
        .or_else(|| git(&["config", "user.name"]))
        .ok_or("git `user.name` is not set: pass `--author-name <name>`")?;
    let author_email = author_email
        .or_else(|| git(&["config", "user.email"]))
        .ok_or("git `user.email` is not set: pass `--author-email <email>`")?;
    let author = Author::new(&author_name, &author_email)?;
    Ok(Options {
        name,
        lib,
        license,
        owner,
        author,
    })
}

/// GitHub owner from a remote URL: `git@github.com:owner/repo.git` or `https://github.com/owner/repo`.
fn owner_from_remote(url: &str) -> Option<String> {
    let mut parts = url.trim_end_matches('/').rsplit(['/', ':']);
    parts.next()?; // repository
    parts
        .next()
        .filter(|owner| !owner.is_empty())
        .map(str::to_owned)
}

fn validate_owner(owner: &str) -> Result<(), String> {
    let well_formed = (1..=39).contains(&owner.len())
        && !owner.starts_with('-')
        && owner.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    if well_formed {
        Ok(())
    } else {
        Err(format!("invalid GitHub user `{owner}`"))
    }
}

fn validate_name(name: &str) -> Result<(), String> {
    let well_formed = name.len() <= 64
        && name.starts_with(|c: char| c.is_ascii_lowercase())
        && !name.ends_with('-')
        && !name.contains("--")
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !well_formed {
        return Err(format!(
            "invalid name `{name}`: use lowercase letters, digits and single dashes, starting with a letter"
        ));
    }
    if matches!(name, "core" | "std" | "alloc" | "test" | "proc-macro" | TOKEN_KEBAB) {
        return Err(format!("`{name}` is a reserved name"));
    }
    Ok(())
}

/// Parses a marker line: `init:<group>:start` or `init:<group>:end` anywhere in the line.
fn marker(line: &str) -> Option<(&str, bool)> {
    let rest = &line[line.find("init:")? + "init:".len()..];
    let (group, rest) = rest.split_once(':')?;
    if group.is_empty() || !group.chars().all(|c| c.is_ascii_lowercase()) {
        return None;
    }
    if rest.starts_with("start") {
        Some((group, true))
    } else if rest.starts_with("end") {
        Some((group, false))
    } else {
        None
    }
}

/// Drops marker blocks whose group `keep` rejects and removes every marker line.
fn apply_markers(text: &str, keep: impl Fn(&str) -> bool) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut open: Vec<(&str, bool)> = Vec::new();
    for line in text.split_inclusive('\n') {
        match marker(line) {
            Some((group, true)) => open.push((group, keep(group))),
            Some((group, false)) => match open.pop() {
                Some((start, _)) if start == group => {},
                _ => return Err(format!("unmatched `init:{group}:end`")),
            },
            None if open.iter().all(|(_, kept)| *kept) => out.push_str(line),
            None => {},
        }
    }
    match open.pop() {
        Some((group, _)) => Err(format!("unclosed `init:{group}:start`")),
        None => Ok(out),
    }
}

fn replace_tokens(text: &str, opts: &Options) -> String {
    let name = &opts.name;
    let snake = name.replace('-', "_");
    // A library-only project has a single crate named after the project, not `<name>-core`.
    let text = if opts.lib {
        text.replace(TOKEN_CORE_KEBAB, name)
            .replace(TOKEN_CORE_SNAKE, &snake)
    } else {
        text.to_owned()
    };
    text.replace(TOKEN_KEBAB, name)
        .replace(TOKEN_SNAKE, &snake)
        .replace(TOKEN_LICENSE, opts.license.spdx())
        // The email contains the owner token: replace it first.
        .replace(TOKEN_AUTHOR_EMAIL, &opts.author.email)
        .replace(TOKEN_AUTHOR_NAME, &opts.author.name)
        .replace(TOKEN_OWNER, &opts.owner)
}

/// Removes the `[[package]]` entry named `name` from a `Cargo.lock`.
fn drop_lock_package(lock: &str, name: &str) -> String {
    let entry = format!("[[package]]\nname = \"{name}\"\n");
    lock.split_inclusive("\n\n")
        .filter(|block| !block.starts_with(&entry))
        .collect()
}

fn keep_group(group: &str, opts: &Options) -> bool {
    match group {
        "template" => false,
        "bin" => !opts.lib,
        "dual" => opts.license == License::Dual,
        "agpl" => opts.license == License::Agpl,
        _ => true,
    }
}

/// Deterministic GUID derived from `name` and `salt`, so every project gets its own MSI codes.
// ponytail: SipHash-based rather than a standard UUIDv5; MSI upgrade/path codes only need uniqueness.
fn guid(name: &str, salt: &str) -> String {
    use std::{
        fmt::Write,
        hash::{DefaultHasher, Hash, Hasher},
    };
    let half = |part: u8| {
        let mut hasher = DefaultHasher::new();
        (name, salt, part).hash(&mut hasher);
        hasher.finish().to_be_bytes()
    };
    let mut bytes = [0u8; 16];
    bytes[..8].copy_from_slice(&half(0));
    bytes[8..].copy_from_slice(&half(1));
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // version 4
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // RFC 4122 variant
    let hex = bytes.iter().fold(String::new(), |mut acc, byte| {
        let _ = write!(acc, "{byte:02X}"); // writing to a String cannot fail
        acc
    });
    format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    )
}

/// Values of `upgrade-guid` and `path-guid` in a `[package.metadata.wix]` table.
fn wix_guids(cargo_toml: &str) -> Vec<String> {
    cargo_toml
        .lines()
        .map(str::trim)
        .filter_map(|line| {
            line.strip_prefix("upgrade-guid = \"")
                .or_else(|| line.strip_prefix("path-guid = \""))
        })
        .filter_map(|rest| rest.strip_suffix('"'))
        .map(str::to_owned)
        .collect()
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let kind = entry.file_type()?; // does not follow symlinks
        // `.git` is a file in worktrees and submodules; symlinks would rewrite their targets.
        if entry.file_name() == ".git" || kind.is_symlink() {
            continue;
        }
        let path = entry.path();
        if kind.is_dir() {
            let skipped = entry
                .file_name()
                .to_str()
                .is_some_and(|name| SKIP_DIRS.contains(&name));
            if !skipped {
                collect_files(&path, out)?;
            }
        } else {
            out.push(path);
        }
    }
    Ok(())
}

fn io_err(path: &Path) -> impl FnOnce(std::io::Error) -> String + '_ {
    move |err| format!("{}: {err}", path.display())
}

fn remove(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(io_err(path))
    } else if path.exists() {
        fs::remove_file(path).map_err(io_err(path))
    } else {
        Ok(())
    }
}

fn rename(from: &Path, to: &Path) -> Result<(), String> {
    fs::rename(from, to).map_err(io_err(from))
}

/// Copies a text asset from `xtask/assets`, replacing tokens on the way.
fn copy_asset(from: &Path, to: &Path, opts: &Options) -> Result<(), String> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(io_err(parent))?;
    }
    let text = fs::read_to_string(from).map_err(io_err(from))?;
    fs::write(to, replace_tokens(&text, opts)).map_err(io_err(to))
}

/// dist-generated release workflow, named after `tag-namespace`.
const RELEASE_WORKFLOW: &str = ".github/workflows/rust-template-v-release.yml";

fn init(root: &Path, opts: &Options) -> Result<(), String> {
    let assets = root.join("xtask/assets");
    if !assets.is_dir() {
        return Err("already initialized (xtask/ is gone)".into());
    }

    // The template's MSI GUIDs must not be shared between projects: derive new ones from the name.
    let guid_swaps: Vec<(String, String)> = if opts.lib {
        Vec::new()
    } else {
        let manifest = fs::read_to_string(root.join("crates/rust-template/Cargo.toml")).unwrap_or_default();
        wix_guids(&manifest)
            .into_iter()
            .map(|old| {
                let fresh = guid(&opts.name, &old);
                (old, fresh)
            })
            .collect()
    };

    println!(
        "Initializing `{}` for {} <{}> (github.com/{})",
        opts.name, opts.author.name, opts.author.email, opts.owner
    );

    // Pass 1: compute every rewrite first so a marker error leaves the tree untouched.
    let mut files = Vec::new();
    collect_files(root, &mut files).map_err(io_err(root))?;
    let mut rewrites = Vec::new();
    for path in files {
        let Ok(text) = fs::read_to_string(&path) else { continue }; // binary file
        // With `--lib` the binary and core crate would both be renamed to `<name>`: drop the binary.
        let source = if opts.lib && path == root.join("Cargo.lock") {
            drop_lock_package(&text, TOKEN_KEBAB)
        } else {
            text.clone()
        };
        let stripped = apply_markers(&source, |group| keep_group(group, opts))
            .map_err(|err| format!("{}: {err}", path.display()))?;
        let mut new = replace_tokens(&stripped, opts);
        for (old, fresh) in &guid_swaps {
            new = new
                .replace(old, fresh)
                .replace(&old.to_lowercase(), &fresh.to_lowercase());
        }
        if new != text {
            rewrites.push((path, new));
        }
    }
    // Pass 2: write.
    println!("  rewriting {} files", rewrites.len());
    for (path, text) in rewrites {
        fs::write(&path, text).map_err(io_err(&path))?;
    }

    let mut doomed = vec![".github/workflows/template.yml", ".cargo"];
    if opts.lib {
        doomed.extend([
            "crates/rust-template",
            "dist-workspace.toml",
            RELEASE_WORKFLOW,
            // Only holds ignores for the dist-generated release workflow.
            ".github/zizmor.yml",
            "flake.nix",
            "flake.lock",
        ]);
    }
    for path in doomed {
        println!("  removing {path}");
        remove(&root.join(path))?;
    }

    apply_license(root, &assets, opts)?;

    let crates = root.join("crates");
    let core = if opts.lib {
        opts.name.clone()
    } else {
        format!("{}-core", opts.name)
    };
    println!(
        "  crates: {core}{}",
        if opts.lib {
            String::new()
        } else {
            format!(", {}", opts.name)
        }
    );
    rename(&crates.join("rust-template-core"), &crates.join(core))?;
    if !opts.lib {
        rename(&crates.join("rust-template"), &crates.join(&opts.name))?;
        // dist names it after `tag-namespace` in dist-workspace.toml.
        rename(
            &root.join(RELEASE_WORKFLOW),
            &root.join(format!(".github/workflows/{}-v-release.yml", opts.name)),
        )?;
    }
    println!("  removing xtask");
    remove(&root.join("xtask"))
}

/// Keeps or swaps in the license files for the chosen license.
fn apply_license(root: &Path, assets: &Path, opts: &Options) -> Result<(), String> {
    println!("  license: {}", opts.license.spdx());
    match opts.license {
        License::Dual => {},
        License::Mit => {
            remove(&root.join("LICENSE-APACHE"))?;
            rename(&root.join("LICENSE-MIT"), &root.join("LICENSE"))?;
        },
        License::Apache => {
            remove(&root.join("LICENSE-MIT"))?;
            rename(&root.join("LICENSE-APACHE"), &root.join("LICENSE"))?;
        },
        License::Agpl => {
            remove(&root.join("LICENSE-MIT"))?;
            remove(&root.join("LICENSE-APACHE"))?;
            copy_asset(&assets.join("LICENSE-AGPL"), &root.join("LICENSE"), opts)?;
            copy_asset(&assets.join("CLA.md"), &root.join("CLA.md"), opts)?;
            copy_asset(&assets.join("cla.yml"), &root.join(".github/workflows/cla.yml"), opts)?;
        },
    }
    Ok(())
}

/// Runs git in `root` and returns its trimmed output, if it succeeded and printed something.
fn git(root: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    let text = text.trim();
    (out.status.success() && !text.is_empty()).then(|| text.to_owned())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the workspace root");
    let result = match args.split_first() {
        Some((cmd, rest)) if cmd == "init" => parse_args(rest, |args| git(root, args)).and_then(|opts| {
            init(root, &opts)?;
            println!("Running `cargo check` (the first run compiles all dependencies)");
            let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
            let ok = Command::new(cargo)
                .args(["check", "--workspace"])
                .current_dir(root)
                .status()
                .is_ok_and(|status| status.success());
            if ok {
                Ok(opts)
            } else {
                Err("`cargo check` failed after init".into())
            }
        }),
        _ => Err(USAGE.to_owned()),
    };
    match result {
        Ok(opts) => {
            println!(
                "\nInitialized `{}`. Next steps:\n  \
                 1. Review the changes and run `just check`\n  \
                 2. Commit them on a branch and open a PR (conventional title, e.g. `chore: initialize from template`)\n  \
                 3. Run `just gh-setup` once, if you haven't yet\n  \
                 4. Do the one-time release setup in README.md (Releasing)",
                opts.name
            );
            ExitCode::SUCCESS
        },
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(ToString::to_string).collect()
    }

    /// Stands in for git: a GitHub `origin` remote and a configured user.
    fn fake_git(args: &[&str]) -> Option<String> {
        match args {
            ["remote", "get-url", "origin"] => Some("git@github.com:octo-org/demo.git".into()),
            ["config", "user.name"] => Some("Octo Cat".into()),
            ["config", "user.email"] => Some("octo@example.com".into()),
            _ => None,
        }
    }

    fn options(lib: bool, license: License) -> Options {
        Options {
            name: "demo".into(),
            lib,
            license,
            owner: "octo-org".into(),
            author: Author {
                name: "Octo Cat".into(),
                email: "octo@example.com".into(),
            },
        }
    }

    #[test]
    fn accepts_valid_names() {
        for name in ["app", "my-app", "app2", "a-b-c"] {
            assert_eq!(validate_name(name), Ok(()), "{name}");
        }
    }

    #[test]
    fn rejects_invalid_names() {
        for name in [
            "",
            "App",
            "-app",
            "app-",
            "a--b",
            "1app",
            "a_b",
            "core",
            "rust-template",
        ] {
            assert!(validate_name(name).is_err(), "{name}");
        }
    }

    #[test]
    fn parses_defaults_from_git() {
        let opts = parse_args(&args(&["demo"]), fake_git).unwrap();
        assert_eq!(opts, options(false, License::Dual));
    }

    #[test]
    fn parses_all_flags() {
        let list = [
            "demo",
            "--lib",
            "--license",
            "agpl",
            "--owner",
            "me",
            "--author-name",
            " Me Too ",
            "--author-email",
            "me@x.dev",
        ];
        let opts = parse_args(&args(&list), |_| None).unwrap();
        assert_eq!(
            opts,
            Options {
                owner: "me".into(),
                author: Author {
                    name: "Me Too".into(),
                    email: "me@x.dev".into(),
                },
                ..options(true, License::Agpl)
            }
        );
    }

    #[test]
    fn requires_owner_and_author_without_git() {
        let all = ["demo", "--owner", "me", "--author-name", "A", "--author-email", "a@b"];
        assert!(parse_args(&args(&all), |_| None).is_ok());
        for skip in [1, 3, 5] {
            let mut list = all.to_vec();
            list.drain(skip..skip + 2);
            assert!(parse_args(&args(&list), |_| None).is_err(), "{list:?}");
        }
    }

    #[test]
    fn reads_owner_from_remotes() {
        for url in [
            "git@github.com:octo/repo.git",
            "https://github.com/octo/repo",
            "https://github.com/octo/repo.git/",
            "ssh://git@github.com/octo/repo",
        ] {
            assert_eq!(owner_from_remote(url).as_deref(), Some("octo"), "{url}");
        }
        assert_eq!(owner_from_remote("repo"), None);
    }

    #[test]
    fn rejects_bad_owner_and_author() {
        for owner in ["", "-x", "a/b", "a b", "a\"b"] {
            assert!(validate_owner(owner).is_err(), "{owner}");
        }
        for (name, email) in [
            ("", "a@b"),
            ("Na\"me", "a@b"),
            ("<x>", "a@b"),
            ("A", ""),
            ("A", "ab"),
            ("A", "<a@b>"),
        ] {
            assert!(Author::new(name, email).is_err(), "{name} {email}");
        }
    }

    #[test]
    fn rejects_bad_args() {
        let cases: [&[&str]; 8] = [
            &[],
            &["--lib"],
            &["demo", "--license"],
            &["demo", "--license", "gpl"],
            &["demo", "--x"],
            &["a", "b"],
            &["demo", "--owner"],
            &["demo", "--author-name"],
        ];
        for list in cases {
            assert!(parse_args(&args(list), fake_git).is_err(), "{list:?}");
        }
    }

    #[test]
    fn strips_dropped_groups_and_all_marker_lines() {
        let text = "a\n# init:bin:start\nb\n# init:bin:end\nc\n<!-- init:dual:start -->\nd\n<!-- init:dual:end -->\n";
        let out = apply_markers(text, |g| g == "dual").unwrap();
        assert_eq!(out, "a\nc\nd\n");
    }

    #[test]
    fn nested_groups_drop_when_any_parent_drops() {
        let text = "# init:bin:start\n# init:agpl:start\nx\n# init:agpl:end\n# init:bin:end\ny";
        assert_eq!(apply_markers(text, |g| g == "agpl").unwrap(), "y");
        assert_eq!(apply_markers(text, |_| true).unwrap(), "x\ny");
    }

    #[test]
    fn ignores_non_marker_mentions() {
        let text = "see `init:<group>:start` in docs\n";
        assert_eq!(apply_markers(text, |_| false).unwrap(), text);
    }

    #[test]
    fn rejects_unbalanced_markers() {
        assert!(apply_markers("# init:bin:start\n", |_| true).is_err());
        assert!(apply_markers("# init:bin:end\n", |_| true).is_err());
        assert!(apply_markers("# init:bin:start\n# init:dual:end\n", |_| true).is_err());
    }

    #[test]
    fn replaces_all_tokens() {
        let text = "rust-template rust-template-core rust_template_core license = \"MIT OR Apache-2.0\"";
        let mut opts = Options {
            name: "foo-bar".into(),
            ..options(false, License::Agpl)
        };
        assert_eq!(
            replace_tokens(text, &opts),
            "foo-bar foo-bar-core foo_bar_core license = \"AGPL-3.0-or-later\""
        );
        opts.lib = true;
        assert_eq!(
            replace_tokens(text, &opts),
            "foo-bar foo-bar foo_bar license = \"AGPL-3.0-or-later\""
        );
        let identity = "Michael Baudler <hello@mrbandler.dev> github.com/mrbandler/x";
        assert_eq!(
            replace_tokens(identity, &opts),
            "Octo Cat <octo@example.com> github.com/octo-org/x"
        );
    }

    #[test]
    fn collect_files_skips_git_entries_and_symlinks() {
        let root = env::temp_dir().join(format!("xtask-collect-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("sub")).unwrap();
        fs::write(root.join("sub/.git"), "gitdir: ../rust-template\n").unwrap();
        fs::write(root.join("a.txt"), "a").unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root.join("a.txt"), root.join("link.txt")).unwrap();
            std::os::unix::fs::symlink(root.join("sub"), root.join("linkdir")).unwrap();
        }
        let mut files = Vec::new();
        collect_files(&root, &mut files).unwrap();
        assert_eq!(files, [root.join("a.txt")]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn keeps_groups_per_options() {
        let opts = options(true, License::Agpl);
        assert!(!keep_group("template", &opts));
        assert!(!keep_group("bin", &opts));
        assert!(!keep_group("dual", &opts));
        assert!(keep_group("agpl", &opts));
    }

    #[test]
    fn guid_is_deterministic_well_formed_and_name_specific() {
        let a = guid("demo", "x");
        assert_eq!(a, guid("demo", "x"));
        assert_ne!(a, guid("other", "x"));
        assert_ne!(a, guid("demo", "y"));
        assert_eq!(a.len(), 36);
        assert_eq!(a.matches('-').count(), 4);
        assert_eq!(&a[14..15], "4");
        assert!(
            a.chars()
                .all(|c| c == '-' || c.is_ascii_digit() || ('A'..='F').contains(&c))
        );
    }

    #[test]
    fn reads_wix_guids() {
        let toml = "[package.metadata.wix]\nupgrade-guid = \"AAA\"\npath-guid = \"BBB\"\nlicense = false\n";
        assert_eq!(wix_guids(toml), ["AAA", "BBB"]);
    }

    #[test]
    fn drops_one_lock_package() {
        let lock = "version = 4\n\n[[package]]\nname = \"rust-template\"\nversion = \"0.1.0\"\n\n[[package]]\nname = \"rust-template-core\"\nversion = \"0.1.0\"\n";
        assert_eq!(
            drop_lock_package(lock, "rust-template"),
            "version = 4\n\n[[package]]\nname = \"rust-template-core\"\nversion = \"0.1.0\"\n"
        );
    }

    /// Builds a minimal template tree in a fresh temp dir.
    fn fixture(tag: &str) -> PathBuf {
        let root = env::temp_dir().join(format!("xtask-fixture-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let files: &[(&str, &str)] = &[
            (
                "Cargo.toml",
                "members = [\n# init:template:start\n\"xtask\",\n# init:template:end\n]\nlicense = \"MIT OR Apache-2.0\"\n# init:bin:start\nclap = \"4\"\n# init:bin:end\n",
            ),
            (
                "README.md",
                "# rust-template\n<!-- init:dual:start -->\ndual\n<!-- init:dual:end -->\n<!-- init:agpl:start -->\ncla\n<!-- init:agpl:end -->\n",
            ),
            (
                "Cargo.lock",
                "[[package]]\nname = \"rust-template\"\n\n[[package]]\nname = \"rust-template-core\"\n",
            ),
            ("LICENSE-MIT", "mit"),
            ("LICENSE-APACHE", "apache"),
            ("dist-workspace.toml", ""),
            ("flake.nix", "pname = \"rust-template\";\n"),
            ("flake.lock", "{}"),
            (".cargo/config.toml", ""),
            (".github/workflows/template.yml", ""),
            (RELEASE_WORKFLOW, ""),
            (".github/zizmor.yml", ""),
            ("crates/rust-template-core/src/lib.rs", "//! rust_template_core\n"),
            ("crates/rust-template/src/main.rs", "use rust_template_core;\n"),
            (
                "crates/rust-template/Cargo.toml",
                "[package.metadata.wix]\nupgrade-guid = \"11111111-1111-4111-8111-111111111111\"\npath-guid = \"22222222-2222-4222-8222-222222222222\"\n",
            ),
            (
                "crates/rust-template/wix/main.wxs",
                "UpgradeCode='11111111-1111-4111-8111-111111111111'\n",
            ),
            ("xtask/assets/LICENSE-AGPL", "agpl"),
            ("xtask/assets/CLA.md", "cla by Michael Baudler"),
            ("xtask/assets/cla.yml", "allowlist: mrbandler\n"),
            ("SECURITY.md", "hello@mrbandler.dev\n"),
        ];
        for (path, content) in files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, content).unwrap();
        }
        root
    }

    fn read(root: &Path, path: &str) -> String {
        fs::read_to_string(root.join(path)).unwrap()
    }

    #[test]
    fn init_binary_project_with_dual_license() {
        let root = fixture("bin");
        init(&root, &options(false, License::Dual)).unwrap();

        assert_eq!(
            read(&root, "Cargo.toml"),
            "members = [\n]\nlicense = \"MIT OR Apache-2.0\"\nclap = \"4\"\n"
        );
        assert_eq!(read(&root, "README.md"), "# demo\ndual\n");
        assert_eq!(read(&root, "crates/demo/src/main.rs"), "use demo_core;\n");
        assert_eq!(
            read(&root, "Cargo.lock"),
            "[[package]]\nname = \"demo\"\n\n[[package]]\nname = \"demo-core\"\n"
        );
        assert!(root.join("crates/demo-core/src/lib.rs").is_file());
        assert!(root.join("LICENSE-MIT").is_file() && root.join("LICENSE-APACHE").is_file());
        assert!(root.join("dist-workspace.toml").is_file());
        assert!(root.join(".github/zizmor.yml").is_file());
        assert!(root.join(".github/workflows/demo-v-release.yml").is_file());
        assert_eq!(read(&root, "flake.nix"), "pname = \"demo\";\n");
        let upgrade = guid("demo", "11111111-1111-4111-8111-111111111111");
        let path = guid("demo", "22222222-2222-4222-8222-222222222222");
        assert_eq!(
            wix_guids(&read(&root, "crates/demo/Cargo.toml")),
            [upgrade.clone(), path]
        );
        assert_eq!(
            read(&root, "crates/demo/wix/main.wxs"),
            format!("UpgradeCode='{upgrade}'\n")
        );
        for gone in [
            "xtask",
            ".cargo",
            ".github/workflows/template.yml",
            "crates/rust-template",
            RELEASE_WORKFLOW,
        ] {
            assert!(!root.join(gone).exists(), "{gone}");
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn init_library_project_with_agpl() {
        let root = fixture("lib");
        init(&root, &options(true, License::Agpl)).unwrap();

        assert_eq!(
            read(&root, "Cargo.toml"),
            "members = [\n]\nlicense = \"AGPL-3.0-or-later\"\n"
        );
        assert_eq!(read(&root, "README.md"), "# demo\ncla\n");
        assert_eq!(read(&root, "crates/demo/src/lib.rs"), "//! demo\n");
        assert_eq!(read(&root, "Cargo.lock"), "[[package]]\nname = \"demo\"\n");
        assert_eq!(read(&root, "LICENSE"), "agpl");
        assert_eq!(read(&root, "CLA.md"), "cla by Octo Cat");
        assert_eq!(read(&root, ".github/workflows/cla.yml"), "allowlist: octo-org\n");
        assert_eq!(read(&root, "SECURITY.md"), "octo@example.com\n");
        for gone in [
            "LICENSE-MIT",
            "LICENSE-APACHE",
            "crates/demo-core",
            "crates/demo/src/main.rs",
            "crates/rust-template",
            "crates/rust-template-core",
            "dist-workspace.toml",
            RELEASE_WORKFLOW,
            ".github/zizmor.yml",
            "flake.nix",
            "flake.lock",
            "xtask",
        ] {
            assert!(!root.join(gone).exists(), "{gone}");
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn init_refuses_to_run_twice() {
        let root = fixture("twice");
        let opts = options(false, License::Mit);
        init(&root, &opts).unwrap();
        assert_eq!(read(&root, "LICENSE"), "mit");
        assert!(
            init(&root, &opts)
                .unwrap_err()
                .contains("already initialized")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn init_leaves_tree_untouched_on_marker_error() {
        let root = fixture("broken");
        fs::write(root.join("broken.md"), "<!-- init:bin:start -->\n").unwrap();
        let before = read(&root, "README.md");
        assert!(init(&root, &options(false, License::Dual)).is_err());
        assert_eq!(read(&root, "README.md"), before);
        assert!(root.join("xtask").exists());
        fs::remove_dir_all(root).unwrap();
    }
}
