//! Pmove single-step differential parity test against the Raven oracle.
//!
//! Drives the ported `mp_bg::bg_pmove::Pmove` over the same fixtures and the
//! same synthetic `animation.cfg` as the C dumpers in `tools/jampgame-oracle/`
//! and byte-compares to the committed goldens. Two goldens:
//!
//! * `golden/pmove_trace.txt` — the `pmworld.h` axial-brush trace stub in
//!   isolation (`TestTraps` transcribes the identical f32 algorithm).
//! * `golden/pmove.txt` — the six on-foot scenarios driven through `Pmove`.
//!
//! `TestTraps` (BgTraps) is the world: an axial-brush trace/pointcontents +
//! `rintf` snap_vector. `TestCallbacks` (GameCallbacks) panics on everything but
//! the two anim restart-check reads, served from the prior-frame anim mirror.
//! See `tools/jampgame-oracle/pmove-spec.md` and `README.md` (pmove section).
//!
//! The world stub, fixture grammar, and seam stubs shared with the saber slice
//! live in `support`.
#![allow(non_snake_case)]

use std::fmt::Write as _;
use std::path::PathBuf;

use mp_bg::bg_panimate::BG_ParseAnimationFile;
use mp_bg::bg_pmove::Pmove;
use mp_game::bg_channel::BgState;
use mp_game::prelude::*;
use testkit::{compare, oracle_dir};

mod support;
use support::*;

fn fixture_dir() -> PathBuf {
    oracle_dir(env!("CARGO_MANIFEST_DIR")).join("fixtures/pmove")
}

// ================================ ps baseline ================================

fn ps_baseline() -> playerState_t {
    let mut ps: playerState_t = unsafe { core::mem::zeroed() };
    ps.pm_type = PM_NORMAL as c_int;
    ps.weapon = WP_MELEE;
    ps.weaponstate = WEAPON_READY as c_int;
    ps.stats[STAT_HEALTH as usize] = 100;
    ps.gravity = 800;
    ps.speed = 250.0;
    ps.basespeed = 250;
    ps.standheight = DEFAULT_MAXS_2; // 40
    ps.crouchheight = CROUCH_MAXS_2; // 16
    ps.viewheight = DEFAULT_VIEWHEIGHT; // 36
    ps.groundEntityNum = ENTITYNUM_NONE;
    ps.clientNum = 0;
    ps.m_iVehicleNum = 0;
    ps.commandTime = 0;
    ps
}

fn apply_ps_override(ps: &mut playerState_t, tok: &[&str]) {
    let name = tok[1];
    match name {
        "origin" => {
            ps.origin = [
                parse_float(tok[2]),
                parse_float(tok[3]),
                parse_float(tok[4]),
            ];
        }
        "velocity" => {
            ps.velocity = [
                parse_float(tok[2]),
                parse_float(tok[3]),
                parse_float(tok[4]),
            ];
        }
        "viewangles" => {
            ps.viewangles = [
                parse_float(tok[2]),
                parse_float(tok[3]),
                parse_float(tok[4]),
            ];
        }
        "delta_angles" => {
            ps.delta_angles = [parse_int(tok[2]), parse_int(tok[3]), parse_int(tok[4])];
        }
        "groundEntityNum" => ps.groundEntityNum = parse_int(tok[2]),
        "pm_flags" => ps.pm_flags = parse_int(tok[2]),
        "pm_type" => ps.pm_type = parse_int(tok[2]),
        "legsAnim" => ps.legsAnim = parse_int(tok[2]),
        "torsoAnim" => ps.torsoAnim = parse_int(tok[2]),
        "weapon" => ps.weapon = parse_int(tok[2]),
        "gravity" => ps.gravity = parse_int(tok[2]),
        "speed" => ps.speed = parse_float(tok[2]),
        "basespeed" => ps.basespeed = parse_int(tok[2]),
        "fallingToDeath" => ps.fallingToDeath = parse_int(tok[2]),
        "clientNum" => ps.clientNum = parse_int(tok[2]),
        other => panic!("unknown ps field '{other}'"),
    }
}

// =================================== dump ====================================

fn dump_step(o: &mut String, step: i32, pm: &pmove_t, ps: &playerState_t, ntr: i64, rng: u32) {
    let _ = writeln!(
        o,
        "s={} t={} org={:08x},{:08x},{:08x} vel={:08x},{:08x},{:08x} \
         va={:08x},{:08x},{:08x} da={},{},{} gnd={} pmf={:x} pmt={} \
         la={}:{} ta={}:{} fl={}{} bob={} vh={} ef={:x} seq={} \
         ev={}:{},{}:{} wt={} ws={} spd={:08x} wl={} wtp={} \
         nt={} mn={:08x} mx={:08x} xy={:08x} air={} f2d={} fjz={:08x} \
         ntr={} rng={:08x}",
        step,
        ps.commandTime,
        f2b(ps.origin[0]),
        f2b(ps.origin[1]),
        f2b(ps.origin[2]),
        f2b(ps.velocity[0]),
        f2b(ps.velocity[1]),
        f2b(ps.velocity[2]),
        f2b(ps.viewangles[0]),
        f2b(ps.viewangles[1]),
        f2b(ps.viewangles[2]),
        ps.delta_angles[0],
        ps.delta_angles[1],
        ps.delta_angles[2],
        ps.groundEntityNum,
        ps.pm_flags as u32,
        ps.pm_time,
        ps.legsAnim,
        ps.legsTimer,
        ps.torsoAnim,
        ps.torsoTimer,
        if ps.legsFlip != 0 { 1 } else { 0 },
        if ps.torsoFlip != 0 { 1 } else { 0 },
        ps.bobCycle,
        ps.viewheight,
        ps.eFlags as u32,
        ps.eventSequence,
        ps.events[0],
        ps.eventParms[0],
        ps.events[1],
        ps.eventParms[1],
        ps.weaponTime,
        ps.weaponstate,
        f2b(ps.speed),
        pm.waterlevel,
        pm.watertype,
        pm.numtouch,
        f2b(pm.mins[2]),
        f2b(pm.maxs[2]),
        f2b(pm.xyspeed),
        if ps.inAirAnim != 0 { 1 } else { 0 },
        ps.fallingToDeath,
        f2b(ps.fd.forceJumpZStart),
        ntr,
        rng,
    );
}

// ============================ scenario driver ================================

fn run_scenario(name: &str) -> String {
    let rows = parse_scenario(&fixture_dir().join(format!("{name}.txt")));

    let mut bg = BgState::new();
    // Size the humanoid anim table so BG_ParseAnimationFile (and Pmove's anim
    // reads) write/read into valid backing — the parser fills animset[token].
    bg.bgHumanoidAnimations
        .resize(MAX_TOTALANIMATIONS as usize, unsafe { core::mem::zeroed() });

    let mut traps = TestTraps::new(fixture_dir());
    let mut cb = TestCallbacks {
        legs_mirror: 0,
        torso_mirror: 0,
    };

    // Load the synthetic humanoid animation set (both sides parse the same file).
    let animset = bg.bgHumanoidAnimations.as_mut_ptr();
    let rc = BG_ParseAnimationFile(
        &mut bg,
        &traps,
        &mut cb,
        c"models/players/_humanoid/animation.cfg".as_ptr(),
        animset,
        qtrue,
    );
    assert_ne!(rc, -1, "failed to load synthetic animation.cfg");

    // Brushes are fixed for the scenario; collect them before the run.
    for row in &rows {
        if let Row::Brush(b) = row {
            traps.brushes.push(*b);
        }
    }

    let mut ps = ps_baseline();

    // pmove_t skeleton; cmd fields patched per step.
    let mut pm: pmove_t = unsafe { core::mem::zeroed() };
    pm.tracemask = MASK_PLAYERSOLID;
    pm.animations = bg.bgHumanoidAnimations.as_mut_ptr();
    pm.gametype = 0;

    let mut arena: Vec<bgEntity_t> = (0..8).map(|_| unsafe { core::mem::zeroed() }).collect();
    pm.entSize = core::mem::size_of::<bgEntity_t>() as c_int;

    let mut o = String::new();
    let _ = writeln!(o, "-- scenario {name} --");
    o.push_str("== pmove ==\n");

    let mut step = 0i32;
    let mut prev_server_time = 0i32;

    for row in &rows {
        match row {
            Row::Brush(_) => {}
            Row::Ps(tok) => {
                let refs: Vec<&str> = tok.iter().map(|s| s.as_str()).collect();
                apply_ps_override(&mut ps, &refs);
            }
            Row::Start => {
                // Freeze the anim mirror to the initial ps and emit the pre-move
                // baseline step (ntr=0).
                cb.legs_mirror = ps.legsAnim;
                cb.torso_mirror = ps.torsoAnim;
                traps.trace_count.set(0);
                // pm.ps / baseEnt must point at the live ps / arena for the dump.
                pm.ps = &mut ps as *mut playerState_t;
                pm.baseEnt = arena.as_mut_ptr() as *mut _;
                let rng = bg.rng.holdrand() as u32; // 32-bit tripwire (fixtures draw nothing)
                dump_step(&mut o, step, &pm, &ps, traps.trace_count.get(), rng);
                step += 1;
            }
            Row::Cmd(c) => {
                for r in 0..c.reps {
                    pm.ps = &mut ps as *mut playerState_t;
                    pm.baseEnt = arena.as_mut_ptr() as *mut _;
                    pm.cmd.forwardmove = c.fwd as i8 as c_schar;
                    pm.cmd.rightmove = c.right as i8 as c_schar;
                    pm.cmd.upmove = c.up as i8 as c_schar;
                    pm.cmd.buttons = c.buttons;
                    pm.cmd.weapon = WP_MELEE as byte;
                    pm.cmd.angles[0] = (c.pitch as i16) as c_int;
                    pm.cmd.angles[1] = ((c.yaw + r * c.yawinc) as i16) as c_int;
                    pm.cmd.angles[2] = (c.roll as i16) as c_int;
                    prev_server_time += c.dt;
                    pm.cmd.serverTime = prev_server_time;

                    traps.trace_count.set(0);
                    Pmove(&mut pm as *mut pmove_t, &mut bg, &traps, &mut cb);

                    // Mirror ps anims into the stub entity for the next step's
                    // restart-check (BG_PlayerStateToEntityState equivalent).
                    cb.legs_mirror = ps.legsAnim;
                    cb.torso_mirror = ps.torsoAnim;

                    let rng = bg.rng.holdrand() as u32; // 32-bit tripwire (fixtures draw nothing)
                    dump_step(&mut o, step, &pm, &ps, traps.trace_count.get(), rng);
                    step += 1;
                }
            }
        }
    }

    o.push_str("== end ==\n");
    o
}

// ============================= the raw-trace golden ===========================

fn run_trace() -> String {
    let text = std::fs::read_to_string(fixture_dir().join("trace.txt")).unwrap();
    let mut brushes: Vec<Brush> = Vec::new();
    let mut sweep_no = 0i32;
    let mut o = String::new();
    o.push_str("== pmove_trace ==\n");

    for line in text.lines() {
        let line = strip_comment(line);
        let tok: Vec<&str> = line.split_whitespace().collect();
        if tok.is_empty() {
            continue;
        }
        match tok[0] {
            "reset" => {
                brushes.clear();
                o.push_str("reset\n");
            }
            "brush" if tok.len() >= 7 => {
                let surf = if tok.len() >= 8 {
                    parse_surf(tok[7])
                } else {
                    0
                };
                brushes.push(Brush {
                    mins: [
                        parse_float(tok[1]),
                        parse_float(tok[2]),
                        parse_float(tok[3]),
                    ],
                    maxs: [
                        parse_float(tok[4]),
                        parse_float(tok[5]),
                        parse_float(tok[6]),
                    ],
                    surface_flags: surf,
                });
                let _ = writeln!(o, "brush {} surf={:x}", brushes.len() - 1, surf);
            }
            "sweep" if tok.len() >= 13 => {
                let start = [
                    parse_float(tok[1]),
                    parse_float(tok[2]),
                    parse_float(tok[3]),
                ];
                let end = [
                    parse_float(tok[4]),
                    parse_float(tok[5]),
                    parse_float(tok[6]),
                ];
                let mins = [
                    parse_float(tok[7]),
                    parse_float(tok[8]),
                    parse_float(tok[9]),
                ];
                let maxs = [
                    parse_float(tok[10]),
                    parse_float(tok[11]),
                    parse_float(tok[12]),
                ];
                let tr = trace_world(&brushes, &start, &mins, &maxs, &end);
                let _ = writeln!(
                    o,
                    "sweep {} as={} ss={} frac={:08x} end={:08x},{:08x},{:08x} \
                     nrm={:08x},{:08x},{:08x} pd={:08x} pt={} psb={} sf={:x} ct={:x} en={}",
                    sweep_no,
                    tr.allsolid,
                    tr.startsolid,
                    f2b(tr.fraction),
                    f2b(tr.endpos[0]),
                    f2b(tr.endpos[1]),
                    f2b(tr.endpos[2]),
                    f2b(tr.plane.normal[0]),
                    f2b(tr.plane.normal[1]),
                    f2b(tr.plane.normal[2]),
                    f2b(tr.plane.dist),
                    tr.plane.r#type,
                    tr.plane.signbits,
                    tr.surfaceFlags as u32,
                    tr.contents as u32,
                    tr.entityNum,
                );
                sweep_no += 1;
            }
            other => panic!("bad trace line: {other}"),
        }
    }
    o.push_str("== end ==\n");
    o
}

// =================================== test ====================================

#[test]
fn pmove_parity() {
    // 1) Prove the axial-brush trace stub in isolation first.
    compare(env!("CARGO_MANIFEST_DIR"), "pmove_trace", &run_trace());

    // 2) Then the six on-foot scenarios through the real Pmove.
    let scenarios = [
        "idle",
        "walk-fwd",
        "strafe-turn",
        "jump-land",
        "fall-onto-box",
        "wall-step",
    ];
    let mut o = String::new();
    for s in scenarios {
        o.push_str(&run_scenario(s));
    }
    compare(env!("CARGO_MANIFEST_DIR"), "pmove", &o);
}
