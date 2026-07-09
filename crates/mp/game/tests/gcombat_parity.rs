//! Differential parity test for three pure-ish `g_combat.c` leaf functions —
//! `RaySphereIntersections`, `G_GetHitLocation`, `CheckArmor` — against the
//! Raven oracle. Reproduces `tests/oracle/golden/gcombat.txt`
//! (generated exclusively by `main_gcombat.c` over the committed
//! `fixtures/gcombat/` inputs) by driving the PORTED functions and
//! byte-comparing to the committed golden.
//!
//! These three functions carry no shared RNG/global state (each reads only its
//! arguments plus, for `CheckArmor`, `level.time`), so one `#[test]` builds the
//! whole dump and compares once. See `tools/jampgame-oracle/README.md` for the
//! harness stamp; the golden is committed, so `cargo test` needs no C toolchain.
#![allow(non_snake_case)]

use std::fmt::Write as _;
use std::path::PathBuf;

use core::ffi::c_void;

use mp_engine_select::Engine;

use mp_game::g_combat::{CheckArmor, G_GetHitLocation, RaySphereIntersections};
use mp_game::prelude::*;
use mp_game::world::{GameContext, GameWorld};

// `YAW` comes from `mp_game::prelude` (canonical `q_math::YAW`, imported above
// via the glob); no test-local shadow needed.

fn oracle_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/oracle")
}

fn fixtures_dir() -> PathBuf {
    oracle_dir().join("fixtures/gcombat")
}

fn read_lines(name: &str) -> String {
    let p = fixtures_dir().join(name);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
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

// --- token parsing (mirrors main_gcombat.c pf/pi) ---
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

// ----------------------------- raysphere -----------------------------------
fn sec_raysphere(o: &mut String) {
    o.push_str("== raysphere ==\n");
    let text = read_lines("raysphere.txt");
    let mut idx = 0;
    for line in text.lines() {
        let tok: Vec<&str> = line.split_whitespace().collect();
        if tok.is_empty() || tok[0].starts_with('#') || tok[0] != "ray" {
            continue;
        }
        let origin: vec3_t = [pf(tok[1]), pf(tok[2]), pf(tok[3])];
        let radius = pf(tok[4]);
        let point: vec3_t = [pf(tok[5]), pf(tok[6]), pf(tok[7])];
        let mut dir: vec3_t = [pf(tok[8]), pf(tok[9]), pf(tok[10])];
        let mut inter: [vec3_t; 2] = [[0.0; 3], [0.0; 3]];
        let n = RaySphereIntersections(origin, radius, point, &mut dir, inter.as_mut_ptr());
        let _ = writeln!(
            o,
            "ray {idx} n {n} dir {:08x} {:08x} {:08x} i0 {:08x} {:08x} {:08x} i1 {:08x} {:08x} {:08x}",
            dir[0].to_bits(), dir[1].to_bits(), dir[2].to_bits(),
            inter[0][0].to_bits(), inter[0][1].to_bits(), inter[0][2].to_bits(),
            inter[1][0].to_bits(), inter[1][1].to_bits(), inter[1][2].to_bits(),
        );
        idx += 1;
    }
}

// ------------------------------ hitloc -------------------------------------
// The target always has a (non-NULL) client (a NULL client leaves `tangles`
// uninitialized in Raven — UB, excluded per porting-rules §19). ctx is unused
// by G_GetHitLocation but part of its signature.
fn sec_hitloc(o: &mut String, ctx: GameContext<'_>) {
    o.push_str("== hitloc ==\n");
    let text = read_lines("hitloc.txt");
    let mut idx = 0;
    for line in text.lines() {
        let tok: Vec<&str> = line.split_whitespace().collect();
        if tok.is_empty() || tok[0].starts_with('#') || tok[0] != "h" {
            continue;
        }
        let mut ent: gentity_t = unsafe { core::mem::zeroed() };
        let mut client: gclient_t = unsafe { core::mem::zeroed() };
        ent.client = &mut client as *mut gclient_t as *mut c_void;
        ent.r.currentAngles[YAW] = pf(tok[1]);
        ent.r.absmin = [pf(tok[2]), pf(tok[3]), pf(tok[4])];
        ent.r.absmax = [pf(tok[5]), pf(tok[6]), pf(tok[7])];
        ent.r.mins[0] = pf(tok[8]);
        ent.r.mins[1] = pf(tok[9]);
        ent.r.maxs[0] = pf(tok[10]);
        ent.r.maxs[1] = pf(tok[11]);
        let ppoint: vec3_t = [pf(tok[12]), pf(tok[13]), pf(tok[14])];
        let hl = G_GetHitLocation(ctx, &mut ent, ppoint);
        let _ = writeln!(o, "h {idx} hl {hl}");
        idx += 1;
    }
}

// ------------------------------- armor -------------------------------------
// Columns: a <armor> <isVehicle 0|1> <electrifyTime> <hasVehicle 0|1>
//            <levelTime> <damage> <dflags>. isVehicle maps 1 -> CLASS_VEHICLE.
fn sec_armor(o: &mut String, ctx: GameContext<'_>) {
    o.push_str("== armor ==\n");
    let text = read_lines("armor.txt");
    let mut dummy_veh: u8 = 0;
    let mut idx = 0;
    for line in text.lines() {
        let tok: Vec<&str> = line.split_whitespace().collect();
        if tok.is_empty() || tok[0].starts_with('#') || tok[0] != "a" {
            continue;
        }
        let mut ent: gentity_t = unsafe { core::mem::zeroed() };
        let mut client: gclient_t = unsafe { core::mem::zeroed() };
        ent.client = &mut client as *mut gclient_t as *mut c_void;
        client.ps.stats[STAT_ARMOR as usize] = pi(tok[1]);
        client.NPC_class = if pi(tok[2]) != 0 {
            class_t::CLASS_VEHICLE
        } else {
            class_t::CLASS_NONE
        };
        client.ps.electrifyTime = pi(tok[3]);
        ent.m_pVehicle = if pi(tok[4]) != 0 {
            &mut dummy_veh as *mut u8 as *mut c_void
        } else {
            core::ptr::null_mut()
        };
        // SAFETY: `ctx.world` is the live boxed GameWorld built in the test.
        unsafe {
            (*ctx.world).level.time = pi(tok[5]);
        }
        let damage = pi(tok[6]);
        let dflags = pi(tok[7]);
        let r = CheckArmor(ctx, &mut ent, damage, dflags);
        let armor = client.ps.stats[STAT_ARMOR as usize];
        let _ = writeln!(o, "a {idx} ret {r} armor {armor}");
        idx += 1;
    }
}

fn run() {
    // A live owned world + a never-invoked engine (none of the three tested
    // functions crosses the syscall seam, so the null syscall pointer is safe).
    let mut world = Box::new(GameWorld::zeroed());
    let engine = Engine::new(core::ptr::null::<c_void>());
    let ctx = GameContext {
        world: &mut *world as *mut GameWorld,
        engine: &engine,
    };

    let mut o = String::new();
    o.push_str("== gcombat ==\n");
    sec_raysphere(&mut o);
    sec_hitloc(&mut o, ctx);
    sec_armor(&mut o, ctx);
    o.push_str("== end ==\n");
    compare("gcombat", &o);
}

#[test]
fn gcombat_parity() {
    // `GameWorld::zeroed()` builds a large by-value temporary; the test-harness
    // worker thread's default stack is too small, so run on a roomy stack.
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}
