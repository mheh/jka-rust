//! Differential parity: the Rust GP2 port must reproduce, byte for byte, the
//! dumps produced by the UNMODIFIED Raven C++ GP2 compiled by
//! `tools/gp2-oracle/run.sh` (goldens under `tools/gp2-oracle/golden/`).
//!
//! The dump format mirrors `tools/gp2-oracle/main.cpp` exactly.

use mp_engine_qcommon::gp2::generic_parser2::GenericParser2;
use mp_engine_qcommon::gp2::gp_group::GpGroup;
use std::fmt::Write as _;
use std::path::PathBuf;

const FIXED_PROBES: [&str; 4] = ["zzz_missing", "name||count", "zzz_missing||name", "NAME"];

fn dump_group(group: GpGroup, depth: usize, out: &mut String) {
    writeln!(out, "G {} |{}|", depth, group.name()).unwrap();

    for pair in group.pairs() {
        write!(out, "P {} |{}|", depth, pair.name()).unwrap();
        for value in pair.values() {
            write!(out, "{value}|").unwrap();
        }
        out.push('\n');
    }

    for pair in group.pairs() {
        let v = group.find_pair_value(pair.name()).unwrap_or("<DEF>");
        writeln!(out, "F {} |{}|{}|", depth, pair.name(), v).unwrap();
    }
    for key in FIXED_PROBES {
        let v = group.find_pair_value(key).unwrap_or("<DEF>");
        writeln!(out, "F {depth} |{key}|{v}|").unwrap();
    }

    for sub in group.subgroups() {
        dump_group(sub, depth + 1, out);
    }
}

fn dump_in_order(group: GpGroup, depth: usize, out: &mut String) {
    writeln!(out, "IG {} |{}|", depth, group.name()).unwrap();
    for pair in group.pairs_in_order() {
        writeln!(out, "IP {} |{}|", depth, pair.name()).unwrap();
    }
    for sub in group.subgroups_in_order() {
        dump_in_order(sub, depth + 1, out);
    }
}

fn dump(text: &str) -> String {
    let mut parser = GenericParser2::new();
    let ok = parser.parse(text, true).is_ok();

    let mut out = String::new();
    writeln!(out, "== parse {} ==", if ok { "ok" } else { "error" }).unwrap();
    dump_group(parser.top_level(), 0, &mut out);
    out.push_str("== inorder ==\n");
    dump_in_order(parser.top_level(), 0, &mut out);
    out.push_str("== write ==\n");
    out.push_str(&parser.write());
    out.push_str("== end ==\n");
    out
}

#[test]
fn matches_oracle_goldens() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../tools/gp2-oracle");
    let mut checked = 0;

    for entry in std::fs::read_dir(root.join("fixtures")).expect("fixtures dir") {
        let fixture = entry.expect("dir entry").path();
        if fixture.extension().and_then(|e| e.to_str()) != Some("gp2") {
            continue;
        }
        let name = fixture.file_stem().unwrap().to_str().unwrap().to_string();
        let text = std::fs::read_to_string(&fixture).expect("read fixture");
        let golden_path = root.join("golden").join(format!("{name}.mp.txt"));
        let golden = std::fs::read_to_string(&golden_path)
            .unwrap_or_else(|_| panic!("missing golden {golden_path:?} — run tools/gp2-oracle/run.sh --regen"));

        assert_eq!(dump(&text), golden, "fixture {name} diverges from the MP C++ oracle");
        checked += 1;
    }

    assert!(checked >= 8, "expected the full fixture set, found {checked}");
}
