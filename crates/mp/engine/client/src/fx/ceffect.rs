//! Raven `CEffect`, the base of the effect hierarchy.
//!
//! DEC-61.1 turns the hierarchy into one enum, so the base class becomes a core
//! block every variant embeds instead of a parent class.
//! Raven's `mNext` pointer disappears with it: the live pool is an indexed slot
//! array, not an intrusive list.
//!
//! Class definition source: `oracle/codemp/client/FxPrimitives.h:108-167`
//! Method source: `oracle/codemp/client/FxPrimitives.cpp:27-35`

#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::mini_ref_entity_s::miniRefEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use native_math::vector::vec3_t;

use crate::fx::emat_impact_effect::EMatImpactEffect;
use crate::fx::fx_flags::FX_SET_SHADER_TIME;

/// The `CEffect` fields every primitive carries.
///
/// `mRefEnt` is the render entity the draw code fills and hands to the renderer.
/// `mFlags` is the merged template flag word, so every group and feature flag
/// reads out of it.
#[derive(Clone, Copy, Debug)]
pub struct EffectCore {
    pub mOrigin1: vec3_t,

    pub mTimeStart: i32,
    pub mTimeEnd: i32,

    pub mFlags: i32,

    pub mMatImpactFX: EMatImpactEffect,
    pub mMatImpactParm: i32,

    /// Size of our object, useful for things that have physics.
    pub mMin: vec3_t,
    pub mMax: vec3_t,

    /// If we have an impact event, we may have to call an effect.
    pub mImpactFxID: i32,
    /// If we have a death event, we may have to call an effect.
    pub mDeathFxID: i32,

    pub mRefEnt: miniRefEntity_t,

    pub mSoundRadius: i32,
    pub mSoundVolume: i32,
}

impl Default for EffectCore {
    /// Raven `CEffect::CEffect` — the four named defaults, everything else zeroed.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:27-35`
    fn default() -> Self {
        EffectCore {
            mOrigin1: [0.0; 3],
            mTimeStart: 0,
            mTimeEnd: 0,
            mFlags: 0,
            mMatImpactFX: EMatImpactEffect::MATIMPACTFX_NONE,
            mMatImpactParm: -1,
            mMin: [0.0; 3],
            mMax: [0.0; 3],
            mImpactFxID: 0,
            mDeathFxID: 0,
            mRefEnt: zeroed_mini_ref_entity(),
            mSoundRadius: -1,
            mSoundVolume: -1,
        }
    }
}

impl EffectCore {
    /// Source: `oracle/codemp/client/FxPrimitives.h:150`
    pub fn SetSTScale(&mut self, s: f32, t: f32) {
        self.mRefEnt.shaderTexCoord[0] = s;
        self.mRefEnt.shaderTexCoord[1] = t;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:152`
    pub fn SetSound(&mut self, vol: i32, rad: i32) {
        self.mSoundRadius = rad;
        self.mSoundVolume = vol;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:153`
    pub fn SetMin(&mut self, min: Option<vec3_t>) {
        self.mMin = min.unwrap_or([0.0; 3]);
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:154`
    pub fn SetMax(&mut self, max: Option<vec3_t>) {
        self.mMax = max.unwrap_or([0.0; 3]);
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:155`
    pub fn SetFlags(&mut self, flags: i32) {
        self.mFlags = flags;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:156`
    pub fn AddFlags(&mut self, flags: i32) {
        self.mFlags |= flags;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:157`
    pub fn ClearFlags(&mut self, flags: i32) {
        self.mFlags &= !flags;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:158`
    pub fn SetOrigin1(&mut self, org: Option<vec3_t>) {
        self.mOrigin1 = org.unwrap_or([0.0; 3]);
    }

    /// Also stamps the shader time when the effect asked for it.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.h:159`
    pub fn SetTimeStart(&mut self, time: i32) {
        self.mTimeStart = time;
        if self.mFlags & FX_SET_SHADER_TIME != 0 {
            self.mRefEnt.shaderTime = time as f32 * 0.001;
        }
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:160`
    pub fn SetTimeEnd(&mut self, time: i32) {
        self.mTimeEnd = time;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:161`
    pub fn SetImpactFxID(&mut self, id: i32) {
        self.mImpactFxID = id;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:162`
    pub fn SetDeathFxID(&mut self, id: i32) {
        self.mDeathFxID = id;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:165`
    pub fn SetMatImpactFX(&mut self, mat_fx: EMatImpactEffect) {
        self.mMatImpactFX = mat_fx;
    }

    /// Source: `oracle/codemp/client/FxPrimitives.h:166`
    pub fn SetMatImpactParm(&mut self, mat_parm: i32) {
        self.mMatImpactParm = mat_parm;
    }
}

/// Raven's `memset( &mRefEnt, 0, sizeof( mRefEnt ))`.
///
/// `refEntityType_t` has no zero-valued default derive, so the fields spell out
/// the all-bits-zero state.
pub fn zeroed_mini_ref_entity() -> miniRefEntity_t {
    miniRefEntity_t {
        reType: refEntityType_t::RT_MODEL,
        renderfx: 0,
        hModel: 0,
        axis: [[0.0; 3]; 3],
        nonNormalizedAxes: 0,
        origin: [0.0; 3],
        oldorigin: [0.0; 3],
        customShader: 0,
        shaderRGBA: [0; 4],
        shaderTexCoord: [0.0; 2],
        radius: 0.0,
        rotation: 0.0,
        shaderTime: 0.0,
        frame: 0,
    }
}
