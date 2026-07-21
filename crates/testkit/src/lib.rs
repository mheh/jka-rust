//! Shared harness support for the differential parity tests.
//!
//! These helpers are consumed only under `[dev-dependencies]` — `testkit` is
//! never a runtime dependency of any shipped crate. It hoists the byte-identical
//! oracle-path resolution, golden byte-compare, fixture walking and token
//! parsing that every `*_parity.rs` integration test used to carry its own copy
//! of.
//!
//! Path-independence: helpers that need a workspace-relative location compute it
//! from `testkit`'s own `CARGO_MANIFEST_DIR`; helpers that need a *consumer*
//! crate's location take that crate's manifest dir as a `&str` parameter (the
//! consumer passes `env!("CARGO_MANIFEST_DIR")`, which resolves at its site).

use std::path::{Path, PathBuf};

/// The workspace root — `crates/testkit`'s manifest dir, two parents up.
pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/testkit has two ancestors")
        .to_path_buf()
}

/// A `tools/<tool>` oracle root under the workspace (e.g. `oracle_root("roff-oracle")`).
pub fn oracle_root(tool: &str) -> PathBuf {
    workspace_root().join("tools").join(tool)
}

/// A consumer crate's `tests/oracle` dir. `manifest_dir` is the consumer's
/// `env!("CARGO_MANIFEST_DIR")`.
pub fn oracle_dir(manifest_dir: &str) -> PathBuf {
    PathBuf::from(manifest_dir).join("tests/oracle")
}

/// Byte-compare `got` against the committed golden
/// `<manifest_dir>/tests/oracle/golden/<name>.txt`, panicking with the first
/// differing line (or a length mismatch) on failure. `manifest_dir` is the
/// consumer's `env!("CARGO_MANIFEST_DIR")`.
pub fn compare(manifest_dir: &str, name: &str, got: &str) {
    let golden_path = oracle_dir(manifest_dir)
        .join("golden")
        .join(format!("{name}.txt"));
    let golden = std::fs::read_to_string(&golden_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", golden_path.display()));
    if got == golden {
        return;
    }
    let g: Vec<&str> = golden.lines().collect();
    let o: Vec<&str> = got.lines().collect();
    for (i, (gl, ol)) in g.iter().zip(o.iter()).enumerate() {
        if gl != ol {
            panic!(
                "{name} parity mismatch at line {} (oracle vs port):\n  oracle: {gl}\n  port:   {ol}",
                i + 1
            );
        }
    }
    panic!(
        "{name} parity length mismatch: oracle {} lines, port {} lines",
        g.len(),
        o.len()
    );
}

/// Recursively visit every regular file under `dir`, calling `sink` with the
/// file's `/`-separated path relative to `dir` and its raw bytes. Used to seed
/// a fixture-backed `MockHost` from a committed `fixtures/` tree.
pub fn walk_fixtures(dir: &Path, sink: &mut dyn FnMut(String, Vec<u8>)) {
    walk_fixtures_rel(dir, dir, sink);
}

fn walk_fixtures_rel(root: &Path, dir: &Path, sink: &mut dyn FnMut(String, Vec<u8>)) {
    for entry in std::fs::read_dir(dir).expect("read fixtures dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            walk_fixtures_rel(root, &path, sink);
        } else {
            let rel = path.strip_prefix(root).expect("under fixtures root");
            let key = rel.to_string_lossy().replace('\\', "/");
            let bytes = std::fs::read(&path).expect("read fixture file");
            sink(key, bytes);
        }
    }
}

/// Parse an int token: decimal (possibly negative), or a `0x`/`0X` hex literal.
pub fn pi(t: &str) -> i32 {
    if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        i64::from_str_radix(h, 16).unwrap() as i32
    } else {
        t.parse::<i64>().unwrap() as i32
    }
}

/// Parse a float token: a plain (possibly negative) integer parsed as
/// `(float)atol`, or an `0xXXXXXXXX` f32 bit pattern.
pub fn pf(t: &str) -> f32 {
    if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        f32::from_bits(u32::from_str_radix(h, 16).unwrap())
    } else {
        t.parse::<i64>().unwrap() as f32
    }
}
