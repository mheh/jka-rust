//! MP `b_public.h` `NPCInfo->scriptFlags` bit values.
//!
//! Plain `#define` bit flags (not an enum), so §C8 makes them `const`s
//! directly.
//!
//! Source: `oracle/oracle/codemp/game/b_public.h:26-52`

use core::ffi::c_int;

pub const SCF_CROUCHED: c_int = 0x0000_0001; // Force ucmd.upmove to be -127
pub const SCF_WALKING: c_int = 0x0000_0002; // Force BUTTON_WALKING to be pressed
pub const SCF_MORELIGHT: c_int = 0x0000_0004; // NPC will have a minlight of 96
pub const SCF_LEAN_RIGHT: c_int = 0x0000_0008; // Force rightmove+BUTTON_USE
pub const SCF_LEAN_LEFT: c_int = 0x0000_0010; // Force leftmove+BUTTON_USE
pub const SCF_RUNNING: c_int = 0x0000_0020; // Takes off walking button, overrides SCF_WALKING
pub const SCF_ALT_FIRE: c_int = 0x0000_0040; // Force to use alt-fire when firing
pub const SCF_NO_RESPONSE: c_int = 0x0000_0080; // NPC will not do generic responses to being used
pub const SCF_FFDEATH: c_int = 0x0000_0100; // Just tells player_die to run the friendly fire deathscript
pub const SCF_NO_COMBAT_TALK: c_int = 0x0000_0200; // NPC will not use their generic combat chatter stuff
pub const SCF_CHASE_ENEMIES: c_int = 0x0000_0400; // NPC chase enemies
pub const SCF_LOOK_FOR_ENEMIES: c_int = 0x0000_0800; // NPC be on the lookout for enemies
pub const SCF_FACE_MOVE_DIR: c_int = 0x0000_1000; // NPC face direction it's moving
pub const SCF_IGNORE_ALERTS: c_int = 0x0000_2000; // NPC ignore alert events
pub const SCF_DONT_FIRE: c_int = 0x0000_4000; // NPC won't shoot
pub const SCF_DONT_FLEE: c_int = 0x0000_8000; // NPC never flees
pub const SCF_FORCED_MARCH: c_int = 0x0001_0000; // NPC that the player must aim at to make him walk
pub const SCF_NO_GROUPS: c_int = 0x0002_0000; // NPC cannot alert groups or be part of a group
pub const SCF_FIRE_WEAPON: c_int = 0x0004_0000; // NPC will fire his (her) weapon
pub const SCF_NO_MIND_TRICK: c_int = 0x0008_0000; // Not succeptible to mind tricks
pub const SCF_USE_CP_NEAREST: c_int = 0x0010_0000; // Will use combat point close to it, not next to player or try and flank player
pub const SCF_NO_FORCE: c_int = 0x0020_0000; // Not succeptible to force powers
pub const SCF_NO_FALLTODEATH: c_int = 0x0040_0000; // NPC will not scream and tumble and fall to hit death over large drops
pub const SCF_NO_ACROBATICS: c_int = 0x0080_0000; // Jedi won't jump, roll or cartwheel
pub const SCF_USE_SUBTITLES: c_int = 0x0100_0000; // Regardless of subtitle setting, this NPC will display subtitles when it speaks lines
pub const SCF_NO_ALERT_TALK: c_int = 0x0200_0000; // Will not say alert sounds, but still can be woken up by alerts
