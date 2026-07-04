//! MP `bg_public.h` `entityState_t::eFlags`/`eFlags2` bit values.
//!
//! Plain `#define` bit flags (not an enum), so §C8 makes them `const`s
//! directly.
//!
//! Source: `oracle/oracle/codemp/game/bg_public.h:558-624`

use core::ffi::c_int;

pub const EF_G2ANIMATING: c_int = 1 << 0; // perform g2 bone anims based on torsoAnim and legsAnim, works for ET_GENERAL -rww
pub const EF_DEAD: c_int = 1 << 1; // don't draw a foe marker over players with EF_DEAD
pub const EF_RADAROBJECT: c_int = 1 << 2; // display on team radar
pub const EF_TELEPORT_BIT: c_int = 1 << 3; // toggled every time the origin abruptly changes
pub const EF_SHADER_ANIM: c_int = 1 << 4; // Animating shader (by s.frame)
pub const EF_PLAYER_EVENT: c_int = 1 << 5;
pub const EF_RAG: c_int = 1 << 6; // ragdoll him even if he's alive
pub const EF_PERMANENT: c_int = 1 << 7; // rww - I am claiming this. (for permanent entities)
pub const EF_NODRAW: c_int = 1 << 8; // may have an event, but no model (unspawned items)
pub const EF_FIRING: c_int = 1 << 9; // for lightning gun
pub const EF_ALT_FIRING: c_int = 1 << 10; // for alt-fires, mostly for lightning guns though
pub const EF_JETPACK_ACTIVE: c_int = 1 << 11; // jetpack is activated
pub const EF_TALK: c_int = 1 << 13; // draw a talk balloon
pub const EF_CONNECTION: c_int = 1 << 14; // draw a connection trouble sprite
pub const EF_BODYPUSH: c_int = 1 << 19; // rww - claiming this for fullbody push effect
pub const EF_DOUBLE_AMMO: c_int = 1 << 20; // Hacky way to get around ammo max
pub const EF_SEEKERDRONE: c_int = 1 << 21; // show seeker drone floating around head
pub const EF_MISSILE_STICK: c_int = 1 << 22; // missiles that stick to the wall.
pub const EF_ITEMPLACEHOLDER: c_int = 1 << 23; // item effect
pub const EF_SOUNDTRACKER: c_int = 1 << 24; // sound position needs to be updated in relation to another entity
pub const EF_DROPPEDWEAPON: c_int = 1 << 25; // it's a dropped weapon
pub const EF_DISINTEGRATION: c_int = 1 << 26; // being disintegrated by the disruptor
pub const EF_INVULNERABLE: c_int = 1 << 27; // just spawned in or whatever, so is protected
pub const EF_CLIENTSMOOTH: c_int = 1 << 28; // standard lerporigin smooth override on client
pub const EF_JETPACK: c_int = 1 << 29; // rww - wearing a jetpack
pub const EF_JETPACK_FLAMING: c_int = 1 << 30; // rww - jetpack fire effect

// These EF2_??? flags were added for NPCs; NOTE: we only allow 10 of these.
pub const EF2_HELD_BY_MONSTER: c_int = 1 << 0; // Being held by something, like a Rancor or a Wampa
pub const EF2_USE_ALT_ANIM: c_int = 1 << 1; // For certain special runs/stands for creatures like the Rancor and Wampa whose runs/stands are conditional
pub const EF2_ALERTED: c_int = 1 << 2; // For certain special anims, for Rancor: means you've had an enemy, so use the more alert stand
pub const EF2_GENERIC_NPC_FLAG: c_int = 1 << 3; // So far, used for Rancor...
pub const EF2_FLYING: c_int = 1 << 4; // Flying (NPC-only)
pub const EF2_HYPERSPACE: c_int = 1 << 5; // Used to both start the hyperspace effect on the predicted client and to let the vehicle know it can now jump into hyperspace
pub const EF2_BRACKET_ENTITY: c_int = 1 << 6; // Draw as bracketed
pub const EF2_SHIP_DEATH: c_int = 1 << 7; // "died in ship" mode
