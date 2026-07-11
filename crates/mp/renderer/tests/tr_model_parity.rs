//! Differential parity: the Rust `mp_renderer` tr-model loader + cache port must
//! reproduce, byte for byte, the dumps produced by the UNMODIFIED Raven
//! `codemp/renderer/tr_model.cpp` compiled by `tools/trmodel-oracle/build.sh`
//! (goldens under `tools/trmodel-oracle/goldens/`).
//!
//! The four units here mirror `docs/subsystems/tr-model.md` § Verification
//! strategy and the two oracle dumpers exactly:
//! * [`load_matches_oracle_golden`] mirrors `dump_load.cpp` — `ServerLoadMDXM`/
//!   `ServerLoadMDXA` header parse + in-place write-backs, the glm→gla
//!   `animIndex` recursion, the `model_t.mdxm`/`.mdxa` NULL-parity seam, the
//!   version-reject / unknown-ident fail paths (return literal `0`, entry stays
//!   hashed — ruling 53), and `R_GetModelByHandle` out-of-range → `models[0]`.
//! * [`cache_hitmiss_matches_oracle_golden`] mirrors `dump_cache.cpp hitmiss` —
//!   disk miss (FS reads) vs cache hit (0 reads, `pqbAlreadyFound`).
//! * [`cache_evict_matches_oracle_golden`] mirrors `dump_cache.cpp evict` —
//!   level-keyed `RE_RegisterModels_LevelLoadEnd` eviction.
//! * [`cache_dumpnonpure_matches_oracle_golden`] mirrors `dump_cache.cpp
//!   dumpnonpure` — `RE_RegisterModels_DumpNonPure` PAK-checksum eviction (the
//!   1/-1 convention, ruling 59), never `*default.gla`.
//!
//! The `matcomp` unit lives with its port in `mp_engine_ghoul2`
//! (`tests/matcomp_parity.rs`), not here (`TRM-D1`(a)/ruling 56a).
//!
//! Host-taking surfaces are driven through the fixture-backed
//! `mp_host_interface::mock::MockHost`; the loader reaches the FS / pak /
//! cvar / console seams through it exactly as the oracle host does. Fixtures and
//! goldens are read from `tools/trmodel-oracle/` and are never edited.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use core::ffi::c_char;

use mp_host_interface::mock::MockHost;
use mp_qshared::shared::{qhandle_t, ForceReload_e};

use mp_renderer::mdx_format::mdxa_header_t::mdxaHeader_t;
use mp_renderer::mdx_format::mdxm_header_t::mdxmHeader_t;
use mp_renderer::mdx_format::mdxm_lod_t::mdxmLOD_t;
use mp_renderer::mdx_format::mdxm_lodsurf_offset_t::mdxmLODSurfOffset_t;
use mp_renderer::mdx_format::mdxm_surface_t::mdxmSurface_t;
use mp_renderer::tr_local::model_s::model_t;
use mp_renderer::tr_local::modtype_t::modtype_t;
use mp_renderer::tr_local::surface_type_t::surfaceType_t;
use mp_renderer::tr_model::render_models::RenderModels;

/// Repo-relative `tools/trmodel-oracle` root (this crate is `crates/mp/renderer`).
fn oracle_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../tools/trmodel-oracle")
}

fn golden(name: &str) -> String {
    let path = oracle_root().join("goldens").join(name);
    fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!("missing golden {path:?} — run tools/trmodel-oracle/build.sh --regen")
    })
}

/// A `MockHost` seeded with every committed fixture under `fixtures/`, keyed by
/// its qpath (the fixture-relative path, e.g. `fixtures/models/test.glm` →
/// `models/test.glm`) — exactly what the oracle host serves from
/// `fixtures/<qpath>`. `*default.gla` is program-internal (the `FakeGLAFile`
/// intercept) and intentionally has no fixture file.
fn fixture_host() -> MockHost {
    let fixtures = oracle_root().join("fixtures");
    let mut host = MockHost::new();
    seed_dir(&fixtures, &fixtures, &mut host);
    host
}

fn seed_dir(root: &Path, dir: &Path, host: &mut MockHost) {
    for entry in fs::read_dir(dir).expect("fixtures dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            seed_dir(root, &path, host);
        } else {
            let qpath = path
                .strip_prefix(root)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/");
            let bytes = fs::read(&path).expect("read fixture");
            host.files.insert(qpath, bytes);
        }
    }
}

/// `dump_load.cpp`'s `tname` — `modtype_t` → the golden's name string.
fn type_name(t: &modtype_t) -> &'static str {
    match t {
        modtype_t::MOD_BAD => "MOD_BAD",
        modtype_t::MOD_MDXM => "MOD_MDXM",
        modtype_t::MOD_MDXA => "MOD_MDXA",
        _ => "MOD_other",
    }
}

/// Render a NUL-terminated Raven `char[]` field (`model_t.name`,
/// `mdxmHeader_t.animName`, `mdxaHeader_t.name`) as the C `%s` would print it.
fn cstr(bytes: &[c_char]) -> String {
    let raw: Vec<u8> = bytes
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    String::from_utf8_lossy(&raw).into_owned()
}

/// Reconstruct `tr.numModels` from the public API. Raven's `dump_load.cpp` reads
/// the file-static counter directly; the port folds it onto `RenderModels` as a
/// `pub(crate)` field with no public accessor (frozen skeleton — see the crate
/// task report), so this test derives the identical value through the pinned
/// public `R_GetModelByHandle` contract: an in-range handle `k in 1..numModels`
/// returns `models[k]` (whose `->index == k`), while any out-of-range handle
/// falls back to `models[0]` (whose `index == 0`). The first `k` that fails
/// `index == k` is `numModels`. This is byte-identical to the counter (the same
/// `index >= tr.numModels` bound the golden's out-of-range cases pin) — a forced
/// divergence in *acquisition*, not in the asserted output.
fn num_models(rm: &RenderModels) -> i32 {
    let mut n: i32 = 1;
    while n <= 1024 {
        if rm.get_model(n).index == n {
            n += 1;
        } else {
            break;
        }
    }
    n
}

/// `dump_load.cpp`'s `dump_model`: the pool-entry line plus the mdxm/mdxa seam
/// (SET where `model_t.mdxm`/`.mdxa` is non-NULL, else NULL).
fn dump_model(out: &mut String, rm: &RenderModels, tag: &str, h: qhandle_t) {
    let m: &model_t = rm.get_model(h);
    writeln!(
        out,
        "{}: handle={} index={} type={} dataSize={} numLods={} name=\"{}\"",
        tag,
        h,
        m.index,
        type_name(&m.r#type),
        m.dataSize,
        m.numLods,
        cstr(&m.name),
    )
    .unwrap();
    writeln!(
        out,
        "     seam: mdxm={} mdxa={}",
        if m.mdxm.is_null() { "NULL" } else { "SET" },
        if m.mdxa.is_null() { "NULL" } else { "SET" },
    )
    .unwrap();
}

/// Capture one `RE_RegisterModels_Info_f` block: the port prints each cache
/// entry + the running total through `host.print`; concatenating the captured
/// chunks reproduces the exact bytes (including the NDEBUG per-entry run-together
/// with the newline only on the total line).
fn info_block(rm: &RenderModels, host: &mut MockHost) -> String {
    host.prints.clear();
    rm.models_info_f(host);
    host.prints.concat()
}

#[test]
fn load_matches_oracle_golden() {
    let golden = golden("load.txt");
    let mut host = fixture_host();
    let mut rm = RenderModels::default();
    let mut out = String::new();

    rm.model_init();
    writeln!(out, "=== init ===").unwrap();
    writeln!(
        out,
        "numModels={}  models[0].type={}",
        num_models(&rm),
        type_name(&rm.get_model(0).r#type),
    )
    .unwrap();

    writeln!(
        out,
        "\n=== register models/test.glm (recurses skeletons/test.gla) ==="
    )
    .unwrap();
    let hglm = rm.register_server_model(&mut host, "models/test.glm");
    writeln!(
        out,
        "register returned handle={}, numModels={}",
        hglm,
        num_models(&rm)
    )
    .unwrap();
    dump_model(&mut out, &rm, "glm", hglm);

    // mdxmHeader read-back: the loader parsed/wrote these fields in place; the
    // `animIndex` is the glm→gla cross-reference it filled during recursion.
    let hgla = {
        let glm = rm.get_model(hglm);
        // SAFETY: on success `model_t.mdxm` points at the cache entry's
        // 16-byte-aligned model block (owned by `rm.cached`, live for `rm`).
        let mm: &mdxmHeader_t = unsafe { &*glm.mdxm };
        writeln!(
            out,
            "  mdxmHeader: ident=0x{:08x} version={} numBones={} numLODs={} ofsLODs={} \
             numSurfaces={} ofsSurfHierarchy={} ofsEnd={} animIndex={} animName=\"{}\"",
            mm.ident as u32,
            mm.version,
            mm.numBones,
            mm.numLODs,
            mm.ofsLODs,
            mm.numSurfaces,
            mm.ofsSurfHierarchy,
            mm.ofsEnd,
            mm.animIndex,
            cstr(&mm.animName),
        )
        .unwrap();

        // Walk to LOD0 surface 0 exactly as `dump_load.cpp` does:
        //   lod  = (byte*)mm + mm->ofsLODs
        //   surf = (byte*)lod + sizeof(mdxmLOD_t)
        //                     + numSurfaces * sizeof(mdxmLODSurfOffset_t)
        // and prove the intel-live write-back forced `surf->ident = SF_MDX`.
        let base = glm.mdxm as *const u8;
        // SAFETY: offsets are within the parsed `ofsEnd`-sized model block.
        let lod = unsafe { base.add(mm.ofsLODs as usize) } as *const mdxmLOD_t;
        let surf_ptr = unsafe {
            (lod as *const u8).add(
                core::mem::size_of::<mdxmLOD_t>()
                    + mm.numSurfaces as usize * core::mem::size_of::<mdxmLODSurfOffset_t>(),
            )
        } as *const mdxmSurface_t;
        let surf: &mdxmSurface_t = unsafe { &*surf_ptr };
        writeln!(
            out,
            "  LOD0 surf0: ident={} (SF_MDX={}) numVerts={} numTriangles={} ofsEnd={}",
            surf.ident,
            surfaceType_t::SF_MDX as i32,
            surf.numVerts,
            surf.numTriangles,
            surf.ofsEnd,
        )
        .unwrap();

        mm.animIndex
    };

    dump_model(&mut out, &rm, "gla", hgla);
    {
        let gla = rm.get_model(hgla);
        // SAFETY: on success `model_t.mdxa` points at the cache entry's block.
        let ma: &mdxaHeader_t = unsafe { &*gla.mdxa };
        writeln!(
            out,
            "  mdxaHeader: ident=0x{:08x} version={} numFrames={} numBones={} ofsFrames={} \
             ofsEnd={} name=\"{}\"",
            ma.ident as u32,
            ma.version,
            ma.numFrames,
            ma.numBones,
            ma.ofsFrames,
            ma.ofsEnd,
            cstr(&ma.name),
        )
        .unwrap();
    }

    writeln!(
        out,
        "\n=== re-register (hash hit -> same handle, no new model) ==="
    )
    .unwrap();
    let hglm2 = rm.register_server_model(&mut host, "models/test.glm");
    writeln!(
        out,
        "re-register handle={} (was {}), numModels={}",
        hglm2,
        hglm,
        num_models(&rm),
    )
    .unwrap();

    writeln!(out, "\n=== version reject (badversion.glm) ===").unwrap();
    let hbadver = rm.register_server_model(&mut host, "badversion.glm");
    writeln!(
        out,
        "first register returned={}, numModels={}",
        hbadver,
        num_models(&rm)
    )
    .unwrap();
    let hbadver2 = rm.register_server_model(&mut host, "badversion.glm");
    writeln!(
        out,
        "re-register returned={} (MOD_BAD entry stays hashed under its nonzero index)",
        hbadver2,
    )
    .unwrap();

    writeln!(out, "\n=== unknown ident (badident.glm) ===").unwrap();
    let hbadid = rm.register_server_model(&mut host, "badident.glm");
    writeln!(
        out,
        "first register returned={}, numModels={}",
        hbadid,
        num_models(&rm)
    )
    .unwrap();
    let hbadid2 = rm.register_server_model(&mut host, "badident.glm");
    writeln!(out, "re-register returned={}", hbadid2).unwrap();

    writeln!(
        out,
        "\n=== R_GetModelByHandle out-of-range -> models[0] (MOD_BAD) ==="
    )
    .unwrap();
    writeln!(
        out,
        "get(0)={}  get(99999)={}  get(-5)={}",
        type_name(&rm.get_model(0).r#type),
        type_name(&rm.get_model(99999).r#type),
        type_name(&rm.get_model(-5).r#type),
    )
    .unwrap();

    assert_eq!(
        out, golden,
        "load/seam/handle dump diverges from the C++ oracle"
    );
}

#[test]
fn cache_hitmiss_matches_oracle_golden() {
    let golden = golden("cache_hitmiss.txt");
    let mut host = fixture_host();
    let mut rm = RenderModels::default();
    let mut out = String::new();

    rm.model_init();

    host.fs_reads = 0;
    let h1 = rm.register_server_model(&mut host, "models/test.glm");
    writeln!(out, "=== first register (disk miss) ===").unwrap();
    writeln!(out, "handle={}  FS disk reads={}", h1, host.fs_reads).unwrap();
    writeln!(out, "cache after first register:").unwrap();
    out.push_str(&info_block(&rm, &mut host));

    // Drop the model pool + hash but keep `CachedModels`, then re-init the null
    // model — the re-register now hits the cached disk images (0 FS reads).
    rm.hunk_clear();
    rm.model_init();

    host.fs_reads = 0;
    let h2 = rm.register_server_model(&mut host, "models/test.glm");
    writeln!(
        out,
        "\n=== re-register after HunkClear+ModelInit (cache hit) ==="
    )
    .unwrap();
    writeln!(
        out,
        "handle={}  FS disk reads={} (0 == served from CachedModels)",
        h2, host.fs_reads,
    )
    .unwrap();
    writeln!(out, "cache after re-register:").unwrap();
    out.push_str(&info_block(&rm, &mut host));

    assert_eq!(
        out, golden,
        "cache hit/miss dump diverges from the C++ oracle"
    );
}

#[test]
fn cache_evict_matches_oracle_golden() {
    let golden = golden("cache_evict.txt");
    let mut host = fixture_host();
    let mut rm = RenderModels::default();
    let mut out = String::new();

    rm.model_init();
    // Force the pool-megs gate open (the dumper's `r_modelpoolmegs->integer = 0`).
    host.set_cvar("r_modelpoolmegs", "0");

    rm.media_level_load_begin(&mut host, "map1", ForceReload_e::eForceReload_NOTHING); // level -> 1
    rm.register_server_model(&mut host, "models/test.glm"); // stamped lvl 1
    writeln!(
        out,
        "=== after level 1 register (GetLevel={}) ===",
        rm.media_get_level()
    )
    .unwrap();
    out.push_str(&info_block(&rm, &mut host));

    rm.media_level_load_begin(&mut host, "map2", ForceReload_e::eForceReload_NOTHING); // level -> 2
    rm.register_server_model(&mut host, "models/modelb.glm"); // stamped lvl 2
    writeln!(
        out,
        "\n=== after level 2 register (GetLevel={}) ===",
        rm.media_get_level()
    )
    .unwrap();
    out.push_str(&info_block(&rm, &mut host));

    let freed = rm.models_level_load_end(&mut host, false);
    writeln!(
        out,
        "\n=== LevelLoadEnd(qfalse), r_modelpoolmegs=0 -> evict stale ==="
    )
    .unwrap();
    writeln!(out, "freed at least one={}", freed as i32).unwrap();
    writeln!(out, "survivors (sorted):").unwrap();
    out.push_str(&info_block(&rm, &mut host));

    assert_eq!(
        out, golden,
        "cache eviction dump diverges from the C++ oracle"
    );
}

#[test]
fn cache_dumpnonpure_matches_oracle_golden() {
    let golden = golden("cache_dumpnonpure.txt");
    let mut host = fixture_host();
    // test.glm + its gla live in a pure PAK (checksums stamped at register);
    // modelb.glm + its gla are disk-only (stamp -1); *default.gla is program-
    // internal (stamp -1 too, but DumpNonPure must never dump it).
    host.pak_files.insert("models/test.glm".to_string(), 111);
    host.pak_files.insert("skeletons/test.gla".to_string(), 222);

    let mut rm = RenderModels::default();
    let mut out = String::new();

    rm.model_init();
    rm.register_server_model(&mut host, "models/test.glm");
    rm.register_server_model(&mut host, "models/modelb.glm");
    rm.register_server_model(&mut host, "*default.gla");
    writeln!(out, "=== registered (before DumpNonPure) ===").unwrap();
    out.push_str(&info_block(&rm, &mut host));

    host.set_cvar("sv_pure", "1");
    rm.media_level_load_begin(&mut host, "map2", ForceReload_e::eForceReload_NOTHING); // -> DumpNonPure
    writeln!(
        out,
        "\n=== after LevelLoadBegin(sv_pure=1) -> DumpNonPure ==="
    )
    .unwrap();
    writeln!(out, "survivors (pure pak matches + *default.gla, sorted):").unwrap();
    out.push_str(&info_block(&rm, &mut host));

    assert_eq!(out, golden, "DumpNonPure dump diverges from the C++ oracle");
}
