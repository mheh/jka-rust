#![allow(non_camel_case_types, non_snake_case)]

use super::tex_mod_t::texMod_t;
use super::wave_form_t::waveForm_t;

/// Raven `texModInfo_t` — one texture-coordinate modifier stage.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:323-348`
#[repr(C)]
pub struct texModInfo_t {
    pub r#type: texMod_t,

    // used for TMOD_TURBULENT and TMOD_STRETCH
    pub wave: waveForm_t,

    // used for TMOD_TRANSFORM
    /// s' = s * m[0][0] + t * m[1][0] + trans[0]
    pub matrix: [[f32; 2]; 2],
    /// t' = s * m[0][1] + t * m[0][1] + trans[1]
    pub translate: [f32; 2],
    // used for TMOD_SCALE
    //(moved to translate)
    //	float			scale[2];			// s *= scale[0]
    //                                    // t *= scale[1]

    // used for TMOD_SCROLL
    //(moved to translate)
    //	float			scroll[2];			// s' = s + scroll[0] * time
    // 										// t' = t + scroll[1] * time

    // + = clockwise
    // - = counterclockwise
    ////(moved to translate[0])
    //	float			rotateSpeed;
}

const _: () = assert!(core::mem::size_of::<texModInfo_t>() == 48);
const _: () = assert!(core::mem::offset_of!(texModInfo_t, r#type) == 0);
const _: () = assert!(core::mem::offset_of!(texModInfo_t, wave) == 4);
const _: () = assert!(core::mem::offset_of!(texModInfo_t, matrix) == 24);
const _: () = assert!(core::mem::offset_of!(texModInfo_t, translate) == 40);
