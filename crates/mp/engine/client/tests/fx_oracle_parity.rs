//! Differential parity: the Rust FX port must reproduce, byte for byte, the
//! dumps the UNMODIFIED Raven FX translation units produce under
//! `tools/fx-oracle/run.sh` (goldens under `tools/fx-oracle/golden/`).
//!
//! This file is the Rust twin of `tools/fx-oracle/main.cpp`. It reads the same
//! scenario scripts, drives the same calls in the same order through
//! `FxHost::Harness`, and prints the same records. `tools/fx-oracle/README.md`
//! is the contract for both the scenario language and the golden format.
//!
//! The goldens are committed, so this gate needs no C++ toolchain
//! (porting-rules §18, DEC-61.5).

#![allow(non_snake_case)]

use std::path::PathBuf;

use mp_engine_client::fx::cfx_range::CFxRange;
use mp_engine_client::fx::cprimitive_template::CPrimitiveTemplate;
use mp_engine_client::fx::ctrail::FX_FeedTrail;
use mp_engine_client::fx::emat_impact_effect::EMatImpactEffect;
use mp_engine_client::fx::fx_export::{
    FX_AddScheduledEffects, FX_AdjustTime, FX_Draw2DEffects, FX_FreeSystem, FX_InitSystem,
    FX_PlayBoltedEffectID, FX_PlayEffect, FX_PlayEffectID, FX_PlayEntityEffectID,
    FX_RegisterEffect,
};
use mp_engine_client::fx::fx_harness::{fx_zero_trace, FxHarness};
use mp_engine_client::fx::fx_host::FxHost;
use mp_engine_client::fx::fx_scheduler::{
    fx_play_effect_file_axis, fx_stop_effect, FX_MAX_EFFECTS,
};
use mp_engine_client::fx::fx_system::{FxRefdef, FxSystem};
use mp_engine_client::fx::fx_util::{
    FX_AddBezier, FX_AddElectricity, FX_AddLine, FX_AddParticle, FX_AddPoly, FX_Free,
};
use mp_qshared::shared::effect_trail_arg::effectTrailArgStruct_t;
use mp_qshared::shared::effect_trail_vert::effectTrailVertStruct_t;
use native_math::vector::vec3_t;

/// The scenario set the harness ships. A shrinking set means a lost gate.
const SCENARIO_FLOOR: usize = 17;

/// Print a float as the raw 32-bit IEEE-754 pattern, the way `fxf` does.
///
/// Source: `tools/fx-oracle/host.cpp:38-50`
fn fxf(v: f32) -> String {
    format!("{:08x}", v.to_bits())
}

/// Print a three-float vector as three bit patterns.
fn fxv(v: &vec3_t) -> String {
    format!("{} {} {}", fxf(v[0]), fxf(v[1]), fxf(v[2]))
}

/// C `atoi` over one token: the longest integer prefix, or zero.
fn atoi(token: &str) -> i32 {
    let bytes = token.as_bytes();
    let mut end = 0;
    if end < bytes.len() && (bytes[end] == b'+' || bytes[end] == b'-') {
        end += 1;
    }
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    token[..end].parse().unwrap_or(0)
}

/// C `atof` over one token, narrowed to `float` the way the driver does.
fn atof(token: &str) -> f32 {
    token.parse::<f64>().unwrap_or(0.0) as f32
}

/// One command line, consumed left to right.
///
/// Source: `tools/fx-oracle/main.cpp:59-112`
struct Args<'a> {
    cmd: &'a str,
    toks: Vec<&'a str>,
    at: usize,
}

impl<'a> Args<'a> {
    fn word(&mut self) -> &'a str {
        let w = self
            .toks
            .get(self.at)
            .unwrap_or_else(|| panic!("fx-oracle: {} wants more arguments", self.cmd));
        self.at += 1;
        w
    }

    fn f(&mut self) -> f32 {
        atof(self.word())
    }

    fn i(&mut self) -> i32 {
        atoi(self.word())
    }

    fn b(&mut self) -> bool {
        self.i() != 0
    }

    fn v3(&mut self) -> vec3_t {
        [self.f(), self.f(), self.f()]
    }

    fn axis(&mut self) -> [vec3_t; 3] {
        [self.v3(), self.v3(), self.v3()]
    }
}

/// The `CFxRange` fields, in the declaration order of `CPrimitiveTemplate`.
///
/// Source: `oracle/codemp/client/FxScheduler.h:167-254`,
/// `tools/fx-oracle/main.cpp:126-178`
fn range_fields(p: &CPrimitiveTemplate) -> Vec<(&'static str, CFxRange)> {
    vec![
        ("spawnDelay", p.mSpawnDelay),
        ("spawnCount", p.mSpawnCount),
        ("life", p.mLife),
        ("origin1X", p.mOrigin1X),
        ("origin1Y", p.mOrigin1Y),
        ("origin1Z", p.mOrigin1Z),
        ("origin2X", p.mOrigin2X),
        ("origin2Y", p.mOrigin2Y),
        ("origin2Z", p.mOrigin2Z),
        ("radius", p.mRadius),
        ("height", p.mHeight),
        ("windModifier", p.mWindModifier),
        ("rotation", p.mRotation),
        ("rotationDelta", p.mRotationDelta),
        ("angle1", p.mAngle1),
        ("angle2", p.mAngle2),
        ("angle3", p.mAngle3),
        ("angle1Delta", p.mAngle1Delta),
        ("angle2Delta", p.mAngle2Delta),
        ("angle3Delta", p.mAngle3Delta),
        ("velX", p.mVelX),
        ("velY", p.mVelY),
        ("velZ", p.mVelZ),
        ("accelX", p.mAccelX),
        ("accelY", p.mAccelY),
        ("accelZ", p.mAccelZ),
        ("gravity", p.mGravity),
        ("density", p.mDensity),
        ("variance", p.mVariance),
        ("redStart", p.mRedStart),
        ("greenStart", p.mGreenStart),
        ("blueStart", p.mBlueStart),
        ("redEnd", p.mRedEnd),
        ("greenEnd", p.mGreenEnd),
        ("blueEnd", p.mBlueEnd),
        ("rgbParm", p.mRGBParm),
        ("alphaStart", p.mAlphaStart),
        ("alphaEnd", p.mAlphaEnd),
        ("alphaParm", p.mAlphaParm),
        ("sizeStart", p.mSizeStart),
        ("sizeEnd", p.mSizeEnd),
        ("sizeParm", p.mSizeParm),
        ("size2Start", p.mSize2Start),
        ("size2End", p.mSize2End),
        ("size2Parm", p.mSize2Parm),
        ("lengthStart", p.mLengthStart),
        ("lengthEnd", p.mLengthEnd),
        ("lengthParm", p.mLengthParm),
        ("texCoordS", p.mTexCoordS),
        ("texCoordT", p.mTexCoordT),
        ("elasticity", p.mElasticity),
    ]
}

/// The five handle lists, in declaration order.
///
/// Source: `oracle/codemp/client/FxScheduler.h:172-176`
fn media_fields(p: &CPrimitiveTemplate) -> Vec<(&'static str, Vec<i32>)> {
    vec![
        ("mediaHandles", p.mMediaHandles.handles().to_vec()),
        ("impactFxHandles", p.mImpactFxHandles.handles().to_vec()),
        ("deathFxHandles", p.mDeathFxHandles.handles().to_vec()),
        ("emitterFxHandles", p.mEmitterFxHandles.handles().to_vec()),
        ("playFxHandles", p.mPlayFxHandles.handles().to_vec()),
    ]
}

/// Print one parsed `SEffectTemplate`.
///
/// Source: `tools/fx-oracle/main.cpp:195-235`
fn dump_template(fx: &FxSystem, out: &mut Vec<String>, handle: i32) {
    if handle < 1
        || handle as usize >= FX_MAX_EFFECTS
        || !fx.scheduler.mEffectTemplates[handle as usize].mInUse
    {
        out.push(format!("TEMPLATE {handle} MISSING"));
        return;
    }
    let effect = &fx.scheduler.mEffectTemplates[handle as usize];

    out.push(format!(
        "TEMPLATE {} name {} repeatDelay {} primitiveCount {}",
        handle, effect.mEffectName, effect.mRepeatDelay, effect.mPrimitiveCount
    ));

    for i in 0..effect.mPrimitiveCount as usize {
        let prim = effect.mPrimitives[i].borrow();

        out.push(format!(
            "PRIM {} name {} type {} flags {} spawnFlags {} matImpactFX {} cullRange {} \
             soundRadius {} soundVolume {}",
            i,
            prim.mName,
            prim.mType as i32,
            prim.mFlags,
            prim.mSpawnFlags,
            prim.mMatImpactFX as i32,
            prim.mCullRange,
            prim.mSoundRadius,
            prim.mSoundVolume
        ));

        for (name, range) in range_fields(&prim) {
            out.push(format!(
                "PRIMRANGE {} {} {} {}",
                i,
                name,
                fxf(range.GetMin()),
                fxf(range.GetMax())
            ));
        }

        out.push(format!(
            "PRIMVEC {} min {} max {}",
            i,
            fxv(&prim.mMin),
            fxv(&prim.mMax)
        ));

        for (name, handles) in media_fields(&prim) {
            let mut line = format!("PRIMMEDIA {} {} {}", i, name, handles.len());
            for h in handles {
                line.push_str(&format!(" {h}"));
            }
            out.push(line);
        }
    }
}

/// Print the pool census.
///
/// Source: `tools/fx-oracle/main.cpp:237-242`
fn dump_state(fx: &FxSystem, out: &mut Vec<String>) {
    out.push(format!(
        "STATE activeFx {} drawnFx {} scheduledFx {} nextFree2DEffect {}",
        fx.activeFx,
        fx.drawnFx,
        fx.scheduler.NumScheduledFx(),
        fx.scheduler.mNextFree2DEffect
    ));
}

/// Build the unit-coloured trail quad the `addtrail` command feeds in.
///
/// Source: `tools/fx-oracle/main.cpp:531-554`
fn trail_vert(origin: vec3_t) -> effectTrailVertStruct_t {
    effectTrailVertStruct_t {
        origin,
        rgb: [1.0, 1.0, 1.0],
        destrgb: [1.0, 1.0, 1.0],
        curRGB: [1.0, 1.0, 1.0],
        alpha: 1.0,
        destAlpha: 1.0,
        curAlpha: 1.0,
        ST: [1.0, 1.0],
        destST: [1.0, 1.0],
        curST: [1.0, 1.0],
    }
}

/// Interpret one scenario and return the record stream, header and trailer included.
///
/// Source: `tools/fx-oracle/main.cpp:246-589`
fn run_scenario(stem: &str, script: &str, fixtures: &[(String, String)]) -> String {
    let mut harness = FxHarness::default();
    for (path, text) in fixtures {
        harness.files.insert(path.clone(), text.clone());
    }
    let mut fx = FxSystem::default();
    let mut clock: i32 = 0;

    // The default view: at the origin, looking down +X. The rig hands
    // `FX_InitSystem` the address of this block and the script keeps writing it,
    // so the port's snapshot follows every `refdef` command instead.
    // Source: `tools/fx-oracle/main.cpp:270-276`
    let mut refdef = FxRefdef {
        vieworg: [0.0; 3],
        viewangles: [0.0; 3],
        viewaxis: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        fov_x: 90.0,
        fov_y: 73.739792,
    };
    fx.refdef = refdef;

    harness.out.push(format!("== fx-oracle {stem} =="));

    for line in script.lines() {
        let toks: Vec<&str> = line.split([' ', '\t']).filter(|t| !t.is_empty()).collect();
        let Some(cmd) = toks.first().copied() else {
            continue;
        };
        if cmd.starts_with('#') {
            continue;
        }
        let mut a = Args { cmd, toks, at: 1 };
        let mut host = FxHost::Harness(&mut harness);

        match cmd {
            "seed" => {
                let seed = a.i();
                host.rng().Rand_Init(seed);
            }

            "refdef" => {
                refdef = FxRefdef {
                    vieworg: a.v3(),
                    viewangles: a.v3(),
                    viewaxis: a.axis(),
                    fov_x: a.f(),
                    fov_y: a.f(),
                };
                fx.refdef = refdef;
            }

            "cvar" => {
                let name = a.word();
                let value = a.word();
                match name {
                    "fx_debug" => fx.fx_debug = atoi(value),
                    "fx_countScale" => fx.fx_countScale = atof(value),
                    "fx_nearCull" => fx.fx_nearCull = atof(value),
                    other => panic!("fx-oracle: the harness scripts no cvar {other}"),
                }
            }

            // The rig hands `FX_InitSystem` the address of its own refdef, which
            // the script has already filled. The port takes a null pointer and a
            // snapshot, so the scripted view is restored over the reset.
            "init" => {
                FX_InitSystem(&mut fx, &mut host, core::ptr::null_mut());
                fx.refdef = refdef;
            }

            "register" => {
                let path = a.word();
                let handle = FX_RegisterEffect(&mut fx, &mut host, path);
                harness.out.push(format!("REGISTER {path} -> {handle}"));
            }

            "dumptemplate" => {
                let handle = a.i();
                dump_template(&fx, &mut harness.out, handle);
            }

            "trace" => {
                let mut tr = fx_zero_trace();
                tr.fraction = a.f();
                tr.endpos = a.v3();
                tr.plane.normal = a.v3();
                tr.startsolid = a.i() as _;
                tr.allsolid = a.i() as _;
                tr.surfaceFlags = a.i();
                tr.entityNum = a.i() as _;
                harness.traces.push_back(tr);
            }

            "pointcontents" => {
                let contents = a.i();
                harness.point_contents.push_back(contents);
            }

            "bolt" => {
                let exists = a.b();
                let origin = a.v3();
                let axis = a.axis();
                harness.bolts.push_back((exists, origin, axis));
            }

            "lerporigin" => {
                let origin = a.v3();
                harness.lerp_origins.push_back(origin);
            }

            "playid" => {
                let id = a.i();
                let org = a.v3();
                let fwd = a.v3();
                let vol = a.i();
                let rad = a.i();
                let portal = a.b();
                FX_PlayEffectID(&mut fx, &mut host, id, org, fwd, vol, rad, portal);
            }

            "play" => {
                let path = a.word();
                let org = a.v3();
                let fwd = a.v3();
                let vol = a.i();
                let rad = a.i();
                FX_PlayEffect(&mut fx, &mut host, path, org, fwd, vol, rad);
            }

            "playbolted" => {
                let id = a.i();
                let org = a.v3();
                let bolt_info = a.i();
                let i_ghoul2 = a.i();
                let i_loop_time = a.i();
                let is_relative = a.b();
                FX_PlayBoltedEffectID(
                    &mut fx,
                    &mut host,
                    id,
                    org,
                    bolt_info,
                    i_ghoul2,
                    i_loop_time,
                    is_relative,
                );
            }

            "playentity" => {
                let id = a.i();
                let org = a.v3();
                let axis = a.axis();
                let bolt_info = a.i();
                let ent_num = a.i();
                let vol = a.i();
                let rad = a.i();
                FX_PlayEntityEffectID(
                    &mut fx, &mut host, id, org, axis, bolt_info, ent_num, vol, rad,
                );
            }

            "playaxis" => {
                let path = a.word();
                let org = a.v3();
                let axis = a.axis();
                let bolt_info = a.i();
                let i_ghoul2 = a.i();
                let fx_parm = a.i();
                let vol = a.i();
                let rad = a.i();
                let i_loop_time = a.i();
                let is_relative = a.b();
                fx_play_effect_file_axis(
                    &mut fx,
                    &mut host,
                    path,
                    Some(org),
                    axis,
                    bolt_info,
                    i_ghoul2,
                    fx_parm,
                    vol,
                    rad,
                    i_loop_time,
                    is_relative,
                );
            }

            "stop" => {
                let path = a.word();
                let bolt_info = a.i();
                let portal = a.b();
                fx_stop_effect(&mut fx, path, bolt_info, portal);
            }

            "addline" => {
                let start = a.v3();
                let end = a.v3();
                let size1 = a.f();
                let size2 = a.f();
                let size_parm = a.f();
                let a1 = a.f();
                let a2 = a.f();
                let a_parm = a.f();
                let s_rgb = a.v3();
                let e_rgb = a.v3();
                let rgb_parm = a.f();
                let kill_time = a.i();
                let shader = a.i();
                let flags = a.i();
                FX_AddLine(
                    &mut fx,
                    &mut host,
                    start,
                    end,
                    size1,
                    size2,
                    size_parm,
                    a1,
                    a2,
                    a_parm,
                    s_rgb,
                    e_rgb,
                    rgb_parm,
                    kill_time,
                    shader,
                    flags,
                    EMatImpactEffect::MATIMPACTFX_NONE,
                    -1,
                    0,
                    -1,
                    -1,
                    -1,
                );
            }

            "addelectricity" => {
                let start = a.v3();
                let end = a.v3();
                let size1 = a.f();
                let size2 = a.f();
                let size_parm = a.f();
                let a1 = a.f();
                let a2 = a.f();
                let a_parm = a.f();
                let s_rgb = a.v3();
                let e_rgb = a.v3();
                let rgb_parm = a.f();
                let chaos = a.f();
                let kill_time = a.i();
                let shader = a.i();
                let flags = a.i();
                FX_AddElectricity(
                    &mut fx,
                    &mut host,
                    start,
                    end,
                    size1,
                    size2,
                    size_parm,
                    a1,
                    a2,
                    a_parm,
                    s_rgb,
                    e_rgb,
                    rgb_parm,
                    chaos,
                    kill_time,
                    shader,
                    flags,
                    EMatImpactEffect::MATIMPACTFX_NONE,
                    -1,
                    0,
                    -1,
                    -1,
                    -1,
                );
            }

            "addbezier" => {
                let start = a.v3();
                let end = a.v3();
                let c1 = a.v3();
                let c1vel = a.v3();
                let c2 = a.v3();
                let c2vel = a.v3();
                let size1 = a.f();
                let size2 = a.f();
                let size_parm = a.f();
                let a1 = a.f();
                let a2 = a.f();
                let a_parm = a.f();
                let s_rgb = a.v3();
                let e_rgb = a.v3();
                let rgb_parm = a.f();
                let kill_time = a.i();
                let shader = a.i();
                let flags = a.i();
                FX_AddBezier(
                    &mut fx, &mut host, start, end, c1, c1vel, c2, c2vel, size1, size2, size_parm,
                    a1, a2, a_parm, s_rgb, e_rgb, rgb_parm, kill_time, shader, flags,
                );
            }

            // Always three verts, which keeps the command line finite.
            "addpoly" => {
                let num_verts = a.i();
                let verts: Vec<vec3_t> = (0..3).map(|_| a.v3()).collect();
                let st: Vec<[f32; 2]> = (0..3).map(|_| [a.f(), a.f()]).collect();
                let vel = a.v3();
                let accel = a.v3();
                let a1 = a.f();
                let a2 = a.f();
                let a_parm = a.f();
                let rgb1 = a.v3();
                let rgb2 = a.v3();
                let rgb_parm = a.f();
                let rot_delta = a.v3();
                let bounce = a.f();
                let motion_delay = a.i();
                let kill_time = a.i();
                let shader = a.i();
                let flags = a.i();
                FX_AddPoly(
                    &mut fx,
                    &mut host,
                    &verts,
                    &st,
                    num_verts,
                    vel,
                    accel,
                    a1,
                    a2,
                    a_parm,
                    rgb1,
                    rgb2,
                    rgb_parm,
                    rot_delta,
                    bounce,
                    motion_delay,
                    kill_time,
                    shader,
                    flags,
                );
            }

            // The `CG_FX_ADDSPRITE` arm: `FX_AddParticle` with rgb 1 1 1.
            // Source: `oracle/codemp/client/cl_cgame.cpp:1210-1229`
            "addsprite" => {
                let org = a.v3();
                let vel = a.v3();
                let accel = a.v3();
                let scale = a.f();
                let dscale = a.f();
                let s_alpha = a.f();
                let e_alpha = a.f();
                let rotation = a.f();
                let bounce = a.f();
                let life = a.i();
                let shader = a.i();
                let flags = a.i();
                let rgb: vec3_t = [1.0, 1.0, 1.0];
                FX_AddParticle(
                    &mut fx,
                    &mut host,
                    org,
                    vel,
                    accel,
                    scale,
                    dscale,
                    0.0,
                    s_alpha,
                    e_alpha,
                    0.0,
                    rgb,
                    rgb,
                    0.0,
                    rotation,
                    0.0,
                    [0.0; 3],
                    [0.0; 3],
                    bounce,
                    0,
                    0,
                    life,
                    shader,
                    flags,
                    EMatImpactEffect::MATIMPACTFX_NONE,
                    -1,
                    0,
                    -1,
                    -1,
                    -1,
                );
            }

            "addtrail" => {
                let verts = [
                    trail_vert(a.v3()),
                    trail_vert(a.v3()),
                    trail_vert(a.v3()),
                    trail_vert(a.v3()),
                ];
                let arg = effectTrailArgStruct_t {
                    mVerts: verts,
                    mShader: a.i(),
                    mSetFlags: a.i(),
                    mKillTime: a.i(),
                };
                FX_FeedTrail(&mut fx, &mut host, &arg);
            }

            // `AdjustTime` takes an absolute time, so the harness accumulates.
            // Source: `oracle/codemp/client/FxSystem.cpp:53-80`
            "advance" => {
                clock += a.i();
                FX_AdjustTime(&mut fx, clock);
                harness.out.push(format!("TIME {clock}"));
            }

            "addscheduled" => {
                let portal = a.b();
                FX_AddScheduledEffects(&mut fx, &mut host, portal);
            }

            "draw2d" => {
                let xscale = a.f();
                let yscale = a.f();
                FX_Draw2DEffects(&mut fx, &mut host, xscale, yscale);
            }

            "dumpstate" => dump_state(&fx, &mut harness.out),

            "free" => {
                FX_FreeSystem(&mut fx, &mut host);
            }

            "reset" => {
                FX_Free(&mut fx, &mut host, false);
            }

            other => panic!("fx-oracle: unknown command '{other}'"),
        }
    }

    harness.out.push("== end ==".to_string());
    format!("{}\n", harness.out.join("\n"))
}

#[test]
fn matches_oracle_goldens() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../tools/fx-oracle");

    // Every scenario sees the whole fixture tree, the way the rig's resolver does.
    let mut fixtures: Vec<(String, String)> = Vec::new();
    for entry in std::fs::read_dir(root.join("fixtures")).expect("fixtures dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("efx") {
            continue;
        }
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let text = std::fs::read_to_string(&path).expect("read fixture");
        fixtures.push((format!("effects/{stem}.efx"), text));
    }
    fixtures.sort();

    let mut scenarios: Vec<PathBuf> = std::fs::read_dir(root.join("scenarios"))
        .expect("scenarios dir")
        .map(|e| e.expect("dir entry").path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("fxs"))
        .collect();
    scenarios.sort();

    let mut checked = 0;
    let mut failures: Vec<String> = Vec::new();
    for scenario in &scenarios {
        let stem = scenario.file_stem().unwrap().to_str().unwrap().to_string();
        let script = std::fs::read_to_string(scenario).expect("read scenario");
        let golden_path = root.join("golden").join(format!("{stem}.txt"));
        let golden = std::fs::read_to_string(&golden_path).unwrap_or_else(|_| {
            panic!("missing golden {golden_path:?} — run tools/fx-oracle/run.sh --regen")
        });

        let produced = run_scenario(&stem, &script, &fixtures);
        if produced != golden {
            let want: Vec<&str> = golden.lines().collect();
            let got: Vec<&str> = produced.lines().collect();
            let at = want
                .iter()
                .zip(got.iter())
                .position(|(w, g)| w != g)
                .unwrap_or(want.len().min(got.len()));
            failures.push(format!(
                "{stem}: record {at} of {}\n    want: {:?}\n    got:  {:?}",
                want.len(),
                want.get(at),
                got.get(at)
            ));
        }
        checked += 1;
    }

    assert!(
        failures.is_empty(),
        "{} of {checked} scenarios diverge from the C++ oracle:\n  {}",
        failures.len(),
        failures.join("\n  ")
    );

    assert!(
        checked >= SCENARIO_FLOOR,
        "expected the full scenario set, found {checked}"
    );
}
