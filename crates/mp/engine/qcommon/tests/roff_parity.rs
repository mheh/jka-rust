//! Differential parity: the Rust `roff` port (`mp_engine_qcommon::roff`) must
//! reproduce, byte for byte, the dumps produced by the UNMODIFIED Raven C++
//! `codemp/qcommon/RoffSystem.cpp` compiled by `tools/roff-oracle/build.sh`
//! (goldens under `tools/roff-oracle/goldens/`, fixtures under
//! `tools/roff-oracle/fixtures/`). See `docs/subsystems/roff.md`
//! § Verification strategy and the two oracle dumpers.
//!
//! The port is driven through its FROZEN public seam — the five arms
//! `cache`/`play`/`update_entities`/`purge_ent`/`clean`
//! (`crates/mp/engine/qcommon/src/roff/roff_system.rs`) — and reaches the world
//! through the fixture-backed [`mp_host_interface::mock::MockHost`] (ruling 32),
//! exactly like the icarus end-to-end unit. The dump helpers mirror
//! `tools/roff-oracle/dump_cache.cpp` / `dump_play.cpp` printf formats character
//! for character; fixtures/goldens are read from `tools/roff-oracle/` and never
//! edited.
//!
//! ## Coverage boundary against the frozen skeleton
//!
//! The oracle dumpers read `theROFFSystem`'s **private** members
//! (`mROFFList` contents, `mROFFEntList.size()`) and call its **debug/query**
//! methods (`GetID`, `List`, `List(id)`, `Unload`). In the ported skeleton
//! those are private (`RoffSystem::{roff_list, roff_ent_list, get_id, unload,
//! list_all, list_one}`), so an *integration* test in `tests/` — a separate
//! crate that sees only `pub` items — cannot dump them. What the public seam +
//! `MockHost` DO reach is asserted here byte-for-byte:
//!   * **Golden A**: the `Cache` return-value / id-minting order (`1,2,3,4,0,0`),
//!     re-cache idempotency, and the two reject `Com_Printf` console lines.
//!   * **Golden B**: the per-frame `SetLerp` writes into the entity's
//!     `s.pos`/`s.apos`/`r` (read back through `MockHost::gentity_mut`) — the
//!     `ApplyROFF`/`ClearLerp` core — plus the `PurgeEnt` return values and its
//!     reject console line.
//!
//! The golden lines that require the private surface (the `mROFFList`/`List`
//! dumps and per-frame `entList=` sizes) and the note-track `VM_Call` text (whose
//! wire encoding is a not-yet-landed body detail) are documented at each site as
//! out of reach for this integration test; see the returned `problems`.

use std::fmt::Write as _;
use std::path::PathBuf;

use mp_engine_qcommon::roff::RoffSystem;
use mp_host_interface::mock::MockHost;

/// Repo-relative `tools/roff-oracle` root (this crate is
/// `crates/mp/engine/qcommon`).
fn oracle_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../tools/roff-oracle")
}

/// Read one committed golden (`goldens/<name>`).
fn read_golden(name: &str) -> String {
    let path = oracle_root().join("goldens").join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("missing golden {path:?} — run tools/roff-oracle/build.sh --regen"))
}

/// A fresh `MockHost` with every fixture registered under the qpath the port's
/// `Cache` requests. `fallbackcase.rof` is registered ONLY under
/// `scripts/fallbackcase.rof` so the bare-name `FS_ReadFile` misses and the
/// `va("scripts/%s.rof", …)` fallback fires (`RoffSystem.cpp` `Cache`); every
/// other fixture is registered under its bare name. The RAW ship-format
/// (4-byte-header) bytes are handed over — the port reads them directly with
/// `i32` headers (ROFF-D4), so no LP64 shim is needed (that shim only exists on
/// the C++ oracle host).
fn host_with_fixtures() -> MockHost {
    let fx = oracle_root().join("fixtures");
    let read = |rel: &str| std::fs::read(fx.join(rel)).unwrap_or_else(|_| panic!("read fixture {rel}"));
    let mut host = MockHost::new();
    host.files.insert("v1_basic.rof".into(), read("v1_basic.rof"));
    host.files.insert("v1_badangle.rof".into(), read("v1_badangle.rof"));
    host.files.insert("v2_notes.rof".into(), read("v2_notes.rof"));
    host.files
        .insert("scripts/fallbackcase.rof".into(), read("scripts/fallbackcase.rof"));
    host.files.insert("bad_version.rof".into(), read("bad_version.rof"));
    host.files.insert("bad_count.rof".into(), read("bad_count.rof"));
    host
}

// ---------------------------------------------------------------------------
// Golden A — parse/cache (reachable subset). Mirrors dump_cache.cpp's
// `=== cache calls ===` block: the `Cache` return + the reject `Com_Printf`s.
// ---------------------------------------------------------------------------

/// Reproduce the reachable half of Golden A: `Cache`'s minted-id order (`GetID`
/// is redundant with the returned id on success), the two `IsROFF`-reject
/// console lines, and re-cache idempotency.
#[test]
fn cache_ids_and_reject_console_match_oracle() {
    let golden = read_golden("cache.txt");
    let mut host = host_with_fixtures();
    let mut roff = RoffSystem::default();

    // Same order dump_cache.cpp's `files[]` uses. `NewID` is `++mID` from 0, so
    // the four valid roffs mint 1,2,3,4 in cache order and the two rejects return
    // 0 (ROFF-D5). `is_client = false` — the DEDICATED live-caller value (ROFF-D3).
    let ids: Vec<i32> = [
        "v1_basic.rof",
        "v1_badangle.rof",
        "v2_notes.rof",
        "fallbackcase.rof",
        "bad_version.rof",
        "bad_count.rof",
    ]
    .iter()
    .map(|f| roff.cache(f, false, &mut host))
    .collect();
    assert_eq!(
        ids,
        vec![1, 2, 3, 4, 0, 0],
        "Cache id-minting / reject order diverges from the C++ oracle"
    );

    // Re-cache idempotency: an already-cached path returns its id, mints nothing.
    assert_eq!(
        roff.cache("v1_basic.rof", false, &mut host),
        1,
        "re-cache of a cached roff must return its existing id"
    );

    // The two reject `Com_Printf` lines (`^1cache failed: roff <%s> …`) are the
    // only fully console-reachable Golden-A lines. Compare `MockHost::prints`
    // against exactly those golden lines.
    let expected_rejects: String = golden
        .lines()
        .filter(|l| l.starts_with("^1cache failed"))
        .map(|l| format!("{l}\n"))
        .collect();
    assert_eq!(
        host.prints.join(""),
        expected_rejects,
        "Cache reject console output diverges from the C++ oracle"
    );
}

// ---------------------------------------------------------------------------
// Golden B — playback trace (reachable subset). Mirrors dump_play.cpp's
// `dump_ent` per frame: the SetLerp writes into s.pos / s.apos and the
// r.mIsRoffing / next_roff_time / currentOrigin row.
// ---------------------------------------------------------------------------

/// Reproduce `dump_play.cpp`'s `dump_ent(entnum)` for one entity: the two
/// trajectory rows (`dump_traj` over `s.pos` and `s.apos`, raw IEEE-754 bits)
/// and the roffing-state row. Read back through `MockHost::gentity_mut` — the
/// same `sharedEntity_t` slot the port's `SV_GentityNum` writes.
fn dump_ent(host: &mut MockHost, entnum: i32, out: &mut String) {
    let e = host.gentity_mut(entnum);

    // `dump_traj("pos", …)`: `%-4s` of "pos" is "pos " → "  pos  type=…".
    let pos = e.s.pos;
    writeln!(
        out,
        "  pos  type={} time={} base=0x{:08x},0x{:08x},0x{:08x} delta=0x{:08x},0x{:08x},0x{:08x}",
        pos.trType as i32,
        pos.trTime,
        pos.trBase[0].to_bits(),
        pos.trBase[1].to_bits(),
        pos.trBase[2].to_bits(),
        pos.trDelta[0].to_bits(),
        pos.trDelta[1].to_bits(),
        pos.trDelta[2].to_bits(),
    )
    .unwrap();

    // `dump_traj("apos", …)`: `%-4s` of "apos" is "apos" → "  apos type=…".
    let apos = e.s.apos;
    writeln!(
        out,
        "  apos type={} time={} base=0x{:08x},0x{:08x},0x{:08x} delta=0x{:08x},0x{:08x},0x{:08x}",
        apos.trType as i32,
        apos.trTime,
        apos.trBase[0].to_bits(),
        apos.trBase[1].to_bits(),
        apos.trBase[2].to_bits(),
        apos.trDelta[0].to_bits(),
        apos.trDelta[1].to_bits(),
        apos.trDelta[2].to_bits(),
    )
    .unwrap();

    writeln!(
        out,
        "  r.mIsRoffing={} next_roff_time={} curOrigin=0x{:08x},0x{:08x},0x{:08x}",
        e.r.mIsRoffing as i32,
        e.next_roff_time,
        e.r.currentOrigin[0].to_bits(),
        e.r.currentOrigin[1].to_bits(),
        e.r.currentOrigin[2].to_bits(),
    )
    .unwrap();
}

/// Mirror `dump_play.cpp`'s `run_playback`, emitting only the reachable
/// `dump_ent` rows (the `- frame … entList=` framing and the `NOTE` rows read
/// the private `mROFFEntList` / note-text wire, so they are omitted here — see
/// the module coverage note). Each scenario starts from a fresh
/// [`RoffSystem::default`] — the owned-state analogue of the oracle harness'
/// `reset()` (`Clean` + drain `mROFFEntList` + `mID = 0`).
///
/// `frame_time` is the driver's known per-format frame step (v1 = 100 ms,
/// v2 = 50 ms); the oracle reads it from the private `mFrameTime`, which this
/// integration test cannot.
#[allow(clippy::too_many_arguments)]
fn run_playback_trace(
    out: &mut String,
    file: &str,
    entnum: i32,
    translate: bool,
    angles: [f32; 3],
    start_time: i32,
    frames: i32,
    frame_time: i32,
) {
    let mut host = host_with_fixtures();
    let mut roff = RoffSystem::default();

    let id = roff.cache(file, false, &mut host);
    host.sv_time = start_time;
    // `host_set_ent_angles`: `s.apos.trBase[PITCH/YAW/ROLL] = pitch/yaw/roll`
    // (`tools/roff-oracle/host.cpp:42-47`). `Play` copies this into `mStartAngles`.
    host.gentity_mut(entnum).s.apos.trBase = angles;

    roff.play(entnum, id, translate, false, &mut host);
    for _ in 0..frames {
        roff.update_entities(false, &mut host);
        dump_ent(&mut host, entnum, out);
        host.sv_time += frame_time;
    }
}

#[test]
fn playback_ent_trace_matches_oracle() {
    let golden = read_golden("play.txt");

    let mut out = String::new();
    // Scenario 1: non-translated v1 playback.
    run_playback_trace(&mut out, "v1_basic.rof", 1, false, [0.0, 0.0, 0.0], 1000, 5, 100);
    // Scenario 2: translated v1 playback (yaw 90) — the `AngleVectors` path.
    run_playback_trace(&mut out, "v1_basic.rof", 2, true, [0.0, 90.0, 0.0], 1000, 5, 100);
    // Scenario 3: v2 note firing (the `NOTE` row itself is omitted — private).
    run_playback_trace(&mut out, "v2_notes.rof", 3, false, [0.0, 0.0, 0.0], 2000, 4, 50);
    // Scenario 4: roff-not-found error path. `Clean(false)` unloads the single
    // cached roff (equivalent to the oracle's `Unload(id)` here) while leaving
    // the ent on the playback list, so `UpdateEntities`' roff lookup misses →
    // the error `Com_Printf` + `mKill` + `ClearLerp` (`TR_STATIONARY`, ROFF).
    {
        let mut host = host_with_fixtures();
        let mut roff = RoffSystem::default();
        let id = roff.cache("v1_basic.rof", false, &mut host);
        host.sv_time = 1000;
        roff.play(4, id, false, false, &mut host);
        roff.clean(false);
        roff.update_entities(false, &mut host);
        dump_ent(&mut host, 4, &mut out);
    }

    // Golden filtered to exactly the `dump_ent` rows (scenarios 1-4; scenario 5
    // emits no ent rows). These three prefixes uniquely select the ent dump.
    let expected: String = golden
        .lines()
        .filter(|l| l.starts_with("  pos ") || l.starts_with("  apos ") || l.starts_with("  r.mIsRoffing"))
        .map(|l| format!("{l}\n"))
        .collect();

    assert_eq!(
        out, expected,
        "playback SetLerp/ClearLerp entity writes diverge from the C++ oracle"
    );
}

#[test]
fn purge_ent_returns_and_console_match_oracle() {
    let golden = read_golden("play.txt");
    let mut host = host_with_fixtures();
    let mut roff = RoffSystem::default();

    // Mirror `run_purge`: two ents playing the same roff, purge one by id
    // (success), then a missing id (fail + reject `Com_Printf`).
    let id = roff.cache("v1_basic.rof", false, &mut host);
    host.sv_time = 1000;
    roff.play(5, id, false, false, &mut host);
    roff.play(6, id, false, false, &mut host);

    assert!(roff.purge_ent(5, false, &mut host), "PurgeEnt(5) must succeed");
    let prints_before = host.prints.len();
    assert!(
        !roff.purge_ent(99, false, &mut host),
        "PurgeEnt(99, missing) must fail"
    );

    // The reject `Com_Printf` line (`^1Purge failed:  Entity <99> not found`).
    let expected_fail = golden
        .lines()
        .find(|l| l.starts_with("^1Purge failed"))
        .map(|l| format!("{l}\n"))
        .expect("play golden carries the PurgeEnt reject line");
    assert_eq!(
        host.prints[prints_before..].join(""),
        expected_fail,
        "PurgeEnt reject console output diverges from the C++ oracle"
    );
}
