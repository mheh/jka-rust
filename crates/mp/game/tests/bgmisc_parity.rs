//! Differential parity test for the jampgame `bg_misc` / `bg_weapons` ports
//! against the Raven oracle. Reproduces `tools/jampgame-oracle/golden/bgmisc.txt`
//! (generated exclusively by `main_bgmisc.c` over the committed
//! `fixtures/bgmisc/` inputs) by driving the PORTED functions and tables and
//! byte-comparing to the committed golden.
//!
//! `bg_misc.c` is "both games misc functions, all completely stateless"
//! (Raven's own header comment), so there is no shared RNG/global state: one
//! `#[test]` builds the whole dump and compares once. See
//! `tools/jampgame-oracle/README.md` for the harness stamp and the trajectory
//! f32-vs-f64 reconciliation notes.
#![allow(non_snake_case)]

use std::ffi::CStr;
use std::fmt::Write as _;
use std::path::PathBuf;

use core::ffi::{c_char, c_int};

use mp_game::bg_misc::{
    BG_CanItemBeGrabbed, BG_EvaluateTrajectory, BG_EvaluateTrajectoryDelta, BG_FindItem,
    BG_FindItemForAmmo, BG_FindItemForHoldable, BG_FindItemForPowerup, BG_FindItemForWeapon,
    BG_PlayerStateToEntityState, BG_PlayerStateToEntityStateExtraPolate,
};
use mp_game::prelude::*;

fn oracle_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../tools/jampgame-oracle")
}

fn fixtures_dir() -> PathBuf {
    oracle_dir().join("fixtures/bgmisc")
}

fn compare(name: &str, got: &str) {
    let golden_path = oracle_dir().join("golden").join(format!("{name}.txt"));
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

// --- token parsing (mirrors main_bgmisc.c pf/pi) ---
// A float token is a plain (possibly negative) integer parsed as (float)atol,
// or an 0xXXXXXXXX f32 bit pattern. An int token is decimal, or 0x hex.
fn pf(t: &str) -> f32 {
    if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        f32::from_bits(u32::from_str_radix(h, 16).unwrap())
    } else {
        t.parse::<i64>().unwrap() as f32
    }
}
fn pi(t: &str) -> i32 {
    if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        i64::from_str_radix(h, 16).unwrap() as i32
    } else {
        t.parse::<i64>().unwrap() as i32
    }
}

fn read_lines(name: &str) -> String {
    let p = fixtures_dir().join(name);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

// C `%s` with `(null)` for a NULL pointer (mirrors main_bgmisc.c pS).
fn cs(p: *mut c_char) -> String {
    if p.is_null() {
        "(null)".to_string()
    } else {
        unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
    }
}

// ----------------------------- ps / es setters -----------------------------
// Mirror main_bgmisc.c ps_set / es_set exactly: the fixture is the single
// source of truth for inputs; both sides dispatch the same field names.
fn ps_set(ps: &mut playerState_t, tok: &[&str]) {
    let f = tok[0];
    match f {
        // scalar ints
        "pm_type" => ps.pm_type = pi(tok[1]),
        "clientNum" => ps.clientNum = pi(tok[1]),
        "weapon" => ps.weapon = pi(tok[1]),
        "weaponstate" => ps.weaponstate = pi(tok[1]),
        "weaponChargeTime" => ps.weaponChargeTime = pi(tok[1]),
        "groundEntityNum" => ps.groundEntityNum = pi(tok[1]),
        "movementDir" => ps.movementDir = pi(tok[1]),
        "eFlags" => ps.eFlags = pi(tok[1]),
        "eFlags2" => ps.eFlags2 = pi(tok[1]),
        "externalEvent" => ps.externalEvent = pi(tok[1]),
        "externalEventParm" => ps.externalEventParm = pi(tok[1]),
        "eventSequence" => ps.eventSequence = pi(tok[1]),
        "entityEventSequence" => ps.entityEventSequence = pi(tok[1]),
        "duelInProgress" => ps.duelInProgress = pi(tok[1]),
        "genericEnemyIndex" => ps.genericEnemyIndex = pi(tok[1]),
        "isJediMaster" => ps.isJediMaster = pi(tok[1]),
        "trueJedi" => ps.trueJedi = pi(tok[1]),
        "trueNonJedi" => ps.trueNonJedi = pi(tok[1]),
        "legsAnim" => ps.legsAnim = pi(tok[1]),
        "torsoAnim" => ps.torsoAnim = pi(tok[1]),
        "legsFlip" => ps.legsFlip = pi(tok[1]),
        "torsoFlip" => ps.torsoFlip = pi(tok[1]),
        "saberInFlight" => ps.saberInFlight = pi(tok[1]),
        "saberEntityNum" => ps.saberEntityNum = pi(tok[1]),
        "saberMove" => ps.saberMove = pi(tok[1]),
        "saberHolstered" => ps.saberHolstered = pi(tok[1]),
        "saberLockFrame" => ps.saberLockFrame = pi(tok[1]),
        "electrifyTime" => ps.electrifyTime = pi(tok[1]),
        "activeForcePass" => ps.activeForcePass = pi(tok[1]),
        "emplacedIndex" => ps.emplacedIndex = pi(tok[1]),
        "holocronBits" => ps.holocronBits = pi(tok[1]),
        "heldByClient" => ps.heldByClient = pi(tok[1]),
        "ragAttach" => ps.ragAttach = pi(tok[1]),
        "iModelScale" => ps.iModelScale = pi(tok[1]),
        "brokenLimbs" => ps.brokenLimbs = pi(tok[1]),
        "hasLookTarget" => ps.hasLookTarget = pi(tok[1]),
        "lookTarget" => ps.lookTarget = pi(tok[1]),
        "m_iVehicleNum" => ps.m_iVehicleNum = pi(tok[1]),
        "loopSound" => ps.loopSound = pi(tok[1]),
        "generic1" => ps.generic1 = pi(tok[1]),
        // float
        "speed" => ps.speed = pf(tok[1]),
        // vec3
        "origin" => ps.origin = [pf(tok[1]), pf(tok[2]), pf(tok[3])],
        "velocity" => ps.velocity = [pf(tok[1]), pf(tok[2]), pf(tok[3])],
        "viewangles" => ps.viewangles = [pf(tok[1]), pf(tok[2]), pf(tok[3])],
        "lastHitLoc" => ps.lastHitLoc = [pf(tok[1]), pf(tok[2]), pf(tok[3])],
        // indexed int arrays
        "events" => ps.events[pi(tok[1]) as usize] = pi(tok[2]),
        "eventParms" => ps.eventParms[pi(tok[1]) as usize] = pi(tok[2]),
        "powerups" => ps.powerups[pi(tok[1]) as usize] = pi(tok[2]),
        "customRGBA" => ps.customRGBA[pi(tok[1]) as usize] = pi(tok[2]),
        "stats" => ps.stats[pi(tok[1]) as usize] = pi(tok[2]),
        "ammo" => ps.ammo[pi(tok[1]) as usize] = pi(tok[2]),
        "persistant" => ps.persistant[pi(tok[1]) as usize] = pi(tok[2]),
        // stat aliases (identical index on both sides)
        "health" => ps.stats[STAT_HEALTH as usize] = pi(tok[1]),
        "maxhealth" => ps.stats[STAT_MAX_HEALTH as usize] = pi(tok[1]),
        "armor" => ps.stats[STAT_ARMOR as usize] = pi(tok[1]),
        "statweapons" => ps.stats[STAT_WEAPONS as usize] = pi(tok[1]),
        "holdables" => ps.stats[STAT_HOLDABLE_ITEMS as usize] = pi(tok[1]),
        // named powerups / persistant (hit the exact enum branch)
        "powerups_ysalamiri" => ps.powerups[PW_YSALAMIRI as usize] = pi(tok[1]),
        "powerups_redflag" => ps.powerups[PW_REDFLAG as usize] = pi(tok[1]),
        "powerups_blueflag" => ps.powerups[PW_BLUEFLAG as usize] = pi(tok[1]),
        "persistant_team" => ps.persistant[PERS_TEAM as usize] = pi(tok[1]),
        // forcedata
        "fd_forcePowersActive" => ps.fd.forcePowersActive = pi(tok[1]),
        "fd_saberAnimLevel" => ps.fd.saberAnimLevel = pi(tok[1]),
        "fd_mtti1" => ps.fd.forceMindtrickTargetIndex = pi(tok[1]),
        "fd_mtti2" => ps.fd.forceMindtrickTargetIndex2 = pi(tok[1]),
        "fd_mtti3" => ps.fd.forceMindtrickTargetIndex3 = pi(tok[1]),
        "fd_mtti4" => ps.fd.forceMindtrickTargetIndex4 = pi(tok[1]),
        _ => panic!("ps_set: unknown field '{f}'"),
    }
}

fn es_set(es: &mut entityState_t, tok: &[&str]) {
    match tok[0] {
        "modelindex" => es.modelindex = pi(tok[1]),
        "modelindex2" => es.modelindex2 = pi(tok[1]),
        "generic1" => es.generic1 = pi(tok[1]),
        "powerups" => es.powerups = pi(tok[1]),
        "eFlags" => es.eFlags = pi(tok[1]),
        f => panic!("es_set: unknown field '{f}'"),
    }
}

// ------------------------------ dump helpers -------------------------------
fn pI(o: &mut String, tag: &str, v: c_int) {
    let _ = writeln!(o, "{tag} {v}");
}
fn pF(o: &mut String, tag: &str, v: f32) {
    let _ = writeln!(o, "{tag} {:08x}", v.to_bits());
}
fn pV(o: &mut String, tag: &str, v: &[f32; 3]) {
    let _ = writeln!(o, "{tag} {:08x} {:08x} {:08x}", v[0].to_bits(), v[1].to_bits(), v[2].to_bits());
}
fn pS(o: &mut String, tag: &str, p: *mut c_char) {
    let _ = writeln!(o, "{tag} {}", cs(p));
}

// ------------------------------- sections ----------------------------------
fn sec_trajectory(o: &mut String) {
    o.push_str("== trajectory ==\n");
    let text = read_lines("trajectory.txt");
    let mut idx = 0;
    for line in text.lines() {
        let tok: Vec<&str> = line.split_whitespace().collect();
        if tok.is_empty() || tok[0].starts_with('#') || tok[0] != "T" {
            continue;
        }
        let ty = match pi(tok[1]) {
            0 => TR_STATIONARY,
            1 => TR_INTERPOLATE,
            2 => TR_LINEAR,
            3 => TR_LINEAR_STOP,
            4 => TR_NONLINEAR_STOP,
            5 => TR_SINE,
            6 => TR_GRAVITY,
            n => panic!("bad trType {n}"),
        };
        let tr = trajectory_t {
            trType: ty,
            trTime: pi(tok[2]),
            trDuration: pi(tok[3]),
            trBase: [pf(tok[4]), pf(tok[5]), pf(tok[6])],
            trDelta: [pf(tok[7]), pf(tok[8]), pf(tok[9])],
        };
        let at = pi(tok[10]);
        let mut rp = [0.0f32; 3];
        let mut rd = [0.0f32; 3];
        BG_EvaluateTrajectory(&tr, at, &mut rp);
        BG_EvaluateTrajectoryDelta(&tr, at, &mut rd);
        let _ = writeln!(
            o,
            "T {idx} et {:08x} {:08x} {:08x} ed {:08x} {:08x} {:08x}",
            rp[0].to_bits(), rp[1].to_bits(), rp[2].to_bits(),
            rd[0].to_bits(), rd[1].to_bits(), rd[2].to_bits()
        );
        idx += 1;
    }
}

fn sec_itemlist(o: &mut String) {
    o.push_str("== itemlist ==\n");
    for i in 0..=bg_numItems as usize {
        let it = &bg_itemlist[i];
        let _ = writeln!(o, "item {i}");
        pS(o, " classname", it.classname);
        pS(o, " pickup_sound", it.pickup_sound);
        for m in 0..4 {
            // Raven MAX_ITEM_MODELS
            let _ = writeln!(o, " world_model{m} {}", cs(it.world_model[m]));
        }
        pS(o, " view_model", it.view_model);
        pS(o, " icon", it.icon);
        pI(o, " quantity", it.quantity);
        pI(o, " giType", it.giType as c_int);
        pI(o, " giTag", it.giTag);
        pS(o, " precaches", it.precaches);
        pS(o, " sounds", it.sounds);
        pS(o, " description", it.description);
    }
}

fn itemidx(it: *mut gitem_t) -> i32 {
    if it.is_null() {
        -1
    } else {
        unsafe { it.offset_from(bg_itemlist.as_ptr()) as i32 }
    }
}

fn sec_findid(o: &mut String) {
    o.push_str("== findid ==\n");
    // BG_FindItemForWeapon / ForAmmo / ForHoldable Com_Error (port panic) on a
    // tag with no item, so only existing tags are queried (see main_bgmisc.c).
    for w in 1..WP_NUM_WEAPONS {
        let _ = writeln!(o, "weapon {w} {}", itemidx(BG_FindItemForWeapon(w)));
    }
    // ammo_t is a non-Copy enum; build a fresh variant per call from its int.
    fn ammo_of(n: i32) -> ammo_t {
        match n {
            1 => AMMO_FORCE,
            2 => AMMO_BLASTER,
            3 => AMMO_POWERCELL,
            4 => AMMO_METAL_BOLTS,
            5 => AMMO_ROCKETS,
            7 => AMMO_THERMAL,
            8 => AMMO_TRIPMINE,
            9 => AMMO_DETPACK,
            _ => unreachable!(),
        }
    }
    for a in [1, 2, 3, 4, 5, 7, 8, 9] {
        let _ = writeln!(o, "ammo {a} {}", itemidx(BG_FindItemForAmmo(ammo_of(a))));
    }
    for h in 1..12 {
        let _ = writeln!(o, "holdable {h} {}", itemidx(BG_FindItemForHoldable(h)));
    }
    for p in 0..=PW_NUM_POWERUPS {
        let _ = writeln!(o, "powerup {p} {}", itemidx(BG_FindItemForPowerup(p)));
    }
    let _ = writeln!(o, "powerup 999 {}", itemidx(BG_FindItemForPowerup(999)));
    let names = [
        "weapon_saber", "weapon_blaster", "ammo_blaster", "item_shield_sm_instant",
        "team_CTF_redflag", "item_medpac", "item_force_enlighten_light", "weapon_stun_baton",
        "nonexistent_item", "",
    ];
    for n in names {
        let c = std::ffi::CString::new(n).unwrap();
        let _ = writeln!(o, "find \"{n}\" {}", itemidx(BG_FindItem(c.as_ptr())));
    }
}

fn sec_weapondata(o: &mut String) {
    o.push_str("== weapondata ==\n");
    for w in 0..WP_NUM_WEAPONS as usize {
        let d = &weaponData[w];
        let _ = writeln!(
            o,
            "wd {w} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            d.ammoIndex, d.ammoLow, d.energyPerShot, d.fireTime, d.range, d.altEnergyPerShot,
            d.altFireTime, d.altRange, d.chargeSubTime, d.altChargeSubTime, d.chargeSub,
            d.altChargeSub, d.maxCharge, d.altMaxCharge
        );
    }
    o.push_str("== ammodata ==\n");
    for a in 0..ammoData.len() {
        let _ = writeln!(o, "ad {a} {}", ammoData[a].max);
    }
}

fn sec_grab(o: &mut String) {
    o.push_str("== grab ==\n");
    let text = read_lines("grab.txt");
    let mut gametype = 0i32;
    let mut ent: entityState_t = unsafe { core::mem::zeroed() };
    let mut ps: playerState_t = unsafe { core::mem::zeroed() };
    let mut have_ps = false;
    for line in text.lines() {
        let tok: Vec<&str> = line.split_whitespace().collect();
        if tok.is_empty() || tok[0].starts_with('#') {
            continue;
        }
        match tok[0] {
            "reset" => {
                ent = unsafe { core::mem::zeroed() };
                ps = unsafe { core::mem::zeroed() };
                gametype = 0;
                have_ps = true;
            }
            "nullps" => have_ps = false,
            "gametype" => gametype = pi(tok[1]),
            "ent" => es_set(&mut ent, &tok[1..]),
            "ps" => ps_set(&mut ps, &tok[1..]),
            "run" => {
                let psp = if have_ps {
                    &ps as *const playerState_t
                } else {
                    core::ptr::null()
                };
                let r = BG_CanItemBeGrabbed(gametype, &ent, psp);
                let _ = writeln!(o, "grab {} {}", tok[1], (r != 0) as i32);
            }
            c => panic!("grab: unknown cmd '{c}'"),
        }
    }
}

fn dump_es(o: &mut String, label: &str, ps: &playerState_t, s: &entityState_t) {
    let _ = writeln!(o, "es {label}");
    pI(o, " eType", s.eType);
    pI(o, " number", s.number);
    pI(o, " pos.trType", s.pos.trType as c_int);
    pV(o, " pos.trBase", &s.pos.trBase);
    pV(o, " pos.trDelta", &s.pos.trDelta);
    pI(o, " apos.trType", s.apos.trType as c_int);
    pV(o, " apos.trBase", &s.apos.trBase);
    pI(o, " trickedentindex", s.trickedentindex);
    pI(o, " trickedentindex2", s.trickedentindex2);
    pI(o, " trickedentindex3", s.trickedentindex3);
    pI(o, " trickedentindex4", s.trickedentindex4);
    pI(o, " forceFrame", s.forceFrame);
    pI(o, " emplacedOwner", s.emplacedOwner);
    pF(o, " speed", s.speed);
    pI(o, " genericenemyindex", s.genericenemyindex);
    pI(o, " activeForcePass", s.activeForcePass);
    pV(o, " angles2", &s.angles2);
    pI(o, " legsAnim", s.legsAnim);
    pI(o, " torsoAnim", s.torsoAnim);
    pI(o, " legsFlip", s.legsFlip);
    pI(o, " torsoFlip", s.torsoFlip);
    pI(o, " clientNum", s.clientNum);
    pI(o, " eFlags", s.eFlags);
    pI(o, " eFlags2", s.eFlags2);
    pI(o, " saberInFlight", s.saberInFlight);
    pI(o, " saberEntityNum", s.saberEntityNum);
    pI(o, " saberMove", s.saberMove);
    pI(o, " forcePowersActive", s.forcePowersActive);
    pI(o, " bolt1", s.bolt1);
    pI(o, " otherEntityNum2", s.otherEntityNum2);
    pI(o, " saberHolstered", s.saberHolstered);
    pI(o, " event", s.event);
    pI(o, " eventParm", s.eventParm);
    pI(o, " weapon", s.weapon);
    pI(o, " groundEntityNum", s.groundEntityNum);
    pI(o, " powerups", s.powerups);
    pI(o, " loopSound", s.loopSound);
    pI(o, " generic1", s.generic1);
    pI(o, " modelindex2", s.modelindex2);
    pI(o, " constantLight", s.constantLight);
    pV(o, " origin2", &s.origin2);
    pI(o, " isJediMaster", s.isJediMaster);
    pI(o, " time2", s.time2);
    pI(o, " fireflag", s.fireflag);
    pI(o, " heldByClient", s.heldByClient);
    pI(o, " ragAttach", s.ragAttach);
    pI(o, " iModelScale", s.iModelScale);
    pI(o, " brokenLimbs", s.brokenLimbs);
    pI(o, " hasLookTarget", s.hasLookTarget);
    pI(o, " lookTarget", s.lookTarget);
    let _ = writeln!(
        o,
        " customRGBA {} {} {} {}",
        s.customRGBA[0], s.customRGBA[1], s.customRGBA[2], s.customRGBA[3]
    );
    pI(o, " m_iVehicleNum", s.m_iVehicleNum);
    pI(o, " ps.entityEventSequence", ps.entityEventSequence);
}

fn sec_ps2es(o: &mut String) {
    o.push_str("== ps2es ==\n");
    let text = read_lines("ps.txt");
    let mut ps: playerState_t = unsafe { core::mem::zeroed() };
    let mut snap = 0i32;
    for line in text.lines() {
        let tok: Vec<&str> = line.split_whitespace().collect();
        if tok.is_empty() || tok[0].starts_with('#') {
            continue;
        }
        match tok[0] {
            "reset" => {
                ps = unsafe { core::mem::zeroed() };
                snap = 0;
            }
            "snap" => snap = pi(tok[1]),
            "ps" => ps_set(&mut ps, &tok[1..]),
            "run" => {
                let mut s: entityState_t = unsafe { core::mem::zeroed() };
                BG_PlayerStateToEntityState(&mut ps, &mut s, if snap != 0 { QTRUE } else { QFALSE });
                dump_es(o, tok[1], &ps, &s);
            }
            c => panic!("ps2es: unknown cmd '{c}'"),
        }
    }
}

// BG_PlayerStateToEntityStateExtraPolate over the same ps.txt scenarios (snap=0,
// fixed extrapolation time); differs from the base only in pos.trType +
// pos.trTime/trDuration, so it reuses dump_es and appends those two fields.
const XP_TIME: c_int = 12345;
fn sec_ps2esxp(o: &mut String) {
    o.push_str("== ps2esxp ==\n");
    let text = read_lines("ps.txt");
    let mut ps: playerState_t = unsafe { core::mem::zeroed() };
    for line in text.lines() {
        let tok: Vec<&str> = line.split_whitespace().collect();
        if tok.is_empty() || tok[0].starts_with('#') {
            continue;
        }
        match tok[0] {
            "reset" => ps = unsafe { core::mem::zeroed() },
            "snap" => {} // ignored: extrapolate dumped with snap=0 only
            "ps" => ps_set(&mut ps, &tok[1..]),
            "run" => {
                let mut s: entityState_t = unsafe { core::mem::zeroed() };
                BG_PlayerStateToEntityStateExtraPolate(&mut ps, &mut s, XP_TIME, QFALSE);
                dump_es(o, tok[1], &ps, &s);
                pI(o, " pos.trTime", s.pos.trTime);
                pI(o, " pos.trDuration", s.pos.trDuration);
            }
            c => panic!("ps2esxp: unknown cmd '{c}'"),
        }
    }
}

#[test]
fn bgmisc_parity() {
    let mut o = String::new();
    o.push_str("== bgmisc ==\n");
    sec_trajectory(&mut o);
    sec_itemlist(&mut o);
    sec_findid(&mut o);
    sec_weapondata(&mut o);
    sec_grab(&mut o);
    sec_ps2es(&mut o);
    sec_ps2esxp(&mut o);
    o.push_str("== end ==\n");
    compare("bgmisc", &o);
}
