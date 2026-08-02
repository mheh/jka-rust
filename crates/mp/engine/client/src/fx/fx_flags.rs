//! FX flag vocabulary.
//!
//! Three separate flag words share this file because Raven declared them in one
//! header block: the generic group flags the parser produces, the per-group
//! flags a primitive stores in `mFlags`, and the spawn flags that only steer
//! spawning.
//!
//! Source: `oracle/codemp/client/FxPrimitives.h:12-98`,
//! `oracle/codemp/client/FxScheduler.h:35-56`

/// Mask for the transition types that carry a parm value.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:13`
pub const FX_PARM_MASK: i32 = 0xC;

/// Mask over the whole generic group-flag nibble.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:14`
pub const FX_GENERIC_MASK: i32 = 0xF;

/// Generic group flag: linear interpolation.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:15`
pub const FX_LINEAR: i32 = 0x1;

/// Generic group flag: random modulation.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:16`
pub const FX_RAND: i32 = 0x2;

/// Generic group flag: non-linear fade that starts at the parm time.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:17`
pub const FX_NONLINEAR: i32 = 0x4;

/// Generic group flag: wave generator, parm is the frequency multiplier.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:18`
pub const FX_WAVE: i32 = 0x8;

/// Generic group flag: clamp, parm is the clamp time.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:19`
pub const FX_CLAMP: i32 = 0xC;

/// Bit position of the alpha group inside `mFlags`.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:22`
pub const FX_ALPHA_SHIFT: i32 = 0;

/// Source: `oracle/codemp/client/FxPrimitives.h:23`
pub const FX_ALPHA_PARM_MASK: i32 = 0x0000000C;
/// Source: `oracle/codemp/client/FxPrimitives.h:24`
pub const FX_ALPHA_LINEAR: i32 = 0x00000001;
/// Source: `oracle/codemp/client/FxPrimitives.h:25`
pub const FX_ALPHA_RAND: i32 = 0x00000002;
/// Source: `oracle/codemp/client/FxPrimitives.h:26`
pub const FX_ALPHA_NONLINEAR: i32 = 0x00000004;
/// Source: `oracle/codemp/client/FxPrimitives.h:27`
pub const FX_ALPHA_WAVE: i32 = 0x00000008;
/// Source: `oracle/codemp/client/FxPrimitives.h:28`
pub const FX_ALPHA_CLAMP: i32 = 0x0000000C;

/// Bit position of the RGB group inside `mFlags`.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:30`
pub const FX_RGB_SHIFT: i32 = 4;

/// Source: `oracle/codemp/client/FxPrimitives.h:31`
pub const FX_RGB_PARM_MASK: i32 = 0x000000C0;
/// Source: `oracle/codemp/client/FxPrimitives.h:32`
pub const FX_RGB_LINEAR: i32 = 0x00000010;
/// Source: `oracle/codemp/client/FxPrimitives.h:33`
pub const FX_RGB_RAND: i32 = 0x00000020;
/// Source: `oracle/codemp/client/FxPrimitives.h:34`
pub const FX_RGB_NONLINEAR: i32 = 0x00000040;
/// Source: `oracle/codemp/client/FxPrimitives.h:35`
pub const FX_RGB_WAVE: i32 = 0x00000080;
/// Source: `oracle/codemp/client/FxPrimitives.h:36`
pub const FX_RGB_CLAMP: i32 = 0x000000C0;

/// Bit position of the size group inside `mFlags`.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:38`
pub const FX_SIZE_SHIFT: i32 = 8;

/// Source: `oracle/codemp/client/FxPrimitives.h:39`
pub const FX_SIZE_PARM_MASK: i32 = 0x00000C00;
/// Source: `oracle/codemp/client/FxPrimitives.h:40`
pub const FX_SIZE_LINEAR: i32 = 0x00000100;
/// Source: `oracle/codemp/client/FxPrimitives.h:41`
pub const FX_SIZE_RAND: i32 = 0x00000200;
/// Source: `oracle/codemp/client/FxPrimitives.h:42`
pub const FX_SIZE_NONLINEAR: i32 = 0x00000400;
/// Source: `oracle/codemp/client/FxPrimitives.h:43`
pub const FX_SIZE_WAVE: i32 = 0x00000800;
/// Source: `oracle/codemp/client/FxPrimitives.h:44`
pub const FX_SIZE_CLAMP: i32 = 0x00000C00;

/// Bit position of the length group inside `mFlags`.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:46`
pub const FX_LENGTH_SHIFT: i32 = 12;

/// Source: `oracle/codemp/client/FxPrimitives.h:47`
pub const FX_LENGTH_PARM_MASK: i32 = 0x0000C000;
/// Source: `oracle/codemp/client/FxPrimitives.h:48`
pub const FX_LENGTH_LINEAR: i32 = 0x00001000;
/// Source: `oracle/codemp/client/FxPrimitives.h:49`
pub const FX_LENGTH_RAND: i32 = 0x00002000;
/// Source: `oracle/codemp/client/FxPrimitives.h:50`
pub const FX_LENGTH_NONLINEAR: i32 = 0x00004000;
/// Source: `oracle/codemp/client/FxPrimitives.h:51`
pub const FX_LENGTH_WAVE: i32 = 0x00008000;
/// Source: `oracle/codemp/client/FxPrimitives.h:52`
pub const FX_LENGTH_CLAMP: i32 = 0x0000C000;

/// Bit position of the size2 group inside `mFlags`.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:54`
pub const FX_SIZE2_SHIFT: i32 = 16;

/// Source: `oracle/codemp/client/FxPrimitives.h:55`
pub const FX_SIZE2_PARM_MASK: i32 = 0x000C0000;
/// Source: `oracle/codemp/client/FxPrimitives.h:56`
pub const FX_SIZE2_LINEAR: i32 = 0x00010000;
/// Source: `oracle/codemp/client/FxPrimitives.h:57`
pub const FX_SIZE2_RAND: i32 = 0x00020000;
/// Source: `oracle/codemp/client/FxPrimitives.h:58`
pub const FX_SIZE2_NONLINEAR: i32 = 0x00040000;
/// Source: `oracle/codemp/client/FxPrimitives.h:59`
pub const FX_SIZE2_WAVE: i32 = 0x00080000;
/// Source: `oracle/codemp/client/FxPrimitives.h:60`
pub const FX_SIZE2_CLAMP: i32 = 0x000C0000;

/// Emitters only. Raven shares the bit with `FX_SIZE2_LINEAR`, which emitters never use.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:63`
pub const FX_PAPER_PHYSICS: i32 = 0x00010000;

/// Full screen flashes only. Shares the bit with `FX_SIZE2_LINEAR`.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:64`
pub const FX_LOCALIZED_FLASH: i32 = 0x00010000;

/// Player view effects only. Shares the bit with `FX_SIZE2_LINEAR`.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:65`
pub const FX_PLAYER_VIEW: i32 = 0x00010000;

/// Draw with the first-person depth hack.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:68`
pub const FX_DEPTH_HACK: i32 = 0x00100000;

/// The primitive is bolted, so it re-reads its origin from a ghoul2 bolt every frame.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:69`
pub const FX_RELATIVE: i32 = 0x00200000;

/// Source: `oracle/codemp/client/FxPrimitives.h:70`
pub const FX_SET_SHADER_TIME: i32 = 0x00400000;

/// Source: `oracle/codemp/client/FxPrimitives.h:71`
pub const FX_EXPENSIVE_PHYSICS: i32 = 0x00800000;

/// Particles only. Trace against ghoul2 instances. Shares the bit with `FX_SIZE2_RAND`.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:75`
pub const FX_GHOUL2_TRACE: i32 = 0x00020000;

/// Decals only. Project the decal as a ghoul2 gore skin. Shares the bit with
/// `FX_SIZE2_NONLINEAR`.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:77`
pub const FX_GHOUL2_DECALS: i32 = 0x00040000;

/// Source: `oracle/codemp/client/FxPrimitives.h:80`
pub const FX_ATTACHED_MODEL: i32 = 0x01000000;

/// Source: `oracle/codemp/client/FxPrimitives.h:82`
pub const FX_APPLY_PHYSICS: i32 = 0x02000000;

/// Trace with the primitive bounding box instead of a point.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:83`
pub const FX_USE_BBOX: i32 = 0x04000000;

/// Fade through the alpha channel instead of modulating RGB.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:85`
pub const FX_USE_ALPHA: i32 = 0x08000000;

/// Source: `oracle/codemp/client/FxPrimitives.h:88`
pub const FX_EMIT_FX: i32 = 0x10000000;

/// Source: `oracle/codemp/client/FxPrimitives.h:90`
pub const FX_DEATH_RUNS_FX: i32 = 0x20000000;

/// Source: `oracle/codemp/client/FxPrimitives.h:91`
pub const FX_KILL_ON_IMPACT: i32 = 0x40000000;

/// Source: `oracle/codemp/client/FxPrimitives.h:92`
pub const FX_IMPACT_RUNS_FX: i32 = -0x80000000; // 0x80000000 as a signed word

/// Lightning only. Taper towards the endpoint. Shares the bit with `FX_ATTACHED_MODEL`.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:95`
pub const FX_TAPER: i32 = 0x01000000;

/// Lightning only. Enable branching. Shares the bit with `FX_APPLY_PHYSICS`.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:96`
pub const FX_BRANCH: i32 = 0x02000000;

/// Lightning only. Grow from start to end over the life. Shares the bit with `FX_USE_BBOX`.
///
/// Source: `oracle/codemp/client/FxPrimitives.h:97`
pub const FX_GROW: i32 = 0x04000000;

/// Spawn flag: pick the origin on a sphere or ellipsoid.
///
/// Source: `oracle/codemp/client/FxScheduler.h:35`
pub const FX_ORG_ON_SPHERE: i32 = 0x00001;

/// Spawn flag: take the axis from the sphere or cylinder point.
///
/// Source: `oracle/codemp/client/FxScheduler.h:36`
pub const FX_AXIS_FROM_SPHERE: i32 = 0x00002;

/// Spawn flag: pick the origin on a cylinder or disk.
///
/// Source: `oracle/codemp/client/FxScheduler.h:38`
pub const FX_ORG_ON_CYLINDER: i32 = 0x00004;

/// Spawn flag: trace forward to find origin2.
///
/// Source: `oracle/codemp/client/FxScheduler.h:40`
pub const FX_ORG2_FROM_TRACE: i32 = 0x00010;

/// Spawn flag: play an impact effect where the trace lands.
///
/// Source: `oracle/codemp/client/FxScheduler.h:41`
pub const FX_TRACE_IMPACT_FX: i32 = 0x00020;

/// Spawn flag: treat the template origin2 as an offset from the trace endpoint.
///
/// Source: `oracle/codemp/client/FxScheduler.h:42`
pub const FX_ORG2_IS_OFFSET: i32 = 0x00040;

/// Spawn flag: skip the axis transform on the origin.
///
/// Source: `oracle/codemp/client/FxScheduler.h:46`
pub const FX_CHEAP_ORG_CALC: i32 = 0x00100;

/// Spawn flag: skip the axis transform on origin2.
///
/// Source: `oracle/codemp/client/FxScheduler.h:47`
pub const FX_CHEAP_ORG2_CALC: i32 = 0x00200;

/// Spawn flag: skip the axis transform on velocity.
///
/// Source: `oracle/codemp/client/FxScheduler.h:48`
pub const FX_VEL_IS_ABSOLUTE: i32 = 0x00400;

/// Spawn flag: skip the axis transform on acceleration.
///
/// Source: `oracle/codemp/client/FxScheduler.h:49`
pub const FX_ACCEL_IS_ABSOLUTE: i32 = 0x00800;

/// Spawn flag: randomly rotate up and right around the forward vector.
///
/// Source: `oracle/codemp/client/FxScheduler.h:51`
pub const FX_RAND_ROT_AROUND_FWD: i32 = 0x01000;

/// Spawn flag: spread the spawn delays evenly instead of picking each at random.
///
/// Source: `oracle/codemp/client/FxScheduler.h:52`
pub const FX_EVEN_DISTRIBUTION: i32 = 0x02000;

/// Spawn flag: pick the color on the line between RGB min and max, not in the cube.
///
/// Source: `oracle/codemp/client/FxScheduler.h:54`
pub const FX_RGB_COMPONENT_INTERP: i32 = 0x04000;

/// Spawn flag: query the wind. Raven left the wind query commented out.
///
/// Source: `oracle/codemp/client/FxScheduler.h:56`
pub const FX_AFFECTED_BY_WIND: i32 = 0x10000;

/// How far a trace-driven origin2 reaches.
///
/// Source: `oracle/codemp/client/FxScheduler.h:25`
pub const FX_MAX_TRACE_DIST: f32 = 16384.0;

/// Longest primitive name the parser stores.
///
/// Source: `oracle/codemp/client/FxScheduler.h:29`
pub const FX_MAX_PRIM_NAME: usize = 32;

/// Directory the scheduler prepends to an unqualified effect path.
///
/// Source: `oracle/codemp/client/FxScheduler.h:23`
pub const FX_FILE_PATH: &str = "effects";
