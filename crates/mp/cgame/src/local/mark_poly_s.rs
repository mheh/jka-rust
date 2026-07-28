#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::poly_s::poly_t;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::shared::qboolean;
use mp_qshared::shared::qhandle_t;

/// Raven `MAX_VERTS_ON_POLY`.
///
/// Source: `oracle/codemp/cgame/cg_local.h:56`
pub const MAX_VERTS_ON_POLY: usize = 10;

/// Raven `markPoly_t`.
///
/// Raven's intrusive `prevMark`/`nextMark` links are gone — DEC-46.3 moves the
/// free list, the live flag and the allocation order into
/// [`EffectPool`](crate::world::effect_pool::EffectPool). The type is
/// module-private (`cg_local.h`), never crosses the engine seam, so the layout
/// is free and the transcription-era size/offset asserts retire with the links
/// (the DEC-31 treatment `weaponInfo_t` already got).
/// Type definition source: `oracle/codemp/cgame/cg_local.h:470-478`
#[repr(C)]
pub struct markPoly_t {
    pub time: i32,
    pub markShader: qhandle_t,
    /// fade alpha instead of rgb
    pub alphaFade: qboolean,
    pub color: [f32; 4],
    pub poly: poly_t,
    pub verts: [polyVert_t; MAX_VERTS_ON_POLY],
}

impl markPoly_t {
    /// Raven `memset( le, 0, sizeof( *le ) )` on every alloc — the seed
    /// [`EffectPool`](crate::world::effect_pool::EffectPool) hands each slot.
    ///
    /// Source: `oracle/codemp/cgame/cg_marks.c:81`
    pub fn zeroed() -> Self {
        // SAFETY: every field is a POD scalar or array of scalars; `poly_t`'s
        // `polyVert_t *verts` is a raw pointer, and null is the value Raven's
        // `memset` leaves there too.
        unsafe { core::mem::zeroed() }
    }
}
