#![allow(non_camel_case_types, non_snake_case)]

use native_math::vector::vec3_t;

use crate::fx::cfx_range::CFxRange;
use crate::fx::cmedia_handles::CMediaHandles;
use crate::fx::emat_impact_effect::EMatImpactEffect;
use crate::fx::eprim_type::EPrimType;

/// Raven `CPrimitiveTemplate` — the parsed `.efx` block that spawns one primitive kind.
///
/// Raven: one huge shared class, because there are never many in memory at once
/// and none get created mid-game.
/// Not every primitive type uses every field.
/// Type definition source: `oracle/codemp/client/FxScheduler.h:152-340`
#[derive(Clone, PartialEq, Debug)]
pub struct CPrimitiveTemplate {
    /// Set on a template that `GetEffectCopy` cloned, so `CreateEffect` frees it when used up.
    pub mCopy: bool,
    /// How many spawns still owe this copy a reference.
    pub mRefCount: i32,

    /// Raven stores `char mName[32]` and fills it with `strcpy`.
    /// A name longer than 31 characters overruns in Raven, so the port truncates
    /// at 31 characters (porting-rules §19).
    pub mName: String,

    pub mType: EPrimType,

    pub mSpawnDelay: CFxRange,
    pub mSpawnCount: CFxRange,
    pub mLife: CFxRange,
    pub mCullRange: i32,

    pub mMediaHandles: CMediaHandles,
    pub mImpactFxHandles: CMediaHandles,
    pub mDeathFxHandles: CMediaHandles,
    pub mEmitterFxHandles: CMediaHandles,
    pub mPlayFxHandles: CMediaHandles,

    /// Passed on to the spawned primitive.
    pub mFlags: i32,
    /// Steers spawning only. Never reaches a primitive.
    pub mSpawnFlags: i32,

    pub mMatImpactFX: EMatImpactEffect,

    pub mMin: vec3_t,
    pub mMax: vec3_t,

    pub mOrigin1X: CFxRange,
    pub mOrigin1Y: CFxRange,
    pub mOrigin1Z: CFxRange,

    pub mOrigin2X: CFxRange,
    pub mOrigin2Y: CFxRange,
    pub mOrigin2Z: CFxRange,

    pub mRadius: CFxRange,
    pub mHeight: CFxRange,
    pub mWindModifier: CFxRange,

    pub mRotation: CFxRange,
    pub mRotationDelta: CFxRange,

    pub mAngle1: CFxRange,
    pub mAngle2: CFxRange,
    pub mAngle3: CFxRange,

    pub mAngle1Delta: CFxRange,
    pub mAngle2Delta: CFxRange,
    pub mAngle3Delta: CFxRange,

    pub mVelX: CFxRange,
    pub mVelY: CFxRange,
    pub mVelZ: CFxRange,

    pub mAccelX: CFxRange,
    pub mAccelY: CFxRange,
    pub mAccelZ: CFxRange,

    pub mGravity: CFxRange,

    pub mDensity: CFxRange,
    pub mVariance: CFxRange,

    pub mRedStart: CFxRange,
    pub mGreenStart: CFxRange,
    pub mBlueStart: CFxRange,

    pub mRedEnd: CFxRange,
    pub mGreenEnd: CFxRange,
    pub mBlueEnd: CFxRange,

    pub mRGBParm: CFxRange,

    pub mAlphaStart: CFxRange,
    pub mAlphaEnd: CFxRange,
    pub mAlphaParm: CFxRange,

    pub mSizeStart: CFxRange,
    pub mSizeEnd: CFxRange,
    pub mSizeParm: CFxRange,

    pub mSize2Start: CFxRange,
    pub mSize2End: CFxRange,
    pub mSize2Parm: CFxRange,

    pub mLengthStart: CFxRange,
    pub mLengthEnd: CFxRange,
    pub mLengthParm: CFxRange,

    pub mTexCoordS: CFxRange,
    pub mTexCoordT: CFxRange,

    pub mElasticity: CFxRange,

    pub mSoundRadius: i32,
    pub mSoundVolume: i32,
}

impl Default for CPrimitiveTemplate {
    /// Raven `CPrimitiveTemplate::CPrimitiveTemplate` — the minimal default values.
    ///
    /// Every field the constructor leaves alone starts at zero, which is what
    /// Raven's uninitialized `CFxRange` members hold after their own default
    /// constructor runs.
    /// Source: `oracle/codemp/client/FxTemplate.cpp:20-69`
    fn default() -> Self {
        let zero = CFxRange::default();
        let mut one = CFxRange::default();
        one.SetRange(1.0, 1.0);

        let mut elasticity = CFxRange::default();
        elasticity.SetRange(0.1, 0.1);

        let mut life = CFxRange::default();
        life.SetRange(50.0, 50.0);

        let mut ten = CFxRange::default();
        ten.SetRange(10.0, 10.0);

        CPrimitiveTemplate {
            // We never start out as a copy or with a name
            mCopy: false,
            mRefCount: 0,
            mName: String::new(),
            mType: EPrimType::None,

            mSpawnDelay: zero,
            mSpawnCount: one,
            mLife: life,
            mCullRange: 0, // no distance culling

            mMediaHandles: CMediaHandles::default(),
            mImpactFxHandles: CMediaHandles::default(),
            mDeathFxHandles: CMediaHandles::default(),
            mEmitterFxHandles: CMediaHandles::default(),
            mPlayFxHandles: CMediaHandles::default(),

            mFlags: 0,
            mSpawnFlags: 0,

            mMatImpactFX: EMatImpactEffect::MATIMPACTFX_NONE,

            mMin: [0.0, 0.0, 0.0],
            mMax: [0.0, 0.0, 0.0],

            mOrigin1X: zero,
            mOrigin1Y: zero,
            mOrigin1Z: zero,

            mOrigin2X: zero,
            mOrigin2Y: zero,
            mOrigin2Z: zero,

            mRadius: ten,
            mHeight: ten,
            mWindModifier: one,

            mRotation: zero,
            mRotationDelta: zero,

            mAngle1: zero,
            mAngle2: zero,
            mAngle3: zero,

            mAngle1Delta: zero,
            mAngle2Delta: zero,
            mAngle3Delta: zero,

            mVelX: zero,
            mVelY: zero,
            mVelZ: zero,

            mAccelX: zero,
            mAccelY: zero,
            mAccelZ: zero,

            mGravity: zero,

            // Raven defaults density high so an emitter does not chuck effects every frame.
            mDensity: ten,
            mVariance: one,

            mRedStart: one,
            mGreenStart: one,
            mBlueStart: one,

            mRedEnd: one,
            mGreenEnd: one,
            mBlueEnd: one,

            mRGBParm: zero,

            mAlphaStart: one,
            mAlphaEnd: one,
            mAlphaParm: zero,

            mSizeStart: one,
            mSizeEnd: one,
            mSizeParm: zero,

            mSize2Start: one,
            mSize2End: one,
            mSize2Parm: zero,

            mLengthStart: one,
            mLengthEnd: one,
            mLengthParm: zero,

            mTexCoordS: one,
            mTexCoordT: one,

            mElasticity: elasticity,

            mSoundRadius: -1,
            mSoundVolume: -1,
        }
    }
}

impl CPrimitiveTemplate {
    /// Raven's `operator=` copies every parsed field but leaves `mCopy` and
    /// `mRefCount` to the caller, which sets them right after.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:72-166`
    pub fn copy_from(&mut self, that: &CPrimitiveTemplate) {
        let mCopy = self.mCopy;
        let mRefCount = self.mRefCount;
        *self = that.clone();
        self.mCopy = mCopy;
        self.mRefCount = mRefCount;
    }

    /// Store a parsed `name` value the way Raven's `strcpy` into `char[32]` does.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:2330-2337`
    pub fn set_name(&mut self, val: &str) {
        let limit = FX_NAME_CHARS.min(val.len());
        let mut end = limit;
        while end > 0 && !val.is_char_boundary(end) {
            end -= 1;
        }
        self.mName = val[..end].to_string();
    }
}

/// How many characters fit in Raven's `char mName[32]` beside the terminator.
///
/// Source: `oracle/codemp/client/FxScheduler.h:29,163`
const FX_NAME_CHARS: usize = 31;
