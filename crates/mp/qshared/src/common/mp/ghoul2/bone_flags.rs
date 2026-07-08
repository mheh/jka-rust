//! Ghoul2 `BONE_*` bit flags (bone-angle apply modes + bone-anim override
//! flags).
//!
//! Plain `#define` bit flags (not an enum), so §C8 makes them `const`s
//! directly.
//!
//! Source: `oracle/oracle/codemp/ghoul2/G2.h:8-26`

use core::ffi::c_int;

/// Raven `BONE_ANGLES_PREMULT`.
///
/// Source: `oracle/oracle/codemp/ghoul2/G2.h:8`
pub const BONE_ANGLES_PREMULT: c_int = 0x0001;

/// Raven `BONE_ANGLES_POSTMULT`.
///
/// Source: `oracle/oracle/codemp/ghoul2/G2.h:9`
pub const BONE_ANGLES_POSTMULT: c_int = 0x0002;

/// Raven `BONE_ANGLES_REPLACE`.
///
/// Source: `oracle/oracle/codemp/ghoul2/G2.h:10`
pub const BONE_ANGLES_REPLACE: c_int = 0x0004;

/// Raven `BONE_NEED_TRANSFORM`.
///
/// Raven: added for a trace optimization. set in routines where a bone is
/// set to be transformed in any way. -rww
///
/// Source: `oracle/oracle/codemp/ghoul2/G2.h:14`
pub const BONE_NEED_TRANSFORM: c_int = 0x8000;

/// Raven `BONE_ANGLES_RAGDOLL`.
///
/// Raven: the rag flags give more details
///
/// Source: `oracle/oracle/codemp/ghoul2/G2.h:17`
pub const BONE_ANGLES_RAGDOLL: c_int = 0x2000;

/// Raven `BONE_ANGLES_IK`.
///
/// Raven: the rag flags give more details
///
/// Source: `oracle/oracle/codemp/ghoul2/G2.h:19`
pub const BONE_ANGLES_IK: c_int = 0x4000;

/// Raven `BONE_ANGLES_TOTAL`.
///
/// Source: `oracle/oracle/codemp/ghoul2/G2.h:21`
pub const BONE_ANGLES_TOTAL: c_int = BONE_ANGLES_PREMULT | BONE_ANGLES_POSTMULT | BONE_ANGLES_REPLACE;

/// Raven `BONE_ANIM_OVERRIDE`.
///
/// Source: `oracle/oracle/codemp/ghoul2/G2.h:22`
pub const BONE_ANIM_OVERRIDE: c_int = 0x0008;

/// Raven `BONE_ANIM_OVERRIDE_LOOP`.
///
/// Source: `oracle/oracle/codemp/ghoul2/G2.h:23`
pub const BONE_ANIM_OVERRIDE_LOOP: c_int = 0x0010;

/// Raven `BONE_ANIM_OVERRIDE_FREEZE`.
///
/// Source: `oracle/oracle/codemp/ghoul2/G2.h:24`
pub const BONE_ANIM_OVERRIDE_FREEZE: c_int = 0x0040 + BONE_ANIM_OVERRIDE;

/// Raven `BONE_ANIM_BLEND`.
///
/// Source: `oracle/oracle/codemp/ghoul2/G2.h:25`
pub const BONE_ANIM_BLEND: c_int = 0x0080;

/// Raven `BONE_ANIM_TOTAL`.
///
/// Source: `oracle/oracle/codemp/ghoul2/G2.h:26`
pub const BONE_ANIM_TOTAL: c_int =
    BONE_ANIM_OVERRIDE | BONE_ANIM_OVERRIDE_LOOP | BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND;
