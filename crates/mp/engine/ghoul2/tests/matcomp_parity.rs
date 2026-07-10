//! Differential parity: the Rust `mp_engine_ghoul2` `matcomp` port must
//! reproduce, byte for byte, the dump produced by the UNMODIFIED Raven
//! `codemp/renderer/matcomp.c` compiled by `tools/trmodel-oracle/build.sh`
//! (golden `tools/trmodel-oracle/goldens/matcomp.txt`).
//!
//! This is the tr-model subsystem's `matcomp` unit (`docs/subsystems/tr-model.md`
//! § Verification strategy, "MC_UnCompressQuat goldens"): matcomp lives in
//! `mp_engine_ghoul2`, not `mp_renderer` (`TRM-D1`(a)/ruling 56a), so its parity
//! test lands here beside it, while the loader/cache parity lives in
//! `mp_renderer` (`tests/tr_model_parity.rs`).
//!
//! The dump format mirrors `tools/trmodel-oracle/dump_matcomp.cpp` exactly:
//! floats are emitted as raw IEEE-754 hex bits (`bits(f)` = `f32::to_bits`) plus
//! a `%.6f` decimal rendering, both for bit-exact, deterministic parity. The
//! golden is committed, so no C++ toolchain is needed here.

use std::fmt::Write as _;
use std::path::PathBuf;

use mp_engine_ghoul2::matcomp::{mc_compress, mc_uncompress, mc_uncompress_quat};

/// Repo-relative `tools/trmodel-oracle` root (this crate is
/// `crates/mp/engine/ghoul2`).
fn oracle_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../tools/trmodel-oracle")
}

/// Reproduce `dump_matcomp.cpp`'s `dump_mat`: the label line, then three rows of
/// `  [%08x %08x %08x %08x]  (%.6f %.6f %.6f %.6f)`.
fn dump_mat(out: &mut String, label: &str, mat: &[[f32; 4]; 3]) {
    writeln!(out, "{label}").unwrap();
    for row in mat {
        writeln!(
            out,
            "  [{:08x} {:08x} {:08x} {:08x}]  ({:.6} {:.6} {:.6} {:.6})",
            row[0].to_bits(),
            row[1].to_bits(),
            row[2].to_bits(),
            row[3].to_bits(),
            row[0],
            row[1],
            row[2],
            row[3],
        )
        .unwrap();
    }
}

/// Pack the leading `n` `u16`s of `vals` little-endian into a fresh zeroed
/// 24-byte `comp` buffer — the dumper's `memset(comp,0,24); memcpy(comp, in, …)`.
fn packed_comp(vals: &[u16]) -> [u8; 24] {
    let mut comp = [0u8; 24];
    for (i, v) in vals.iter().enumerate() {
        let le = v.to_le_bytes();
        comp[i * 2] = le[0];
        comp[i * 2 + 1] = le[1];
    }
    comp
}

#[test]
fn matcomp_matches_oracle_golden() {
    let golden_path = oracle_root().join("goldens").join("matcomp.txt");
    let golden = std::fs::read_to_string(&golden_path).unwrap_or_else(|_| {
        panic!("missing golden {golden_path:?} — run tools/trmodel-oracle/build.sh --regen")
    });

    let mut out = String::new();

    // --- MC_UnCompressQuat: 7 x u16 (quat wxyz + 3 xlat) over a spread ---------
    writeln!(out, "=== MC_UnCompressQuat (7 x u16: quat wxyz + xlat) ===").unwrap();
    // Mirrors `dump_matcomp.cpp`'s `inputs[][7]` literals exactly.
    let inputs: [[u16; 7]; 5] = [
        [16383 * 2, 16383 * 2, 16383 * 2, 16383 * 2, 512 * 64, 512 * 64, 512 * 64],
        [16383 * 3, 16383 * 2, 16383 * 2, 16383 * 2, 0, 0, 0],
        [20000, 10000, 30000, 5000, 40000, 20000, 1000],
        [0, 0, 0, 0, 65535, 0, 32768],
        [65535, 65535, 65535, 65535, 1, 2, 3],
    ];
    for (i, input) in inputs.iter().enumerate() {
        let comp = packed_comp(input);
        let mut mat = [[0.0f32; 4]; 3];
        mc_uncompress_quat(&mut mat, &comp);
        dump_mat(&mut out, &format!("quat[{i}]"), &mat);
    }

    // --- MC_Compress -> MC_UnCompress round-trip -------------------------------
    writeln!(out, "\n=== MC_Compress -> MC_UnCompress round-trip ===").unwrap();
    // Mirrors `dump_matcomp.cpp`'s `mats[][3][4]` literals exactly.
    let mats: [[[f32; 4]; 3]; 2] = [
        [[1.0, 0.0, 0.0, 10.0], [0.0, 1.0, 0.0, -20.0], [0.0, 0.0, 1.0, 30.0]],
        [
            [0.5, -0.5, 0.25, 100.0],
            [-1.0, 1.0, -1.0, -100.0],
            [0.1, 0.2, 0.3, 0.0],
        ],
    ];
    for (i, m) in mats.iter().enumerate() {
        let mut comp = [0u8; 24];
        mc_compress(m, &mut comp);
        // `printf("comp[%zu]:", i)` then a leading-space `%02x` per byte.
        write!(out, "comp[{i}]:").unwrap();
        for b in comp.iter() {
            write!(out, " {b:02x}").unwrap();
        }
        out.push('\n');

        let mut mat = [[0.0f32; 4]; 3];
        mc_uncompress(&mut mat, &comp);
        dump_mat(&mut out, &format!("uncompress[{i}]"), &mat);
    }

    assert_eq!(out, golden, "matcomp dump diverges from the C++ oracle");
}
