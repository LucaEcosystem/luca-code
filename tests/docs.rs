//! Documentation integrity checks.
//!
//! These tests keep the public Markdown honest: every relative link resolves,
//! the required guides exist, and stale instructions (a `luca run` subcommand,
//! a private-repository note, an old test count) do not creep back in.
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_markdown(directory: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if matches!(
                name.as_ref(),
                "target" | ".git" | "node_modules" | "contributor"
            ) {
                continue;
            }
            collect_markdown(&path, files);
        } else if name.ends_with(".md") {
            files.push(path);
        }
    }
}

fn public_markdown() -> Vec<PathBuf> {
    let root = repo_root();
    let mut files = Vec::new();
    for name in [
        "README.md",
        "CHANGELOG.md",
        "CONTRIBUTING.md",
        "SECURITY.md",
        "RELEASING.md",
    ] {
        let path = root.join(name);
        if path.exists() {
            files.push(path);
        }
    }
    collect_markdown(&root.join("docs"), &mut files);
    collect_markdown(&root.join(".github"), &mut files);
    for name in [
        "examples/README.md",
        "editors/vscode/README.md",
        "editors/vscode/CHANGELOG.md",
    ] {
        let path = root.join(name);
        if path.exists() {
            files.push(path);
        }
    }
    files.sort();
    files
}

/// Extract inline Markdown link targets, ignoring fenced code blocks.
fn link_targets(text: &str) -> Vec<(usize, String)> {
    let mut targets = Vec::new();
    let mut in_fence = false;
    for (index, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let mut rest = line;
        while let Some(start) = rest.find("](") {
            let after = &rest[start + 2..];
            let Some(end) = after.find(')') else {
                break;
            };
            let target = after[..end].trim();
            targets.push((index + 1, target.to_owned()));
            rest = &after[end + 1..];
        }
    }
    targets
}

fn is_external(target: &str) -> bool {
    target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("mailto:")
        || target.starts_with("data:")
}

/// True when the text mentions a `luca run` subcommand, without matching the
/// ordinary sentence "Luca runs your file".
fn mentions_luca_run(lower: &str) -> bool {
    const NEEDLE: &str = "luca run";
    let mut cursor = 0;
    while let Some(offset) = lower[cursor..].find(NEEDLE) {
        let end = cursor + offset + NEEDLE.len();
        let next = lower[end..].chars().next();
        if !next.is_some_and(|character| character.is_ascii_alphanumeric() || character == '_') {
            return true;
        }
        cursor = end;
    }
    false
}

#[test]
fn every_relative_markdown_link_resolves() {
    let root = repo_root();
    let mut broken = Vec::new();
    for file in public_markdown() {
        let text = std::fs::read_to_string(&file).expect("markdown is readable");
        for (line, raw_target) in link_targets(&text) {
            if raw_target.is_empty() || raw_target.starts_with('#') || is_external(&raw_target) {
                continue;
            }
            let path_part = raw_target.split('#').next().unwrap_or("").trim();
            let path_part = path_part.trim_start_matches('<').trim_end_matches('>');
            if path_part.is_empty() {
                continue;
            }
            let resolved = if path_part.starts_with('/') {
                root.join(path_part.trim_start_matches('/'))
            } else {
                file.parent().expect("file has a directory").join(path_part)
            };
            if !resolved.exists() {
                broken.push(format!(
                    "{}:{}: {} -> {}",
                    file.strip_prefix(&root).unwrap_or(&file).display(),
                    line,
                    raw_target,
                    resolved.display()
                ));
            }
        }
    }
    assert!(
        broken.is_empty(),
        "broken relative links:\n{}",
        broken.join("\n")
    );
}

#[test]
fn required_guides_exist() {
    let root = repo_root();
    for relative in [
        "README.md",
        "CHANGELOG.md",
        "CONTRIBUTING.md",
        "SECURITY.md",
        "RELEASING.md",
        "LICENSE",
        "docs/learn/install.md",
        "docs/learn/first-program.md",
        "docs/learn/troubleshooting.md",
        "docs/learn/vscode.md",
        "docs/learn/examples-gallery.md",
        "docs/learn/known-limitations.md",
        "docs/learn/tour/index.md",
        "docs/reference/language-reference.md",
        "docs/reference/stdlib.md",
        "docs/reference/cli.md",
        "docs/reference/glossary.md",
        "examples/README.md",
        ".github/pull_request_template.md",
        ".github/ISSUE_TEMPLATE/bug_report.yml",
        ".github/ISSUE_TEMPLATE/feature_request.yml",
    ] {
        assert!(root.join(relative).exists(), "missing {relative}");
    }
}

#[test]
fn public_docs_do_not_reference_removed_or_stale_instructions() {
    let root = repo_root();
    let mut offenders = Vec::new();
    for file in public_markdown() {
        // The changelog legitimately records historical test counts and wording.
        if file.file_name().is_some_and(|name| name == "CHANGELOG.md") {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("markdown is readable");
        for (line, content) in text.lines().enumerate() {
            let lower = content.to_ascii_lowercase();
            if mentions_luca_run(&lower) {
                offenders.push(format!(
                    "{}:{}: mentions removed `luca run` subcommand",
                    file.strip_prefix(&root).unwrap_or(&file).display(),
                    line + 1
                ));
            }
            if lower.contains("repository is private")
                || lower.contains("while the repository is private")
            {
                offenders.push(format!(
                    "{}:{}: describes the public repository as private",
                    file.strip_prefix(&root).unwrap_or(&file).display(),
                    line + 1
                ));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "stale documentation:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn readme_states_the_current_test_count_and_limitations() {
    let readme =
        std::fs::read_to_string(repo_root().join("README.md")).expect("README is readable");
    assert!(
        readme.contains("179 automated tests"),
        "README should state the verified test count"
    );
    assert!(
        readme.contains("docs/learn/known-limitations.md"),
        "README should link the known limitations"
    );
}

#[test]
fn examples_gallery_lists_every_top_level_example() {
    let root = repo_root();
    let gallery = std::fs::read_to_string(root.join("docs/learn/examples-gallery.md"))
        .expect("gallery is readable");
    let mut missing = Vec::new();
    for entry in std::fs::read_dir(root.join("examples")).expect("examples directory") {
        let path = entry.expect("entry").path();
        if path
            .extension()
            .is_some_and(|extension| extension == "lucc")
        {
            let name = path.file_name().unwrap().to_string_lossy();
            let reference = format!("examples/{name}");
            if !gallery.contains(&reference) {
                missing.push(reference);
            }
        }
    }
    assert!(
        missing.is_empty(),
        "examples missing from the gallery: {missing:?}"
    );
}
