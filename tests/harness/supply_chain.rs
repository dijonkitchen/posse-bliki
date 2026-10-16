//! Supply chain (spec/build-invariants.md, ADR 2026-10-06-rust-build-zero-dependencies, ADR 2026-10-14-supply-chain-pinned-actions): no crates,
//! every GitHub Action pinned to a full commit SHA, and no `-latest` runners.

use std::path::Path;

fn repo() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn cargo_lock_lists_only_this_package() {
    let lock = std::fs::read_to_string(repo().join("Cargo.lock")).unwrap();
    let packages = lock.lines().filter(|l| l.trim() == "[[package]]").count();
    assert_eq!(
        packages, 1,
        "Cargo.lock lists a dependency; ADR 2026-10-06-rust-build-zero-dependencies needs a new ADR first"
    );
}

#[test]
fn actions_are_pinned_to_commit_shas() {
    let mut unpinned = Vec::new();
    let dir = repo().join(".github/workflows");
    let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).collect();
    files.sort();
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap();
        for (n, line) in text.lines().enumerate() {
            let line = line.trim_start().trim_start_matches("- ");
            let Some(spec) = line.strip_prefix("uses:") else {
                continue;
            };
            let spec = spec.split('#').next().unwrap().trim();
            // Local actions (./path) live in this repository.
            if spec.starts_with("./") {
                continue;
            }
            let sha = spec.rsplit_once('@').map(|(_, r)| r).unwrap_or("");
            if sha.len() != 40 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
                unpinned.push(format!("{}:{}: {spec}", file.display(), n + 1));
            }
        }
    }
    assert!(
        unpinned.is_empty(),
        "pin these actions to a full commit SHA, with the version in a comment:\n{}",
        unpinned.join("\n")
    );
}

#[test]
fn runners_name_an_os_release() {
    let mut moving = Vec::new();
    let dir = repo().join(".github/workflows");
    let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).collect();
    files.sort();
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap();
        for (n, line) in text.lines().enumerate() {
            if let Some(label) = line.trim_start().strip_prefix("runs-on:") {
                if label.trim().ends_with("-latest") {
                    moving.push(format!("{}:{}: {}", file.display(), n + 1, label.trim()));
                }
            }
        }
    }
    assert!(
        moving.is_empty(),
        "name an OS release (e.g. ubuntu-24.04), not a -latest label:\n{}",
        moving.join("\n")
    );
}
