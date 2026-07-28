//! Port of `oracle/codemp/cgame/fx_demp2.c` — DEMP2 projectile, shock-sphere and impact effects. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::shared::q_math::{_VectorCopy, VectorNormalize2};
use mp_qshared::shared::vec3_t;

use crate::cg_localents::CG_AllocLocalEntity;
use crate::local::centity_s::centity_t;
use crate::local::le_type_t::leType_t;
use crate::local::weapon_info_s::weaponInfo_t;
use crate::trap;
use crate::world::cg_context::CgContext;

/// Raven `FX_DEMP2_ProjectileThink` — plays the DEMP2 bolt flight effect at
/// the entity's lerped origin, oriented along its velocity (straight up if
/// the entity isn't moving).
///
/// `weapon` goes unread in Raven's body; kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_demp2.c:11-21`
pub fn FX_DEMP2_ProjectileThink(ctx: &mut CgContext, cent: &centity_t, _weapon: &weaponInfo_t) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(cent.currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.demp2ProjectileEffect,
        &cent.lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/// Raven `FX_DEMP2_HitWall` — plays the DEMP2 wall-impact effect at the hit
/// point, oriented along the surface normal.
/// Source: `oracle/codemp/cgame/fx_demp2.c:29-32`
pub fn FX_DEMP2_HitWall(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t) {
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.demp2WallImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `FX_DEMP2_HitPlayer` — plays the DEMP2 flesh-impact effect against
/// a hit target.
///
/// `humanoid` goes unread in Raven's body (unlike the blaster's equivalent,
/// DEMP2 has no separate droid-impact effect); kept for signature parity.
/// Source: `oracle/codemp/cgame/fx_demp2.c:40-43`
pub fn FX_DEMP2_HitPlayer(ctx: &mut CgContext, origin: &vec3_t, normal: &vec3_t, humanoid: bool) {
    let _ = humanoid;

    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.demp2FleshImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/// Raven `RF_VOLUMETRIC` — fake volumetric shading.
/// Source: `oracle/codemp/cgame/tr_types.h:24`
const RF_VOLUMETRIC: c_int = 0x00020;

/// Raven `FX_DEMP2_AltDetonate` — spawns a fading shell-model local entity
/// at the alt-fire detonation point.
/// Source: `oracle/codemp/cgame/fx_demp2.c:240-259`
pub fn FX_DEMP2_AltDetonate(ctx: &mut CgContext, org: &vec3_t, size: f32) {
    let handle = CG_AllocLocalEntity(ctx.world);
    let now = ctx.world.cg.time;
    let demp2ShellShader = ctx.world.cgs.media.demp2ShellShader;
    let demp2Shell = ctx.world.cgs.media.demp2Shell;

    let ex = ctx
        .world
        .cg_localEntities
        .get_mut(handle)
        .expect("FX_DEMP2_AltDetonate: fresh slot");

    ex.leType = leType_t::LE_FADE_SCALE_MODEL;
    ex.refEntity = refEntity_t::zeroed();

    ex.refEntity.renderfx |= RF_VOLUMETRIC;

    ex.startTime = now;
    ex.endTime = ex.startTime + 800; //1600;

    ex.radius = size;
    ex.refEntity.customShader = demp2ShellShader;
    ex.refEntity.hModel = demp2Shell;
    _VectorCopy(*org, &mut ex.refEntity.origin);

    ex.color[0] = 255.0;
    ex.color[1] = 255.0;
    ex.color[2] = 255.0;
}

/// Raven `FX_DEMP2_AltBeam` — dead: the entire body is commented out in the
/// oracle ("NOTENOTE Fix this after trap calls for all primitives are
/// created"), so this is a no-op preserved verbatim.
/// Source: `oracle/codemp/cgame/fx_demp2.c:50-237`
pub fn FX_DEMP2_AltBeam(
    _start: &vec3_t,
    _end: &vec3_t,
    _normal: &vec3_t,
    _targ1: &vec3_t,
    _targ2: &vec3_t,
) {
}
