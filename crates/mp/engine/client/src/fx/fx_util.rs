//! Raven `FxUtil.cpp` — the live effect pool and the eleven spawn factories.
//!
//! DEC-61.2 moves `effectList`, `nextValidEffect`, `activeFx`, `drawnFx`, and
//! `fxInitialized` onto `FxSystem`. The pool stays a fixed slot array walked in
//! index order, because the draw order is parity surface.
//!
//! Source: `oracle/codemp/client/FxUtil.cpp`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_engine_qcommon::cvar_fns::Cvar_Get;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::shared::cvar::{CVAR_ARCHIVE, CVAR_TEMP};
use native_math::vector::vec3_t;

use crate::fx::cbezier::CBezier;
use crate::fx::ccylinder::CCylinder;
use crate::fx::celectricity::CElectricity;
use crate::fx::cemitter::CEmitter;
use crate::fx::cflash::CFlash;
use crate::fx::clight::CLight;
use crate::fx::cline::CLine;
use crate::fx::coriented_particle::COrientedParticle;
use crate::fx::cparticle::ParticleCore;
use crate::fx::cpoly::{CPoly, MAX_CPOLY_VERTS};
use crate::fx::ctail::CTail;
use crate::fx::emat_impact_effect::EMatImpactEffect;
use crate::fx::fx_flags::{
    FX_ALPHA_PARM_MASK, FX_ALPHA_WAVE, FX_KILL_ON_IMPACT, FX_LENGTH_PARM_MASK, FX_LENGTH_WAVE,
    FX_RELATIVE, FX_RGB_PARM_MASK, FX_RGB_WAVE, FX_SIZE2_PARM_MASK, FX_SIZE2_WAVE,
    FX_SIZE_PARM_MASK, FX_SIZE_WAVE,
};
use crate::fx::fx_host::FxHost;
use crate::fx::fx_primitive::FxPrimitive;
use crate::fx::fx_scheduler::fx_scheduler_clean;
use crate::fx::fx_system::{FxRefdef, FxSystem};

/// How many effects the pool holds at once.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:10`
pub const MAX_EFFECTS: usize = 1800;

/// Raven's local `PI`, which is not the `q_shared.h` value.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:20`
pub const FX_PI: f32 = 3.14159;

/// One slot of the live pool.
///
/// Raven tests `mEffect != 0` for occupancy. The port keeps `mInUse` beside the
/// payload, because the update loop moves the primitive out and back while a
/// nested spawn may look for a free slot.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:13-18`
#[derive(Clone, Debug, Default)]
pub struct SEffectList {
    pub mEffect: Option<FxPrimitive>,
    pub mKillTime: i32,
    pub mPortal: bool,
    pub mInUse: bool,
}

/// Raven `FX_Free` — drop every live effect, then clean the scheduler.
///
/// Raven deletes without calling `Die`, so no death effect fires here.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:35-51`
pub fn FX_Free(fx: &mut FxSystem, host: &mut FxHost<'_, '_>, templates: bool) -> bool {
    for slot in fx.effects.iter_mut() {
        slot.mEffect = None;
        slot.mInUse = false;
    }

    fx.activeFx = 0;

    fx_scheduler_clean(fx, host, templates, 0);
    true
}

/// Raven `FX_Stop` — drop the live effects but keep the templates.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:58-73`
pub fn FX_Stop(fx: &mut FxSystem, host: &mut FxHost<'_, '_>) {
    for slot in fx.effects.iter_mut() {
        slot.mEffect = None;
        slot.mInUse = false;
    }

    fx.activeFx = 0;

    fx_scheduler_clean(fx, host, false, 0);
}

/// Raven `FX_Init` — prep the pool, register the cvars, reset the clock.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:80-104`
pub fn FX_Init(fx: &mut FxSystem, host: &mut FxHost<'_, '_>, refdef: *mut refdef_t) -> c_int {
    if !fx.fxInitialized {
        fx.fxInitialized = true;

        for slot in fx.effects.iter_mut() {
            slot.mEffect = None;
            slot.mInUse = false;
        }
    }
    fx.next_valid_effect = 0;

    if let FxHost::Engine { view, .. } = host {
        fx.cvar_fx_debug = Some(Cvar_Get(view, "fx_debug", "0", CVAR_TEMP));
        fx.cvar_fx_countScale = Some(Cvar_Get(view, "fx_countScale", "1", CVAR_ARCHIVE));
        fx.cvar_fx_nearCull = Some(Cvar_Get(view, "fx_nearCull", "16", CVAR_ARCHIVE));
    }

    fx.clock.ReInit();
    fx.refdef_ptr = refdef;
    fx.refdef = FxRefdef::default();

    // Raven returns `true` through an `int` return type.
    1
}

/// Raven `FX_SetRefDef`.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:106-109`
pub fn FX_SetRefDef(fx: &mut FxSystem, refdef: *mut refdef_t) {
    fx.refdef_ptr = refdef;
}

/// Raven `FX_FreeMember` — run the death effect, free the slot, remember it.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:114-124`
pub fn FX_FreeMember(fx: &mut FxSystem, host: &mut FxHost<'_, '_>, index: usize) {
    if let Some(mut effect) = fx.effects[index].mEffect.take() {
        effect.Die(host, fx);
    }
    fx.effects[index].mInUse = false;

    // May as well mark this to be used next
    fx.next_valid_effect = index;

    fx.activeFx -= 1;
}

/// Raven `FX_GetValidEffect` — the next free slot, trashing slot zero if full.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:135-164`
pub fn FX_GetValidEffect(fx: &mut FxSystem, host: &mut FxHost<'_, '_>) -> usize {
    if !fx.effects[fx.next_valid_effect].mInUse {
        return fx.next_valid_effect;
    }

    // Blah..plow through the list till we find something that is currently untainted
    for i in 0..MAX_EFFECTS {
        if !fx.effects[i].mInUse {
            return i;
        }
    }

    host.Print("FX system out of effects\n");

    // Hmmm.. just trashing the first effect in the list is a poor approach
    FX_FreeMember(fx, host, 0);

    fx.next_valid_effect
}

/// Raven `FX_Add` — update every live effect and hand the survivors to the renderer.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:171-215`
pub fn FX_Add(fx: &mut FxSystem, host: &mut FxHost<'_, '_>, portal: bool) {
    fx.drawnFx = 0;

    // but stop when there can't be any more left!
    let mut num_fx = fx.activeFx;
    for i in 0..MAX_EFFECTS {
        if num_fx == 0 {
            break;
        }
        if !fx.effects[i].mInUse {
            continue;
        }

        num_fx -= 1;
        if portal != fx.effects[i].mPortal {
            // this one does not render in this scene
            continue;
        }

        // Effect is active
        if fx.clock.mTime > fx.effects[i].mKillTime {
            // Clean up old effects, calling any death effects as needed.
            // This flag just has to be cleared otherwise death effects might not
            // happen correctly.
            if let Some(effect) = fx.effects[i].mEffect.as_mut() {
                effect.core_mut().ClearFlags(FX_KILL_ON_IMPACT);
            }
            FX_FreeMember(fx, host, i);
        } else {
            // Borrow the primitive out for the update. The slot stays in use, so a
            // nested spawn cannot claim it while the update runs.
            let Some(mut effect) = fx.effects[i].mEffect.take() else {
                continue;
            };
            let alive = effect.Update(host, fx);
            fx.effects[i].mEffect = Some(effect);

            if !alive {
                // We've been marked for death
                FX_FreeMember(fx, host, i);
            }
        }
    }

    if fx.fx_debug != 0 && !portal {
        let active = fx.activeFx;
        let drawn = fx.drawnFx;
        let scheduled = fx.scheduler.NumScheduledFx();
        host.Print(&format!("Active    FX: {active}\n"));
        host.Print(&format!("Drawn     FX: {drawn}\n"));
        host.Print(&format!("Scheduled FX: {scheduled}\n"));
    }
}

/// Raven `FX_AddPrimitive` — hand the built primitive to the pool.
///
/// Returns the slot it landed in, so `FX_AddElectricity` can run `Initialize`
/// on the stored copy the way Raven runs it through the still-live pointer.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:226-239`
pub fn FX_AddPrimitive(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    mut effect: FxPrimitive,
    kill_time: i32,
) -> usize {
    let index = FX_GetValidEffect(fx, host);

    // Stash these in the primitive so it has easy access to the vals
    effect.core_mut().SetTimeStart(fx.clock.mTime);
    effect.core_mut().SetTimeEnd(fx.clock.mTime + kill_time);

    fx.effects[index].mKillTime = fx.clock.mTime + kill_time;
    // global set in AddScheduledEffects
    fx.effects[index].mPortal = fx.gEffectsInPortal;
    fx.effects[index].mEffect = Some(effect);
    fx.effects[index].mInUse = true;

    fx.activeFx += 1;

    index
}

/// The RGB, alpha, size, size2, and length parm blocks every factory repeats.
///
/// A wave parm becomes a frequency in radians per millisecond. Any other parm is
/// a percentage of the life, offset by the current time.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:283-317`
fn parm_value(
    flags: i32,
    parm_mask: i32,
    wave: i32,
    parm: f32,
    kill_time: i32,
    time: i32,
) -> Option<f32> {
    if (flags & parm_mask) == wave {
        Some(parm * FX_PI * 0.001)
    } else if flags & parm_mask != 0 {
        // parm should be a value from 0-100..
        Some(parm * 0.01 * kill_time as f32 + time as f32)
    } else {
        None
    }
}

/// Raven `FX_AddParticle`.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:244-335`
#[allow(clippy::too_many_arguments)]
pub fn FX_AddParticle(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    org: vec3_t,
    vel: vec3_t,
    accel: vec3_t,
    size1: f32,
    size2: f32,
    size_parm: f32,
    alpha1: f32,
    alpha2: f32,
    alpha_parm: f32,
    s_rgb: vec3_t,
    e_rgb: vec3_t,
    rgb_parm: f32,
    rotation: f32,
    rotation_delta: f32,
    min: vec3_t,
    max: vec3_t,
    elasticity: f32,
    death_id: i32,
    impact_id: i32,
    kill_time: i32,
    shader: i32,
    flags: i32,
    mat_impact_fx: EMatImpactEffect,
    fx_parm: i32,
    i_ghoul2: i32,
    ent_num: i32,
    model_num: i32,
    bolt_num: i32,
) {
    if fx.clock.mFrameTime < 1 {
        // disallow adding effects when the system is paused
        return;
    }

    let mut p = ParticleCore::default();
    let time = fx.clock.mTime;

    if flags & FX_RELATIVE != 0 && i_ghoul2 > 0 {
        p.e.SetOrigin1(None);
        p.SetOrgOffset(Some(org));
        p.SetBoltinfo(i_ghoul2, ent_num, model_num, bolt_num);
    } else {
        p.e.SetOrigin1(Some(org));
    }
    // Raven sets the origin a second time, outside the branch.
    p.e.SetOrigin1(Some(org));
    p.e.SetMatImpactFX(mat_impact_fx);
    p.e.SetMatImpactParm(fx_parm);
    p.SetVel(Some(vel));
    p.SetAccel(Some(accel));

    p.SetRGBStart(Some(s_rgb));
    p.SetRGBEnd(Some(e_rgb));
    if let Some(v) = parm_value(
        flags,
        FX_RGB_PARM_MASK,
        FX_RGB_WAVE,
        rgb_parm,
        kill_time,
        time,
    ) {
        p.SetRGBParm(v);
    }

    p.SetAlphaStart(alpha1);
    p.SetAlphaEnd(alpha2);
    if let Some(v) = parm_value(
        flags,
        FX_ALPHA_PARM_MASK,
        FX_ALPHA_WAVE,
        alpha_parm,
        kill_time,
        time,
    ) {
        p.SetAlphaParm(v);
    }

    p.SetSizeStart(size1);
    p.SetSizeEnd(size2);
    if let Some(v) = parm_value(
        flags,
        FX_SIZE_PARM_MASK,
        FX_SIZE_WAVE,
        size_parm,
        kill_time,
        time,
    ) {
        p.SetSizeParm(v);
    }

    p.e.SetFlags(flags);
    p.SetShader(shader);
    p.SetRotation(rotation);
    p.SetRotationDelta(rotation_delta);
    p.SetElasticity(elasticity);
    p.e.SetMin(Some(min));
    p.e.SetMax(Some(max));
    p.e.SetDeathFxID(death_id);
    p.e.SetImpactFxID(impact_id);

    p.Init(host);

    FX_AddPrimitive(fx, host, FxPrimitive::Particle(p), kill_time);
}

/// Raven `FX_AddLine`.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:342-422`
#[allow(clippy::too_many_arguments)]
pub fn FX_AddLine(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    start: vec3_t,
    end: vec3_t,
    size1: f32,
    size2: f32,
    size_parm: f32,
    alpha1: f32,
    alpha2: f32,
    alpha_parm: f32,
    s_rgb: vec3_t,
    e_rgb: vec3_t,
    rgb_parm: f32,
    kill_time: i32,
    shader: i32,
    flags: i32,
    mat_impact_fx: EMatImpactEffect,
    fx_parm: i32,
    i_ghoul2: i32,
    ent_num: i32,
    model_num: i32,
    bolt_num: i32,
) {
    if fx.clock.mFrameTime < 1 {
        return;
    }

    let mut l = CLine::default();
    let time = fx.clock.mTime;

    if flags & FX_RELATIVE != 0 && i_ghoul2 > 0 {
        l.p.e.SetOrigin1(None);
        // offset from bolt pos
        l.p.SetOrgOffset(Some(start));
        // vel is the vector offset from bolt+orgOffset
        l.p.SetVel(Some(end));
        l.p.SetBoltinfo(i_ghoul2, ent_num, model_num, bolt_num);
    } else {
        l.p.e.SetOrigin1(Some(start));
        l.SetOrigin2(end);
    }
    l.p.e.SetMatImpactFX(mat_impact_fx);
    l.p.e.SetMatImpactParm(fx_parm);

    l.p.SetRGBStart(Some(s_rgb));
    l.p.SetRGBEnd(Some(e_rgb));
    if let Some(v) = parm_value(
        flags,
        FX_RGB_PARM_MASK,
        FX_RGB_WAVE,
        rgb_parm,
        kill_time,
        time,
    ) {
        l.p.SetRGBParm(v);
    }

    l.p.SetAlphaStart(alpha1);
    l.p.SetAlphaEnd(alpha2);
    if let Some(v) = parm_value(
        flags,
        FX_ALPHA_PARM_MASK,
        FX_ALPHA_WAVE,
        alpha_parm,
        kill_time,
        time,
    ) {
        l.p.SetAlphaParm(v);
    }

    l.p.SetSizeStart(size1);
    l.p.SetSizeEnd(size2);
    if let Some(v) = parm_value(
        flags,
        FX_SIZE_PARM_MASK,
        FX_SIZE_WAVE,
        size_parm,
        kill_time,
        time,
    ) {
        l.p.SetSizeParm(v);
    }

    l.p.SetShader(shader);
    l.p.e.SetFlags(flags);

    l.p.e.SetSTScale(1.0, 1.0);

    FX_AddPrimitive(fx, host, FxPrimitive::Line(l), kill_time);
}

/// Raven `FX_AddElectricity`.
///
/// `Initialize` runs after the primitive lands in the pool, because it reads the
/// start and end times `FX_AddPrimitive` stamps.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:428-514`
#[allow(clippy::too_many_arguments)]
pub fn FX_AddElectricity(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    start: vec3_t,
    end: vec3_t,
    size1: f32,
    size2: f32,
    size_parm: f32,
    alpha1: f32,
    alpha2: f32,
    alpha_parm: f32,
    s_rgb: vec3_t,
    e_rgb: vec3_t,
    rgb_parm: f32,
    chaos: f32,
    kill_time: i32,
    shader: i32,
    flags: i32,
    mat_impact_fx: EMatImpactEffect,
    fx_parm: i32,
    i_ghoul2: i32,
    ent_num: i32,
    model_num: i32,
    bolt_num: i32,
) {
    if fx.clock.mFrameTime < 1 {
        return;
    }

    let mut el = CElectricity::default();
    let time = fx.clock.mTime;

    if flags & FX_RELATIVE != 0 && i_ghoul2 > 0 {
        el.l.p.e.SetOrigin1(None);
        el.l.p.SetOrgOffset(Some(start));
        el.l.p.SetVel(Some(end));
        el.l.p.SetBoltinfo(i_ghoul2, ent_num, model_num, bolt_num);
    } else {
        el.l.p.e.SetOrigin1(Some(start));
        el.l.SetOrigin2(end);
    }
    el.l.p.e.SetMatImpactFX(mat_impact_fx);
    el.l.p.e.SetMatImpactParm(fx_parm);

    el.l.p.SetRGBStart(Some(s_rgb));
    el.l.p.SetRGBEnd(Some(e_rgb));
    if let Some(v) = parm_value(
        flags,
        FX_RGB_PARM_MASK,
        FX_RGB_WAVE,
        rgb_parm,
        kill_time,
        time,
    ) {
        el.l.p.SetRGBParm(v);
    }

    el.l.p.SetAlphaStart(alpha1);
    el.l.p.SetAlphaEnd(alpha2);
    if let Some(v) = parm_value(
        flags,
        FX_ALPHA_PARM_MASK,
        FX_ALPHA_WAVE,
        alpha_parm,
        kill_time,
        time,
    ) {
        el.l.p.SetAlphaParm(v);
    }

    el.l.p.SetSizeStart(size1);
    el.l.p.SetSizeEnd(size2);
    if let Some(v) = parm_value(
        flags,
        FX_SIZE_PARM_MASK,
        FX_SIZE_WAVE,
        size_parm,
        kill_time,
        time,
    ) {
        el.l.p.SetSizeParm(v);
    }

    el.l.p.SetShader(shader);
    el.l.p.e.SetFlags(flags);
    el.SetChaos(chaos);

    el.l.p.e.SetSTScale(1.0, 1.0);

    let index = FX_AddPrimitive(fx, host, FxPrimitive::Electricity(el), kill_time);
    let now = fx.clock.mTime;
    let draw = host.rng().flrand(0.0, 1.0);
    if let Some(FxPrimitive::Electricity(stored)) = fx.effects[index].mEffect.as_mut() {
        stored.Initialize(now, draw);
    }
}

/// Raven `FX_AddTail`.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:520-621`
#[allow(clippy::too_many_arguments)]
pub fn FX_AddTail(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    org: vec3_t,
    vel: vec3_t,
    accel: vec3_t,
    size1: f32,
    size2: f32,
    size_parm: f32,
    length1: f32,
    length2: f32,
    length_parm: f32,
    alpha1: f32,
    alpha2: f32,
    alpha_parm: f32,
    s_rgb: vec3_t,
    e_rgb: vec3_t,
    rgb_parm: f32,
    min: vec3_t,
    max: vec3_t,
    elasticity: f32,
    death_id: i32,
    impact_id: i32,
    kill_time: i32,
    shader: i32,
    flags: i32,
    mat_impact_fx: EMatImpactEffect,
    fx_parm: i32,
    i_ghoul2: i32,
    ent_num: i32,
    model_num: i32,
    bolt_num: i32,
) {
    if fx.clock.mFrameTime < 1 {
        return;
    }

    let mut t = CTail::default();
    let time = fx.clock.mTime;

    if flags & FX_RELATIVE != 0 && i_ghoul2 > 0 {
        t.p.e.SetOrigin1(None);
        t.p.SetOrgOffset(Some(org));
        t.p.SetBoltinfo(i_ghoul2, ent_num, model_num, bolt_num);
    } else {
        t.p.e.SetOrigin1(Some(org));
    }
    t.p.e.SetMatImpactFX(mat_impact_fx);
    t.p.e.SetMatImpactParm(fx_parm);
    t.p.SetVel(Some(vel));
    t.p.SetAccel(Some(accel));

    t.p.SetRGBStart(Some(s_rgb));
    t.p.SetRGBEnd(Some(e_rgb));
    if let Some(v) = parm_value(
        flags,
        FX_RGB_PARM_MASK,
        FX_RGB_WAVE,
        rgb_parm,
        kill_time,
        time,
    ) {
        t.p.SetRGBParm(v);
    }

    t.p.SetAlphaStart(alpha1);
    t.p.SetAlphaEnd(alpha2);
    if let Some(v) = parm_value(
        flags,
        FX_ALPHA_PARM_MASK,
        FX_ALPHA_WAVE,
        alpha_parm,
        kill_time,
        time,
    ) {
        t.p.SetAlphaParm(v);
    }

    t.p.SetSizeStart(size1);
    t.p.SetSizeEnd(size2);
    if let Some(v) = parm_value(
        flags,
        FX_SIZE_PARM_MASK,
        FX_SIZE_WAVE,
        size_parm,
        kill_time,
        time,
    ) {
        t.p.SetSizeParm(v);
    }

    t.SetLengthStart(length1);
    t.SetLengthEnd(length2);
    if let Some(v) = parm_value(
        flags,
        FX_LENGTH_PARM_MASK,
        FX_LENGTH_WAVE,
        length_parm,
        kill_time,
        time,
    ) {
        t.SetLengthParm(v);
    }

    t.p.e.SetFlags(flags);
    t.p.SetShader(shader);
    t.p.SetElasticity(elasticity);
    t.p.e.SetMin(Some(min));
    t.p.e.SetMax(Some(max));
    t.p.e.SetSTScale(1.0, 1.0);
    t.p.e.SetDeathFxID(death_id);
    t.p.e.SetImpactFxID(impact_id);

    FX_AddPrimitive(fx, host, FxPrimitive::Tail(t), kill_time);
}

/// Raven `FX_AddCylinder`.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:627-737`
#[allow(clippy::too_many_arguments)]
pub fn FX_AddCylinder(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    start: vec3_t,
    normal: vec3_t,
    size1s: f32,
    size1e: f32,
    size1_parm: f32,
    size2s: f32,
    size2e: f32,
    size2_parm: f32,
    length1: f32,
    length2: f32,
    length_parm: f32,
    alpha1: f32,
    alpha2: f32,
    alpha_parm: f32,
    rgb1: vec3_t,
    rgb2: vec3_t,
    rgb_parm: f32,
    kill_time: i32,
    shader: i32,
    flags: i32,
    mat_impact_fx: EMatImpactEffect,
    fx_parm: i32,
    i_ghoul2: i32,
    ent_num: i32,
    model_num: i32,
    bolt_num: i32,
    trace_end: bool,
) {
    if fx.clock.mFrameTime < 1 {
        return;
    }

    let mut c = CCylinder::default();
    let time = fx.clock.mTime;

    if flags & FX_RELATIVE != 0 && i_ghoul2 > 0 {
        c.t.p.e.SetOrigin1(None);
        c.t.p.SetOrgOffset(Some(start));
        c.t.p.SetBoltinfo(i_ghoul2, ent_num, model_num, bolt_num);
    } else {
        c.t.p.e.SetOrigin1(Some(start));
    }
    c.SetTraceEnd(trace_end);

    c.t.p.e.SetMatImpactFX(mat_impact_fx);
    c.t.p.e.SetMatImpactParm(fx_parm);
    // Raven sets the origin a second time, outside the branch.
    c.t.p.e.SetOrigin1(Some(start));
    c.SetNormal(normal);

    c.t.p.SetRGBStart(Some(rgb1));
    c.t.p.SetRGBEnd(Some(rgb2));
    if let Some(v) = parm_value(
        flags,
        FX_RGB_PARM_MASK,
        FX_RGB_WAVE,
        rgb_parm,
        kill_time,
        time,
    ) {
        c.t.p.SetRGBParm(v);
    }

    c.t.p.SetSizeStart(size1s);
    c.t.p.SetSizeEnd(size1e);
    if let Some(v) = parm_value(
        flags,
        FX_SIZE_PARM_MASK,
        FX_SIZE_WAVE,
        size1_parm,
        kill_time,
        time,
    ) {
        c.t.p.SetSizeParm(v);
    }

    c.SetSize2Start(size2s);
    c.SetSize2End(size2e);
    if let Some(v) = parm_value(
        flags,
        FX_SIZE2_PARM_MASK,
        FX_SIZE2_WAVE,
        size2_parm,
        kill_time,
        time,
    ) {
        c.SetSize2Parm(v);
    }

    c.t.SetLengthStart(length1);
    c.t.SetLengthEnd(length2);
    if let Some(v) = parm_value(
        flags,
        FX_LENGTH_PARM_MASK,
        FX_LENGTH_WAVE,
        length_parm,
        kill_time,
        time,
    ) {
        c.t.SetLengthParm(v);
    }

    c.t.p.SetAlphaStart(alpha1);
    c.t.p.SetAlphaEnd(alpha2);
    if let Some(v) = parm_value(
        flags,
        FX_ALPHA_PARM_MASK,
        FX_ALPHA_WAVE,
        alpha_parm,
        kill_time,
        time,
    ) {
        c.t.p.SetAlphaParm(v);
    }

    c.t.p.SetShader(shader);
    c.t.p.e.SetFlags(flags);

    FX_AddPrimitive(fx, host, FxPrimitive::Cylinder(c), kill_time);
}

/// Raven `FX_AddEmitter`.
///
/// Raven asserts and does nothing when a bolted emitter is requested, so the
/// relative branch stays unimplemented here too.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:742-835`
#[allow(clippy::too_many_arguments)]
pub fn FX_AddEmitter(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    org: vec3_t,
    vel: vec3_t,
    accel: vec3_t,
    size1: f32,
    size2: f32,
    size_parm: f32,
    alpha1: f32,
    alpha2: f32,
    alpha_parm: f32,
    rgb1: vec3_t,
    rgb2: vec3_t,
    rgb_parm: f32,
    angs: vec3_t,
    delta_angs: vec3_t,
    min: vec3_t,
    max: vec3_t,
    elasticity: f32,
    death_id: i32,
    impact_id: i32,
    emitter_id: i32,
    density: f32,
    variance: f32,
    kill_time: i32,
    model: i32,
    flags: i32,
    mat_impact_fx: EMatImpactEffect,
    fx_parm: i32,
) {
    if fx.clock.mFrameTime < 1 {
        return;
    }

    let mut em = CEmitter::default();
    let time = fx.clock.mTime;

    em.p.e.SetMatImpactFX(mat_impact_fx);
    em.p.e.SetMatImpactParm(fx_parm);
    em.p.e.SetOrigin1(Some(org));
    em.p.SetVel(Some(vel));
    em.p.SetAccel(Some(accel));

    em.p.SetRGBStart(Some(rgb1));
    em.p.SetRGBEnd(Some(rgb2));
    if let Some(v) = parm_value(
        flags,
        FX_RGB_PARM_MASK,
        FX_RGB_WAVE,
        rgb_parm,
        kill_time,
        time,
    ) {
        em.p.SetRGBParm(v);
    }

    em.p.SetSizeStart(size1);
    em.p.SetSizeEnd(size2);
    if let Some(v) = parm_value(
        flags,
        FX_SIZE_PARM_MASK,
        FX_SIZE_WAVE,
        size_parm,
        kill_time,
        time,
    ) {
        em.p.SetSizeParm(v);
    }

    em.p.SetAlphaStart(alpha1);
    em.p.SetAlphaEnd(alpha2);
    if let Some(v) = parm_value(
        flags,
        FX_ALPHA_PARM_MASK,
        FX_ALPHA_WAVE,
        alpha_parm,
        kill_time,
        time,
    ) {
        em.p.SetAlphaParm(v);
    }

    em.SetAngles(Some(angs));
    em.SetAngleDelta(Some(delta_angs));
    em.p.e.SetFlags(flags);
    em.SetModel(model);
    em.p.SetElasticity(elasticity);
    em.p.e.SetMin(Some(min));
    em.p.e.SetMax(Some(max));
    em.p.e.SetDeathFxID(death_id);
    em.p.e.SetImpactFxID(impact_id);
    em.SetEmitterFxID(emitter_id);
    em.SetDensity(density);
    em.SetVariance(variance);
    em.SetOldTime(time);

    em.SetLastOrg(Some(org));
    em.SetLastVel(Some(vel));

    FX_AddPrimitive(fx, host, FxPrimitive::Emitter(em), kill_time);
}

/// Raven `FX_AddLight`.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:840-902`
#[allow(clippy::too_many_arguments)]
pub fn FX_AddLight(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    org: vec3_t,
    size1: f32,
    size2: f32,
    size_parm: f32,
    rgb1: vec3_t,
    rgb2: vec3_t,
    rgb_parm: f32,
    kill_time: i32,
    flags: i32,
    mat_impact_fx: EMatImpactEffect,
    fx_parm: i32,
    i_ghoul2: i32,
    ent_num: i32,
    model_num: i32,
    bolt_num: i32,
) {
    if fx.clock.mFrameTime < 1 {
        return;
    }

    let mut li = CLight::default();
    let time = fx.clock.mTime;

    if flags & FX_RELATIVE != 0 && i_ghoul2 > 0 {
        li.e.SetOrigin1(None);
        li.SetOrgOffset(Some(org));
        li.SetBoltinfo(i_ghoul2, ent_num, model_num, bolt_num);
    } else {
        li.e.SetOrigin1(Some(org));
    }
    li.e.SetMatImpactFX(mat_impact_fx);
    li.e.SetMatImpactParm(fx_parm);

    li.SetRGBStart(Some(rgb1));
    li.SetRGBEnd(Some(rgb2));
    if let Some(v) = parm_value(
        flags,
        FX_RGB_PARM_MASK,
        FX_RGB_WAVE,
        rgb_parm,
        kill_time,
        time,
    ) {
        li.SetRGBParm(v);
    }

    li.SetSizeStart(size1);
    li.SetSizeEnd(size2);
    if let Some(v) = parm_value(
        flags,
        FX_SIZE_PARM_MASK,
        FX_SIZE_WAVE,
        size_parm,
        kill_time,
        time,
    ) {
        li.SetSizeParm(v);
    }

    li.e.SetFlags(flags);

    FX_AddPrimitive(fx, host, FxPrimitive::Light(li), kill_time);
}

/// Raven `FX_AddOrientedParticle`.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:908-999`
#[allow(clippy::too_many_arguments)]
pub fn FX_AddOrientedParticle(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    org: vec3_t,
    norm: vec3_t,
    vel: vec3_t,
    accel: vec3_t,
    size1: f32,
    size2: f32,
    size_parm: f32,
    alpha1: f32,
    alpha2: f32,
    alpha_parm: f32,
    rgb1: vec3_t,
    rgb2: vec3_t,
    rgb_parm: f32,
    rotation: f32,
    rotation_delta: f32,
    min: vec3_t,
    max: vec3_t,
    bounce: f32,
    death_id: i32,
    impact_id: i32,
    kill_time: i32,
    shader: i32,
    flags: i32,
    mat_impact_fx: EMatImpactEffect,
    fx_parm: i32,
    i_ghoul2: i32,
    ent_num: i32,
    model_num: i32,
    bolt_num: i32,
) {
    if fx.clock.mFrameTime < 1 {
        return;
    }

    let mut o = COrientedParticle::default();
    let time = fx.clock.mTime;

    if flags & FX_RELATIVE != 0 && i_ghoul2 > 0 {
        o.p.e.SetOrigin1(None);
        o.p.SetOrgOffset(Some(org));
        o.p.SetBoltinfo(i_ghoul2, ent_num, model_num, bolt_num);
    } else {
        o.p.e.SetOrigin1(Some(org));
    }
    o.p.e.SetMatImpactFX(mat_impact_fx);
    o.p.e.SetMatImpactParm(fx_parm);
    // Raven sets the origin a second time, outside the branch.
    o.p.e.SetOrigin1(Some(org));
    o.SetNormal(norm);
    o.p.SetVel(Some(vel));
    o.p.SetAccel(Some(accel));

    o.p.SetRGBStart(Some(rgb1));
    o.p.SetRGBEnd(Some(rgb2));
    if let Some(v) = parm_value(
        flags,
        FX_RGB_PARM_MASK,
        FX_RGB_WAVE,
        rgb_parm,
        kill_time,
        time,
    ) {
        o.p.SetRGBParm(v);
    }

    o.p.SetAlphaStart(alpha1);
    o.p.SetAlphaEnd(alpha2);
    if let Some(v) = parm_value(
        flags,
        FX_ALPHA_PARM_MASK,
        FX_ALPHA_WAVE,
        alpha_parm,
        kill_time,
        time,
    ) {
        o.p.SetAlphaParm(v);
    }

    o.p.SetSizeStart(size1);
    o.p.SetSizeEnd(size2);
    if let Some(v) = parm_value(
        flags,
        FX_SIZE_PARM_MASK,
        FX_SIZE_WAVE,
        size_parm,
        kill_time,
        time,
    ) {
        o.p.SetSizeParm(v);
    }

    o.p.e.SetFlags(flags);
    o.p.SetShader(shader);
    o.p.SetRotation(rotation);
    o.p.SetRotationDelta(rotation_delta);
    o.p.SetElasticity(bounce);
    o.p.e.SetMin(Some(min));
    o.p.e.SetMax(Some(max));
    o.p.e.SetDeathFxID(death_id);
    o.p.e.SetImpactFxID(impact_id);

    FX_AddPrimitive(fx, host, FxPrimitive::OrientedParticle(o), kill_time);
}

/// Raven `FX_AddPoly`.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:1005-1072`
#[allow(clippy::too_many_arguments)]
pub fn FX_AddPoly(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    verts: &[vec3_t],
    st: &[[f32; 2]],
    num_verts: i32,
    vel: vec3_t,
    accel: vec3_t,
    alpha1: f32,
    alpha2: f32,
    alpha_parm: f32,
    rgb1: vec3_t,
    rgb2: vec3_t,
    rgb_parm: f32,
    rotation_delta: vec3_t,
    bounce: f32,
    motion_delay: i32,
    kill_time: i32,
    shader: i32,
    flags: i32,
) {
    if fx.clock.mFrameTime < 1 || verts.is_empty() {
        // disallow adding effects when the system is paused or the user doesn't
        // pass in a vert array
        return;
    }

    let mut poly = CPoly::default();
    let time = fx.clock.mTime;

    // Do a cheesy copy of the verts and texture coords into our own structure
    let count = (num_verts as usize).min(MAX_CPOLY_VERTS);
    for i in 0..count {
        poly.mOrg[i] = verts[i];
        poly.mST[i] = st[i];
    }

    poly.p.SetVel(Some(vel));
    poly.p.SetAccel(Some(accel));

    poly.p.SetRGBStart(Some(rgb1));
    poly.p.SetRGBEnd(Some(rgb2));
    if let Some(v) = parm_value(
        flags,
        FX_RGB_PARM_MASK,
        FX_RGB_WAVE,
        rgb_parm,
        kill_time,
        time,
    ) {
        poly.p.SetRGBParm(v);
    }

    poly.p.SetAlphaStart(alpha1);
    poly.p.SetAlphaEnd(alpha2);
    if let Some(v) = parm_value(
        flags,
        FX_ALPHA_PARM_MASK,
        FX_ALPHA_WAVE,
        alpha_parm,
        kill_time,
        time,
    ) {
        poly.p.SetAlphaParm(v);
    }

    poly.p.e.SetFlags(flags);
    poly.p.SetShader(shader);
    poly.SetRot(Some(rotation_delta));
    poly.p.SetElasticity(bounce);
    poly.SetMotionTimeStamp(time, motion_delay);
    poly.SetNumVerts(num_verts);

    // Now that we've set our data up, let's process it into a useful format
    poly.PolyInit(fx);

    FX_AddPrimitive(fx, host, FxPrimitive::Poly(poly), kill_time);
}

/// Raven `FX_AddFlash`.
///
/// A zero shader is bad input, and Raven answers it by drawing nothing.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:1078-1155`
#[allow(clippy::too_many_arguments)]
pub fn FX_AddFlash(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    origin: vec3_t,
    size1: f32,
    size2: f32,
    size_parm: f32,
    alpha1: f32,
    alpha2: f32,
    alpha_parm: f32,
    s_rgb: vec3_t,
    e_rgb: vec3_t,
    rgb_parm: f32,
    kill_time: i32,
    shader: i32,
    flags: i32,
    mat_impact_fx: EMatImpactEffect,
    fx_parm: i32,
) {
    if fx.clock.mFrameTime < 1 {
        return;
    }

    if shader == 0 {
        // yeah..this is bad, I guess, but SP seems to handle it by not drawing
        // the flash, so I will too.
        return;
    }

    let mut f = CFlash::default();
    let time = fx.clock.mTime;

    f.p.e.SetMatImpactFX(mat_impact_fx);
    f.p.e.SetMatImpactParm(fx_parm);
    f.p.e.SetOrigin1(Some(origin));

    f.p.SetRGBStart(Some(s_rgb));
    f.p.SetRGBEnd(Some(e_rgb));
    if let Some(v) = parm_value(
        flags,
        FX_RGB_PARM_MASK,
        FX_RGB_WAVE,
        rgb_parm,
        kill_time,
        time,
    ) {
        f.p.SetRGBParm(v);
    }

    f.p.SetAlphaStart(alpha1);
    f.p.SetAlphaEnd(alpha2);
    if let Some(v) = parm_value(
        flags,
        FX_ALPHA_PARM_MASK,
        FX_ALPHA_WAVE,
        alpha_parm,
        kill_time,
        time,
    ) {
        f.p.SetAlphaParm(v);
    }

    f.p.SetSizeStart(size1);
    f.p.SetSizeEnd(size2);
    if let Some(v) = parm_value(
        flags,
        FX_SIZE_PARM_MASK,
        FX_SIZE_WAVE,
        size_parm,
        kill_time,
        time,
    ) {
        f.p.SetSizeParm(v);
    }

    f.p.SetShader(shader);
    f.p.e.SetFlags(flags);

    f.Init(fx);

    FX_AddPrimitive(fx, host, FxPrimitive::Flash(f), kill_time);
}

/// Raven `FX_AddBezier`.
///
/// Source: `oracle/codemp/client/FxUtil.cpp:1160-1232`
#[allow(clippy::too_many_arguments)]
pub fn FX_AddBezier(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    start: vec3_t,
    end: vec3_t,
    control1: vec3_t,
    control1_vel: vec3_t,
    control2: vec3_t,
    control2_vel: vec3_t,
    size1: f32,
    size2: f32,
    size_parm: f32,
    alpha1: f32,
    alpha2: f32,
    alpha_parm: f32,
    s_rgb: vec3_t,
    e_rgb: vec3_t,
    rgb_parm: f32,
    kill_time: i32,
    shader: i32,
    flags: i32,
) {
    if fx.clock.mFrameTime < 1 {
        return;
    }

    let mut b = CBezier::default();
    let time = fx.clock.mTime;

    b.l.p.e.SetOrigin1(Some(start));
    b.l.SetOrigin2(end);

    b.SetControlPoints(control1, control2);
    b.SetControlVel(control1_vel, control2_vel);

    b.l.p.SetRGBStart(Some(s_rgb));
    b.l.p.SetRGBEnd(Some(e_rgb));
    if let Some(v) = parm_value(
        flags,
        FX_RGB_PARM_MASK,
        FX_RGB_WAVE,
        rgb_parm,
        kill_time,
        time,
    ) {
        b.l.p.SetRGBParm(v);
    }

    b.l.p.SetAlphaStart(alpha1);
    b.l.p.SetAlphaEnd(alpha2);
    if let Some(v) = parm_value(
        flags,
        FX_ALPHA_PARM_MASK,
        FX_ALPHA_WAVE,
        alpha_parm,
        kill_time,
        time,
    ) {
        b.l.p.SetAlphaParm(v);
    }

    b.l.p.SetSizeStart(size1);
    b.l.p.SetSizeEnd(size2);
    if let Some(v) = parm_value(
        flags,
        FX_SIZE_PARM_MASK,
        FX_SIZE_WAVE,
        size_parm,
        kill_time,
        time,
    ) {
        b.l.p.SetSizeParm(v);
    }

    b.l.p.SetShader(shader);
    b.l.p.e.SetFlags(flags);

    b.l.p.e.SetSTScale(1.0, 1.0);

    FX_AddPrimitive(fx, host, FxPrimitive::Bezier(b), kill_time);
}
