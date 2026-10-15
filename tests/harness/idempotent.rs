//! Build twice: byte-identical output (spec/build-invariants.md).

use std::collections::BTreeMap;
use std::path::Path;

use crate::support::{build, content_dir, temp_dir};

fn tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                walk(root, &p, out);
            } else {
                out.insert(
                    p.strip_prefix(root).unwrap().display().to_string(),
                    std::fs::read(&p).unwrap(),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

#[test]
fn idempotent() {
    let (a, b) = (temp_dir("idem-a"), temp_dir("idem-b"));
    build(&content_dir(), &a, &[]).unwrap();
    build(&content_dir(), &b, &[]).unwrap();
    let (ta, tb) = (tree(&a), tree(&b));
    let diff: Vec<&String> = ta
        .keys()
        .chain(tb.keys())
        .filter(|k| ta.get(*k) != tb.get(*k))
        .collect();
    assert!(diff.is_empty(), "build is not idempotent; differing files: {diff:?}");
}
