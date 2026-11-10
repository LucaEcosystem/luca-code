//! Repository metadata checks.
//!
//! These tests pin the release identity, keep the extension and CLI versions in
//! sync, ensure internal engineering docs stay unpublished, and guard the
//! zero-dependency / non-publishing guarantees.
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(repo_root().join(relative))
        .unwrap_or_else(|_| panic!("read {relative}"))
}

fn cargo_version() -> String {
    let cargo = read("Cargo.toml");
    for line in cargo.lines() {
        if let Some(value) = line.strip_prefix("version = \"") {
            return value.trim_end_matches('"').to_owned();
        }
    }
    panic!("Cargo.toml has a package version");
}

fn product_version() -> String {
    let lib = read("src/lib.rs");
    for line in lib.lines() {
        if let Some(value) = line.split("PRODUCT_VERSION: &str = \"").nth(1) {
            return value.split('"').next().unwrap_or(value).to_owned();
        }
    }
    panic!("src/lib.rs has PRODUCT_VERSION");
}

fn extension_manifest() -> String {
    read("editors/vscode/package.json")
}

#[test]
fn package_and_product_versions_agree() {
    let cargo = cargo_version();
    let product = product_version();
    assert_eq!(cargo, "1.0.3");
    assert_eq!(product, "1.0.3 Beta");
    assert!(
        product.starts_with(&cargo),
        "product version starts with the package version"
    );
    assert!(extension_manifest().contains(&format!("\"version\": \"{cargo}\"")));
}

#[test]
fn cargo_manifest_stays_dependency_free_and_licensed() {
    let cargo = read("Cargo.toml");
    assert!(cargo.contains("license-file = \"LICENSE\""));
    let dependencies = cargo
        .split("[dependencies]")
        .nth(1)
        .expect("dependencies section")
        .split("\n[")
        .next()
        .expect("dependencies body");
    for line in dependencies.lines() {
        let line = line.trim();
        assert!(
            line.is_empty() || line.starts_with('#'),
            "unexpected dependency in the main crate: {line}"
        );
    }
    assert!(read("LICENSE").contains("Luca Code License"));
}

#[test]
fn changelog_tracks_unreleased_and_current_release() {
    let changelog = read("CHANGELOG.md");
    assert!(
        changelog.contains("## Unreleased"),
        "changelog has an Unreleased section"
    );
    assert!(
        changelog.contains("## 1.0.3 Beta"),
        "changelog records 1.0.3 Beta"
    );
    assert!(
        changelog.contains("No version bump yet"),
        "audit work is explicitly unreleased"
    );
}

#[test]
fn internal_engineering_docs_stay_unpublished() {
    let ignore = read(".gitignore");
    for entry in [
        "/AGENTS.md",
        "/OPENCODE_BUILD_PROMPT.md",
        "/docs/contributor/",
    ] {
        assert!(ignore.contains(entry), ".gitignore keeps {entry} local");
    }
    let root = repo_root();
    assert!(
        root.join("AGENTS.md").exists(),
        "internal AGENTS.md is still local"
    );
    assert!(
        root.join("docs/contributor").is_dir(),
        "internal contributor docs are still local"
    );
}

#[test]
fn ci_runs_tests_lints_docs_and_packages_without_publishing() {
    let ci = read(".github/workflows/ci.yml");
    for step in [
        "cargo test",
        "cargo clippy --all-targets --all-features -- -D warnings",
        "cargo doc --no-deps",
        "npm test",
    ] {
        assert!(ci.contains(step), "CI includes: {step}");
    }
    assert!(
        !ci.contains("cargo publish"),
        "CI never publishes the crate"
    );
    assert!(
        !ci.contains("vsce publish"),
        "CI never publishes the extension"
    );
    assert!(
        ci.contains("Nothing here publishes"),
        "CI states its non-publishing boundary"
    );
}

#[test]
fn install_scripts_and_logos_are_present() {
    let root = repo_root();
    for relative in [
        "scripts/install.sh",
        "scripts/install.ps1",
        "logo/luca code logo light.svg",
        "logo/luca code logo dark.svg",
        "logo/luca code icon.svg",
        "editors/vscode/LICENSE",
        "editors/vscode/CHANGELOG.md",
    ] {
        assert!(root.join(relative).exists(), "missing {relative}");
    }
}

#[test]
fn extension_manifest_declares_check_and_run_commands() {
    let manifest = extension_manifest();
    for command in ["luca.runFile", "luca.checkFile", "luca.createFile"] {
        assert!(manifest.contains(command), "manifest declares {command}");
    }
    for field in ["\"bugs\"", "\"homepage\"", "\"license\""] {
        assert!(manifest.contains(field), "manifest includes {field}");
    }
}
