//! Stage-R referee: the differential driver.
//!
//! Replays identical `usercmd_t` streams through Raven's UNMODIFIED jampgame
//! (the oracle dylib) and through our Rust `jampgame` cdylib under ONE
//! deterministic mock engine, then byte-diffs every `playerState_t` (per
//! connected client) and `entityState_t` (per active entity) at every frame.
//! The first divergent field at the first divergent frame is a named,
//! bisectable bug — this test is a FINDING generator, so a divergence is the
//! expected, valuable outcome; the comparison is never weakened to make it pass,
//! and `mp_game` is never touched here to chase parity.
//!
//! # Prerequisites (why this is `#[ignore]`d)
//! Two artifacts must exist before the driver runs:
//!   1. `tools/referee-oracle/build.sh` → `build/liboraclejampgame.dylib`
//!      (Raven's unmodified game module; needs Homebrew `gcc`).
//!   2. `cargo build --workspace` → `libjampgame.dylib` (our cdylib).
//! Run it explicitly:
//! ```sh
//! tools/referee-oracle/build.sh
//! cargo build --workspace
//! cargo test -p jampgame --test referee -- --ignored --test-threads=1 --nocapture
//! ```
//! The `reflog_roundtrip` test below is NOT ignored (no C++ toolchain needed).
//!
//! # Mock determinism audit (the harness must be a pure function of inputs)
//! `tests/common/mod.rs`'s mock was audited for nondeterminism:
//!   * cvar / configstring / userinfo tables are `BTreeMap` (ordered iteration).
//!   * no wall-clock, no `HashMap`, no RNG, no address-dependent replies.
//!   * `G_GET_USERCMD` is a pure function of the injected replay input.
//!   * string-bearing syscalls (`SET_CONFIGSTRING`/`SEND_SERVER_COMMAND`/
//!     `SEND_CONSOLE_COMMAND`/`PRINT`) are captured by their pointed-to DATA, so
//!     the two runs' differing heap addresses never enter the syscall digest.
//! The ONE call-count-coupled reply is `G_MILLISECONDS` (a monotonic counter):
//! it feeds profiling/timing paths, not snapshot state. If a module stored a
//! `trap_Milliseconds` value into `ps`/`es` AND the two modules called it a
//! different number of times, that would surface as a (legitimate) divergence.
//!
//! # RNG seeding verdict (see the report accompanying this file)
//! `Q_irand`/`Q_flrand`/`flrand`/`irand` (the fork-3 `holdrand` LCG) are bit
//! identical on both sides: compile-time `holdrand = 0x89abcdef`, never reseeded
//! by `srand` in either tree — so that stream matches. BUT the oracle's
//! `G_InitGame` calls `srand(randomSeed)` and its `random()`/`crandom()`/bare
//! `rand()` macros route through **libc rand** in this native build (`bg_lib.c`'s
//! own `rand`/`srand` compile only under `Q3_VM`). Our port routes the same
//! macros through `bg_lib.c`'s 69069-LCG (`BgState::rng.randSeed`) seeded by the
//! same `randomSeed`. Those two streams DIVERGE — any `ps`/`es` field fed by
//! `random()`/`crandom()` is expected to diverge. We REPORT this; we do not hack
//! the port.

#![allow(non_snake_case)]

mod common;

use std::path::{Path, PathBuf};

use common::reflog::{self, Scenario};
use common::{
    referee_arm, referee_begin_frame, referee_error, referee_frame_syscall_digest,
    referee_frame_syscalls, referee_import_name, referee_load, referee_locate, referee_reset,
    referee_set_cvar, referee_set_map, referee_set_userinfo, referee_set_usercmd, referee_vm_call,
    run_on_engine_thread_fn, LocateData,
};

use mp_abi::game::exports::MpGameExport;
use mp_qshared::common::mp::gentity::gentity_t;
use mp_qshared::common::mp::qcommon::player_state::forcedata_t;
use mp_qshared::common::mp::qcommon::{entityState_t, playerState_t};
use mp_qshared::shared::limits::MAX_GENTITIES;
use mp_qshared::shared::trajectory::trajectory_t;

// ===========================================================================
// Field-name lookup tables — exhaustive (offset, name) over each snapshot type,
// built with `core::mem::offset_of!` so they are checked against the real
// #[repr(C)] layout at compile time. `fd`/`pos`/`apos` are single entries here;
// a byte landing inside them recurses into the nested table below.
// ===========================================================================

/// `&[(offset, name)]` in declaration (ascending offset) order.
macro_rules! field_offsets {
    ($ty:ty; $($f:ident),+ $(,)?) => {
        &[ $( (core::mem::offset_of!($ty, $f), stringify!($f)) ),+ ]
    };
}

struct FieldTable {
    kind: &'static str,
    fields: &'static [(usize, &'static str)],
    /// Total `size_of` — pins the table's owning type at compile time and bounds
    /// the last field's byte range; kept even though lookups walk `fields`.
    #[allow(dead_code)]
    size: usize,
}

const PS: FieldTable = FieldTable {
    kind: "playerState_t",
    size: core::mem::size_of::<playerState_t>(),
    fields: field_offsets!(playerState_t;
        commandTime, pm_type, bobCycle, pm_flags, pm_time, origin, velocity, moveDir,
        weaponTime, weaponChargeTime, weaponChargeSubtractTime, gravity, speed, basespeed,
        delta_angles, slopeRecalcTime, useTime, groundEntityNum, legsTimer, legsAnim, torsoTimer,
        torsoAnim, legsFlip, torsoFlip, movementDir, eFlags, eFlags2, eventSequence, events,
        eventParms, externalEvent, externalEventParm, externalEventTime, clientNum, weapon,
        weaponstate, viewangles, viewheight, damageEvent, damageYaw, damagePitch, damageCount,
        damageType, painTime, painDirection, yawAngle, yawing, pitchAngle, pitching, stats,
        persistant, powerups, ammo, generic1, loopSound, jumppad_ent, ping, pmove_framecount,
        jumppad_frame, entityEventSequence, lastOnGround, saberInFlight, saberMove, saberBlocking,
        saberBlocked, saberLockTime, saberLockEnemy, saberLockFrame, saberLockHits,
        saberLockHitCheckTime, saberLockHitIncrementTime, saberLockAdvance, saberEntityNum,
        saberEntityDist, saberEntityState, saberThrowDelay, saberCanThrow, saberDidThrowTime,
        saberDamageDebounceTime, saberHitWallSoundDebounceTime, saberEventFlags, rocketLockIndex,
        rocketLastValidTime, rocketLockTime, rocketTargetTime, emplacedIndex, emplacedTime,
        isJediMaster, forceRestricted, trueJedi, trueNonJedi, saberIndex, genericEnemyIndex,
        droneFireTime, droneExistTime, activeForcePass, hasDetPackPlanted, holocronsCarried,
        holocronCantTouch, holocronCantTouchTime, holocronBits, electrifyTime, saberAttackSequence,
        saberIdleWound, saberAttackWound, saberBlockTime, otherKiller, otherKillerTime,
        otherKillerDebounceTime, fd, forceJumpFlip, forceHandExtend, forceHandExtendTime,
        forceRageDrainTime, forceDodgeAnim, quickerGetup, groundTime, footstepTime, otherSoundTime,
        otherSoundLen, forceGripMoveInterval, forceGripChangeMovetype, forceKickFlip, duelIndex,
        duelTime, duelInProgress, saberAttackChainCount, saberHolstered, forceAllowDeactivateTime,
        zoomMode, zoomTime, zoomLocked, zoomFov, zoomLockTime, fallingToDeath, useDelay, inAirAnim,
        lastHitLoc, heldByClient, ragAttach, iModelScale, brokenLimbs, hasLookTarget, lookTarget,
        customRGBA, standheight, crouchheight, m_iVehicleNum, vehOrientation, vehBoarding,
        vehSurfaces, vehTurnaroundIndex, vehTurnaroundTime, vehWeaponsLinked, hyperSpaceTime,
        hyperSpaceAngles, hackingTime, hackingBaseTime, jetpackFuel, cloakFuel, userInt1, userInt2,
        userInt3, userFloat1, userFloat2, userFloat3, userVec1, userVec2,
    ),
};

const FD: FieldTable = FieldTable {
    kind: "forcedata_t",
    size: core::mem::size_of::<forcedata_t>(),
    fields: field_offsets!(forcedata_t;
        forcePowerDebounce, forcePowersKnown, forcePowersActive, forcePowerSelected,
        forceButtonNeedRelease, forcePowerDuration, forcePower, forcePowerMax,
        forcePowerRegenDebounceTime, forcePowerLevel, forcePowerBaseLevel, forceUsingAdded,
        forceJumpZStart, forceJumpCharge, forceJumpSound, forceJumpAddTime, forceGripEntityNum,
        forceGripDamageDebounceTime, forceGripBeingGripped, forceGripCripple, forceGripUseTime,
        forceGripSoundTime, forceGripStarted, forceHealTime, forceHealAmount,
        forceMindtrickTargetIndex, forceMindtrickTargetIndex2, forceMindtrickTargetIndex3,
        forceMindtrickTargetIndex4, forceRageRecoveryTime, forceDrainEntNum, forceDrainTime,
        forceDoInit, forceSide, forceRank, forceDeactivateAll, killSoundEntIndex, sentryDeployed,
        saberAnimLevelBase, saberAnimLevel, saberDrawAnimLevel, suicides, privateDuelTime,
    ),
};

const ES: FieldTable = FieldTable {
    kind: "entityState_t",
    size: core::mem::size_of::<entityState_t>(),
    fields: field_offsets!(entityState_t;
        number, eType, eFlags, eFlags2, pos, apos, time, time2, origin, origin2, angles, angles2,
        bolt1, bolt2, trickedentindex, trickedentindex2, trickedentindex3, trickedentindex4, speed,
        fireflag, genericenemyindex, activeForcePass, emplacedOwner, otherEntityNum, otherEntityNum2,
        groundEntityNum, constantLight, loopSound, loopIsSoundset, soundSetIndex, modelGhoul2,
        g2radius, modelindex, modelindex2, clientNum, frame, saberInFlight, saberEntityNum,
        saberMove, forcePowersActive, saberHolstered, isJediMaster, isPortalEnt, solid, event,
        eventParm, owner, teamowner, shouldtarget, powerups, weapon, legsAnim, torsoAnim, legsFlip,
        torsoFlip, forceFrame, generic1, heldByClient, ragAttach, iModelScale, brokenLimbs,
        boltToPlayer, hasLookTarget, lookTarget, customRGBA, health, maxhealth, npcSaber1, npcSaber2,
        csSounds_Std, csSounds_Combat, csSounds_Extra, csSounds_Jedi, surfacesOn, surfacesOff,
        boneIndex1, boneIndex2, boneIndex3, boneIndex4, boneOrient, boneAngles1, boneAngles2,
        boneAngles3, boneAngles4, NPC_class, m_iVehicleNum, userInt1, userInt2, userInt3, userFloat1,
        userFloat2, userFloat3, userVec1, userVec2,
    ),
};

const TR: FieldTable = FieldTable {
    kind: "trajectory_t",
    size: core::mem::size_of::<trajectory_t>(),
    fields: field_offsets!(trajectory_t; trType, trTime, trDuration, trBase, trDelta),
};

/// The field whose byte range contains `off` (last field with offset <= off).
fn field_at(t: &FieldTable, off: usize) -> (usize, &'static str) {
    let mut best = t.fields[0];
    for &e in t.fields {
        if e.0 <= off {
            best = e;
        } else {
            break;
        }
    }
    best
}

/// `"field+N"` (or `"parent.sub+N"` when the byte lands inside a nested struct).
fn describe(t: &FieldTable, off: usize) -> String {
    let (start, name) = field_at(t, off);
    let inner = off - start;
    let nested = match (t.kind, name) {
        ("playerState_t", "fd") => Some(&FD),
        ("entityState_t", "pos") | ("entityState_t", "apos") => Some(&TR),
        _ => None,
    };
    if let Some(sub) = nested {
        let (ss, sn) = field_at(sub, inner);
        format!("{name}.{sn}+{}", inner - ss)
    } else {
        format!("{name}+{inner}")
    }
}

// ===========================================================================
// Per-frame snapshot
// ===========================================================================

struct FrameSnap {
    level_time: i32,
    /// Raw `playerState_t` bytes per connected client (0..clients).
    clients: Vec<Vec<u8>>,
    /// Sorted active (inuse) entity slot indices.
    inuse: Vec<u32>,
    /// Raw `entityState_t` bytes keyed by slot, for inuse slots only.
    ents: Vec<(u32, Vec<u8>)>,
    /// Syscall stream issued this frame (pointer-free): import numbers + texts.
    sc_imports: Vec<isize>,
    sc_texts: Vec<(isize, i32, String)>,
    sc_digest: u64,
}

impl FrameSnap {
    fn ent(&self, slot: u32) -> Option<&[u8]> {
        self.ents
            .iter()
            .find(|(s, _)| *s == slot)
            .map(|(_, b)| b.as_slice())
    }
}

unsafe fn read_bytes(ptr: *const u8, len: usize) -> Vec<u8> {
    core::slice::from_raw_parts(ptr, len).to_vec()
}

fn snapshot(locate: &LocateData, num_clients: i32, level_time: i32) -> FrameSnap {
    let ent_base = locate.g_ents as *const u8;
    let ent_stride = locate.sizeof_g_entity_t as usize;
    let cl_base = locate.clients as *const u8;
    let cl_stride = locate.sizeof_g_client as usize;
    let inuse_off = core::mem::offset_of!(gentity_t, inuse);
    let ps_size = core::mem::size_of::<playerState_t>();
    let es_size = core::mem::size_of::<entityState_t>();

    // Clients: playerState_t is at offset 0 of gclient_s (server pins it first).
    let mut clients = Vec::with_capacity(num_clients as usize);
    for c in 0..num_clients as usize {
        let p = unsafe { cl_base.add(c * cl_stride) };
        clients.push(unsafe { read_bytes(p, ps_size) });
    }

    // Entities: entityState_t `s` is at offset 0 of gentity_t; iterate the whole
    // physical g_entities[MAX_GENTITIES] array, snapshotting only inuse slots.
    let mut inuse = Vec::new();
    let mut ents = Vec::new();
    for slot in 0..MAX_GENTITIES {
        let base = unsafe { ent_base.add(slot * ent_stride) };
        let flag = unsafe { *(base.add(inuse_off) as *const i32) };
        if flag != 0 {
            inuse.push(slot as u32);
            ents.push((slot as u32, unsafe { read_bytes(base, es_size) }));
        }
    }

    let (sc_imports, sc_texts_raw) = referee_frame_syscalls();
    let sc_texts = sc_texts_raw
        .into_iter()
        .map(|(i, a, s)| (i, a as i32, s))
        .collect();
    FrameSnap {
        level_time,
        clients,
        inuse,
        ents,
        sc_imports,
        sc_texts,
        sc_digest: referee_frame_syscall_digest(),
    }
}

// ===========================================================================
// Scenario drive (one module, sequential in one process)
// ===========================================================================

/// The map-entity token stream. One variant: a worldspawn + three spaced FFA
/// spawn points. With <=3 spawns, `SelectRandomFurthestSpawnPoint`'s
/// `rnd = random()*(numSpots/2)` is 0 regardless of the RNG value, so the spawn
/// choice is deterministic even though the two runs' `random()` streams differ
/// (see the RNG verdict) — divergence, if any, comes from gameplay, not spawn
/// roulette.
fn map_tokens(_map: &str) -> Vec<&'static str> {
    vec![
        "{", "classname", "worldspawn", "}", //
        "{", "classname", "info_player_deathmatch", "origin", "0 0 50", "angle", "0", "}", //
        "{", "classname", "info_player_deathmatch", "origin", "384 0 50", "angle", "180", "}", //
        "{", "classname", "info_player_deathmatch", "origin", "0 384 50", "angle", "90", "}",
    ]
}

fn drive(dylib: &Path, sc: &Scenario) -> Vec<FrameSnap> {
    referee_reset();
    // Deterministic offline replay: synchronous clients so G_RunClient runs
    // ClientThink_real from the latched usercmd every frame (async engine
    // ClientThink dispatch does not exist in this harness). Both runs identical.
    referee_set_cvar("g_synchronousClients", "1");
    referee_set_map(&map_tokens(&sc.map));
    for (&num, ui) in &sc.userinfos {
        referee_set_userinfo(num, ui);
    }

    referee_arm();
    let module = referee_load(dylib);
    let vm = module.entry();

    let init = referee_vm_call(vm, MpGameExport::GAME_INIT, &[sc.starttime as isize, sc.seed as isize, 0]);
    assert_eq!(init, 0, "GAME_INIT returned {init}");
    assert!(referee_error().is_none(), "GAME_INIT G_ERROR: {:?}", referee_error());
    let locate = referee_locate().expect("GAME_INIT must call G_LOCATE_GAME_DATA");

    // Warm-up frames before wiring clients (mirrors g_main.c:2949 "3 game frames
    // before Connect").
    let mut t = sc.starttime;
    for _ in 0..3 {
        t += sc.msec;
        let r = referee_vm_call(vm, MpGameExport::GAME_RUN_FRAME, &[t as isize]);
        assert_eq!(r, 0, "warm-up GAME_RUN_FRAME returned {r}");
    }

    for c in 0..sc.clients {
        let cr = referee_vm_call(vm, MpGameExport::GAME_CLIENT_CONNECT, &[c as isize, 1, 0]);
        assert_eq!(cr, 0, "GAME_CLIENT_CONNECT({c}) rejected (ret {cr})");
        let br = referee_vm_call(vm, MpGameExport::GAME_CLIENT_BEGIN, &[c as isize]);
        assert_eq!(br, 0, "GAME_CLIENT_BEGIN({c}) returned {br}");
        assert!(referee_error().is_none(), "client {c} begin G_ERROR: {:?}", referee_error());
    }

    let mut snaps = Vec::with_capacity(sc.frames as usize);
    for f in 0..sc.frames {
        t += sc.msec;
        // Inject this frame's replay input for every client (G_RunClient
        // overwrites serverTime with level.time, per the format contract).
        for c in 0..sc.clients {
            referee_set_usercmd(c, sc.cmd(f, c).to_usercmd(t));
        }
        referee_begin_frame();
        // Latch each client's cmd (trap_GetUsercmd inside ClientThink).
        for c in 0..sc.clients {
            let r = referee_vm_call(vm, MpGameExport::GAME_CLIENT_THINK, &[c as isize]);
            assert_eq!(r, 0, "GAME_CLIENT_THINK({c}) frame {f} returned {r}");
        }
        let r = referee_vm_call(vm, MpGameExport::GAME_RUN_FRAME, &[t as isize]);
        assert_eq!(r, 0, "GAME_RUN_FRAME frame {f} returned {r}");
        assert!(referee_error().is_none(), "frame {f} G_ERROR: {:?}", referee_error());
        snaps.push(snapshot(&locate, sc.clients, t));
    }

    referee_vm_call(vm, MpGameExport::GAME_SHUTDOWN, &[0]);
    drop(module);
    snaps
}

// ===========================================================================
// Diff
// ===========================================================================

/// First differing byte index of two equal-length slices.
fn first_diff(a: &[u8], b: &[u8]) -> Option<usize> {
    let n = a.len().min(b.len());
    (0..n).find(|&i| a[i] != b[i]).or_else(|| {
        if a.len() != b.len() {
            Some(n)
        } else {
            None
        }
    })
}

/// The 4-byte word containing `off`, decoded as hex / i32 / f32.
fn word_str(bytes: &[u8], off: usize) -> String {
    let w = off & !3;
    if w + 4 > bytes.len() {
        return "<oob>".into();
    }
    let b = [bytes[w], bytes[w + 1], bytes[w + 2], bytes[w + 3]];
    let i = i32::from_le_bytes(b);
    let f = f32::from_le_bytes(b);
    format!("0x{:08x} (i32 {i}, f32 {f})", u32::from_le_bytes(b))
}

fn report_state_diff(kind: &str, tbl: &FieldTable, a: &[u8], b: &[u8], out: &mut String) -> bool {
    let Some(off) = first_diff(a, b) else {
        return false;
    };
    let field = describe(tbl, off);
    out.push_str(&format!(
        "  {kind}: first diff at byte {off} (field {field}) [{}]\n      oracle {}\n      rust   {}\n",
        tbl.kind,
        word_str(a, off),
        word_str(b, off),
    ));
    true
}

/// Compare the two runs; on the first divergent frame, build the structured
/// report and return it (Err); on full match return the pass summary (Ok).
fn diff_runs(sc: &Scenario, a: &[FrameSnap], b: &[FrameSnap]) -> Result<String, String> {
    assert_eq!(a.len(), b.len(), "frame counts differ ({} vs {})", a.len(), b.len());

    let mut total_ents = 0usize;
    let mut total_sc = 0usize;

    for (f, (sa, sb)) in a.iter().zip(b.iter()).enumerate() {
        total_ents += sa.inuse.len();
        total_sc += sa.sc_imports.len();

        let mut r = String::new();
        let mut diverged = false;

        // (1) playerState per client.
        for c in 0..sc.clients as usize {
            if report_state_diff(
                &format!("client {c} playerState"),
                &PS,
                &sa.clients[c],
                &sb.clients[c],
                &mut r,
            ) {
                diverged = true;
            }
        }

        // (2) active-entity set.
        if sa.inuse != sb.inuse {
            diverged = true;
            let only_a: Vec<u32> = sa.inuse.iter().copied().filter(|s| !sb.inuse.contains(s)).collect();
            let only_b: Vec<u32> = sb.inuse.iter().copied().filter(|s| !sa.inuse.contains(s)).collect();
            r.push_str(&format!(
                "  active-entity set differs: only-oracle={only_a:?} only-rust={only_b:?}\n"
            ));
        }

        // (3) entityState for slots active in BOTH.
        for &slot in &sa.inuse {
            if let (Some(ea), Some(eb)) = (sa.ent(slot), sb.ent(slot)) {
                if report_state_diff(
                    &format!("entity {slot} entityState"),
                    &ES,
                    ea,
                    eb,
                    &mut r,
                ) {
                    diverged = true;
                }
            }
        }

        // (4) syscall stream.
        if sa.sc_digest != sb.sc_digest {
            diverged = true;
            let idx = (0..sa.sc_imports.len().min(sb.sc_imports.len()))
                .find(|&i| sa.sc_imports[i] != sb.sc_imports[i]);
            match idx {
                Some(i) => {
                    r.push_str(&format!(
                        "  syscall stream: first divergent call #{i}: oracle={} rust={}\n",
                        referee_import_name(sa.sc_imports[i]),
                        referee_import_name(sb.sc_imports[i]),
                    ));
                    // Decoded window of calls around the divergence from BOTH
                    // streams — shows which loop each side is in. Never weakens
                    // the comparison; report-only.
                    let win = 12usize;
                    let lo = i.saturating_sub(win);
                    let hi_a = (i + win + 1).min(sa.sc_imports.len());
                    let hi_b = (i + win + 1).min(sb.sc_imports.len());
                    r.push_str("    --- oracle stream window ---\n");
                    for j in lo..hi_a {
                        let mark = if j == i { " <==" } else { "" };
                        r.push_str(&format!(
                            "      #{j}: {}{mark}\n",
                            referee_import_name(sa.sc_imports[j])
                        ));
                    }
                    r.push_str("    --- rust   stream window ---\n");
                    for j in lo..hi_b {
                        let mark = if j == i { " <==" } else { "" };
                        r.push_str(&format!(
                            "      #{j}: {}{mark}\n",
                            referee_import_name(sb.sc_imports[j])
                        ));
                    }
                }
                None => {
                    r.push_str(&format!(
                        "  syscall stream: same import prefix, lengths/texts differ (oracle {} calls, rust {} calls)\n",
                        sa.sc_imports.len(),
                        sb.sc_imports.len(),
                    ));
                    // One import stream is a prefix of the other; show the tail
                    // window from the shorter length onward from BOTH streams.
                    let common = sa.sc_imports.len().min(sb.sc_imports.len());
                    let lo = common.saturating_sub(6);
                    r.push_str("    --- oracle stream tail ---\n");
                    for j in lo..sa.sc_imports.len() {
                        r.push_str(&format!("      #{j}: {}\n", referee_import_name(sa.sc_imports[j])));
                    }
                    r.push_str("    --- rust   stream tail ---\n");
                    for j in lo..sb.sc_imports.len() {
                        r.push_str(&format!("      #{j}: {}\n", referee_import_name(sb.sc_imports[j])));
                    }
                }
            }
            if sa.sc_texts != sb.sc_texts {
                if let Some(i) = (0..sa.sc_texts.len().min(sb.sc_texts.len()))
                    .find(|&i| sa.sc_texts[i] != sb.sc_texts[i])
                {
                    r.push_str(&format!(
                        "    first divergent text #{i}: oracle={:?} rust={:?}\n",
                        sa.sc_texts[i], sb.sc_texts[i]
                    ));
                }
            }
        }

        if diverged {
            return Err(format!(
                "REFEREE DIVERGENCE — scenario '{}', frame {f} (level.time {}):\n{}",
                sc.name, sa.level_time, r
            ));
        }
    }

    Ok(format!(
        "referee PASS — scenario '{}': {} frames byte-identical; \
         {} client-states, {} entity-states, {} syscalls compared",
        sc.name,
        a.len(),
        a.len() * sc.clients as usize,
        total_ents,
        total_sc,
    ))
}

// ===========================================================================
// Artifact locations
// ===========================================================================

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root (crates/jampgame/../..)")
        .to_path_buf()
}

fn oracle_dylib() -> PathBuf {
    // Mirrors `common::dylib_filename`'s platform cfg-branch style; the oracle's
    // own filename convention comes from `tools/referee-oracle/build.sh`
    // (Darwin: liboraclejampgame.dylib, Linux: liboraclejampgame.so). Windows is
    // not built by build.sh yet — the name here is a placeholder for when it is.
    #[cfg(target_os = "macos")]
    let name = "liboraclejampgame.dylib";
    #[cfg(all(unix, not(target_os = "macos")))]
    let name = "liboraclejampgame.so";
    #[cfg(windows)]
    let name = "liboraclejampgame.dll";

    repo_root().join("tools/referee-oracle/build").join(name)
}

fn logs_dir() -> PathBuf {
    repo_root().join("tools/referee-oracle/logs")
}

// ===========================================================================
// Crash-isolated orchestration
//
// The referee is a findings generator whose whole point is to catch bugs — and
// a real one (the Rust port's `SortRanks`/`qsort` ABI mismatch, which SIGSEGVs
// with >=2 connected clients) would otherwise abort the harness before it could
// report anything. So each module's drive runs in a CHILD process (a re-exec of
// this test binary with `REFEREE_CHILD` set): the child does exactly ONE drive
// and serializes its per-frame snapshots to a file; the parent runs A then B as
// children, then diffs. A child killed by a signal is reported as a crash
// finding with the faulting module named — never an opaque parent-process
// SIGSEGV. This keeps the settled "sequential A, then B, then diff" shape while
// making a crashing port a DIAGNOSIS, not a dead harness.
// ===========================================================================

const CHILD_ENV: &str = "REFEREE_CHILD";
const OUT_ENV: &str = "REFEREE_OUT";

fn rust_cdylib() -> PathBuf {
    // Determinism self-test hook: `REFEREE_SELFTEST=1` drives the ORACLE as both
    // sides (A and B). The result MUST be a byte-identical PASS — it proves the
    // mock harness is a pure function of inputs, so any oracle-vs-rust divergence
    // is a genuine port difference, not harness nondeterminism.
    if std::env::var("REFEREE_SELFTEST").is_ok() {
        return oracle_dylib();
    }
    common::locate_cargo_cdylib(&common::dylib_filename("jampgame"))
}

/// Dispatch: parent orchestrates two children + diff; a child does one drive.
fn run_referee(test_name: &str, sc: Scenario) {
    if let Ok(role) = std::env::var(CHILD_ENV) {
        child_drive(&role, sc);
        return;
    }

    let oracle = oracle_dylib();
    assert!(
        oracle.exists(),
        "oracle dylib missing at {}. Build it: tools/referee-oracle/build.sh",
        oracle.display()
    );
    let rust = rust_cdylib();
    assert!(rust.exists(), "rust cdylib missing; run `cargo build --workspace`");

    let a = spawn_child(test_name, "oracle");
    let b = spawn_child(test_name, "rust");

    match diff_runs(&sc, &a, &b) {
        Ok(summary) => eprintln!("\n===== {summary} =====\n"),
        Err(report) => panic!("\n{report}"),
    }
}

/// Child: run exactly one module's drive and serialize the snapshots.
fn child_drive(role: &str, sc: Scenario) {
    let dylib = match role {
        "oracle" => oracle_dylib(),
        "rust" => rust_cdylib(),
        other => panic!("unknown REFEREE_CHILD role {other:?}"),
    };
    let out = PathBuf::from(std::env::var(OUT_ENV).expect("REFEREE_OUT"));
    let role = role.to_string();
    run_on_engine_thread_fn(move || {
        eprintln!("[referee:{role}] drive {}", dylib.display());
        let snaps = drive(&dylib, &sc);
        std::fs::write(&out, encode_snaps(&snaps)).expect("write snapshot file");
        eprintln!("[referee:{role}] {} frames snapshotted", snaps.len());
    });
}

/// Spawn a crash-isolated child for one module, then load back its snapshots.
/// A signal-killed child is reported as a crash finding (test failure).
fn spawn_child(test_name: &str, role: &str) -> Vec<FrameSnap> {
    let out = std::env::temp_dir().join(format!(
        "referee-{test_name}-{role}-{}.bin",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&out);
    let exe = std::env::current_exe().expect("current_exe");
    let status = std::process::Command::new(exe)
        .args([test_name, "--exact", "--ignored", "--nocapture", "--test-threads=1"])
        .env(CHILD_ENV, role)
        .env(OUT_ENV, &out)
        .status()
        .expect("spawn referee child");

    if !status.success() {
        let how = describe_exit(&status);
        panic!(
            "\nREFEREE CRASH FINDING — the '{role}' module {how} during scenario \
             '{test_name}'.\n  This is a referee finding: one side failed to complete the \
             replay while the other did.\n  Re-run that side alone under a debugger:\n    \
             {CHILD_ENV}={role} {OUT_ENV}=/tmp/x <this-test-binary> {test_name} --exact \
             --ignored --nocapture\n"
        );
    }

    let bytes = std::fs::read(&out).unwrap_or_else(|e| {
        panic!("'{role}' child produced no snapshot file ({e}); it likely failed silently")
    });
    let _ = std::fs::remove_file(&out);
    decode_snaps(&bytes)
}

#[cfg(unix)]
fn describe_exit(status: &std::process::ExitStatus) -> String {
    use std::os::unix::process::ExitStatusExt;
    if let Some(sig) = status.signal() {
        format!("was killed by signal {sig} (e.g. 11=SIGSEGV, 6=SIGABRT) — a hard crash")
    } else {
        format!("exited with {status} (a panic/G_ERROR — see its output above)")
    }
}

#[cfg(not(unix))]
fn describe_exit(status: &std::process::ExitStatus) -> String {
    format!("exited unsuccessfully ({status})")
}

// ===========================================================================
// Snapshot (de)serialization — a compact length-prefixed binary format for the
// child->parent handoff. Pure `to_le_bytes`; no external dep.
// ===========================================================================

fn put_u32(o: &mut Vec<u8>, v: u32) {
    o.extend_from_slice(&v.to_le_bytes());
}
fn put_i64(o: &mut Vec<u8>, v: i64) {
    o.extend_from_slice(&v.to_le_bytes());
}
fn put_bytes(o: &mut Vec<u8>, b: &[u8]) {
    put_u32(o, b.len() as u32);
    o.extend_from_slice(b);
}

fn encode_snaps(snaps: &[FrameSnap]) -> Vec<u8> {
    let mut o = Vec::new();
    put_u32(&mut o, snaps.len() as u32);
    for s in snaps {
        o.extend_from_slice(&s.level_time.to_le_bytes());
        put_u32(&mut o, s.clients.len() as u32);
        for c in &s.clients {
            put_bytes(&mut o, c);
        }
        put_u32(&mut o, s.ents.len() as u32);
        for (slot, b) in &s.ents {
            put_u32(&mut o, *slot);
            put_bytes(&mut o, b);
        }
        put_u32(&mut o, s.sc_imports.len() as u32);
        for &i in &s.sc_imports {
            put_i64(&mut o, i as i64);
        }
        put_u32(&mut o, s.sc_texts.len() as u32);
        for (imp, arg, text) in &s.sc_texts {
            put_i64(&mut o, *imp as i64);
            o.extend_from_slice(&arg.to_le_bytes());
            put_bytes(&mut o, text.as_bytes());
        }
        o.extend_from_slice(&s.sc_digest.to_le_bytes());
    }
    o
}

struct Reader<'a> {
    b: &'a [u8],
    p: usize,
}
impl<'a> Reader<'a> {
    fn u32(&mut self) -> u32 {
        let v = u32::from_le_bytes(self.b[self.p..self.p + 4].try_into().unwrap());
        self.p += 4;
        v
    }
    fn i32(&mut self) -> i32 {
        self.u32() as i32
    }
    fn i64(&mut self) -> i64 {
        let v = i64::from_le_bytes(self.b[self.p..self.p + 8].try_into().unwrap());
        self.p += 8;
        v
    }
    fn u64(&mut self) -> u64 {
        let v = u64::from_le_bytes(self.b[self.p..self.p + 8].try_into().unwrap());
        self.p += 8;
        v
    }
    fn bytes(&mut self) -> Vec<u8> {
        let n = self.u32() as usize;
        let v = self.b[self.p..self.p + n].to_vec();
        self.p += n;
        v
    }
}

fn decode_snaps(bytes: &[u8]) -> Vec<FrameSnap> {
    let mut r = Reader { b: bytes, p: 0 };
    let frames = r.u32() as usize;
    let mut out = Vec::with_capacity(frames);
    for _ in 0..frames {
        let level_time = r.i32();
        let nc = r.u32() as usize;
        let clients = (0..nc).map(|_| r.bytes()).collect();
        let ne = r.u32() as usize;
        let mut ents = Vec::with_capacity(ne);
        let mut inuse = Vec::with_capacity(ne);
        for _ in 0..ne {
            let slot = r.u32();
            let b = r.bytes();
            inuse.push(slot);
            ents.push((slot, b));
        }
        let ni = r.u32() as usize;
        let sc_imports = (0..ni).map(|_| r.i64() as isize).collect();
        let nt = r.u32() as usize;
        let sc_texts = (0..nt)
            .map(|_| {
                let imp = r.i64() as isize;
                let arg = r.i32();
                let text = String::from_utf8_lossy(&r.bytes()).into_owned();
                (imp, arg, text)
            })
            .collect();
        let sc_digest = r.u64();
        out.push(FrameSnap {
            level_time,
            clients,
            inuse,
            ents,
            sc_imports,
            sc_texts,
            sc_digest,
        });
    }
    out
}

// ===========================================================================
// Tests
// ===========================================================================

/// Parser/serializer round-trips AND the committed logs still match their
/// generators — guards against silent fixture drift. NOT ignored: no toolchain.
#[test]
fn reflog_roundtrip() {
    for sc in [reflog::gen_idle(), reflog::gen_solo(), reflog::gen_melee_brawl()] {
        let text = reflog::to_text(&sc);
        let reparsed = reflog::parse(&text);
        assert_eq!(sc, reparsed, "reflog parse/serialize round-trip failed for '{}'", sc.name);

        let path = logs_dir().join(format!("{}.reflog", sc.name));
        let committed = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("committed log {} missing ({e}); regenerate via `regenerate_logs`", path.display()));
        assert_eq!(
            committed, text,
            "committed {} has drifted from its generator; regenerate deliberately via the \
             `regenerate_logs` ignored test",
            path.display()
        );
    }
}

/// Deliberate log (re)generation — writes the committed fixtures from the
/// generators. Run explicitly when a generator changes:
/// `cargo test -p jampgame --test referee regenerate_logs -- --ignored`.
#[test]
#[ignore = "writes committed fixtures; run deliberately after changing a generator"]
fn regenerate_logs() {
    let dir = logs_dir();
    std::fs::create_dir_all(&dir).unwrap();
    for sc in [reflog::gen_idle(), reflog::gen_solo(), reflog::gen_melee_brawl()] {
        let path = dir.join(format!("{}.reflog", sc.name));
        std::fs::write(&path, reflog::to_text(&sc)).unwrap();
        eprintln!("[referee] wrote {}", path.display());
    }
}

#[test]
#[ignore = "requires tools/referee-oracle/build.sh + cargo build --workspace; run with --ignored"]
fn referee_solo() {
    run_referee("referee_solo", reflog::gen_solo());
}

#[test]
#[ignore = "requires tools/referee-oracle/build.sh + cargo build --workspace; run with --ignored"]
fn referee_idle() {
    run_referee("referee_idle", reflog::gen_idle());
}

#[test]
#[ignore = "requires tools/referee-oracle/build.sh + cargo build --workspace; run with --ignored"]
fn referee_melee_brawl() {
    run_referee("referee_melee_brawl", reflog::gen_melee_brawl());
}
