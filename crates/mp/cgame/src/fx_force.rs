//! Port of `oracle/codemp/cgame/fx_force.c` — force-power visual effects. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use mp_qshared::shared::q_math::_VectorScale;
use mp_qshared::shared::vec3_t;

use crate::trap;
use crate::world::cg_context::CgContext;

/// Raven `FX_ForceDrained` - fires the force-drained hit effect, flipping `dir` in place to aim it back at the source.
///
/// Source: `oracle/codemp/cgame/fx_force.c:11-15`
pub fn FX_ForceDrained(ctx: &mut CgContext, origin: &vec3_t, dir: &mut vec3_t) {
    _VectorScale(*dir, -1.0, dir);
    trap::FX_PlayEffectID(
        ctx.engine,
        ctx.world.cgs.effects.forceDrained,
        origin,
        dir,
        -1,
        -1,
    );
}
