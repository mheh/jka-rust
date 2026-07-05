//! `EF_*` entity flag bits.
//!
//! Source: `oracle/oracle/codemp/game/bg_public.h:560-612`

/// Perform g2 bone anims based on torsoAnim and legsAnim, works for ET_GENERAL -rww.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:560`
pub const EF_G2ANIMATING: i32 = 1 << 0;

/// Don't draw a foe marker over players with EF_DEAD.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:561`
pub const EF_DEAD: i32 = 1 << 1;

/// Display on team radar.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:564`
pub const EF_RADAROBJECT: i32 = 1 << 2;

/// Toggled every time the origin abruptly changes.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:566`
pub const EF_TELEPORT_BIT: i32 = 1 << 3;

/// Animating shader (by s.frame).
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:568`
pub const EF_SHADER_ANIM: i32 = 1 << 4;

/// Source: `oracle/oracle/codemp/game/bg_public.h:570`
pub const EF_PLAYER_EVENT: i32 = 1 << 5;

/// Ragdoll him even if he's alive.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:576`
pub const EF_RAG: i32 = 1 << 6;

/// rww - I am claiming this. (for permanent entities)
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:579`
pub const EF_PERMANENT: i32 = 1 << 7;

/// May have an event, but no model (unspawned items).
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:581`
pub const EF_NODRAW: i32 = 1 << 8;

/// For lightning gun.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:582`
pub const EF_FIRING: i32 = 1 << 9;

/// For alt-fires, mostly for lightning guns though.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:583`
pub const EF_ALT_FIRING: i32 = 1 << 10;

/// Jetpack is activated.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:584`
pub const EF_JETPACK_ACTIVE: i32 = 1 << 11;

/// Not used.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:586`
pub const EF_NOT_USED_1: i32 = 1 << 12;

/// Draw a talk balloon.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:588`
pub const EF_TALK: i32 = 1 << 13;

/// Draw a connection trouble sprite.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:589`
pub const EF_CONNECTION: i32 = 1 << 14;

/// Not used.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:590`
pub const EF_NOT_USED_6: i32 = 1 << 15;

/// Not used.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:592`
pub const EF_NOT_USED_2: i32 = 1 << 16;

/// Not used.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:593`
pub const EF_NOT_USED_3: i32 = 1 << 17;

/// Not used.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:594`
pub const EF_NOT_USED_4: i32 = 1 << 18;

/// rww - claiming this for fullbody push effect.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:596`
pub const EF_BODYPUSH: i32 = 1 << 19;

/// Hacky way to get around ammo max.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:598`
pub const EF_DOUBLE_AMMO: i32 = 1 << 20;

/// Show seeker drone floating around head.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:599`
pub const EF_SEEKERDRONE: i32 = 1 << 21;

/// Missiles that stick to the wall.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:600`
pub const EF_MISSILE_STICK: i32 = 1 << 22;

/// Item effect.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:601`
pub const EF_ITEMPLACEHOLDER: i32 = 1 << 23;

/// Sound position needs to be updated in relation to another entity.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:602`
pub const EF_SOUNDTRACKER: i32 = 1 << 24;

/// It's a dropped weapon.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:603`
pub const EF_DROPPEDWEAPON: i32 = 1 << 25;

/// Being disintegrated by the disruptor.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:604`
pub const EF_DISINTEGRATION: i32 = 1 << 26;

/// Just spawned in or whatever, so is protected.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:605`
pub const EF_INVULNERABLE: i32 = 1 << 27;

/// Standard lerporigin smooth override on client.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:607`
pub const EF_CLIENTSMOOTH: i32 = 1 << 28;

/// rww - wearing a jetpack.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:609`
pub const EF_JETPACK: i32 = 1 << 29;

/// rww - jetpack fire effect.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:610`
pub const EF_JETPACK_FLAMING: i32 = 1 << 30;

/// Not used.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:612`
pub const EF_NOT_USED_5: i32 = 1 << 31;
