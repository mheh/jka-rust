//! MP game-tier `setType_t` + `setTable` — ICARUS SET_* property enum and its
//! name/id lookup table.
//!
//! `setType_t` is duplicated from `mp_engine_icarus::q3_interface::set_type_t`
//! (Q3_Interface.h) rather than imported: `mp/game` cannot depend on the
//! engine tier (`docs/workspace-architecture.md` — only `ghoul2` is shared
//! between `engine/*` and `cgame`; `icarus` is not), but `g_ICARUScb.c`
//! (game-tier ICARUS callbacks) needs this enum and its `setTable` string
//! lookup at the game layer. Kept byte-identical to the icarus-crate port.
//!
//! Type definition source: `oracle/oracle/codemp/icarus/Q3_Interface.h:6-255`
//! `setTable` source: `oracle/oracle/codemp/game/g_ICARUScb.c:52-241`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};
use mp_qshared::shared::string_id_table::stringID_table_t;

/// Raven `setType_t` — ICARUS entity properties and script parameters.
///
/// Type definition source: `oracle/oracle/codemp/icarus/Q3_Interface.h:6-255`
#[repr(i32)]
pub enum setType_t {
	// Parm strings
	/// Set entity parm1
	SET_PARM1 = 0,
	/// Set entity parm2
	SET_PARM2,
	/// Set entity parm3
	SET_PARM3,
	/// Set entity parm4
	SET_PARM4,
	/// Set entity parm5
	SET_PARM5,
	/// Set entity parm6
	SET_PARM6,
	/// Set entity parm7
	SET_PARM7,
	/// Set entity parm8
	SET_PARM8,
	/// Set entity parm9
	SET_PARM9,
	/// Set entity parm10
	SET_PARM10,
	/// Set entity parm11
	SET_PARM11,
	/// Set entity parm12
	SET_PARM12,
	/// Set entity parm13
	SET_PARM13,
	/// Set entity parm14
	SET_PARM14,
	/// Set entity parm15
	SET_PARM15,
	/// Set entity parm16
	SET_PARM16,

	// Scripts and other file paths
	/// Script to run when spawned
	SET_SPAWNSCRIPT,
	/// Script to run when used
	SET_USESCRIPT,
	/// Script to run when startled
	SET_AWAKESCRIPT,
	/// Script run when find an enemy for the first time
	SET_ANGERSCRIPT,
	/// Script to run when you shoot
	SET_ATTACKSCRIPT,
	/// Script to run when killed someone
	SET_VICTORYSCRIPT,
	/// Script to run when you can't find your enemy
	SET_LOSTENEMYSCRIPT,
	/// Script to run when hit
	SET_PAINSCRIPT,
	/// Script to run when hit and low health
	SET_FLEESCRIPT,
	/// Script to run when killed
	SET_DEATHSCRIPT,
	/// Script to run after a delay
	SET_DELAYEDSCRIPT,
	/// Script to run when blocked by teammate
	SET_BLOCKEDSCRIPT,
	/// Script to run when player has shot own team repeatedly
	SET_FFIRESCRIPT,
	/// Script to run when player kills a teammate
	SET_FFDEATHSCRIPT,
	/// Script to run when player kills a teammate
	SET_MINDTRICKSCRIPT,
	/// Play a video (inGame)
	SET_VIDEO_PLAY,
	/// Script to run when skipping the running cinematic
	SET_CINEMATIC_SKIPSCRIPT,

	// Standard strings
	/// Set enemy by targetname
	SET_ENEMY,
	/// Set for BS_FOLLOW_LEADER
	SET_LEADER,
	/// Move to this navgoal then continue script
	SET_NAVGOAL,
	/// Set captureGoal by targetname
	SET_CAPTURE,
	/// Set angles toward ent by targetname
	SET_VIEWTARGET,
	/// Set angles toward ent by targetname, will continue to face them (only in BS_CINEMATIC)
	SET_WATCHTARGET,
	/// Set/change your targetname
	SET_TARGETNAME,
	/// Set/change what to use when hit
	SET_PAINTARGET,
	/// All ents with this cameraGroup will be focused on
	SET_CAMERA_GROUP,
	/// What tag on all clients to try and track
	SET_CAMERA_GROUP_TAG,
	/// Object for NPC to look at
	SET_LOOK_TARGET,
	/// Object to place on NPC right hand bolt
	SET_ADDRHANDBOLT_MODEL,
	/// Object to remove from NPC right hand bolt
	SET_REMOVERHANDBOLT_MODEL,
	/// Object to place on NPC left hand bolt
	SET_ADDLHANDBOLT_MODEL,
	/// Object to remove from NPC left hand bolt
	SET_REMOVELHANDBOLT_MODEL,
	/// Color of text RED,WHITE,BLUE, YELLOW
	SET_CAPTIONTEXTCOLOR,
	/// Color of text RED,WHITE,BLUE, YELLOW
	SET_CENTERTEXTCOLOR,
	/// Color of text RED,WHITE,BLUE, YELLOW
	SET_SCROLLTEXTCOLOR,
	/// Copy the origin of the ent with targetname to your origin
	SET_COPY_ORIGIN,
	/// This NPC will attack the target NPC's enemies
	SET_DEFEND_TARGET,
	/// Set/change your target
	SET_TARGET,
	/// Set/change your target2, on NPC's, this fires when they're knocked out by the red hypo
	SET_TARGET2,
	/// What trigger_location you're in - Can only be gotten, not set!
	SET_LOCATION,
	/// Target that is fired when someone completes the BS_REMOVE behaviorState
	SET_REMOVE_TARGET,
	/// Load the savegame that was auto-saved when you started the holodeck
	SET_LOADGAME,
	/// Lock legs to a certain yaw angle (or "off" or "auto" uses current)
	SET_LOCKYAW,
	/// This name will appear when ent is scanned by tricorder
	SET_FULLNAME,
	/// Make the player look through this ent's eyes - also shunts player movement control to this ent
	SET_VIEWENTITY,
	/// Looping sound to play on entity
	SET_LOOPSOUND,
	/// Specify name of entity to freeze
	SET_ICARUS_FREEZE,
	/// Specify name of entity to unfreeze
	SET_ICARUS_UNFREEZE,

	/// Key of text string to print
	SET_SCROLLTEXT,
	/// Key of text string to print in LCARS frame
	SET_LCARSTEXT,

	// Vectors
	/// Set origin explicitly or with TAG
	SET_ORIGIN,
	/// Set angles explicitly or with TAG
	SET_ANGLES,
	/// Set origin here as soon as the area is clear
	SET_TELEPORT_DEST,

	// Floats
	/// Velocity along X axis
	SET_XVELOCITY,
	/// Velocity along Y axis
	SET_YVELOCITY,
	/// Velocity along Z axis
	SET_ZVELOCITY,
	/// Vertical offset from original origin (offset/ent's speed * 1000ms is duration)
	SET_Z_OFFSET,
	/// Pitch for NPC to turn to
	SET_DPITCH,
	/// Yaw for NPC to turn to
	SET_DYAW,
	/// Speed-up slow down game (0 - 1.0)
	SET_TIMESCALE,
	/// When following an ent with the camera, apply this z ofs
	SET_CAMERA_GROUP_Z_OFS,
	/// How far away NPC can see
	SET_VISRANGE,
	/// How far an NPC can hear
	SET_EARSHOT,
	/// How often to look for enemies (0 - 1.0)
	SET_VIGILANCE,
	/// Change this ent's gravity - 800 default
	SET_GRAVITY,
	/// Set face to Aux expression for number of seconds
	SET_FACEAUX,
	/// Set face to Blink expression for number of seconds
	SET_FACEBLINK,
	/// Set face to Blinkfrown expression for number of seconds
	SET_FACEBLINKFROWN,
	/// Set face to Frown expression for number of seconds
	SET_FACEFROWN,
	/// Set face to Normal expression for number of seconds
	SET_FACENORMAL,
	/// Set face to Eyes closed
	SET_FACEEYESCLOSED,
	/// Set face to Eyes open
	SET_FACEEYESOPENED,
	/// Change an entity's wait field
	SET_WAIT,
	/// How far away to stay from leader in BS_FOLLOW_LEADER
	SET_FOLLOWDIST,
	/// Scale the entity model
	SET_SCALE,

	// Ints
	/// Hold lower anim for number of milliseconds
	SET_ANIM_HOLDTIME_LOWER,
	/// Hold upper anim for number of milliseconds
	SET_ANIM_HOLDTIME_UPPER,
	/// Hold lower and upper anims for number of milliseconds
	SET_ANIM_HOLDTIME_BOTH,
	/// Change health
	SET_HEALTH,
	/// Change armor
	SET_ARMOR,
	/// Change walkSpeed
	SET_WALKSPEED,
	/// Change runSpeed
	SET_RUNSPEED,
	/// Change yawSpeed
	SET_YAWSPEED,
	/// Change aggression 1-5
	SET_AGGRESSION,
	/// Change aim 1-5
	SET_AIM,
	/// Change ent's friction - 6 default
	SET_FRICTION,
	/// How far the ent can shoot - 0 uses weapon
	SET_SHOOTDIST,
	/// Horizontal field of view
	SET_HFOV,
	/// Vertical field of view
	SET_VFOV,
	/// How many milliseconds to wait before running delayscript
	SET_DELAYSCRIPTTIME,
	/// NPC move forward -127(back) to 127
	SET_FORWARDMOVE,
	/// NPC move right -127(left) to 127
	SET_RIGHTMOVE,
	/// Frame to start animation sequence on
	SET_STARTFRAME,
	/// Frame to end animation sequence on
	SET_ENDFRAME,
	/// Frame to set animation sequence to
	SET_ANIMFRAME,
	/// Change an entity's count field
	SET_COUNT,
	/// Time between shots for an NPC - reset to defaults when changes weapon
	SET_SHOT_SPACING,
	/// Amount of time until Mission Status should be shown after death
	SET_MISSIONSTATUSTIME,
	/// Width of NPC bounding box
	SET_WIDTH,

	// Booleans
	/// Do not react to pain
	SET_IGNOREPAIN,
	/// Do not acquire enemies
	SET_IGNOREENEMIES,
	/// Do not get enemy set by allies in area (ambush)
	SET_IGNOREALERTS,
	/// Others won't shoot you
	SET_DONTSHOOT,
	/// Others won't pick you as enemy
	SET_NOTARGET,
	/// Don't fire your weapon
	SET_DONTFIRE,
	/// Keep current enemy until dead
	SET_LOCKED_ENEMY,
	/// Force NPC to crouch
	SET_CROUCHED,
	/// Force NPC to move at walkSpeed
	SET_WALKING,
	/// Force NPC to move at runSpeed
	SET_RUNNING,
	/// NPC will chase after enemies
	SET_CHASE_ENEMIES,
	/// NPC will be on the lookout for enemies
	SET_LOOK_FOR_ENEMIES,
	/// NPC will face in the direction it's moving
	SET_FACE_MOVE_DIR,
	/// NPC will not run from danger
	SET_DONT_FLEE,
	/// NPC will not move unless you aim at him
	SET_FORCED_MARCH,
	/// Can take damage down to 1 but not die
	SET_UNDYING,
	/// Will not avoid other NPCs or architecture
	SET_NOAVOID,
	/// Make yourself notsolid or solid
	SET_SOLID,
	/// Can be activated by the player's "use" button
	SET_PLAYER_USABLE,
	/// For non-NPCs, loop your animation sequence
	SET_LOOP_ANIM,
	/// Player interface on/off
	SET_INTERFACE,
	/// NPC has no shields (Borg do not adapt)
	SET_SHIELDS,
	/// Makes an NPC not solid and not visible
	SET_INVISIBLE,
	/// Draws only in mirrors/portals
	SET_VAMPIRE,
	/// Force Invincibility effect, also godmode
	SET_FORCE_INVINCIBLE,
	/// Makes an NPC greet teammates
	SET_GREET_ALLIES,
	/// Makes video playback fade in
	SET_VIDEO_FADE_IN,
	/// Makes video playback fade out
	SET_VIDEO_FADE_OUT,
	/// Makes it so player cannot move
	SET_PLAYER_LOCKED,
	/// Makes it so player cannot switch weapons
	SET_LOCK_PLAYER_WEAPONS,
	/// Stops this ent from taking impact damage
	SET_NO_IMPACT_DAMAGE,
	/// Stops this ent from taking knockback from weapons
	SET_NO_KNOCKBACK,
	/// Force NPC to use altfire when shooting
	SET_ALT_FIRE,
	/// NPCs will do generic responses when this is on (usescripts override generic responses as well)
	SET_NO_RESPONSE,
	/// Completely unkillable
	SET_INVINCIBLE,
	/// Turns on Mission Status Screen
	SET_MISSIONSTATUSACTIVE,
	/// NPCs will not do their combat talking noises when this is on
	SET_NO_COMBAT_TALK,
	/// NPCs will not do their combat talking noises when this is on
	SET_NO_ALERT_TALK,
	/// Player has turned on his own - scripts will stop, NPCs will turn on him and level changes load the brig
	SET_TREASONED,
	/// Allows turning off an animating shader in a script
	SET_DISABLE_SHADER_ANIM,
	/// Sets a shader with an image map to be under frame control
	SET_SHADER_ANIM,
	/// Turns saber on/off
	SET_SABERACTIVE,
	/// Only set this on things you move with script commands that you want to open/close area portals (Default is off)
	SET_ADJUST_AREA_PORTALS,
	/// When true, only a heavy weapon class missile/laser can damage this ent
	SET_DMG_BY_HEAVY_WEAP_ONLY,
	/// When true, ion_cannon is shielded from any kind of damage
	SET_SHIELDED,
	/// This NPC cannot alert groups or be part of a group
	SET_NO_GROUPS,
	/// Makes NPC will hold down the fire button, until this is set to false
	SET_FIRE_WEAPON,
	/// Makes NPC immune to jedi mind-trick
	SET_NO_MINDTRICK,
	/// In lieu of using a target_activate or target_deactivate
	SET_INACTIVE,
	/// Provides an alternate way of changing func_usable to be visible or not, DOES NOT AFFECT SOLID
	SET_FUNC_USABLE_VISIBLE,
	/// Increment secret areas found counter
	SET_SECRET_AREA_FOUND,
	/// Display Mission Status screen before advancing to next level
	SET_MISSION_STATUS_SCREEN,
	/// End of game dissolve into star background and credits
	SET_END_SCREENDISSOLVE,
	/// NPCs will use their closest combat points, not try and find ones next to the player, or flank player
	SET_USE_CP_NEAREST,
	/// NPC will have a minlight of 96
	SET_MORELIGHT,
	/// NPC will not be affected by force powers
	SET_NO_FORCE,
	/// NPC will not scream and tumble and fall to hit death over large drops
	SET_NO_FALLTODEATH,
	/// NPC will not be dismemberable if you set this to false (default is true)
	SET_DISMEMBERABLE,
	/// Jedi won't jump, roll or cartwheel
	SET_NO_ACROBATICS,
	/// When true NPC will always display subtitle regardless of subtitle setting
	SET_USE_SUBTITLES,
	/// Removes entities that could muck up cinematics, explosives, turrets, seekers
	SET_CLEAN_DAMAGING_ENTS,
	/// Turns on/off HUD
	SET_HUD,

	// Calls
	/// Cannot set this, only get it - valid values are 0 through 3
	SET_SKILL,

	// Special tables
	/// Torso and head anim
	SET_ANIM_UPPER,
	/// Legs anim
	SET_ANIM_LOWER,
	/// Set same anim on torso and legs
	SET_ANIM_BOTH,
	/// Your team
	SET_PLAYER_TEAM,
	/// Team in which to look for enemies
	SET_ENEMY_TEAM,
	/// Change current bState
	SET_BEHAVIOR_STATE,
	/// Change fallback bState
	SET_DEFAULT_BSTATE,
	/// Set/Change a temp bState
	SET_TEMP_BSTATE,
	/// Events you can initiate
	SET_EVENT,
	/// Change/Stow/Drop weapon
	SET_WEAPON,
	/// Give items
	SET_ITEM,
	/// Set the state of the dynamic music
	SET_MUSIC_STATE,

	/// Change force power level
	SET_FORCE_HEAL_LEVEL,
	/// Change force power level
	SET_FORCE_JUMP_LEVEL,
	/// Change force power level
	SET_FORCE_SPEED_LEVEL,
	/// Change force power level
	SET_FORCE_PUSH_LEVEL,
	/// Change force power level
	SET_FORCE_PULL_LEVEL,
	/// Change force power level
	SET_FORCE_MINDTRICK_LEVEL,
	/// Change force power level
	SET_FORCE_GRIP_LEVEL,
	/// Change force power level
	SET_FORCE_LIGHTNING_LEVEL,
	/// Change force power level
	SET_SABER_THROW,
	/// Change force power level
	SET_SABER_DEFENSE,
	/// Change force power level
	SET_SABER_OFFENSE,

	/// Show objective on mission screen
	SET_OBJECTIVE_SHOW,
	/// Hide objective from mission screen
	SET_OBJECTIVE_HIDE,
	/// Mark objective as completed
	SET_OBJECTIVE_SUCCEEDED,
	/// Mark objective as failed
	SET_OBJECTIVE_FAILED,

	/// Mission failed screen activates
	SET_MISSIONFAILED,

	/// Show tactical info on mission objectives screen
	SET_TACTICAL_SHOW,
	/// Hide tactical info on mission objectives screen
	SET_TACTICAL_HIDE,
	/// Force all objectives to be hidden
	SET_OBJECTIVE_CLEARALL,

	/// Text to appear in mission status screen
	SET_MISSIONSTATUSTEXT,
	/// Brings up specified menu screen
	SET_MENU_SCREEN,

	/// Show closing credits
	SET_CLOSINGCREDITS,

	// In-bhc tables
	/// Lean left, right or stop leaning
	SET_LEAN,

	/// Count sentinel
	SET_,
}

pub static setTable: [stringID_table_t; 212] = [
    stringID_table_t { name: c"SET_SPAWNSCRIPT".as_ptr() as *mut c_char, id: setType_t::SET_SPAWNSCRIPT as c_int },
    stringID_table_t { name: c"SET_USESCRIPT".as_ptr() as *mut c_char, id: setType_t::SET_USESCRIPT as c_int },
    stringID_table_t { name: c"SET_AWAKESCRIPT".as_ptr() as *mut c_char, id: setType_t::SET_AWAKESCRIPT as c_int },
    stringID_table_t { name: c"SET_ANGERSCRIPT".as_ptr() as *mut c_char, id: setType_t::SET_ANGERSCRIPT as c_int },
    stringID_table_t { name: c"SET_ATTACKSCRIPT".as_ptr() as *mut c_char, id: setType_t::SET_ATTACKSCRIPT as c_int },
    stringID_table_t { name: c"SET_VICTORYSCRIPT".as_ptr() as *mut c_char, id: setType_t::SET_VICTORYSCRIPT as c_int },
    stringID_table_t { name: c"SET_PAINSCRIPT".as_ptr() as *mut c_char, id: setType_t::SET_PAINSCRIPT as c_int },
    stringID_table_t { name: c"SET_FLEESCRIPT".as_ptr() as *mut c_char, id: setType_t::SET_FLEESCRIPT as c_int },
    stringID_table_t { name: c"SET_DEATHSCRIPT".as_ptr() as *mut c_char, id: setType_t::SET_DEATHSCRIPT as c_int },
    stringID_table_t { name: c"SET_DELAYEDSCRIPT".as_ptr() as *mut c_char, id: setType_t::SET_DELAYEDSCRIPT as c_int },
    stringID_table_t { name: c"SET_BLOCKEDSCRIPT".as_ptr() as *mut c_char, id: setType_t::SET_BLOCKEDSCRIPT as c_int },
    stringID_table_t { name: c"SET_FFIRESCRIPT".as_ptr() as *mut c_char, id: setType_t::SET_FFIRESCRIPT as c_int },
    stringID_table_t { name: c"SET_FFDEATHSCRIPT".as_ptr() as *mut c_char, id: setType_t::SET_FFDEATHSCRIPT as c_int },
    stringID_table_t { name: c"SET_MINDTRICKSCRIPT".as_ptr() as *mut c_char, id: setType_t::SET_MINDTRICKSCRIPT as c_int },
    stringID_table_t { name: c"SET_NO_MINDTRICK".as_ptr() as *mut c_char, id: setType_t::SET_NO_MINDTRICK as c_int },
    stringID_table_t { name: c"SET_ORIGIN".as_ptr() as *mut c_char, id: setType_t::SET_ORIGIN as c_int },
    stringID_table_t { name: c"SET_TELEPORT_DEST".as_ptr() as *mut c_char, id: setType_t::SET_TELEPORT_DEST as c_int },
    stringID_table_t { name: c"SET_ANGLES".as_ptr() as *mut c_char, id: setType_t::SET_ANGLES as c_int },
    stringID_table_t { name: c"SET_XVELOCITY".as_ptr() as *mut c_char, id: setType_t::SET_XVELOCITY as c_int },
    stringID_table_t { name: c"SET_YVELOCITY".as_ptr() as *mut c_char, id: setType_t::SET_YVELOCITY as c_int },
    stringID_table_t { name: c"SET_ZVELOCITY".as_ptr() as *mut c_char, id: setType_t::SET_ZVELOCITY as c_int },
    stringID_table_t { name: c"SET_Z_OFFSET".as_ptr() as *mut c_char, id: setType_t::SET_Z_OFFSET as c_int },
    stringID_table_t { name: c"SET_ENEMY".as_ptr() as *mut c_char, id: setType_t::SET_ENEMY as c_int },
    stringID_table_t { name: c"SET_LEADER".as_ptr() as *mut c_char, id: setType_t::SET_LEADER as c_int },
    stringID_table_t { name: c"SET_NAVGOAL".as_ptr() as *mut c_char, id: setType_t::SET_NAVGOAL as c_int },
    stringID_table_t { name: c"SET_ANIM_UPPER".as_ptr() as *mut c_char, id: setType_t::SET_ANIM_UPPER as c_int },
    stringID_table_t { name: c"SET_ANIM_LOWER".as_ptr() as *mut c_char, id: setType_t::SET_ANIM_LOWER as c_int },
    stringID_table_t { name: c"SET_ANIM_BOTH".as_ptr() as *mut c_char, id: setType_t::SET_ANIM_BOTH as c_int },
    stringID_table_t { name: c"SET_ANIM_HOLDTIME_LOWER".as_ptr() as *mut c_char, id: setType_t::SET_ANIM_HOLDTIME_LOWER as c_int },
    stringID_table_t { name: c"SET_ANIM_HOLDTIME_UPPER".as_ptr() as *mut c_char, id: setType_t::SET_ANIM_HOLDTIME_UPPER as c_int },
    stringID_table_t { name: c"SET_ANIM_HOLDTIME_BOTH".as_ptr() as *mut c_char, id: setType_t::SET_ANIM_HOLDTIME_BOTH as c_int },
    stringID_table_t { name: c"SET_PLAYER_TEAM".as_ptr() as *mut c_char, id: setType_t::SET_PLAYER_TEAM as c_int },
    stringID_table_t { name: c"SET_ENEMY_TEAM".as_ptr() as *mut c_char, id: setType_t::SET_ENEMY_TEAM as c_int },
    stringID_table_t { name: c"SET_BEHAVIOR_STATE".as_ptr() as *mut c_char, id: setType_t::SET_BEHAVIOR_STATE as c_int },
    stringID_table_t { name: c"SET_BEHAVIOR_STATE".as_ptr() as *mut c_char, id: setType_t::SET_BEHAVIOR_STATE as c_int },
    stringID_table_t { name: c"SET_HEALTH".as_ptr() as *mut c_char, id: setType_t::SET_HEALTH as c_int },
    stringID_table_t { name: c"SET_ARMOR".as_ptr() as *mut c_char, id: setType_t::SET_ARMOR as c_int },
    stringID_table_t { name: c"SET_DEFAULT_BSTATE".as_ptr() as *mut c_char, id: setType_t::SET_DEFAULT_BSTATE as c_int },
    stringID_table_t { name: c"SET_CAPTURE".as_ptr() as *mut c_char, id: setType_t::SET_CAPTURE as c_int },
    stringID_table_t { name: c"SET_DPITCH".as_ptr() as *mut c_char, id: setType_t::SET_DPITCH as c_int },
    stringID_table_t { name: c"SET_DYAW".as_ptr() as *mut c_char, id: setType_t::SET_DYAW as c_int },
    stringID_table_t { name: c"SET_EVENT".as_ptr() as *mut c_char, id: setType_t::SET_EVENT as c_int },
    stringID_table_t { name: c"SET_TEMP_BSTATE".as_ptr() as *mut c_char, id: setType_t::SET_TEMP_BSTATE as c_int },
    stringID_table_t { name: c"SET_COPY_ORIGIN".as_ptr() as *mut c_char, id: setType_t::SET_COPY_ORIGIN as c_int },
    stringID_table_t { name: c"SET_VIEWTARGET".as_ptr() as *mut c_char, id: setType_t::SET_VIEWTARGET as c_int },
    stringID_table_t { name: c"SET_WEAPON".as_ptr() as *mut c_char, id: setType_t::SET_WEAPON as c_int },
    stringID_table_t { name: c"SET_ITEM".as_ptr() as *mut c_char, id: setType_t::SET_ITEM as c_int },
    stringID_table_t { name: c"SET_WALKSPEED".as_ptr() as *mut c_char, id: setType_t::SET_WALKSPEED as c_int },
    stringID_table_t { name: c"SET_RUNSPEED".as_ptr() as *mut c_char, id: setType_t::SET_RUNSPEED as c_int },
    stringID_table_t { name: c"SET_YAWSPEED".as_ptr() as *mut c_char, id: setType_t::SET_YAWSPEED as c_int },
    stringID_table_t { name: c"SET_AGGRESSION".as_ptr() as *mut c_char, id: setType_t::SET_AGGRESSION as c_int },
    stringID_table_t { name: c"SET_AIM".as_ptr() as *mut c_char, id: setType_t::SET_AIM as c_int },
    stringID_table_t { name: c"SET_FRICTION".as_ptr() as *mut c_char, id: setType_t::SET_FRICTION as c_int },
    stringID_table_t { name: c"SET_GRAVITY".as_ptr() as *mut c_char, id: setType_t::SET_GRAVITY as c_int },
    stringID_table_t { name: c"SET_IGNOREPAIN".as_ptr() as *mut c_char, id: setType_t::SET_IGNOREPAIN as c_int },
    stringID_table_t { name: c"SET_IGNOREENEMIES".as_ptr() as *mut c_char, id: setType_t::SET_IGNOREENEMIES as c_int },
    stringID_table_t { name: c"SET_IGNOREALERTS".as_ptr() as *mut c_char, id: setType_t::SET_IGNOREALERTS as c_int },
    stringID_table_t { name: c"SET_DONTSHOOT".as_ptr() as *mut c_char, id: setType_t::SET_DONTSHOOT as c_int },
    stringID_table_t { name: c"SET_DONTFIRE".as_ptr() as *mut c_char, id: setType_t::SET_DONTFIRE as c_int },
    stringID_table_t { name: c"SET_LOCKED_ENEMY".as_ptr() as *mut c_char, id: setType_t::SET_LOCKED_ENEMY as c_int },
    stringID_table_t { name: c"SET_NOTARGET".as_ptr() as *mut c_char, id: setType_t::SET_NOTARGET as c_int },
    stringID_table_t { name: c"SET_LEAN".as_ptr() as *mut c_char, id: setType_t::SET_LEAN as c_int },
    stringID_table_t { name: c"SET_CROUCHED".as_ptr() as *mut c_char, id: setType_t::SET_CROUCHED as c_int },
    stringID_table_t { name: c"SET_WALKING".as_ptr() as *mut c_char, id: setType_t::SET_WALKING as c_int },
    stringID_table_t { name: c"SET_RUNNING".as_ptr() as *mut c_char, id: setType_t::SET_RUNNING as c_int },
    stringID_table_t { name: c"SET_CHASE_ENEMIES".as_ptr() as *mut c_char, id: setType_t::SET_CHASE_ENEMIES as c_int },
    stringID_table_t { name: c"SET_LOOK_FOR_ENEMIES".as_ptr() as *mut c_char, id: setType_t::SET_LOOK_FOR_ENEMIES as c_int },
    stringID_table_t { name: c"SET_FACE_MOVE_DIR".as_ptr() as *mut c_char, id: setType_t::SET_FACE_MOVE_DIR as c_int },
    stringID_table_t { name: c"SET_ALT_FIRE".as_ptr() as *mut c_char, id: setType_t::SET_ALT_FIRE as c_int },
    stringID_table_t { name: c"SET_DONT_FLEE".as_ptr() as *mut c_char, id: setType_t::SET_DONT_FLEE as c_int },
    stringID_table_t { name: c"SET_FORCED_MARCH".as_ptr() as *mut c_char, id: setType_t::SET_FORCED_MARCH as c_int },
    stringID_table_t { name: c"SET_NO_RESPONSE".as_ptr() as *mut c_char, id: setType_t::SET_NO_RESPONSE as c_int },
    stringID_table_t { name: c"SET_NO_COMBAT_TALK".as_ptr() as *mut c_char, id: setType_t::SET_NO_COMBAT_TALK as c_int },
    stringID_table_t { name: c"SET_NO_ALERT_TALK".as_ptr() as *mut c_char, id: setType_t::SET_NO_ALERT_TALK as c_int },
    stringID_table_t { name: c"SET_UNDYING".as_ptr() as *mut c_char, id: setType_t::SET_UNDYING as c_int },
    stringID_table_t { name: c"SET_TREASONED".as_ptr() as *mut c_char, id: setType_t::SET_TREASONED as c_int },
    stringID_table_t { name: c"SET_DISABLE_SHADER_ANIM".as_ptr() as *mut c_char, id: setType_t::SET_DISABLE_SHADER_ANIM as c_int },
    stringID_table_t { name: c"SET_SHADER_ANIM".as_ptr() as *mut c_char, id: setType_t::SET_SHADER_ANIM as c_int },
    stringID_table_t { name: c"SET_INVINCIBLE".as_ptr() as *mut c_char, id: setType_t::SET_INVINCIBLE as c_int },
    stringID_table_t { name: c"SET_NOAVOID".as_ptr() as *mut c_char, id: setType_t::SET_NOAVOID as c_int },
    stringID_table_t { name: c"SET_SHOOTDIST".as_ptr() as *mut c_char, id: setType_t::SET_SHOOTDIST as c_int },
    stringID_table_t { name: c"SET_TARGETNAME".as_ptr() as *mut c_char, id: setType_t::SET_TARGETNAME as c_int },
    stringID_table_t { name: c"SET_TARGET".as_ptr() as *mut c_char, id: setType_t::SET_TARGET as c_int },
    stringID_table_t { name: c"SET_TARGET2".as_ptr() as *mut c_char, id: setType_t::SET_TARGET2 as c_int },
    stringID_table_t { name: c"SET_LOCATION".as_ptr() as *mut c_char, id: setType_t::SET_LOCATION as c_int },
    stringID_table_t { name: c"SET_PAINTARGET".as_ptr() as *mut c_char, id: setType_t::SET_PAINTARGET as c_int },
    stringID_table_t { name: c"SET_TIMESCALE".as_ptr() as *mut c_char, id: setType_t::SET_TIMESCALE as c_int },
    stringID_table_t { name: c"SET_VISRANGE".as_ptr() as *mut c_char, id: setType_t::SET_VISRANGE as c_int },
    stringID_table_t { name: c"SET_EARSHOT".as_ptr() as *mut c_char, id: setType_t::SET_EARSHOT as c_int },
    stringID_table_t { name: c"SET_VIGILANCE".as_ptr() as *mut c_char, id: setType_t::SET_VIGILANCE as c_int },
    stringID_table_t { name: c"SET_HFOV".as_ptr() as *mut c_char, id: setType_t::SET_HFOV as c_int },
    stringID_table_t { name: c"SET_VFOV".as_ptr() as *mut c_char, id: setType_t::SET_VFOV as c_int },
    stringID_table_t { name: c"SET_DELAYSCRIPTTIME".as_ptr() as *mut c_char, id: setType_t::SET_DELAYSCRIPTTIME as c_int },
    stringID_table_t { name: c"SET_FORWARDMOVE".as_ptr() as *mut c_char, id: setType_t::SET_FORWARDMOVE as c_int },
    stringID_table_t { name: c"SET_RIGHTMOVE".as_ptr() as *mut c_char, id: setType_t::SET_RIGHTMOVE as c_int },
    stringID_table_t { name: c"SET_LOCKYAW".as_ptr() as *mut c_char, id: setType_t::SET_LOCKYAW as c_int },
    stringID_table_t { name: c"SET_SOLID".as_ptr() as *mut c_char, id: setType_t::SET_SOLID as c_int },
    stringID_table_t { name: c"SET_CAMERA_GROUP".as_ptr() as *mut c_char, id: setType_t::SET_CAMERA_GROUP as c_int },
    stringID_table_t { name: c"SET_CAMERA_GROUP_Z_OFS".as_ptr() as *mut c_char, id: setType_t::SET_CAMERA_GROUP_Z_OFS as c_int },
    stringID_table_t { name: c"SET_CAMERA_GROUP_TAG".as_ptr() as *mut c_char, id: setType_t::SET_CAMERA_GROUP_TAG as c_int },
    stringID_table_t { name: c"SET_LOOK_TARGET".as_ptr() as *mut c_char, id: setType_t::SET_LOOK_TARGET as c_int },
    stringID_table_t { name: c"SET_ADDRHANDBOLT_MODEL".as_ptr() as *mut c_char, id: setType_t::SET_ADDRHANDBOLT_MODEL as c_int },
    stringID_table_t { name: c"SET_REMOVERHANDBOLT_MODEL".as_ptr() as *mut c_char, id: setType_t::SET_REMOVERHANDBOLT_MODEL as c_int },
    stringID_table_t { name: c"SET_ADDLHANDBOLT_MODEL".as_ptr() as *mut c_char, id: setType_t::SET_ADDLHANDBOLT_MODEL as c_int },
    stringID_table_t { name: c"SET_REMOVELHANDBOLT_MODEL".as_ptr() as *mut c_char, id: setType_t::SET_REMOVELHANDBOLT_MODEL as c_int },
    stringID_table_t { name: c"SET_FACEAUX".as_ptr() as *mut c_char, id: setType_t::SET_FACEAUX as c_int },
    stringID_table_t { name: c"SET_FACEBLINK".as_ptr() as *mut c_char, id: setType_t::SET_FACEBLINK as c_int },
    stringID_table_t { name: c"SET_FACEBLINKFROWN".as_ptr() as *mut c_char, id: setType_t::SET_FACEBLINKFROWN as c_int },
    stringID_table_t { name: c"SET_FACEFROWN".as_ptr() as *mut c_char, id: setType_t::SET_FACEFROWN as c_int },
    stringID_table_t { name: c"SET_FACENORMAL".as_ptr() as *mut c_char, id: setType_t::SET_FACENORMAL as c_int },
    stringID_table_t { name: c"SET_FACEEYESCLOSED".as_ptr() as *mut c_char, id: setType_t::SET_FACEEYESCLOSED as c_int },
    stringID_table_t { name: c"SET_FACEEYESOPENED".as_ptr() as *mut c_char, id: setType_t::SET_FACEEYESOPENED as c_int },
    stringID_table_t { name: c"SET_SCROLLTEXT".as_ptr() as *mut c_char, id: setType_t::SET_SCROLLTEXT as c_int },
    stringID_table_t { name: c"SET_LCARSTEXT".as_ptr() as *mut c_char, id: setType_t::SET_LCARSTEXT as c_int },
    stringID_table_t { name: c"SET_SCROLLTEXTCOLOR".as_ptr() as *mut c_char, id: setType_t::SET_SCROLLTEXTCOLOR as c_int },
    stringID_table_t { name: c"SET_CAPTIONTEXTCOLOR".as_ptr() as *mut c_char, id: setType_t::SET_CAPTIONTEXTCOLOR as c_int },
    stringID_table_t { name: c"SET_CENTERTEXTCOLOR".as_ptr() as *mut c_char, id: setType_t::SET_CENTERTEXTCOLOR as c_int },
    stringID_table_t { name: c"SET_PLAYER_USABLE".as_ptr() as *mut c_char, id: setType_t::SET_PLAYER_USABLE as c_int },
    stringID_table_t { name: c"SET_STARTFRAME".as_ptr() as *mut c_char, id: setType_t::SET_STARTFRAME as c_int },
    stringID_table_t { name: c"SET_ENDFRAME".as_ptr() as *mut c_char, id: setType_t::SET_ENDFRAME as c_int },
    stringID_table_t { name: c"SET_ANIMFRAME".as_ptr() as *mut c_char, id: setType_t::SET_ANIMFRAME as c_int },
    stringID_table_t { name: c"SET_LOOP_ANIM".as_ptr() as *mut c_char, id: setType_t::SET_LOOP_ANIM as c_int },
    stringID_table_t { name: c"SET_INTERFACE".as_ptr() as *mut c_char, id: setType_t::SET_INTERFACE as c_int },
    stringID_table_t { name: c"SET_SHIELDS".as_ptr() as *mut c_char, id: setType_t::SET_SHIELDS as c_int },
    stringID_table_t { name: c"SET_NO_KNOCKBACK".as_ptr() as *mut c_char, id: setType_t::SET_NO_KNOCKBACK as c_int },
    stringID_table_t { name: c"SET_INVISIBLE".as_ptr() as *mut c_char, id: setType_t::SET_INVISIBLE as c_int },
    stringID_table_t { name: c"SET_VAMPIRE".as_ptr() as *mut c_char, id: setType_t::SET_VAMPIRE as c_int },
    stringID_table_t { name: c"SET_FORCE_INVINCIBLE".as_ptr() as *mut c_char, id: setType_t::SET_FORCE_INVINCIBLE as c_int },
    stringID_table_t { name: c"SET_GREET_ALLIES".as_ptr() as *mut c_char, id: setType_t::SET_GREET_ALLIES as c_int },
    stringID_table_t { name: c"SET_PLAYER_LOCKED".as_ptr() as *mut c_char, id: setType_t::SET_PLAYER_LOCKED as c_int },
    stringID_table_t { name: c"SET_LOCK_PLAYER_WEAPONS".as_ptr() as *mut c_char, id: setType_t::SET_LOCK_PLAYER_WEAPONS as c_int },
    stringID_table_t { name: c"SET_NO_IMPACT_DAMAGE".as_ptr() as *mut c_char, id: setType_t::SET_NO_IMPACT_DAMAGE as c_int },
    stringID_table_t { name: c"SET_PARM1".as_ptr() as *mut c_char, id: setType_t::SET_PARM1 as c_int },
    stringID_table_t { name: c"SET_PARM2".as_ptr() as *mut c_char, id: setType_t::SET_PARM2 as c_int },
    stringID_table_t { name: c"SET_PARM3".as_ptr() as *mut c_char, id: setType_t::SET_PARM3 as c_int },
    stringID_table_t { name: c"SET_PARM4".as_ptr() as *mut c_char, id: setType_t::SET_PARM4 as c_int },
    stringID_table_t { name: c"SET_PARM5".as_ptr() as *mut c_char, id: setType_t::SET_PARM5 as c_int },
    stringID_table_t { name: c"SET_PARM6".as_ptr() as *mut c_char, id: setType_t::SET_PARM6 as c_int },
    stringID_table_t { name: c"SET_PARM7".as_ptr() as *mut c_char, id: setType_t::SET_PARM7 as c_int },
    stringID_table_t { name: c"SET_PARM8".as_ptr() as *mut c_char, id: setType_t::SET_PARM8 as c_int },
    stringID_table_t { name: c"SET_PARM9".as_ptr() as *mut c_char, id: setType_t::SET_PARM9 as c_int },
    stringID_table_t { name: c"SET_PARM10".as_ptr() as *mut c_char, id: setType_t::SET_PARM10 as c_int },
    stringID_table_t { name: c"SET_PARM11".as_ptr() as *mut c_char, id: setType_t::SET_PARM11 as c_int },
    stringID_table_t { name: c"SET_PARM12".as_ptr() as *mut c_char, id: setType_t::SET_PARM12 as c_int },
    stringID_table_t { name: c"SET_PARM13".as_ptr() as *mut c_char, id: setType_t::SET_PARM13 as c_int },
    stringID_table_t { name: c"SET_PARM14".as_ptr() as *mut c_char, id: setType_t::SET_PARM14 as c_int },
    stringID_table_t { name: c"SET_PARM15".as_ptr() as *mut c_char, id: setType_t::SET_PARM15 as c_int },
    stringID_table_t { name: c"SET_PARM16".as_ptr() as *mut c_char, id: setType_t::SET_PARM16 as c_int },
    stringID_table_t { name: c"SET_DEFEND_TARGET".as_ptr() as *mut c_char, id: setType_t::SET_DEFEND_TARGET as c_int },
    stringID_table_t { name: c"SET_WAIT".as_ptr() as *mut c_char, id: setType_t::SET_WAIT as c_int },
    stringID_table_t { name: c"SET_COUNT".as_ptr() as *mut c_char, id: setType_t::SET_COUNT as c_int },
    stringID_table_t { name: c"SET_SHOT_SPACING".as_ptr() as *mut c_char, id: setType_t::SET_SHOT_SPACING as c_int },
    stringID_table_t { name: c"SET_VIDEO_PLAY".as_ptr() as *mut c_char, id: setType_t::SET_VIDEO_PLAY as c_int },
    stringID_table_t { name: c"SET_VIDEO_FADE_IN".as_ptr() as *mut c_char, id: setType_t::SET_VIDEO_FADE_IN as c_int },
    stringID_table_t { name: c"SET_VIDEO_FADE_OUT".as_ptr() as *mut c_char, id: setType_t::SET_VIDEO_FADE_OUT as c_int },
    stringID_table_t { name: c"SET_REMOVE_TARGET".as_ptr() as *mut c_char, id: setType_t::SET_REMOVE_TARGET as c_int },
    stringID_table_t { name: c"SET_LOADGAME".as_ptr() as *mut c_char, id: setType_t::SET_LOADGAME as c_int },
    stringID_table_t { name: c"SET_MENU_SCREEN".as_ptr() as *mut c_char, id: setType_t::SET_MENU_SCREEN as c_int },
    stringID_table_t { name: c"SET_OBJECTIVE_SHOW".as_ptr() as *mut c_char, id: setType_t::SET_OBJECTIVE_SHOW as c_int },
    stringID_table_t { name: c"SET_OBJECTIVE_HIDE".as_ptr() as *mut c_char, id: setType_t::SET_OBJECTIVE_HIDE as c_int },
    stringID_table_t { name: c"SET_OBJECTIVE_SUCCEEDED".as_ptr() as *mut c_char, id: setType_t::SET_OBJECTIVE_SUCCEEDED as c_int },
    stringID_table_t { name: c"SET_OBJECTIVE_FAILED".as_ptr() as *mut c_char, id: setType_t::SET_OBJECTIVE_FAILED as c_int },
    stringID_table_t { name: c"SET_MISSIONFAILED".as_ptr() as *mut c_char, id: setType_t::SET_MISSIONFAILED as c_int },
    stringID_table_t { name: c"SET_TACTICAL_SHOW".as_ptr() as *mut c_char, id: setType_t::SET_TACTICAL_SHOW as c_int },
    stringID_table_t { name: c"SET_TACTICAL_HIDE".as_ptr() as *mut c_char, id: setType_t::SET_TACTICAL_HIDE as c_int },
    stringID_table_t { name: c"SET_FOLLOWDIST".as_ptr() as *mut c_char, id: setType_t::SET_FOLLOWDIST as c_int },
    stringID_table_t { name: c"SET_SCALE".as_ptr() as *mut c_char, id: setType_t::SET_SCALE as c_int },
    stringID_table_t { name: c"SET_OBJECTIVE_CLEARALL".as_ptr() as *mut c_char, id: setType_t::SET_OBJECTIVE_CLEARALL as c_int },
    stringID_table_t { name: c"SET_MISSIONSTATUSTEXT".as_ptr() as *mut c_char, id: setType_t::SET_MISSIONSTATUSTEXT as c_int },
    stringID_table_t { name: c"SET_WIDTH".as_ptr() as *mut c_char, id: setType_t::SET_WIDTH as c_int },
    stringID_table_t { name: c"SET_CLOSINGCREDITS".as_ptr() as *mut c_char, id: setType_t::SET_CLOSINGCREDITS as c_int },
    stringID_table_t { name: c"SET_SKILL".as_ptr() as *mut c_char, id: setType_t::SET_SKILL as c_int },
    stringID_table_t { name: c"SET_MISSIONSTATUSTIME".as_ptr() as *mut c_char, id: setType_t::SET_MISSIONSTATUSTIME as c_int },
    stringID_table_t { name: c"SET_FULLNAME".as_ptr() as *mut c_char, id: setType_t::SET_FULLNAME as c_int },
    stringID_table_t { name: c"SET_FORCE_HEAL_LEVEL".as_ptr() as *mut c_char, id: setType_t::SET_FORCE_HEAL_LEVEL as c_int },
    stringID_table_t { name: c"SET_FORCE_JUMP_LEVEL".as_ptr() as *mut c_char, id: setType_t::SET_FORCE_JUMP_LEVEL as c_int },
    stringID_table_t { name: c"SET_FORCE_SPEED_LEVEL".as_ptr() as *mut c_char, id: setType_t::SET_FORCE_SPEED_LEVEL as c_int },
    stringID_table_t { name: c"SET_FORCE_PUSH_LEVEL".as_ptr() as *mut c_char, id: setType_t::SET_FORCE_PUSH_LEVEL as c_int },
    stringID_table_t { name: c"SET_FORCE_PULL_LEVEL".as_ptr() as *mut c_char, id: setType_t::SET_FORCE_PULL_LEVEL as c_int },
    stringID_table_t { name: c"SET_FORCE_MINDTRICK_LEVEL".as_ptr() as *mut c_char, id: setType_t::SET_FORCE_MINDTRICK_LEVEL as c_int },
    stringID_table_t { name: c"SET_FORCE_GRIP_LEVEL".as_ptr() as *mut c_char, id: setType_t::SET_FORCE_GRIP_LEVEL as c_int },
    stringID_table_t { name: c"SET_FORCE_LIGHTNING_LEVEL".as_ptr() as *mut c_char, id: setType_t::SET_FORCE_LIGHTNING_LEVEL as c_int },
    stringID_table_t { name: c"SET_SABER_THROW".as_ptr() as *mut c_char, id: setType_t::SET_SABER_THROW as c_int },
    stringID_table_t { name: c"SET_SABER_DEFENSE".as_ptr() as *mut c_char, id: setType_t::SET_SABER_DEFENSE as c_int },
    stringID_table_t { name: c"SET_SABER_OFFENSE".as_ptr() as *mut c_char, id: setType_t::SET_SABER_OFFENSE as c_int },
    stringID_table_t { name: c"SET_VIEWENTITY".as_ptr() as *mut c_char, id: setType_t::SET_VIEWENTITY as c_int },
    stringID_table_t { name: c"SET_WATCHTARGET".as_ptr() as *mut c_char, id: setType_t::SET_WATCHTARGET as c_int },
    stringID_table_t { name: c"SET_SABERACTIVE".as_ptr() as *mut c_char, id: setType_t::SET_SABERACTIVE as c_int },
    stringID_table_t { name: c"SET_ADJUST_AREA_PORTALS".as_ptr() as *mut c_char, id: setType_t::SET_ADJUST_AREA_PORTALS as c_int },
    stringID_table_t { name: c"SET_DMG_BY_HEAVY_WEAP_ONLY".as_ptr() as *mut c_char, id: setType_t::SET_DMG_BY_HEAVY_WEAP_ONLY as c_int },
    stringID_table_t { name: c"SET_SHIELDED".as_ptr() as *mut c_char, id: setType_t::SET_SHIELDED as c_int },
    stringID_table_t { name: c"SET_NO_GROUPS".as_ptr() as *mut c_char, id: setType_t::SET_NO_GROUPS as c_int },
    stringID_table_t { name: c"SET_FIRE_WEAPON".as_ptr() as *mut c_char, id: setType_t::SET_FIRE_WEAPON as c_int },
    stringID_table_t { name: c"SET_INACTIVE".as_ptr() as *mut c_char, id: setType_t::SET_INACTIVE as c_int },
    stringID_table_t { name: c"SET_FUNC_USABLE_VISIBLE".as_ptr() as *mut c_char, id: setType_t::SET_FUNC_USABLE_VISIBLE as c_int },
    stringID_table_t { name: c"SET_MISSION_STATUS_SCREEN".as_ptr() as *mut c_char, id: setType_t::SET_MISSION_STATUS_SCREEN as c_int },
    stringID_table_t { name: c"SET_END_SCREENDISSOLVE".as_ptr() as *mut c_char, id: setType_t::SET_END_SCREENDISSOLVE as c_int },
    stringID_table_t { name: c"SET_LOOPSOUND".as_ptr() as *mut c_char, id: setType_t::SET_LOOPSOUND as c_int },
    stringID_table_t { name: c"SET_ICARUS_FREEZE".as_ptr() as *mut c_char, id: setType_t::SET_ICARUS_FREEZE as c_int },
    stringID_table_t { name: c"SET_ICARUS_UNFREEZE".as_ptr() as *mut c_char, id: setType_t::SET_ICARUS_UNFREEZE as c_int },
    stringID_table_t { name: c"SET_USE_CP_NEAREST".as_ptr() as *mut c_char, id: setType_t::SET_USE_CP_NEAREST as c_int },
    stringID_table_t { name: c"SET_MORELIGHT".as_ptr() as *mut c_char, id: setType_t::SET_MORELIGHT as c_int },
    stringID_table_t { name: c"SET_CINEMATIC_SKIPSCRIPT".as_ptr() as *mut c_char, id: setType_t::SET_CINEMATIC_SKIPSCRIPT as c_int },
    stringID_table_t { name: c"SET_NO_FORCE".as_ptr() as *mut c_char, id: setType_t::SET_NO_FORCE as c_int },
    stringID_table_t { name: c"SET_NO_FALLTODEATH".as_ptr() as *mut c_char, id: setType_t::SET_NO_FALLTODEATH as c_int },
    stringID_table_t { name: c"SET_DISMEMBERABLE".as_ptr() as *mut c_char, id: setType_t::SET_DISMEMBERABLE as c_int },
    stringID_table_t { name: c"SET_NO_ACROBATICS".as_ptr() as *mut c_char, id: setType_t::SET_NO_ACROBATICS as c_int },
    stringID_table_t { name: c"SET_MUSIC_STATE".as_ptr() as *mut c_char, id: setType_t::SET_MUSIC_STATE as c_int },
    stringID_table_t { name: c"SET_USE_SUBTITLES".as_ptr() as *mut c_char, id: setType_t::SET_USE_SUBTITLES as c_int },
    stringID_table_t { name: c"SET_CLEAN_DAMAGING_ENTS".as_ptr() as *mut c_char, id: setType_t::SET_CLEAN_DAMAGING_ENTS as c_int },
    stringID_table_t { name: c"SET_HUD".as_ptr() as *mut c_char, id: setType_t::SET_HUD as c_int },
    stringID_table_t { name: c"".as_ptr() as *mut c_char, id: setType_t::SET_ as c_int },
];
