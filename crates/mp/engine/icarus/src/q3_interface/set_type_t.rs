#![allow(non_camel_case_types, non_snake_case)]

/// Raven `setType_t` — ICARUS entity properties and script parameters.
///
/// Type definition source: `oracle/codemp/icarus/Q3_Interface.h:6-255`
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
