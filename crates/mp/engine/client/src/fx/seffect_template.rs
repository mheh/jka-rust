#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_void;

use mp_qshared::shared::MAX_QPATH;

/// `FX_MAX_EFFECT_COMPONENTS` — how many primitives an effect can hold, this should be plenty.
///
/// Source: `oracle/codemp/client/FxScheduler.h:28`
pub const FX_MAX_EFFECT_COMPONENTS: usize = 24;

/// Raven `SEffectTemplate` — a single effect template (a set of primitives) as loaded from an
/// `.efx` file.
///
/// Raven: none.
/// Type definition source: `oracle/codemp/client/FxScheduler.h:346-360`
#[repr(C)]
pub struct SEffectTemplate {
    pub mInUse: bool,
    pub mCopy: bool,
    pub mEffectName: [i8; MAX_QPATH as usize],
    pub mPrimitiveCount: i32,
    pub mRepeatDelay: i32,
    //TODO: Port CPrimitiveTemplate
    // Source: oracle/codemp/client/FxScheduler.h:353
    pub mPrimitives: [*mut c_void; FX_MAX_EFFECT_COMPONENTS],
}

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<SEffectTemplate>() == 272);
    assert!(core::mem::offset_of!(SEffectTemplate, mInUse) == 0);
    assert!(core::mem::offset_of!(SEffectTemplate, mCopy) == 1);
    assert!(core::mem::offset_of!(SEffectTemplate, mEffectName) == 2);
    assert!(core::mem::offset_of!(SEffectTemplate, mPrimitiveCount) == 68);
    assert!(core::mem::offset_of!(SEffectTemplate, mRepeatDelay) == 72);
    assert!(core::mem::offset_of!(SEffectTemplate, mPrimitives) == 80);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<SEffectTemplate>() == 172);
    assert!(core::mem::offset_of!(SEffectTemplate, mInUse) == 0);
    assert!(core::mem::offset_of!(SEffectTemplate, mCopy) == 1);
    assert!(core::mem::offset_of!(SEffectTemplate, mEffectName) == 2);
    assert!(core::mem::offset_of!(SEffectTemplate, mPrimitiveCount) == 68);
    assert!(core::mem::offset_of!(SEffectTemplate, mRepeatDelay) == 72);
    assert!(core::mem::offset_of!(SEffectTemplate, mPrimitives) == 76);
};
