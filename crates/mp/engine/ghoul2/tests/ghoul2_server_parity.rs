//! Differential parity: the Rust `mp_engine_ghoul2` server-side bone/arena port
//! must reproduce, byte for byte, the dumps produced by the UNMODIFIED Raven
//! C++ ghoul2 TUs compiled by `tools/ghoul2-server-oracle/build.sh` (goldens
//! under `tools/ghoul2-server-oracle/goldens/`).
//!
//! The three covered units mirror `tools/ghoul2-server-oracle/README.md`
//! (`docs/subsystems/ghoul2-server.md` § Verification strategy) and the three
//! oracle dumpers exactly:
//! * [`arena_matches_oracle_golden`] mirrors `dump_arena.cpp` — the
//!   `Ghoul2InfoArray` packed-handle scheme (`G2SV-D6`).
//! * [`bolts_matches_oracle_golden`] mirrors `dump_bolts.cpp` — the bolt-list
//!   add/find/remove/prune bookkeeping (`G2_bolts.cpp`).
//! * [`surfaces_matches_oracle_golden`] mirrors `dump_surfaces.cpp` — the
//!   generated-surface list bookkeeping (`G2_surfaces.cpp`).
//!
//! Goldens/fixtures are read from `tools/ghoul2-server-oracle/` and are never
//! edited; each Rust dump format matches its dumper's `printf`s character for
//! character. The surface unit drives the port's host-consuming
//! `g2_add_surface` through `mp_host_interface::mock::MockHost` (a synthetic
//! in-memory `mdxmHeader_t` with `numLODs=2`, exactly the dumper's tiny
//! `mdxmHeader_t hdr; hdr.numLODs = 2;` — no loader, no disk fixture).

use std::fmt::Write as _;
use std::path::PathBuf;

use mp_engine_ghoul2::bolts::{
    g2_add_bolt_surf_num, g2_find_bolt_bone_num, g2_find_bolt_surface_num, g2_init_bolt_list,
    g2_remove_bolt, g2_remove_redundant_bolts,
};
use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_ghoul2::info_array::{ghoul2_info_array_free, Ghoul2InfoArray};
use mp_engine_ghoul2::shared::cghoul2_info::CGhoul2Info;
use mp_engine_ghoul2::shared::surface_info_t::surfaceInfo_t;
use mp_engine_ghoul2::surfaces::{g2_add_surface, g2_find_override_surface, g2_remove_surface};
use mp_host_interface::mock::MockHost;

/// Repo-relative `tools/ghoul2-server-oracle` root (this crate is
/// `crates/mp/engine/ghoul2`).
fn oracle_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../tools/ghoul2-server-oracle")
}

// ---------------------------------------------------------------------------
// Handle-arithmetic constants the arena dumper hard-codes locally. Private in
// `info_array.rs`, so restated here (same values, same Source) — the dumper
// itself `#define`s them at the top of `dump_arena.cpp`.
// ---------------------------------------------------------------------------

/// Raven `#define G2_MODEL_BITS (10)` — the slot-index field width; a handle's
/// generation is `handle >> G2_MODEL_BITS`.
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:305`
const G2_MODEL_BITS: i32 = 10;

/// Raven `#define MAX_G2_MODELS (1024)` — the arena's fixed slot count; the
/// stale-handle probe is `handle + MAX_G2_MODELS` (next unissued generation).
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:304`
const MAX_G2_MODELS: i32 = 1024;

/// Raven `#define G2_INDEX_MASK (MAX_G2_MODELS - 1)` — extracts the slot index
/// (`handle & G2_INDEX_MASK`).
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:308`
const G2_INDEX_MASK: i32 = MAX_G2_MODELS - 1;

/// Raven `#define G2SURFACEFLAG_GENERATED (0x00000200)` — generated-surface
/// marker on a bolt/surface slot; the bolt dumper passes it to the surface
/// finder and every generated bolt carries it as `surfaceType`.
/// Source: `oracle/codemp/renderer/mdx_format.h:50`
const G2SURFACEFLAG_GENERATED: i32 = 0x0000_0200;

/// `mdxmHeader_t::numLODs` byte offset — where `G2_DecideTraceLod` reads the
/// LOD count the surface unit's clamp keys off (matches `surfaces.rs`'s own
/// `MDXM_OFS_NUM_LODS`).
/// Source: `oracle/codemp/renderer/mdx_format.h:165`
const MDXM_OFS_NUM_LODS: usize = 144;

/// `sizeof(mdxmHeader_t)` — a synthetic header block this wide is more than the
/// surface unit's `decide_trace_lod` ever reads (only `numLODs` at offset 144).
/// Source: `oracle/codemp/ghoul2/G2_surfaces.cpp:118` (`sizeof(mdxmHeader_t)`)
const MDXM_HEADER_SIZE: usize = 164;

// ---------------------------------------------------------------------------
// Unit 1: Ghoul2InfoArray arena (mirrors dump_arena.cpp)
// ---------------------------------------------------------------------------

/// Reproduce `dump_arena.cpp`'s `show`: `%-24s handle=%d idx=%d gen=%d
/// valid=%d`. `idx`/`gen` are the packed-handle bit fields (`h & G2_INDEX_MASK`,
/// `h >> G2_MODEL_BITS`); `valid` is the arena's `IsValid(h)` (`bool` → `%d`).
fn arena_show(out: &mut String, arr: &Ghoul2InfoArray, label: &str, h: i32) {
    writeln!(
        out,
        "{:<24} handle={} idx={} gen={} valid={}",
        label,
        h,
        h & G2_INDEX_MASK,
        h >> G2_MODEL_BITS,
        arr.is_valid(h) as i32,
    )
    .unwrap();
}

#[test]
fn arena_matches_oracle_golden() {
    let golden = std::fs::read_to_string(oracle_root().join("goldens").join("arena.txt"))
        .expect("read arena golden");

    // Raven's `TheGhoul2InfoArray()` singleton is the `Ghoul2System.info_array`
    // field (`G2SV-D5`); `Delete` moved UP to `Ghoul2System::delete` (ruling
    // 29), so the driver threads `&mut Ghoul2System` and reaches `New`/`IsValid`
    // through its `info_array` field.
    let mut g2 = Ghoul2System::default();
    let mut out = String::new();

    writeln!(out, "== fresh new / initial generation ==").unwrap();
    let h0 = g2.info_array.new_handle();
    arena_show(&mut out, &g2.info_array, "new h0", h0);
    let h1 = g2.info_array.new_handle();
    arena_show(&mut out, &g2.info_array, "new h1", h1);
    let h2 = g2.info_array.new_handle();
    arena_show(&mut out, &g2.info_array, "new h2", h2);

    writeln!(out, "\n== IsValid predicate ==").unwrap();
    writeln!(
        out,
        "IsValid(0)       = {}  (null handle -> false)",
        g2.info_array.is_valid(0) as i32
    )
    .unwrap();
    writeln!(
        out,
        "IsValid(h0)      = {}",
        g2.info_array.is_valid(h0) as i32
    )
    .unwrap();
    writeln!(
        out,
        "IsValid(h0|junk) = {}  (stale generation -> false)",
        g2.info_array.is_valid(h0 + MAX_G2_MODELS) as i32
    )
    .unwrap();

    writeln!(out, "\n== delete + LIFO reuse + generation bump ==").unwrap();
    g2.delete(h1); // idx1 -> gen2, front of free list
    arena_show(&mut out, &g2.info_array, "after delete h1", h1); // stale handle now invalid
    let r1 = g2.info_array.new_handle(); // reuse idx1
    arena_show(&mut out, &g2.info_array, "reuse -> new r1", r1);
    g2.delete(r1); // idx1 -> gen3
    let r2 = g2.info_array.new_handle();
    arena_show(&mut out, &g2.info_array, "reuse -> new r2", r2);
    g2.delete(r2); // idx1 -> gen4
    let r3 = g2.info_array.new_handle();
    arena_show(&mut out, &g2.info_array, "reuse -> new r3", r3);

    writeln!(out, "\n== multi-slot free-list ordering ==").unwrap();
    g2.delete(h0); // push_front(idx0)
    g2.delete(h2); // push_front(idx2) -> now ahead of idx0
    let n1 = g2.info_array.new_handle();
    arena_show(&mut out, &g2.info_array, "new after 2 deletes", n1); // expect idx2 (LIFO)
    let n2 = g2.info_array.new_handle();
    arena_show(&mut out, &g2.info_array, "new again", n2); // expect idx0

    ghoul2_info_array_free(&mut g2); // `Ghoul2InfoArray_Free()` — no dump output

    assert_eq!(
        out, golden,
        "arena handle scheme diverges from the C++ oracle"
    );
}

// ---------------------------------------------------------------------------
// Unit 2: bolt list (mirrors dump_bolts.cpp)
// ---------------------------------------------------------------------------

/// A zero-initialized `surfaceInfo_t` — the dumper's `slist.resize(4)`
/// value-initializes four POD entries (all fields `0`). `surfaceInfo_t` derives
/// no `Default`/`Clone`, so the four slots are built explicitly.
fn zero_surface() -> surfaceInfo_t {
    surfaceInfo_t {
        offFlags: 0,
        surface: 0,
        genBarycentricJ: 0.0,
        genBarycentricI: 0.0,
        genPolySurfaceIndex: 0,
        genLod: 0,
    }
}

/// Reproduce `dump_bolts.cpp`'s `dump`: `%-26s size=%d` then, per slot,
/// ` | [%zu] bone=%d surf=%d type=%d used=%d`.
fn dump_bolts(
    out: &mut String,
    label: &str,
    b: &[mp_engine_ghoul2::shared::bolt_info_t::boltInfo_t],
) {
    write!(out, "{:<26} size={}", label, b.len()).unwrap();
    for (i, bolt) in b.iter().enumerate() {
        write!(
            out,
            " | [{}] bone={} surf={} type={} used={}",
            i, bolt.boneNumber, bolt.surfaceNumber, bolt.surfaceType, bolt.boltUsed
        )
        .unwrap();
    }
    out.push('\n');
}

#[test]
fn bolts_matches_oracle_golden() {
    let golden = std::fs::read_to_string(oracle_root().join("goldens").join("bolts.txt"))
        .expect("read bolts golden");

    let mut gh = CGhoul2Info::default();
    gh.valid = true; // Raven `gh.mValid = true` (asserts are NDEBUG no-ops)
    let slist: Vec<surfaceInfo_t> = (0..4).map(|_| zero_surface()).collect(); // slist.resize(4)
    let mut bolts = Vec::new();

    let mut out = String::new();

    writeln!(out, "== add generated-surface bolts ==").unwrap();
    let a0 = g2_add_bolt_surf_num(&gh, &mut bolts, &slist, 2);
    writeln!(out, "add surf 2 -> {a0}").unwrap();
    let a1 = g2_add_bolt_surf_num(&gh, &mut bolts, &slist, 0);
    writeln!(out, "add surf 0 -> {a1}").unwrap();
    dump_bolts(&mut out, "after 2 adds", &bolts);

    writeln!(out, "\n== duplicate add bumps boltUsed ==").unwrap();
    let a2 = g2_add_bolt_surf_num(&gh, &mut bolts, &slist, 2); // existing -> ++used
    writeln!(out, "re-add surf 2 -> {a2}").unwrap();
    dump_bolts(&mut out, "after re-add", &bolts);

    writeln!(out, "\n== add out-of-range surface (>= slist.size) ==").unwrap();
    let a3 = g2_add_bolt_surf_num(&gh, &mut bolts, &slist, 9); // 9 >= 4 -> -1
    writeln!(out, "add surf 9 -> {a3}").unwrap();

    writeln!(out, "\n== finders ==").unwrap();
    writeln!(
        out,
        "find surf 2 (flags=G2SURFACEFLAG_GENERATED) -> {}",
        g2_find_bolt_surface_num(&bolts, 2, G2SURFACEFLAG_GENERATED)
    )
    .unwrap();
    writeln!(
        out,
        "find surf 0 (flags=0) -> {}",
        g2_find_bolt_surface_num(&bolts, 0, 0)
    )
    .unwrap();
    writeln!(
        out,
        "find surf 3 (absent) -> {}",
        g2_find_bolt_surface_num(&bolts, 3, 0)
    )
    .unwrap();
    // The dumper's label reads "find bone -1" but the call passes `0`.
    writeln!(
        out,
        "find bone -1 (none set) -> {}",
        g2_find_bolt_bone_num(&bolts, 0)
    )
    .unwrap();

    writeln!(out, "\n== remove (boltUsed decrement + tail resize) ==").unwrap();
    // surf 2 was added twice (used=2): first remove just decrements.
    let i2 = g2_find_bolt_surface_num(&bolts, 2, G2SURFACEFLAG_GENERATED);
    writeln!(
        out,
        "remove idx {i2} (used 2->1) -> {}",
        g2_remove_bolt(&mut bolts, i2) as i32
    )
    .unwrap();
    dump_bolts(&mut out, "after 1st remove", &bolts);
    writeln!(
        out,
        "remove idx {i2} again (used 1->0, frees slot) -> {}",
        g2_remove_bolt(&mut bolts, i2) as i32
    )
    .unwrap();
    dump_bolts(&mut out, "after 2nd remove", &bolts);

    writeln!(
        out,
        "\n== RemoveRedundantBolts (drop bolts to inactive surfaces) =="
    )
    .unwrap();
    // Re-add two surface bolts, then mark surface 0 inactive.
    g2_add_bolt_surf_num(&gh, &mut bolts, &slist, 0);
    g2_add_bolt_surf_num(&gh, &mut bolts, &slist, 1);
    dump_bolts(&mut out, "before prune", &bolts);
    let active_surfaces = [0, 1, 1, 1]; // surface 0 inactive
    let active_bones = [1, 1, 1, 1];
    g2_remove_redundant_bolts(&mut bolts, &slist, &active_surfaces, &active_bones);
    dump_bolts(&mut out, "after prune (surf0 gone)", &bolts);

    writeln!(out, "\n== Init clears the list ==").unwrap();
    g2_init_bolt_list(&mut bolts);
    dump_bolts(&mut out, "after init", &bolts);

    assert_eq!(out, golden, "bolt list diverges from the C++ oracle");
}

// ---------------------------------------------------------------------------
// Unit 3: generated-surface list (mirrors dump_surfaces.cpp)
// ---------------------------------------------------------------------------

/// Reproduce `dump_surfaces.cpp`'s `dump`: `%-26s size=%d` then, per slot,
/// ` | [%zu] off=%d surf=%d poly=%d lod=%d` (offFlags, surface,
/// genPolySurfaceIndex, genLod).
fn dump_surfaces(out: &mut String, label: &str, s: &[surfaceInfo_t]) {
    write!(out, "{:<26} size={}", label, s.len()).unwrap();
    for (i, e) in s.iter().enumerate() {
        write!(
            out,
            " | [{}] off={} surf={} poly={} lod={}",
            i, e.offFlags, e.surface, e.genPolySurfaceIndex, e.genLod
        )
        .unwrap();
    }
    out.push('\n');
}

#[test]
fn surfaces_matches_oracle_golden() {
    let golden = std::fs::read_to_string(oracle_root().join("goldens").join("surfaces.txt"))
        .expect("read surfaces golden");

    // The dumper's tiny `mdxmHeader_t hdr; hdr.numLODs = 2;` reached over
    // `currentModel->mdxm`: here a synthetic in-memory mdxm block (numLODs=2 at
    // its byte offset) served by `MockHost::model_mdxm` for model handle 1. No
    // loader, no disk fixture — G2_DecideTraceLod's only model-memory touch.
    let mut host = MockHost::new();
    let mut mdxm = vec![0u8; MDXM_HEADER_SIZE];
    mdxm[MDXM_OFS_NUM_LODS..MDXM_OFS_NUM_LODS + 4].copy_from_slice(&2i32.to_ne_bytes());
    // `ofsEnd`(160) sizes `MdxmView::from_block`.
    mdxm[160..164].copy_from_slice(&(MDXM_HEADER_SIZE as i32).to_ne_bytes());
    host.mdxm_blocks.insert(1, mdxm);

    let mut gh = CGhoul2Info::default();
    gh.valid = true; // Raven `gh.mValid = true`
    gh.lod_bias = 0; // Raven `gh.mLodBias = 0`
    gh.model = 1; // resolves `currentModel->mdxm` via `MockHost` handle 1

    let mut out = String::new();

    writeln!(out, "== add generated surfaces ==").unwrap();
    // G2_AddSurface(ghoul2, surfaceNumber, polyNumber, BarycentricI, BarycentricJ, lod)
    let s0 = g2_add_surface(&mut host, &mut gh, 7, 3, 0.25, 0.5, 0);
    writeln!(out, "add (surf=7,poly=3,lod=0) -> {s0}").unwrap();
    let s1 = g2_add_surface(&mut host, &mut gh, 9, 1, 0.1, 0.2, 1);
    writeln!(out, "add (surf=9,poly=1,lod=1) -> {s1}").unwrap();
    dump_surfaces(&mut out, "after 2 adds", &gh.slist);

    writeln!(out, "\n== lod clamp (lod>=numLODs -> numLODs-1) ==").unwrap();
    let s2 = g2_add_surface(&mut host, &mut gh, 4, 4, 0.0, 0.0, 5); // lod=5 -> clamp 1
    writeln!(out, "add (surf=4,poly=4,lod=5) -> {s2} (genLod clamped)").unwrap();
    dump_surfaces(&mut out, "after clamp add", &gh.slist);

    writeln!(
        out,
        "\n== find override surface (matches surface==10000 marker) =="
    )
    .unwrap();
    let f = g2_find_override_surface(10000, &gh.slist).is_some();
    writeln!(
        out,
        "find 10000 -> {} (idx0)",
        if f { "found" } else { "null" }
    )
    .unwrap();
    writeln!(
        out,
        "find 12345 (absent) -> {}",
        if g2_find_override_surface(12345, &gh.slist).is_some() {
            "found"
        } else {
            "null"
        }
    )
    .unwrap();

    writeln!(out, "\n== remove middle then tail (tail resize) ==").unwrap();
    // Remove idx1 (marks surface=-1, no tail resize since idx2 still active).
    writeln!(
        out,
        "remove idx1 -> {}",
        g2_remove_surface(&mut gh.slist, 1) as i32
    )
    .unwrap();
    dump_surfaces(&mut out, "after remove idx1", &gh.slist);
    // Remove idx2 (tail): now idx1,idx2 both -1 -> resize drops both.
    writeln!(
        out,
        "remove idx2 -> {}",
        g2_remove_surface(&mut gh.slist, 2) as i32
    )
    .unwrap();
    dump_surfaces(&mut out, "after remove idx2 (resize)", &gh.slist);

    writeln!(out, "\n== add reuses the freed (-1) slot before growing ==").unwrap();
    let s3 = g2_add_surface(&mut host, &mut gh, 2, 2, 0.0, 0.0, 0);
    writeln!(out, "add (surf=2,poly=2) -> {s3} (reused freed slot)").unwrap();
    dump_surfaces(&mut out, "after reuse add", &gh.slist);

    assert_eq!(
        out, golden,
        "generated-surface list diverges from the C++ oracle"
    );
}
