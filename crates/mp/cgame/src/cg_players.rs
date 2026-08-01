//! Port of `oracle/codemp/cgame/cg_players.c` — player models, animation, ghoul2 setup and player effects. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::f64::consts::PI;
use core::ffi::{c_char, c_int, c_short, c_void};
use core::mem::ManuallyDrop;
use core::ptr::null_mut;

use mp_abi::cgame::syscalls::CG_CM_MARKFRAGMENTS::markFragment_t;
use mp_bg::bg_g2_utils::BG_AttachToRancor;
use mp_bg::bg_misc::{
    forcePowerDarkLight, BG_EmplacedView, BG_EvaluateTrajectory, BG_GiveMeVectorFromMatrix,
    BG_IsValidCharacterModel, BG_ValidateSkinForTeam,
};
use mp_bg::bg_panimate::{
    BG_FlippingAnim, BG_InDeathAnim, BG_ParseAnimationEvtFile, BG_ParseAnimationFile,
    BG_SaberInAttack, BG_SaberStartTransAnim, BG_SuperBreakWinAnim,
};
use mp_bg::bg_pmove::{
    BG_G2ATSTAngles, BG_G2PlayerAngles, BG_IK_MoveArm, PM_RunningAnim, PM_WalkingAnim,
};
use mp_bg::bg_saber::{SFL2_NO_BLADE, SFL2_NO_DLIGHT, SFL2_NO_WALL_MARKS};
use mp_bg::bg_saberLoad::{
    BG_SI_SetDesiredLength, BG_SI_SetLength, BG_SI_SetLengthGradual,
    WP_SaberBladeUseSecondBladeStyle, WP_SetSaber,
};
use mp_bg::bg_saga::{BG_SiegeFindClassByName, BG_SiegeFindClassIndexByName};
use mp_bg::bg_vehicleLoad::{
    AttachRidersGeneric, BG_GetVehicleModelName, BG_GetVehicleSkinName, BG_VehicleGetIndex,
};
use mp_bg::cstr_util::cstr_to_str;
use mp_bg::local::{bgToggleableSurfaceDebris, bgToggleableSurfaces, bg_customSiegeSoundNames};
use mp_bg::public::anim_event_type::animEventType_t;
use mp_bg::public::anim_number::animNumber_t;
use mp_bg::public::anim_table::animTable;
use mp_bg::public::animevent::{animevent_t, MAX_RANDOM_ANIM_SOUNDS};
use mp_bg::public::bg_loaded_events::MAX_ANIM_EVENTS;
use mp_bg::public::broken_limb::brokenLimb_t;
use mp_bg::public::configstring::{CS_G2BONES, CS_MODELS, CS_PLAYERS};
use mp_bg::public::duel_team::duelTeam_t::DUELTEAM_DOUBLE;
use mp_bg::public::entity_effects::{
    EF2_GENERIC_NPC_FLAG, EF2_HELD_BY_MONSTER, EF2_HYPERSPACE, EF2_SHIP_DEATH,
};
use mp_bg::public::entity_flags::{
    EF_BODYPUSH, EF_CONNECTION, EF_DEAD, EF_DISINTEGRATION, EF_INVULNERABLE, EF_JETPACK,
    EF_JETPACK_ACTIVE, EF_JETPACK_FLAMING, EF_NODRAW, EF_PERMANENT, EF_RAG, EF_SEEKERDRONE,
    EF_TALK,
};
use mp_bg::public::entity_type::entityType_t;
use mp_bg::public::footstep_type::footstepType_t;
use mp_bg::public::g2_model_parts::g2ModelParts_t;
use mp_bg::public::gametype::{
    GT_CTY, GT_DUEL, GT_HOLOCRON, GT_JEDIMASTER, GT_POWERDUEL, GT_SIEGE, GT_TEAM,
};
use mp_bg::public::gender::gender_t;
use mp_bg::public::hyperspace::HYPERSPACE_TIME;
use mp_bg::public::pers_enum::persEnum_t::PERS_TEAM;
use mp_bg::public::pmtype::pmtype_t::PM_INTERMISSION;
use mp_bg::public::powerup::{
    PW_BLUEFLAG, PW_CLOAKED, PW_DISINT_4, PW_FORCE_BOON, PW_FORCE_ENLIGHTENED_DARK,
    PW_FORCE_ENLIGHTENED_LIGHT, PW_NEUTRALFLAG, PW_PULL, PW_QUAD, PW_REDFLAG, PW_SHIELDHIT,
    PW_SPEED, PW_SPEEDBURST, PW_YSALAMIRI,
};
use mp_bg::public::saber_move_data_table::saberMoveData;
use mp_bg::public::team::{TEAM_BLUE, TEAM_FREE, TEAM_RED, TEAM_SPECTATOR};
use mp_bg::saga::siege_team_t::SIEGETEAM_TEAM1;
use mp_bg::vehicles::vehicle_s::{MAX_VEHICLE_EXHAUSTS, MAX_VEHICLE_MUZZLES, MAX_VEHICLE_TURRETS};
use mp_bg::vehicles::vehicle_type_t::vehicleType_t;
use mp_bg::weapons::weapon_t::{WP_EMPLACED_GUN, WP_SABER, WP_STUN_BATON};
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::tr_types::{
    RF_DISTORTION, RF_FORCEPOST, RF_FORCE_ENT_ALPHA, RF_LIGHTING_ORIGIN, RF_MINLIGHT, RF_NODEPTH,
    RF_RGB_TINT, RF_SHADOW_ONLY, RF_SHADOW_PLANE, RF_THIRD_PERSON,
};
use mp_qshared::common::mp::game::class_t::class_t;
use mp_qshared::common::mp::ghoul2::bone_flags::{
    BONE_ANGLES_POSTMULT, BONE_ANIM_BLEND, BONE_ANIM_OVERRIDE, BONE_ANIM_OVERRIDE_FREEZE,
    BONE_ANIM_OVERRIDE_LOOP,
};
use mp_qshared::common::mp::ghoul2::sskin_gore_data::SSkinGoreData;
use mp_qshared::common::mp::qcommon::collision_record::{G2Trace_t, MAX_G2_COLLISIONS};
use mp_qshared::common::mp::qcommon::saber::blade_info::MAX_BLADES;
use mp_qshared::common::mp::qcommon::saber::saber_colors::{
    saber_colors_t, SABER_BLUE, SABER_GREEN, SABER_ORANGE, SABER_PURPLE, SABER_RED, SABER_YELLOW,
};
use mp_qshared::common::mp::qcommon::saber::saber_flags::{SFL_BOLT_TO_WRIST, SFL_RETURN_DAMAGE};
use mp_qshared::common::mp::qcommon::saber::saber_info::MAX_SABERS;
use mp_qshared::common::mp::qcommon::{
    entityState_t, saberInfo_t, sharedRagDollParams_t, sharedRagDollUpdateParams_t, PMF_FOLLOW,
};
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::com_parse::{COM_ParseExt, COM_ParseString, QSharedScratch};
use mp_qshared::shared::force_powers::{
    FORCE_DARKSIDE, FORCE_LEVEL_2, FORCE_LEVEL_3, FORCE_LIGHTSIDE,
};
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, vec3_origin,
    vectoangles, AngleVectors, AnglesToAxis, CrossProduct, Distance, DistanceSquared,
    MatrixMultiply, VectorClear, VectorInverse, VectorLength, VectorNormalize, VectorSet, PITCH,
    ROLL, YAW,
};
use mp_qshared::shared::q_string::COM_StripExtension;
use mp_qshared::shared::surface_flags::{
    MATERIAL_CANVAS, MATERIAL_CARPET, MATERIAL_DIRT, MATERIAL_FABRIC, MATERIAL_GRAVEL,
    MATERIAL_HOLLOWMETAL, MATERIAL_HOLLOWWOOD, MATERIAL_LONGGRASS, MATERIAL_MASK, MATERIAL_MUD,
    MATERIAL_PLASTIC, MATERIAL_RUBBER, MATERIAL_SAND, MATERIAL_SHORTGRASS, MATERIAL_SNOW,
    MATERIAL_SOLIDMETAL, MATERIAL_SOLIDWOOD,
};
use mp_qshared::shared::trajectory::{trType_t, trajectory_t};
use mp_qshared::shared::{
    addElectricityArgStruct_t, addbezierArgStruct_t, addpolyArgStruct_t, addspriteArgStruct_t,
    effectTrailArgStruct_t, fileHandle_t, mdxaBone_t, orientation_t, qboolean, qfalse, qhandle_t,
    qtrue, sfxHandle_t, sharedEIKMoveState, sharedERagPhase, vec3_t, CollisionRecord_t,
    Eorientations, CHAN_AUTO, CHAN_BODY, CHAN_LESS_ATTEN, CHAN_WEAPON, CONTENTS_BODY,
    CONTENTS_LAVA, CONTENTS_SLIME, CONTENTS_SOLID, CONTENTS_WATER, ENTITYNUM_NONE, ENTITYNUM_WORLD,
    FP_ABSORB, FP_GRIP, FP_PROTECT, FP_RAGE, FP_SABERTHROW, FP_SABER_DEFENSE, FP_SABER_OFFENSE,
    FP_SEE, FP_SPEED, FS_READ, MASK_PLAYERSOLID, MASK_SOLID, MAX_CLIENTS, MAX_CLIENTS_I32,
    MAX_GENTITIES, MAX_QPATH, NUM_FORCE_POWERS, SURF_NOIMPACT,
};
use native_string::{
    atoi, buf_to_string, cstr, strcat_string, strncpyz_string, Info_ValueForKey, Q_stricmp,
    Q_stricmpn, Q_strncpyz, Q_strncpyzBytes,
};

use crate::bg_channel::{CgBgTraps, CgGameCallbacks};
use crate::cg_draw::CG_InFighter;
use crate::cg_ents::{
    forceHolocronModels, CG_AddBracketedEnt, CG_AddRadarEnt, CG_Disintegration,
    CG_ManualEntityRender, CG_SetGhoul2Info, ScaleModelAxis,
};
use crate::cg_event::CG_ReattachLimb;
use crate::cg_localents::CG_AllocLocalEntity;
use crate::cg_main::{
    CG_ConfigString, CG_Error, CG_Printf, Com_Printf, DEFAULT_BLUETEAM_NAME, DEFAULT_MODEL,
    DEFAULT_REDTEAM_NAME,
};
use crate::cg_marks::{CG_AllocMark, CG_ImpactMark};
use crate::cg_predict::{CG_G2Trace, CG_Trace};
use crate::cg_servercmds::CG_HandleNPCSounds;
use crate::cg_weapons::{CG_AddPlayerWeapon, CG_CopyG2WeaponInstance, CG_G2WeaponInstance};
use crate::local::centity_s::centity_t;
use crate::local::client_info_t::{
    clientInfo_t, MAX_CUSTOM_DUEL_SOUNDS, MAX_CUSTOM_SIEGE_SOUNDS, MAX_CUSTOM_SOUNDS, MAX_TEAMNAME,
};
use crate::local::footstep_t::footstep_t;
use crate::local::le_type_t::leType_t;
use crate::local::lerp_frame_t::lerpFrame_t;
use crate::local::mark_poly_s::MAX_VERTS_ON_POLY;
use crate::local::player_state_ref::PlayerStateRef;
use crate::local::vehicle_id::VehicleId;
use crate::trap;
use crate::vehicle_npc::{
    G_CreateAnimalNPC, G_CreateFighterNPC, G_CreateSpeederNPC, G_CreateWalkerNPC,
};
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

// ---------------------------------------------------------------------------
// File-scope constants and tables
// ---------------------------------------------------------------------------

/// Raven `DEFAULT_FEMALE_SOUNDPATH`.
///
/// Raven: the commented-out original was `"chars/tavion/misc"`.
/// Source: `oracle/codemp/cgame/cg_players.c:797`
const DEFAULT_FEMALE_SOUNDPATH: &str = "chars/mp_generic_female/misc";

/// Raven `DEFAULT_MALE_SOUNDPATH`.
///
/// Raven: the commented-out original was `"chars/kyle/misc"`.
/// Source: `oracle/codemp/cgame/cg_players.c:798`
const DEFAULT_MALE_SOUNDPATH: &str = "chars/mp_generic_male/misc";

/// Raven `MAX_SURF_LIST_SIZE` — the `surfOff`/`surfOn` accumulator bound
/// `CG_ParseSurfsFile` folds its comma-joined surface list into.
/// Source: `oracle/codemp/cgame/cg_players.c:313`
const MAX_SURF_LIST_SIZE: usize = 1024;

/// Raven `MAX_SHIELD_TIME` — a double literal, so the `cg.time + MAX_SHIELD_TIME`
/// it feeds is a double expression that truncates back to `int`.
/// Source: `oracle/codemp/cgame/cg_players.c:5073`
const MAX_SHIELD_TIME: f64 = 2000.0;

/// Raven `MIN_SHIELD_TIME` — same double literal, the shield-alpha divisor.
/// Source: `oracle/codemp/cgame/cg_players.c:5074`
const MIN_SHIELD_TIME: f64 = 2000.0;

// `animevent_t.eventData` index constants — Raven's `bg_public.h` `#define`s that
// name the slots in a parsed animation event's payload. They have no ported
// cross-crate home yet, so they land beside `CG_PlayerAnimEventDo`, the only
// reader in this file. Values transcribed verbatim.
// Source: `oracle/codemp/game/bg_public.h:273-302`

/// `AEV_SOUND` payload — first random-sound-index slot.
/// Source: `oracle/codemp/game/bg_public.h:274`
const AED_SOUNDINDEX_START: usize = 0;
/// `AEV_SOUND` payload — how many random sounds to pick from.
/// Source: `oracle/codemp/game/bg_public.h:276`
const AED_SOUND_NUMRANDOMSNDS: usize = MAX_RANDOM_ANIM_SOUNDS;
/// `AEV_SOUNDCHAN` payload — the `soundChannel_t` to play on.
/// Source: `oracle/codemp/game/bg_public.h:279`
const AED_SOUNDCHANNEL: usize = MAX_RANDOM_ANIM_SOUNDS + 2;
/// `AEV_FOOTSTEP` payload — the `footstepType_t`.
/// Source: `oracle/codemp/game/bg_public.h:281`
const AED_FOOTSTEP_TYPE: usize = 0;
/// `AEV_EFFECT` payload — the effect index.
/// Source: `oracle/codemp/game/bg_public.h:284`
const AED_EFFECTINDEX: usize = 0;
/// `AEV_EFFECT` payload — the resolved bolt index (cached after first lookup).
/// Source: `oracle/codemp/game/bg_public.h:285`
const AED_BOLTINDEX: usize = 1;
/// `AEV_EFFECT` payload — which ghoul2 model the bolt lives on.
/// Source: `oracle/codemp/game/bg_public.h:287`
const AED_MODELINDEX: usize = 3;
/// `AEV_SABER_SWING` payload — which saber swung.
/// Source: `oracle/codemp/game/bg_public.h:296`
const AED_SABER_SWING_SABERNUM: usize = 0;
/// `AEV_SABER_SWING` payload — swing strength (fast/medium/strong).
/// Source: `oracle/codemp/game/bg_public.h:297`
const AED_SABER_SWING_TYPE: usize = 1;
/// `AEV_SABER_SPIN` payload — which saber is spinning.
/// Source: `oracle/codemp/game/bg_public.h:300`
const AED_SABER_SPIN_SABERNUM: usize = 0;
/// `AEV_SABER_SPIN` payload — spin sound variant.
/// Source: `oracle/codemp/game/bg_public.h:301`
const AED_SABER_SPIN_TYPE: usize = 1;
/// `AEV_SOUND`/`AEV_SOUNDCHAN` payload — chance-to-play, 0 means always.
/// Source: `oracle/codemp/game/bg_public.h:277`
const AED_SOUND_PROBABILITY: usize = MAX_RANDOM_ANIM_SOUNDS + 1;
/// `AEV_FOOTSTEP` payload — chance-to-play, 0 means always.
/// Source: `oracle/codemp/game/bg_public.h:282`
const AED_FOOTSTEP_PROBABILITY: usize = 1;
/// `AEV_EFFECT` payload — chance-to-play, 0 means always.
/// Source: `oracle/codemp/game/bg_public.h:286`
const AED_EFFECT_PROBABILITY: usize = 2;
/// `AEV_FIRE` payload — chance-to-fire, 0 means always.
/// Source: `oracle/codemp/game/bg_public.h:290`
const AED_FIRE_PROBABILITY: usize = 1;
/// `AEV_SABER_SWING` payload — chance-to-play, 0 means always.
/// Source: `oracle/codemp/game/bg_public.h:298`
const AED_SABER_SWING_PROBABILITY: usize = 2;
/// `AEV_SABER_SPIN` payload — chance-to-play, 0 means always.
/// Source: `oracle/codemp/game/bg_public.h:302`
const AED_SABER_SPIN_PROBABILITY: usize = 2;

/// Raven `cg_customSoundNames[MAX_CUSTOM_SOUNDS]` — the base per-client custom
/// sound set, scanned until the `NULL` sentinel (`None`). The leading `*` is
/// stripped by every reader.
/// Source: `oracle/codemp/cgame/cg_players.c:25-41`
pub static cg_customSoundNames: [Option<&str>; MAX_CUSTOM_SOUNDS] = [
    Some("*death1"),
    Some("*death2"),
    Some("*death3"),
    Some("*jump1"),
    Some("*pain25"),
    Some("*pain50"),
    Some("*pain75"),
    Some("*pain100"),
    Some("*falling1"),
    Some("*choke1"),
    Some("*choke2"),
    Some("*choke3"),
    Some("*gasp"),
    Some("*land1"),
    Some("*taunt"),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

/// Raven `cg_customDuelSoundNames[MAX_CUSTOM_DUEL_SOUNDS]`.
///
/// Raven: Used for DUEL taunts.
/// Source: `oracle/codemp/cgame/cg_players.c:142-158`
pub static cg_customDuelSoundNames: [Option<&str>; MAX_CUSTOM_DUEL_SOUNDS] = [
    // Raven: Say when acquire an enemy when didn't have one before
    Some("*anger1"),
    Some("*anger2"),
    Some("*anger3"),
    // Raven: Say when killed an enemy
    Some("*victory1"),
    Some("*victory2"),
    Some("*victory3"),
    Some("*taunt1"),
    Some("*taunt2"),
    Some("*taunt3"),
    Some("*deflect1"),
    Some("*deflect2"),
    Some("*deflect3"),
    Some("*gloat1"),
    Some("*gloat2"),
    Some("*gloat3"),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

/// Raven `MAX_CUSTOM_COMBAT_SOUNDS`.
/// Source: `oracle/codemp/cgame/cg_local.h:187`
pub const MAX_CUSTOM_COMBAT_SOUNDS: usize = 40;

/// Raven `MAX_CUSTOM_EXTRA_SOUNDS`.
/// Source: `oracle/codemp/cgame/cg_local.h:188`
pub const MAX_CUSTOM_EXTRA_SOUNDS: usize = 40;

/// Raven `MAX_CUSTOM_JEDI_SOUNDS`.
/// Source: `oracle/codemp/cgame/cg_local.h:189`
pub const MAX_CUSTOM_JEDI_SOUNDS: usize = 40;

/// Raven `cg_customCombatSoundNames[MAX_CUSTOM_COMBAT_SOUNDS]` — the enemy/hazard-team supplement set, scanned until the `NULL`
/// sentinel (`None`).
///
/// Raven: (keep numbers in ascending order in order for variant-capping to
/// work).
/// Source: `oracle/codemp/cgame/cg_players.c:47-67`
pub static cg_customCombatSoundNames: [Option<&str>; MAX_CUSTOM_COMBAT_SOUNDS] = [
    Some("*anger1"),
    Some("*anger2"),
    Some("*anger3"),
    Some("*victory1"),
    Some("*victory2"),
    Some("*victory3"),
    Some("*confuse1"),
    Some("*confuse2"),
    Some("*confuse3"),
    Some("*pushed1"),
    Some("*pushed2"),
    Some("*pushed3"),
    Some("*choke1"),
    Some("*choke2"),
    Some("*choke3"),
    Some("*ffwarn"),
    Some("*ffturn"),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

/// Raven `cg_customExtraSoundNames[MAX_CUSTOM_EXTRA_SOUNDS]` — the stormtrooper supplement set, scanned until the `NULL`
/// sentinel (`None`).
///
/// Raven: (keep numbers in ascending order in order for variant-capping to
/// work).
/// Source: `oracle/codemp/cgame/cg_players.c:71-109`
pub static cg_customExtraSoundNames: [Option<&str>; MAX_CUSTOM_EXTRA_SOUNDS] = [
    Some("*chase1"),
    Some("*chase2"),
    Some("*chase3"),
    Some("*cover1"),
    Some("*cover2"),
    Some("*cover3"),
    Some("*cover4"),
    Some("*cover5"),
    Some("*detected1"),
    Some("*detected2"),
    Some("*detected3"),
    Some("*detected4"),
    Some("*detected5"),
    Some("*lost1"),
    Some("*outflank1"),
    Some("*outflank2"),
    Some("*escaping1"),
    Some("*escaping2"),
    Some("*escaping3"),
    Some("*giveup1"),
    Some("*giveup2"),
    Some("*giveup3"),
    Some("*giveup4"),
    Some("*look1"),
    Some("*look2"),
    Some("*sight1"),
    Some("*sight2"),
    Some("*sight3"),
    Some("*sound1"),
    Some("*sound2"),
    Some("*sound3"),
    Some("*suspicious1"),
    Some("*suspicious2"),
    Some("*suspicious3"),
    Some("*suspicious4"),
    Some("*suspicious5"),
    None,
    None,
    None,
    None,
];

/// Raven `cg_customJediSoundNames[MAX_CUSTOM_JEDI_SOUNDS]` — the jedi supplement set, scanned until the `NULL`
/// sentinel (`None`).
///
/// Raven: (keep numbers in ascending order in order for variant-capping to
/// work).
/// Source: `oracle/codemp/cgame/cg_players.c:113-140`
pub static cg_customJediSoundNames: [Option<&str>; MAX_CUSTOM_JEDI_SOUNDS] = [
    Some("*combat1"),
    Some("*combat2"),
    Some("*combat3"),
    Some("*jdetected1"),
    Some("*jdetected2"),
    Some("*jdetected3"),
    Some("*taunt1"),
    Some("*taunt2"),
    Some("*taunt3"),
    Some("*jchase1"),
    Some("*jchase2"),
    Some("*jchase3"),
    Some("*jlost1"),
    Some("*jlost2"),
    Some("*jlost3"),
    Some("*deflect1"),
    Some("*deflect2"),
    Some("*deflect3"),
    Some("*gloat1"),
    Some("*gloat2"),
    Some("*gloat3"),
    Some("*pushfail"),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

/// Raven `SHIPSURF_FRONT` — the impact-damage surface indices. `bg_vehicles.h`'s
/// `#define`s have no ported cross-crate home (mp_game keeps its own copy in
/// `g_vehicles.rs`, which cgame must not depend on), so they land beside their
/// reader.
/// Source: `oracle/codemp/game/bg_vehicles.h:426-429`
const SHIPSURF_FRONT: c_int = 0;
const SHIPSURF_BACK: c_int = 1;
const SHIPSURF_RIGHT: c_int = 2;
const SHIPSURF_LEFT: c_int = 3;

/// Raven `SHIPSURF_DAMAGE_*` — the brokenLimbs damage-severity bit indices.
/// Source: `oracle/codemp/game/bg_vehicles.h:432-439`
const SHIPSURF_DAMAGE_FRONT_LIGHT: c_int = 0;
const SHIPSURF_DAMAGE_BACK_LIGHT: c_int = 1;
const SHIPSURF_DAMAGE_FRONT_HEAVY: c_int = 4;
const SHIPSURF_DAMAGE_BACK_HEAVY: c_int = 5;

/// Raven `FLYBYSOUNDTIME` — the per-vehicle flyby-sound debounce, in ms.
/// Source: `oracle/codemp/cgame/cg_players.c:7979`
const FLYBYSOUNDTIME: c_int = 2000;

/// Raven `BG_NUM_TOGGLEABLE_SURFACES`.
/// Source: `oracle/codemp/game/bg_public.h:138`
const BG_NUM_TOGGLEABLE_SURFACES: usize = 31;

/// Raven `JETPACK_MODEL`.
/// Source: `oracle/codemp/cgame/cg_players.c:7889`
const JETPACK_MODEL: &str = "models/weapons2/jetpack/model.glm";

/// Raven `cg_effectorStringTable[]` — the ragdoll effectors a drag is allowed
/// to kick around.
///
/// Raven: list of valid ragdoll effectors. The entries Raven commented out
/// ("thoracic", "rhand", "rradiusX", "ceyebrow") stay commented out; the `NULL`
/// terminator its `while` loop walks to becomes the slice length.
/// Source: `oracle/codemp/cgame/cg_players.c:3440-3455`
const cg_effectorStringTable: [&str; 8] = [
    //	"thoracic",
    //	"rhand",
    "lhand", "rtibia", "ltibia", "rtalus", "ltalus", //	"rradiusX",
    "lradiusX", "rfemurX", "lfemurX",
    //	"ceyebrow",
];

/// Raven `TURN_ON` — the `G2SURFACEFLAG_OFF`-cleared bit, spelled as a
/// surface-toggle flag to re-show a ghoul2 surface.
/// Source: `oracle/codemp/cgame/cg_players.c:20`
const TURN_ON: c_int = 0x00000000;

/// Raven `TURN_OFF` — the `G2SURFACEFLAG_OFF` bit spelled as a surface-toggle
/// name, used to hide boba_fett's jetpack surfaces.
/// Source: `oracle/codemp/cgame/cg_players.c:21`
const TURN_OFF: c_int = 0x00000100;

/// Raven `SPEED_TRAIL_DISTANCE` — the per-frame spacing (scaled by velocity)
/// between the two force-speed motion-trail refents behind a running player.
/// Source: `oracle/codemp/cgame/cg_players.c:6520`
const SPEED_TRAIL_DISTANCE: c_int = 6;

/// Raven `RARMBIT` — the `torsoBolt` bit for the right arm limb.
/// Source: `oracle/codemp/cgame/cg_players.c:7922`
const RARMBIT: c_int = 1 << (g2ModelParts_t::G2_MODELPART_RARM as c_int - 10);

/// Raven `RHANDBIT` — the `torsoBolt` bit for the right hand limb.
/// Source: `oracle/codemp/cgame/cg_players.c:7923`
const RHANDBIT: c_int = 1 << (g2ModelParts_t::G2_MODELPART_RHAND as c_int - 10);

/// Raven `WAISTBIT` — the `torsoBolt` bit for the waist limb.
/// Source: `oracle/codemp/cgame/cg_players.c:7924`
const WAISTBIT: c_int = 1 << (g2ModelParts_t::G2_MODELPART_WAIST as c_int - 10);

/// Raven `REFRACT_EFFECT_DURATION` — how long the render-to-texture force-push
/// distortion takes to expand and fade.
/// Source: `oracle/codemp/cgame/cg_players.c:4797`
const REFRACT_EFFECT_DURATION: c_int = 500;

/// Raven `FOOTSTEP_DISTANCE` — how far below the foot `_PlayerFootStep` traces
/// down looking for a surface to step on.
/// Source: `oracle/codemp/cgame/cg_players.c:1988`
const FOOTSTEP_DISTANCE: f32 = 32.0;

/// Raven `SHADOW_DISTANCE` — how far below the player `CG_PlayerShadow` traces
/// for the projection/mark shadow (stencil traces the full 4096 instead).
/// Source: `oracle/codemp/cgame/cg_players.c:4611`
const SHADOW_DISTANCE: f32 = 128.0;

/// Raven `SABER_TRAIL_TIME` — a float literal, the default trail-slice lifetime
/// when the move data doesn't name one.
/// Source: `oracle/codemp/cgame/cg_players.c:5986`
const SABER_TRAIL_TIME: f32 = 40.0;

/// Raven `FX_USE_ALPHA` — the trail-arg flag telling the effects system to honor
/// the per-vertex alpha we filled in.
/// Source: `oracle/codemp/cgame/cg_players.c:5987`
const FX_USE_ALPHA: c_int = 0x0800_0000;

/// Raven `cg_pushBoneNames[]` — the bones `CG_ForcePushBodyBlur` bolts a blur
/// effect onto, walked until the `NULL` sentinel (`None`).
/// Source: `oracle/codemp/cgame/cg_players.c:4945-4956`
const cg_pushBoneNames: [Option<&str>; 9] = [
    Some("cranium"),
    Some("lower_lumbar"),
    Some("rhand"),
    Some("lhand"),
    Some("ltibia"),
    Some("rtibia"),
    Some("lradius"),
    Some("rradius"),
    None,
];

/// Raven `MAX_MARK_FRAGMENTS` — the fragment-buffer bound `CG_CreateSaberMarks`
/// hands `trap_CM_MarkFragments`.
/// Source: `oracle/codemp/cgame/cg_players.c:5458`
const MAX_MARK_FRAGMENTS: usize = 128;

/// Raven `MAX_MARK_POINTS` — the point-buffer bound beside
/// [`MAX_MARK_FRAGMENTS`].
/// Source: `oracle/codemp/cgame/cg_players.c:5459`
const MAX_MARK_POINTS: usize = 384;

/// Raven `CG_MAX_SABER_COMP_TIME`.
///
/// Raven: last registered saber entity hit must match within this many ms for
/// the client effect to take place.
/// Source: `oracle/codemp/cgame/cg_players.c:5736`
const CG_MAX_SABER_COMP_TIME: c_int = 400;

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

/// Raven `CG_ColorFromString` — decodes a 1..7 colour index string into an RGB
/// triple; anything outside that range comes back white.
/// Source: `oracle/codemp/cgame/cg_players.c:701-722`
pub fn CG_ColorFromString(v: &str, color: &mut vec3_t) {
    VectorClear(color);

    let val = atoi(v);

    if !(1..=7).contains(&val) {
        VectorSet(color, 1.0, 1.0, 1.0);
        return;
    }

    if val & 1 != 0 {
        color[2] = 1.0;
    }
    if val & 2 != 0 {
        color[1] = 1.0;
    }
    if val & 4 != 0 {
        color[0] = 1.0;
    }
}

/// Raven `CG_ColorFromInt` — the already-parsed twin of
/// [`CG_ColorFromString`].
/// Source: `oracle/codemp/cgame/cg_players.c:729-746`
pub fn CG_ColorFromInt(val: c_int, color: &mut vec3_t) {
    VectorClear(color);

    if !(1..=7).contains(&val) {
        VectorSet(color, 1.0, 1.0, 1.0);
        return;
    }

    if val & 1 != 0 {
        color[2] = 1.0;
    }
    if val & 2 != 0 {
        color[1] = 1.0;
    }
    if val & 4 != 0 {
        color[0] = 1.0;
    }
}

/// Raven `CG_G2SkelForModel` — the animation-set index for a ghoul2 instance,
/// found by swapping the `.gla` filename for its sibling `animation.cfg`.
/// A GLA name with no `/` leaves the index at -1.
/// Source: `oracle/codemp/cgame/cg_players.c:749-767`
pub fn CG_G2SkelForModel(ctx: &mut CgContext, g2: *mut c_void) -> c_int {
    let mut animIndex: c_int = -1;

    let mut GLAName = trap::G2API_GetGLAName(ctx.engine, g2, 0, MAX_QPATH);

    if let Some(slash) = GLAName.rfind('/') {
        GLAName.truncate(slash);
        GLAName.push_str("/animation.cfg");

        let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
        let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
        let filename = cstr(&GLAName);

        animIndex = BG_ParseAnimationFile(
            &mut ctx.world.bg_state,
            &traps,
            &mut callbacks,
            filename.as_ptr(),
            null_mut(),
            qfalse,
        );
    }

    animIndex
}

/// Raven `CG_G2EvIndexForModel` — the animation-event-set index for a ghoul2
/// instance, parsed from the `animevents.cfg` beside its `.gla`.
/// Source: `oracle/codemp/cgame/cg_players.c:770-795`
pub fn CG_G2EvIndexForModel(ctx: &mut CgContext, g2: *mut c_void, animIndex: c_int) -> c_int {
    let mut evtIndex: c_int = -1;

    if animIndex == -1 {
        debug_assert!(false, "shouldn't happen, bad animIndex");
        return -1;
    }

    let mut GLAName = trap::G2API_GetGLAName(ctx.engine, g2, 0, MAX_QPATH);

    if let Some(slash) = GLAName.rfind('/') {
        // Raven cuts just past the slash, leaving the directory with its
        // trailing separator.
        GLAName.truncate(slash + 1);

        let bgNumAnimEvents = ctx.world.bg_state.bgNumAnimEvents;
        let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
        let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
        evtIndex = BG_ParseAnimationEvtFile(
            &mut ctx.world.bg_state,
            &traps,
            &mut callbacks,
            &GLAName,
            animIndex,
            bgNumAnimEvents,
        );
    }

    evtIndex
}

/// Raven `CG_LoadCISounds` — registers a client's custom sound set, following
/// the `sounds*.cfg` redirect (and its `f` gender marker) when the model ships
/// one, otherwise falling back to the generic male/female path.
/// Source: `oracle/codemp/cgame/cg_players.c:799-996`
pub fn CG_LoadCISounds(ctx: &mut CgContext, ci: &mut clientInfo_t, modelloaded: bool) {
    let mut f = 0;
    let mut isFemale = false;
    #[allow(unused_assignments)]
    let mut fLen;

    let dir = buf_to_string(&ci.modelName.iter().map(|&c| c as u8).collect::<Vec<u8>>());
    let skinName = buf_to_string(&ci.skinName.iter().map(|&c| c as u8).collect::<Vec<u8>>());

    // PORT-NOTE: Raven's `!ci->skinName` tests an array, so it is always false;
    // only the `Q_stricmp` half of the disjunction ever decides.
    if Q_stricmp("default", &skinName) == 0 {
        // try default sounds.cfg first
        fLen = trap::FS_FOpenFile(
            ctx.engine,
            &format!("models/players/{}/sounds.cfg", dir),
            &mut f,
            FS_READ,
        );
        if f == 0 {
            // no?  Look for _default sounds.cfg
            fLen = trap::FS_FOpenFile(
                ctx.engine,
                &format!("models/players/{}/sounds_default.cfg", dir),
                &mut f,
                FS_READ,
            );
        }
    } else {
        // use the .skin associated with this skin
        fLen = trap::FS_FOpenFile(
            ctx.engine,
            &format!("models/players/{}/sounds_{}.cfg", dir, skinName),
            &mut f,
            FS_READ,
        );
        if f == 0 {
            // fall back to default sounds
            fLen = trap::FS_FOpenFile(
                ctx.engine,
                &format!("models/players/{}/sounds.cfg", dir),
                &mut f,
                FS_READ,
            );
        }
    }

    let mut soundpath = [0u8; MAX_QPATH];

    if f != 0 {
        // PORT-NOTE: Raven reads `fLen` bytes into a MAX_QPATH buffer and then
        // writes `soundpath[fLen] = 0` — a file longer than 63 bytes overruns
        // the stack (§19 UB). The port clamps to the buffer instead.
        let fClamped = (fLen.max(0) as usize).min(MAX_QPATH - 1);
        trap::FS_Read(ctx.engine, &mut soundpath[..fClamped], f);
        soundpath[fClamped] = 0;

        let mut i = fClamped as isize;

        while i >= 0 && soundpath[i as usize] != b'\n' {
            if soundpath[i as usize] == b'f' {
                isFemale = true;
                soundpath[i as usize] = 0;
            }

            i -= 1;
        }

        i = 0;

        while soundpath[i as usize] != 0
            && soundpath[i as usize] != b'\r'
            && soundpath[i as usize] != b'\n'
        {
            i += 1;
        }
        soundpath[i as usize] = 0;

        trap::FS_FCloseFile(ctx.engine, f);
    }

    // the redirect line, once the gender marker and the line ending are cut off
    let soundpath = buf_to_string(&soundpath);

    ci.gender = if isFemale {
        gender_t::GENDER_FEMALE
    } else {
        gender_t::GENDER_MALE
    };

    trap::S_ShutUp(ctx.engine, true);

    for i in 0..MAX_CUSTOM_SOUNDS {
        let s = match cg_customSoundNames[i] {
            Some(s) => s,
            None => break,
        };

        // strip the extension because we might want .mp3's
        let soundName = COM_StripExtension(&s[1..]);

        ci.sounds[i] = 0;
        // if the model didn't load use the sounds of the default model
        if !soundpath.is_empty() {
            ci.sounds[i] = trap::S_RegisterSound(
                ctx.engine,
                &format!("sound/chars/{}/misc/{}", soundpath, soundName),
            );
        } else if modelloaded {
            ci.sounds[i] = trap::S_RegisterSound(
                ctx.engine,
                &format!("sound/chars/{}/misc/{}", dir, soundName),
            );
        }

        if ci.sounds[i] == 0 {
            // failed the load, try one out of the generic path
            let generic = if isFemale {
                DEFAULT_FEMALE_SOUNDPATH
            } else {
                DEFAULT_MALE_SOUNDPATH
            };
            ci.sounds[i] =
                trap::S_RegisterSound(ctx.engine, &format!("sound/{}/{}", generic, soundName));
        }
    }

    if ctx.world.cgs.gametype >= GT_TEAM || ctx.world.cvars.cg_buildScript.integer != 0 {
        // load the siege sounds then
        for i in 0..MAX_CUSTOM_SIEGE_SOUNDS {
            let s = match bg_customSiegeSoundNames[i] {
                Some(s) => s.to_str().unwrap_or(""),
                None => break,
            };

            // strip the extension because we might want .mp3's
            let soundName = COM_StripExtension(&s[1..]);

            ci.siegeSounds[i] = 0;
            // if the model didn't load use the sounds of the default model
            //
            // PORT-NOTE: this arm's path has no `chars/` prefix, unlike the
            // other two sound loops — Raven's own asymmetry, kept.
            if !soundpath.is_empty() {
                ci.siegeSounds[i] = trap::S_RegisterSound(
                    ctx.engine,
                    &format!("sound/{}/{}", soundpath, soundName),
                );
            } else if modelloaded {
                ci.siegeSounds[i] = trap::S_RegisterSound(
                    ctx.engine,
                    &format!("sound/chars/{}/misc/{}", dir, soundName),
                );
            }

            if ci.siegeSounds[i] == 0 {
                // failed the load, try one out of the generic path
                let generic = if isFemale {
                    DEFAULT_FEMALE_SOUNDPATH
                } else {
                    DEFAULT_MALE_SOUNDPATH
                };
                ci.siegeSounds[i] =
                    trap::S_RegisterSound(ctx.engine, &format!("sound/{}/{}", generic, soundName));
            }
        }
    }

    if ctx.world.cgs.gametype == GT_DUEL
        || ctx.world.cgs.gametype == GT_POWERDUEL
        || ctx.world.cvars.cg_buildScript.integer != 0
    {
        // load the Duel sounds then
        for i in 0..MAX_CUSTOM_DUEL_SOUNDS {
            let s = match cg_customDuelSoundNames[i] {
                Some(s) => s,
                None => break,
            };

            // strip the extension because we might want .mp3's
            let soundName = COM_StripExtension(&s[1..]);

            ci.duelSounds[i] = 0;
            // if the model didn't load use the sounds of the default model
            if !soundpath.is_empty() {
                ci.duelSounds[i] = trap::S_RegisterSound(
                    ctx.engine,
                    &format!("sound/chars/{}/misc/{}", soundpath, soundName),
                );
            } else if modelloaded {
                ci.duelSounds[i] = trap::S_RegisterSound(
                    ctx.engine,
                    &format!("sound/chars/{}/misc/{}", dir, soundName),
                );
            }

            if ci.duelSounds[i] == 0 {
                // failed the load, try one out of the generic path
                let generic = if isFemale {
                    DEFAULT_FEMALE_SOUNDPATH
                } else {
                    DEFAULT_MALE_SOUNDPATH
                };
                ci.duelSounds[i] =
                    trap::S_RegisterSound(ctx.engine, &format!("sound/{}/{}", generic, soundName));
            }
        }
    }

    trap::S_ShutUp(ctx.engine, false);
}

/// Raven `CG_InitG2SaberData` — builds the ghoul2 instance for one of a
/// client's sabers and bolts every blade tag on it.
/// Source: `oracle/codemp/cgame/cg_players.c:1151-1203`
pub fn CG_InitG2SaberData(ctx: &mut CgContext, saberNum: usize, ci: &mut clientInfo_t) {
    let model = buf_to_string(
        &ci.saber[saberNum]
            .model
            .iter()
            .map(|&c| c as u8)
            .collect::<Vec<u8>>(),
    );
    let skin = ci.saber[saberNum].skin;
    let saberFlags = ci.saber[saberNum].saberFlags;
    let numBlades = ci.saber[saberNum].numBlades;

    trap::G2API_InitGhoul2Model(
        ctx.engine,
        &mut ci.ghoul2Weapons[saberNum] as *mut *mut c_void,
        &model,
        0,
        skin,
        0,
        0,
        0,
    );

    if !ci.ghoul2Weapons[saberNum].is_null() {
        let g2 = ci.ghoul2Weapons[saberNum];
        let mut k = 0;

        if skin != 0 {
            trap::G2API_SetSkin(ctx.engine, g2, 0, skin, skin);
        }

        if saberFlags & SFL_BOLT_TO_WRIST != 0 {
            trap::G2API_SetBoltInfo(ctx.engine, g2, 0, 3 + saberNum as c_int);
        } else {
            trap::G2API_SetBoltInfo(ctx.engine, g2, 0, saberNum as c_int);
        }

        while k < numBlades {
            let tagName = format!("*blade{}", k + 1);
            let tagBolt = trap::G2API_AddBolt(ctx.engine, g2, 0, &tagName);

            if tagBolt == -1 {
                if k == 0 {
                    // guess this is an 0ldsk3wl saber
                    let tagBolt = trap::G2API_AddBolt(ctx.engine, g2, 0, "*flash");

                    if tagBolt == -1 {
                        debug_assert!(false, "no *blade1 and no *flash on this saber");
                    }
                    break;
                }

                // Raven re-tests the same value it just branched on, so this
                // arm always takes the break.
                debug_assert!(false, "missing blade tag on this saber");
                break;
            }

            k += 1;
        }
    }
}

/// Raven `CG_CopyClientInfoModel` — moves one client's registered model, bolts
/// and sound handles onto another, duplicating (never sharing) the ghoul2
/// instance.
/// Source: `oracle/codemp/cgame/cg_players.c:1211-1281`
pub fn CG_CopyClientInfoModel(ctx: &mut CgContext, from: &clientInfo_t, to: &mut clientInfo_t) {
    _VectorCopy(from.headOffset, &mut to.headOffset);
    // to->footsteps = from->footsteps;
    to.gender = from.gender;

    to.legsModel = from.legsModel;
    to.legsSkin = from.legsSkin;
    to.torsoModel = from.torsoModel;
    to.torsoSkin = from.torsoSkin;
    //to->headModel = from->headModel;
    //to->headSkin = from->headSkin;
    to.modelIcon = from.modelIcon;

    to.newAnims = from.newAnims;

    //to->ghoul2Model = from->ghoul2Model;
    //rww - Trying to use the same ghoul2 pointer for two seperate clients == DISASTER
    //
    // PORT-NOTE: Raven's `assert(to->ghoul2Model != from->ghoul2Model)` fires
    // whenever both are NULL (the ordinary unregistered case), so it is not
    // ported as a runtime assert — retail compiles it out.

    if !to.ghoul2Model.is_null() && trap::G2_HaveWeGhoul2Models(ctx.engine, to.ghoul2Model) {
        trap::G2API_CleanGhoul2Models(ctx.engine, &mut to.ghoul2Model as *mut *mut c_void);
    }
    if !from.ghoul2Model.is_null() && trap::G2_HaveWeGhoul2Models(ctx.engine, from.ghoul2Model) {
        trap::G2API_DuplicateGhoul2Instance(
            ctx.engine,
            from.ghoul2Model,
            &mut to.ghoul2Model as *mut *mut c_void,
        );
    }

    // Raven: Don't do this, I guess. Just leave the saber info in the original,
    // so it will be properly initialized. (The saber-copy block that used to
    // live here is commented out in the oracle, `cg_players.c:1242-1268`.)

    to.bolt_head = from.bolt_head;
    to.bolt_lhand = from.bolt_lhand;
    to.bolt_rhand = from.bolt_rhand;
    to.bolt_motion = from.bolt_motion;
    to.bolt_llumbar = from.bolt_llumbar;

    to.siegeIndex = from.siegeIndex;

    to.sounds = from.sounds;
    to.siegeSounds = from.siegeSounds;
    to.duelSounds = from.duelSounds;
}

/// Raven `CG_LoadDeferredPlayers` — arms the queue flag; the actual load
/// happens at the top of the next `CG_Player`.
/// Source: `oracle/codemp/cgame/cg_players.c:1976-1978`
pub fn CG_LoadDeferredPlayers(world: &mut CgWorld) {
    world.players.cgQueueLoad = true;
}

/// Raven `CG_InRoll` — is this entity mid-roll (either a getup-roll or a plain
/// one) with time left on the animation?
/// Source: `oracle/codemp/cgame/cg_players.c:2721-2744`
pub fn CG_InRoll(world: &CgWorld, cent: &centity_t) -> bool {
    let legsAnim = cent.currentState.legsAnim;

    if legsAnim == animNumber_t::BOTH_GETUP_BROLL_B as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_BROLL_F as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_BROLL_L as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_BROLL_R as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_FROLL_B as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_FROLL_F as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_FROLL_L as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_FROLL_R as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_F as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_B as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_R as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_L as c_int
    {
        if cent.pe.legs.animationTime > world.cg.time {
            return true;
        }
    }

    false
}

/// Raven `CG_InRollAnim` — the plain-roll half of [`CG_InRoll`], with no
/// animation-time test.
/// Source: `oracle/codemp/cgame/cg_players.c:2746-2757`
pub fn CG_InRollAnim(cent: &centity_t) -> bool {
    let legsAnim = cent.currentState.legsAnim;

    legsAnim == animNumber_t::BOTH_ROLL_F as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_B as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_R as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_L as c_int
}

/// Raven `CG_FirstAnimFrame` — false only when the speed scale is unchanged;
/// the frame-position tests it used to make are commented out in the oracle
/// ("I don't care where it is in the anim now, I am going to pick up from the
/// same bone frame").
/// Source: `oracle/codemp/cgame/cg_players.c:3144-3175`
pub fn CG_FirstAnimFrame(lf: &lerpFrame_t, torsoOnly: bool, speedScale: f32) -> bool {
    if torsoOnly {
        if lf.animationTorsoSpeed == speedScale {
            return false;
        }
    } else if lf.animationSpeed == speedScale {
        return false;
    }

    true
}

/// Raven `CG_G2SetBoneAngles` — straight pass-through to the trap.
///
/// Raven: we want to hold off on setting the bone angles until the end of the
/// frame, because every time we set them the entire skeleton has to be
/// reconstructed... We don't want to go with the delayed approach, we want our
/// bolt points and everything to be updated in realtime. We'll just take the
/// reconstructs and live with them. (The deferred `cgBoneAnglePostSet` path is
/// `#if 0`'d out in the oracle, `cg_players.c:3370-3391`.)
/// Source: `oracle/codemp/cgame/cg_players.c:3365-3396`
#[allow(clippy::too_many_arguments)]
pub fn CG_G2SetBoneAngles(
    ctx: &mut CgContext,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boneName: &str,
    angles: &vec3_t,
    flags: c_int,
    up: c_int,
    right: c_int,
    forward: c_int,
    modelList: Option<&mut qhandle_t>,
    blendTime: c_int,
    currentTime: c_int,
) {
    trap::G2API_SetBoneAngles(
        ctx.engine,
        ghoul2,
        modelIndex,
        boneName,
        angles,
        flags,
        up,
        right,
        forward,
        modelList,
        blendTime,
        currentTime,
    );
}

/// Raven `CG_Rag_Trace` — the ragdoll trace callback: world brushes only, so
/// the hit is reported as the world or as nothing at all.
/// Source: `oracle/codemp/cgame/cg_players.c:3407-3411`
#[allow(clippy::too_many_arguments, unused_variables)]
pub fn CG_Rag_Trace(
    ctx: &mut CgContext,
    result: &mut trace_t,
    start: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    end: &vec3_t,
    skipNumber: c_int,
    mask: c_int,
) {
    trap::CM_BoxTrace(
        ctx.engine,
        result,
        start,
        end,
        Some(mins),
        Some(maxs),
        0,
        mask,
    );
    result.entityNum = if result.fraction != 1.0 {
        ENTITYNUM_WORLD as c_short
    } else {
        ENTITYNUM_NONE as c_short
    };
}

/// Raven `CG_RagAnimForPositioning` — picks the deadflop pose that matches
/// which way the ragdoll's pelvis is facing.
/// Source: `oracle/codemp/cgame/cg_players.c:3460-3482`
pub fn CG_RagAnimForPositioning(ctx: &mut CgContext, cent: &centity_t) -> c_int {
    let mut dir: vec3_t = [0.0; 3];
    let mut matrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };

    let engine = ctx.engine;
    let ghoul2 = cent.ghoul2;
    let turAngles = cent.turAngles;
    let lerpOrigin = cent.lerpOrigin;
    let modelScale = cent.modelScale;
    let time = ctx.world.cg.time;

    debug_assert!(!ghoul2.is_null());
    let bolt = trap::G2API_AddBolt(engine, ghoul2, 0, "pelvis");
    debug_assert!(bolt > -1);

    trap::G2API_GetBoltMatrix(
        engine,
        ghoul2,
        0,
        bolt,
        &mut matrix,
        &turAngles,
        &lerpOrigin,
        time,
        Some(&mut ctx.world.cgs.gameModels[0]),
        &modelScale,
    );
    BG_GiveMeVectorFromMatrix(&matrix, Eorientations::NEGATIVE_Z as c_int, &mut dir);

    if dir[2] > 0.0 {
        // facing up
        animNumber_t::BOTH_DEADFLOP2 as c_int
    } else {
        // facing down
        animNumber_t::BOTH_DEADFLOP1 as c_int
    }
}

/// Raven `CG_G2SetHeadBlink` — closes (or reopens) the eyes; one blink in
/// twenty is a wink, which shuts only the left eye and runs faster.
/// Source: `oracle/codemp/cgame/cg_players.c:3969-4006`
pub fn CG_G2SetHeadBlink(ctx: &mut CgContext, cent: &centity_t, bStart: bool) {
    let engine = ctx.engine;
    let ghoul2 = cent.ghoul2;

    let mut desiredAngles: vec3_t = [0.0; 3];
    let mut blendTime: c_int = 80;
    let mut bWink = false;
    let hReye = trap::G2API_AddBolt(engine, ghoul2, 0, "reye");
    let hLeye = trap::G2API_AddBolt(engine, ghoul2, 0, "leye");

    if hLeye == -1 {
        return;
    }

    VectorClear(&mut desiredAngles);

    if bStart {
        desiredAngles[YAW] = -50.0;
        if ctx.world.bg_state.rng.random() > 0.95 {
            bWink = true;
            blendTime /= 3;
        }
    }

    let time = ctx.world.cg.time;
    trap::G2API_SetBoneAngles(
        engine,
        ghoul2,
        0,
        "leye",
        &desiredAngles,
        BONE_ANGLES_POSTMULT,
        Eorientations::POSITIVE_Y as c_int,
        Eorientations::POSITIVE_Z as c_int,
        Eorientations::POSITIVE_X as c_int,
        None,
        blendTime,
        time,
    );

    if hReye == -1 {
        return;
    }

    if !bWink {
        trap::G2API_SetBoneAngles(
            engine,
            ghoul2,
            0,
            "reye",
            &desiredAngles,
            BONE_ANGLES_POSTMULT,
            Eorientations::POSITIVE_Y as c_int,
            Eorientations::POSITIVE_Z as c_int,
            Eorientations::POSITIVE_X as c_int,
            None,
            blendTime,
            time,
        );
    }
}

/// Raven `CG_G2SetHeadAnim` — plays a face animation on the head bone. A
/// negative `frameLerp` in the table means the animation runs backwards.
///
/// §F19: a model whose `animation.cfg` never parsed leaves `localAnimIndex` at
/// -1, which Raven indexes `bgAllAnims[]` with (a defined-nothing read one
/// entry before the table); `-1 as usize` would panic here, so we early-out.
/// Source: `oracle/codemp/cgame/cg_players.c:4013-4057`
pub fn CG_G2SetHeadAnim(ctx: &mut CgContext, cent: &centity_t, anim: c_int) {
    let engine = ctx.engine;
    let ghoul2 = cent.ghoul2;

    let blendTime: c_int = 50;
    // no skeleton parsed yet, nothing to look the animation up in
    if cent.localAnimIndex < 0 {
        return;
    }
    // `bgAllAnims[..].anims` is still `mp_bg`'s raw `animation_t*` table; the
    // whole workspace reads it through a pointer (see `g_vehicles.rs`,
    // `NPC_reactions.rs`, ui's `UI_ParseAnimationFile`).
    let animations = ctx.world.bg_state.bgAllAnims[cent.localAnimIndex as usize].anims;
    let mut animFlags: c_int = BONE_ANIM_OVERRIDE; //| BONE_ANIM_BLEND;

    // animSpeed is 1.0 if the frameLerp (ms/frame) is 50 (20 fps).
    let cg_timescale = ctx.world.cvars.cg_timescale.value;
    let timeScaleMod: f32 = if cg_timescale != 0.0 {
        (1.0f64 / cg_timescale as f64) as f32
    } else {
        1.0
    };

    // SAFETY: `anims` is the animation table `BG_ParseAnimationFile` allocated
    // for this skeleton; Raven indexes it unchecked and so do we.
    let (numFrames, firstFrame, frameLerp) = unsafe {
        let a = &*animations.offset(anim as isize);
        (a.numFrames as c_int, a.firstFrame as c_int, a.frameLerp)
    };

    let animSpeed = 50.0f32 / frameLerp as f32 * timeScaleMod;
    let firstFrameOut;
    let lastFrame;

    if numFrames <= 0 {
        return;
    }
    if anim == animNumber_t::FACE_DEAD as c_int {
        animFlags |= BONE_ANIM_OVERRIDE_FREEZE;
    }
    // animSpeed is 1.0 if the frameLerp (ms/frame) is 50 (20 fps).
    if animSpeed < 0.0 {
        // play anim backwards
        lastFrame = firstFrame - 1;
        firstFrameOut = (numFrames - 1) + firstFrame;
    } else {
        firstFrameOut = firstFrame;
        lastFrame = numFrames + firstFrame;
    }

    // Raven's "are we already animating the head" early-out is commented out
    // (`cg_players.c:4046-4051`), so the anim is always set.
    trap::G2API_SetBoneAnim(
        engine,
        ghoul2,
        0,
        "face",
        firstFrameOut,
        lastFrame,
        animFlags,
        animSpeed,
        ctx.world.cg.time,
        -1.0,
        blendTime,
    );
}

/// Raven `CG_PlayerFloatSprite` — the sprite that floats above a player's head
/// (chat bubble, flag icon). On your own body it renders in mirrors only.
/// Source: `oracle/codemp/cgame/cg_players.c:4505-4527`
pub fn CG_PlayerFloatSprite(ctx: &mut CgContext, cent: &centity_t, shader: qhandle_t) {
    let rf;

    // PORT-NOTE: Raven derefs `cg.snap` unguarded; `cg_t.snap` is still the C1
    // a missing snapshot reads as "not us"
    let ourClientNum = ctx.world.cg.snap_ref().map_or(-1, |snap| snap.ps.clientNum);

    if cent.currentState.number == ourClientNum && ctx.world.cg.renderingThirdPerson == 0 {
        rf = RF_THIRD_PERSON; // only show in mirrors
    } else {
        rf = 0;
    }

    let mut ent = refEntity_t::zeroed();
    _VectorCopy(cent.lerpOrigin, &mut ent.origin);
    ent.origin[2] += 48.0;
    ent.reType = refEntityType_t::RT_SPRITE;
    ent.customShader = shader;
    ent.radius = 10.0;
    ent.renderfx = rf;
    ent.shaderRGBA[0] = 255;
    ent.shaderRGBA[1] = 255;
    ent.shaderRGBA[2] = 255;
    ent.shaderRGBA[3] = 255;
    trap::R_AddRefEntityToScene(ctx.engine, &ent);
}

/// Raven `CG_PlayerSplash` — the water-wake quad under a player whose feet are
/// in liquid and whose head is not.
/// Source: `oracle/codemp/cgame/cg_players.c:4717-4795`
pub fn CG_PlayerSplash(ctx: &mut CgContext, cent: &centity_t) {
    let engine = ctx.engine;
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut trace = trace_t::zeroed();
    let mut verts = [polyVert_t {
        xyz: [0.0; 3],
        st: [0.0; 2],
        modulate: [0; 4],
    }; 4];

    if ctx.world.cvars.cg_shadows.integer == 0 {
        return;
    }

    _VectorCopy(cent.lerpOrigin, &mut end);
    end[2] -= 24.0;

    // if the feet aren't in liquid, don't make a mark
    // this won't handle moving water brushes, but they wouldn't draw right anyway...
    let contents = trap::CM_PointContents(engine, &end, 0);
    if contents & (CONTENTS_WATER | CONTENTS_SLIME | CONTENTS_LAVA) == 0 {
        return;
    }

    _VectorCopy(cent.lerpOrigin, &mut start);
    start[2] += 32.0;

    // if the head isn't out of liquid, don't make a mark
    let contents = trap::CM_PointContents(engine, &start, 0);
    if contents & (CONTENTS_SOLID | CONTENTS_WATER | CONTENTS_SLIME | CONTENTS_LAVA) != 0 {
        return;
    }

    // trace down to find the surface
    trap::CM_BoxTrace(
        engine,
        &mut trace,
        &start,
        &end,
        None,
        None,
        0,
        CONTENTS_WATER | CONTENTS_SLIME | CONTENTS_LAVA,
    );

    if trace.fraction == 1.0 {
        return;
    }

    // create a mark polygon
    _VectorCopy(trace.endpos, &mut verts[0].xyz);
    verts[0].xyz[0] -= 32.0;
    verts[0].xyz[1] -= 32.0;
    verts[0].st[0] = 0.0;
    verts[0].st[1] = 0.0;
    verts[0].modulate[0] = 255;
    verts[0].modulate[1] = 255;
    verts[0].modulate[2] = 255;
    verts[0].modulate[3] = 255;

    _VectorCopy(trace.endpos, &mut verts[1].xyz);
    verts[1].xyz[0] -= 32.0;
    verts[1].xyz[1] += 32.0;
    verts[1].st[0] = 0.0;
    verts[1].st[1] = 1.0;
    verts[1].modulate[0] = 255;
    verts[1].modulate[1] = 255;
    verts[1].modulate[2] = 255;
    verts[1].modulate[3] = 255;

    _VectorCopy(trace.endpos, &mut verts[2].xyz);
    verts[2].xyz[0] += 32.0;
    verts[2].xyz[1] += 32.0;
    verts[2].st[0] = 1.0;
    verts[2].st[1] = 1.0;
    verts[2].modulate[0] = 255;
    verts[2].modulate[1] = 255;
    verts[2].modulate[2] = 255;
    verts[2].modulate[3] = 255;

    _VectorCopy(trace.endpos, &mut verts[3].xyz);
    verts[3].xyz[0] += 32.0;
    verts[3].xyz[1] -= 32.0;
    verts[3].st[0] = 1.0;
    verts[3].st[1] = 0.0;
    verts[3].modulate[0] = 255;
    verts[3].modulate[1] = 255;
    verts[3].modulate[2] = 255;
    verts[3].modulate[3] = 255;

    trap::R_AddPolyToScene(engine, ctx.world.cgs.media.wakeMarkShader, &verts);
}

/// Raven `CG_PlayerShieldHit` — arms the personal-shield flash on an entity and
/// points it back down the incoming damage direction. Bigger hits hold longer,
/// capped at two seconds.
/// Source: `oracle/codemp/cgame/cg_players.c:5077-5104`
pub fn CG_PlayerShieldHit(world: &mut CgWorld, entitynum: c_int, dir: &mut vec3_t, amount: c_int) {
    // PORT-NOTE: Raven guards with `tr_types.h`'s MAX_ENTITIES (2048), twice
    // the 1024-entry `cg_entities` array, so 1024..2047 is an out-of-bounds
    // read (§19). The port guards at the array bound instead.
    if entitynum < 0 || entitynum >= MAX_GENTITIES as c_int {
        return;
    }

    let time = if amount > 100 {
        (world.cg.time as f64 + MAX_SHIELD_TIME) as c_int // 2 sec.
    } else {
        world.cg.time + 500 + amount * 15
    };

    let cent = world.entity_mut(entitynum as usize);

    if time > cent.damageTime {
        cent.damageTime = time;
        let incoming = *dir;
        _VectorScale(incoming, -1.0, dir);
        vectoangles(*dir, &mut cent.damageAngles);
    }
}

/// Raven `CG_DrawPlayerShield` — the half-shield shell over the last hit
/// direction, fading out and swelling as it goes. Never drawn on a corpse.
/// Source: `oracle/codemp/cgame/cg_players.c:5107-5142`
pub fn CG_DrawPlayerShield(ctx: &mut CgContext, cent: &centity_t, origin: &vec3_t) {
    // Don't draw the shield when the player is dead.
    if cent.currentState.eFlags & EF_DEAD != 0 {
        return;
    }

    let mut ent = refEntity_t::zeroed();

    _VectorCopy(*origin, &mut ent.origin);
    ent.origin[2] += 10.0;
    AnglesToAxis(cent.damageAngles, ent.axis.as_mut_ptr());

    let jitter = ctx.world.bg_state.rng.random() * 16.0;
    let mut alpha = (255.0f64 * ((cent.damageTime - ctx.world.cg.time) as f64 / MIN_SHIELD_TIME)
        + jitter as f64) as c_int;
    if alpha > 255 {
        alpha = 255;
    }

    // Make it bigger, but tighter if more solid
    let scale = (1.4f64 - (alpha as f32 as f64 * (0.4f64 / 255.0))) as f32; // Range from 1.0 to 1.4
    let axis0 = ent.axis[0];
    _VectorScale(axis0, scale, &mut ent.axis[0]);
    let axis1 = ent.axis[1];
    _VectorScale(axis1, scale, &mut ent.axis[1]);
    let axis2 = ent.axis[2];
    _VectorScale(axis2, scale, &mut ent.axis[2]);

    ent.hModel = ctx.world.cgs.media.halfShieldModel;
    ent.customShader = ctx.world.cgs.media.halfShieldShader;
    ent.shaderRGBA[0] = alpha as u8;
    ent.shaderRGBA[1] = alpha as u8;
    ent.shaderRGBA[2] = alpha as u8;
    ent.shaderRGBA[3] = 255;
    trap::R_AddRefEntityToScene(ctx.engine, &ent);
}

/// Raven `CG_LightVerts` — lights a poly's vertices from the world lightgrid;
/// a face pointing away from the light gets the ambient term only.
///
/// Raven's `numVerts` is dropped in favour of the slice length — callers hand
/// over exactly the vertices they mean to light.
/// Source: `oracle/codemp/cgame/cg_players.c:5167-5207`
pub fn CG_LightVerts(ctx: &mut CgContext, normal: &vec3_t, verts: &mut [polyVert_t]) -> bool {
    let mut ambientLight: vec3_t = [0.0; 3];
    let mut lightDir: vec3_t = [0.0; 3];
    let mut directedLight: vec3_t = [0.0; 3];

    let point = verts[0].xyz;
    trap::R_LightForPoint(
        ctx.engine,
        &point,
        &mut ambientLight,
        &mut directedLight,
        &mut lightDir,
    );

    for vert in verts.iter_mut() {
        let incoming = _DotProduct(*normal, lightDir);
        if incoming <= 0.0 {
            vert.modulate[0] = ambientLight[0] as u8;
            vert.modulate[1] = ambientLight[1] as u8;
            vert.modulate[2] = ambientLight[2] as u8;
            vert.modulate[3] = 255;
            continue;
        }

        let mut j = (ambientLight[0] + incoming * directedLight[0]) as c_int;
        if j > 255 {
            j = 255;
        }
        vert.modulate[0] = j as u8;

        let mut j = (ambientLight[1] + incoming * directedLight[1]) as c_int;
        if j > 255 {
            j = 255;
        }
        vert.modulate[1] = j as u8;

        let mut j = (ambientLight[2] + incoming * directedLight[2]) as c_int;
        if j > 255 {
            j = 255;
        }
        vert.modulate[2] = j as u8;

        vert.modulate[3] = 255;
    }

    true
}

/// Raven `CG_RGBForSaberColor` — the blade tint for one of the six saber
/// colours. Anything outside the six leaves `rgb` exactly as the caller had it.
/// Source: `oracle/codemp/cgame/cg_players.c:5209-5232`
pub fn CG_RGBForSaberColor(color: saber_colors_t, rgb: &mut vec3_t) {
    match color {
        SABER_RED => VectorSet(rgb, 1.0, 0.2, 0.2),
        SABER_ORANGE => VectorSet(rgb, 1.0, 0.5, 0.1),
        SABER_YELLOW => VectorSet(rgb, 1.0, 1.0, 0.2),
        SABER_GREEN => VectorSet(rgb, 0.2, 1.0, 0.2),
        SABER_BLUE => VectorSet(rgb, 0.2, 0.4, 1.0),
        SABER_PURPLE => VectorSet(rgb, 0.9, 0.2, 1.0),
        _ => {}
    }
}

/// Raven `CG_GetTagWorldPosition` — where a model's tag ends up once the
/// model's own origin and axis are applied.
///
/// Raven: Can pass in NULL for the axis.
/// Source: `oracle/codemp/cgame/cg_players.c:5437-5456`
pub fn CG_GetTagWorldPosition(
    ctx: &mut CgContext,
    model: &refEntity_t,
    tag: &str,
    pos: &mut vec3_t,
    axis: Option<&mut [vec3_t; 3]>,
) {
    let mut orientation = orientation_t {
        origin: [0.0; 3],
        axis: [[0.0; 3]; 3],
    };

    // Get the requested tag
    trap::R_LerpTag(
        ctx.engine,
        &mut orientation,
        model.hModel,
        model.oldframe,
        model.frame,
        1.0 - model.backlerp,
        tag,
    );

    _VectorCopy(model.origin, pos);
    for i in 0..3 {
        let running = *pos;
        _VectorMA(running, orientation.origin[i], model.axis[i], pos);
    }

    if let Some(axis) = axis {
        MatrixMultiply(&orientation.axis, &model.axis, axis);
    }
}

/// Raven `CG_G2TraceCollide` — re-runs a landed trace against the hit entity's
/// ghoul2 skeleton. A miss rewrites the trace as "hit nothing"; a hit moves the
/// endpoint and normal onto the collided polygon.
/// Source: `oracle/codemp/cgame/cg_players.c:5632-5689`
pub fn CG_G2TraceCollide(
    ctx: &mut CgContext,
    tr: &mut trace_t,
    mins: Option<&vec3_t>,
    maxs: Option<&vec3_t>,
    lastValidStart: &vec3_t,
    lastValidEnd: &vec3_t,
) -> bool {
    let mut angles: vec3_t = [0.0; 3];
    let mut fRadius: f32 = 0.0;

    if let (Some(mins), Some(maxs)) = (mins, maxs) {
        if mins[0] != 0.0 || maxs[0] != 0.0 {
            fRadius = (maxs[0] - mins[0]) / 2.0;
        }
    }

    // Raven's `memset(&G2Trace, 0, sizeof(G2Trace))` then the -1 entity-number
    // sweep, in one go.
    let mut G2Trace: G2Trace_t = [CollisionRecord_t {
        mDistance: 0.0,
        mEntityNum: -1,
        mModelIndex: 0,
        mPolyIndex: 0,
        mSurfaceIndex: 0,
        mCollisionPosition: [0.0; 3],
        mCollisionNormal: [0.0; 3],
        mFlags: 0,
        mMaterial: 0,
        mLocation: 0,
        mBarycentricI: 0.0,
        mBarycentricJ: 0.0,
    }; MAX_G2_COLLISIONS];

    let engine = ctx.engine;
    let time = ctx.world.cg.time;
    let g2TraceLod = ctx.world.cvars.cg_g2TraceLod.integer;
    let optvehtrace = ctx.world.cvars.cg_optvehtrace.integer;

    let g2Hit = ctx.world.entity(tr.entityNum as usize);
    let ghoul2 = g2Hit.ghoul2;
    let number = g2Hit.currentState.number;
    let eType = g2Hit.currentState.eType;
    let NPC_class = g2Hit.currentState.NPC_class;
    let hasVehicle = g2Hit.m_pVehicle.is_some();
    let lerpOrigin = g2Hit.lerpOrigin;
    let modelScale = g2Hit.modelScale;
    let yaw = g2Hit.lerpAngles[YAW];

    if ghoul2.is_null() {
        return false;
    }

    angles[ROLL] = 0.0;
    angles[PITCH] = 0.0;
    angles[YAW] = yaw;

    if optvehtrace != 0
        && eType == entityType_t::ET_NPC as c_int
        && NPC_class == class_t::CLASS_VEHICLE as c_int
        && hasVehicle
    {
        trap::G2API_CollisionDetectCache(
            engine,
            &mut G2Trace,
            ghoul2,
            &angles,
            &lerpOrigin,
            time,
            number,
            lastValidStart,
            lastValidEnd,
            &modelScale,
            0,
            g2TraceLod,
            fRadius,
        );
    } else {
        trap::G2API_CollisionDetect(
            engine,
            &mut G2Trace,
            ghoul2,
            &angles,
            &lerpOrigin,
            time,
            number,
            lastValidStart,
            lastValidEnd,
            &modelScale,
            0,
            g2TraceLod,
            fRadius,
        );
    }

    if G2Trace[0].mEntityNum != number {
        tr.fraction = 1.0;
        tr.entityNum = ENTITYNUM_NONE as c_short;
        tr.startsolid = 0;
        tr.allsolid = 0;
        false
    } else {
        // Yay!
        _VectorCopy(G2Trace[0].mCollisionPosition, &mut tr.endpos);
        _VectorCopy(G2Trace[0].mCollisionNormal, &mut tr.plane.normal);
        true
    }
}

/// Raven `CG_AddGhoul2Mark` — puts a gore decal on a ghoul2 skin along the
/// start→end ray. Bails when the instance is already carrying `cg_ghoul2Marks`
/// decals, or when the ray is too short to have a direction.
/// Source: `oracle/codemp/cgame/cg_players.c:5738-5796`
#[allow(clippy::too_many_arguments)]
pub fn CG_AddGhoul2Mark(
    ctx: &mut CgContext,
    shader: c_int,
    size: f32,
    start: &vec3_t,
    end: &vec3_t,
    entnum: c_int,
    entposition: &vec3_t,
    entangle: f32,
    ghoul2: *mut c_void,
    scale: &mut vec3_t,
    lifeTime: c_int,
) {
    debug_assert!(!ghoul2.is_null());

    // Raven memsets the block; every field the fill below skips stays zero, and
    // the C6b referee compares all 144 bytes.
    // SAFETY: SSkinGoreData is #[repr(C)] plain data; all-zero bytes are valid.
    let mut goreSkin: SSkinGoreData = unsafe { core::mem::zeroed() };

    if trap::G2API_GetNumGoreMarks(ctx.engine, ghoul2, 0) >= ctx.world.cvars.cg_ghoul2Marks.integer
    {
        // you've got too many marks already
        return;
    }

    goreSkin.growDuration = -1; // default expandy time
    goreSkin.goreScaleStartFraction = 1.0; // default start scale
    goreSkin.frontFaces = qtrue;
    goreSkin.backFaces = qtrue;
    goreSkin.lifeTime = lifeTime; //last randomly 10-20 seconds
                                  //rwwFIXMEFIXME: fade has sorting issues with other non-fading decals, disabled until fixed

    goreSkin.baseModelOnly = qfalse;

    goreSkin.currentTime = ctx.world.cg.time;
    goreSkin.entNum = entnum;
    goreSkin.SSize = size;
    goreSkin.TSize = size;
    goreSkin.theta = ctx.world.bg_state.rng.flrand(0.0, 6.28);
    goreSkin.shader = shader;

    // PORT-NOTE: Raven's else arm is `VectorCopy(goreSkin.scale, scale)` — the
    // arguments are the wrong way round, so it copies the memset-zeroed
    // `goreSkin.scale` OVER the caller's vector and leaves the gore scale at
    // zero. Kept: the caller really does get stomped.
    if scale[0] == 0.0 && scale[1] == 0.0 && scale[2] == 0.0 {
        VectorSet(&mut goreSkin.scale, 1.0, 1.0, 1.0);
    } else {
        _VectorCopy(goreSkin.scale, scale);
    }

    _VectorCopy(*start, &mut goreSkin.hitLocation);

    _VectorSubtract(*end, *start, &mut goreSkin.rayDirection);
    if VectorNormalize(&mut goreSkin.rayDirection) < 0.1 {
        return;
    }

    _VectorCopy(*entposition, &mut goreSkin.position);
    goreSkin.angles[YAW] = entangle;

    trap::G2API_AddSkinGore(ctx.engine, ghoul2, &mut goreSkin);
}

/// Raven `CG_IsMindTricked` — is this client hidden from us by mind trick? The
/// four bitmask args are the four 16-client blocks of `trickedentindex`; force
/// sight beats the trick outright.
/// Source: `oracle/codemp/cgame/cg_players.c:6482-6518`
pub fn CG_IsMindTricked(
    world: &CgWorld,
    trickIndex1: c_int,
    trickIndex2: c_int,
    trickIndex3: c_int,
    trickIndex4: c_int,
    client: c_int,
) -> bool {
    let checkIn;
    let mut sub = 0;

    if world.entity(client as usize).currentState.forcePowersActive & (1 << FP_SEE) != 0 {
        return false;
    }

    if client > 47 {
        checkIn = trickIndex4;
        sub = 48;
    } else if client > 31 {
        checkIn = trickIndex3;
        sub = 32;
    } else if client > 15 {
        checkIn = trickIndex2;
        sub = 16;
    } else {
        checkIn = trickIndex1;
    }

    checkIn & (1 << (client - sub)) != 0
}

/// Raven `CG_DrawPlayerSphere` — the force-effect shell (invulnerability,
/// ysalamiri, enlightenment…) drawn around a player. Never on a corpse, and
/// never on your own body in first person past the first pass.
/// Source: `oracle/codemp/cgame/cg_players.c:6522-6626`
pub fn CG_DrawPlayerSphere(
    ctx: &mut CgContext,
    cent: &centity_t,
    origin: &vec3_t,
    scale: f32,
    shader: c_int,
) {
    // Don't draw the shield when the player is dead.
    if cent.currentState.eFlags & EF_DEAD != 0 {
        return;
    }

    let mut ent = refEntity_t::zeroed();
    let mut ang: vec3_t = [0.0; 3];
    let mut viewDir: vec3_t = [0.0; 3];

    _VectorCopy(*origin, &mut ent.origin);
    ent.origin[2] += 9.0;

    let vieworg = ctx.world.cg.refdef.vieworg;
    _VectorSubtract(ent.origin, vieworg, &mut ent.axis[0]);
    let vLen = VectorLength(ent.axis[0]);
    if vLen <= 0.1 {
        // Entity is right on vieworg.  quit.
        return;
    }

    _VectorCopy(ent.axis[0], &mut viewDir);
    VectorInverse(&mut viewDir);
    VectorNormalize(&mut viewDir);

    vectoangles(ent.axis[0], &mut ang);
    ang[ROLL] += 180.0;
    ang[PITCH] += 180.0;
    AnglesToAxis(ang, ent.axis.as_mut_ptr());

    let axis0 = ent.axis[0];
    _VectorScale(axis0, scale, &mut ent.axis[0]);
    let axis1 = ent.axis[1];
    _VectorScale(axis1, scale, &mut ent.axis[1]);
    let axis2 = ent.axis[2];
    _VectorScale(axis2, scale, &mut ent.axis[2]);

    ent.nonNormalizedAxes = qtrue;

    ent.hModel = ctx.world.cgs.media.halfShieldModel;
    ent.customShader = shader;

    trap::R_AddRefEntityToScene(ctx.engine, &ent);

    if ctx.world.cg.renderingThirdPerson == 0
        && cent.currentState.number == ctx.world.cg.predictedPlayerState.clientNum
    {
        // don't do the rest then
        return;
    }
    if ctx.world.cvars.cg_renderToTextureFX.integer == 0 {
        return;
    }

    ang[PITCH] -= 180.0;
    AnglesToAxis(ang, ent.axis.as_mut_ptr());

    let axis0 = ent.axis[0];
    _VectorScale(axis0, scale * 0.5, &mut ent.axis[0]);
    let axis1 = ent.axis[1];
    _VectorScale(axis1, scale * 0.5, &mut ent.axis[1]);
    let axis2 = ent.axis[2];
    _VectorScale(axis2, scale * 0.5, &mut ent.axis[2]);

    ent.renderfx = RF_DISTORTION | RF_FORCE_ENT_ALPHA;
    if shader == ctx.world.cgs.media.invulnerabilityShader {
        // ok, ok, this is a little hacky. sorry!
        ent.shaderRGBA[0] = 0;
        ent.shaderRGBA[1] = 255;
        ent.shaderRGBA[2] = 0;
        ent.shaderRGBA[3] = 100;
    } else if shader == ctx.world.cgs.media.ysalimariShader {
        ent.shaderRGBA[0] = 255;
        ent.shaderRGBA[1] = 255;
        ent.shaderRGBA[2] = 0;
        ent.shaderRGBA[3] = 100;
    } else if shader == ctx.world.cgs.media.endarkenmentShader {
        ent.shaderRGBA[0] = 100;
        ent.shaderRGBA[1] = 0;
        ent.shaderRGBA[2] = 0;
        ent.shaderRGBA[3] = 20;
    } else if shader == ctx.world.cgs.media.enlightenmentShader {
        ent.shaderRGBA[0] = 255;
        ent.shaderRGBA[1] = 255;
        ent.shaderRGBA[2] = 255;
        ent.shaderRGBA[3] = 20;
    } else {
        // ysal red/blue, boon
        ent.shaderRGBA[0] = 255;
        ent.shaderRGBA[1] = 255;
        ent.shaderRGBA[2] = 255;
        ent.shaderRGBA[3] = 20;
    }

    ent.radius = 256.0;

    let entOrigin = ent.origin;
    _VectorMA(entOrigin, 40.0, viewDir, &mut ent.origin);

    ent.customShader = trap::R_RegisterShader(ctx.engine, "effects/refract_2");
    trap::R_AddRefEntityToScene(ctx.engine, &ent);
}

/// Raven `CG_AddLightningBeam` — the wobbling force-lightning bezier between
/// two points; the "chaos" that bends the control points is three sine waves off
/// `cg.time`, so it is barely chaotic at all.
/// Source: `oracle/codemp/cgame/cg_players.c:6628-6697`
pub fn CG_AddLightningBeam(ctx: &mut CgContext, start: &vec3_t, end: &vec3_t) {
    let mut dir: vec3_t = [0.0; 3];
    let mut chaos: vec3_t = [0.0; 3];
    let mut c1: vec3_t = [0.0; 3];
    let mut c2: vec3_t = [0.0; 3];
    let mut v1: vec3_t = [0.0; 3];
    let mut v2: vec3_t = [0.0; 3];

    _VectorSubtract(*end, *start, &mut dir);
    let len = VectorNormalize(&mut dir);

    // Get the base control points, we'll work from there
    _VectorMA(*start, 0.3333 * len, dir, &mut c1);
    _VectorMA(*start, 0.6666 * len, dir, &mut c2);

    // get some chaos values that really aren't very chaotic :)
    let time = ctx.world.cg.time;
    let s1 = (((time as f32 * 0.005) as f64).sin() * 2.0
        + ctx.world.bg_state.rng.crandom() * 0.2f32 as f64) as f32;
    let s2 = ((time as f32 * 0.001) as f64).sin() as f32;
    let s3 = ((time as f32 * 0.011) as f64).sin() as f32;

    VectorSet(
        &mut chaos,
        len * 0.01 * s1,
        len * 0.02 * s2,
        len * 0.04 * (s1 + s2 + s3),
    );

    let c1in = c1;
    _VectorAdd(c1in, chaos, &mut c1);
    _VectorScale(chaos, 4.0, &mut v1);

    VectorSet(
        &mut chaos,
        -len * 0.02 * s3,
        len * 0.01 * (s1 * s2),
        -len * 0.02 * (s1 + s2 * s3),
    );

    let c2in = c2;
    _VectorAdd(c2in, chaos, &mut c2);
    _VectorScale(chaos, 2.0, &mut v2);

    VectorSet(&mut chaos, 1.0, 1.0, 1.0);

    // PORT-NOTE: `v1`, `v2` and that last `chaos` set are never read again in
    // Raven — dead stores, kept so the arithmetic reads like the oracle.
    let _ = (v1, v2, chaos);

    let sRGB: vec3_t = [255.0, 255.0, 255.0];
    let shader = trap::R_RegisterShader(ctx.engine, "gfx/misc/electric2");

    let mut b = addbezierArgStruct_t {
        start: *start,
        end: *end,
        control1: c1,
        control1Vel: vec3_origin,
        control2: c2,
        control2Vel: vec3_origin,
        size1: 6.0,
        size2: 6.0,
        sizeParm: 0.0,
        alpha1: 0.0,
        alpha2: 0.2,
        alphaParm: 0.5,
        sRGB,
        eRGB: sRGB,
        rgbParm: 0.0,
        killTime: 50,
        shader,
        flags: 0x00000001, // FX_ALPHA_LINEAR
    };

    trap::FX_AddBezier(ctx.engine, &mut b);
}

/// Raven `CG_ThereIsAMaster` — is anyone carrying the Jedi Master mantle right
/// now?
/// Source: `oracle/codemp/cgame/cg_players.c:6744-6762`
pub fn CG_ThereIsAMaster(world: &CgWorld) -> bool {
    for i in 0..MAX_CLIENTS {
        if world.entity(i).currentState.isJediMaster != 0 {
            return true;
        }
    }

    false
}

/// Raven `CG_HandleAppendedSkin` — splits a `model*skin` string in place and
/// registers the skin out of the model's own folder. `modelName` comes back cut
/// at the `*`; a `|` in the skin name means a three-part skin.
/// Source: `oracle/codemp/cgame/cg_players.c:6817-6868`
pub fn CG_HandleAppendedSkin(ctx: &mut CgContext, modelName: &mut String) -> qhandle_t {
    let mut skinID: qhandle_t = 0;

    // see if it has a skin name
    let p = modelName.rfind('*');

    if let Some(p) = p {
        // found a *, we should have a model name before it and a skin name after it.
        let skinName = modelName[p + 1..].to_string();
        modelName.truncate(p);

        if !skinName.is_empty() {
            // got it, register the skin under the model path.
            let mut baseFolder = modelName.clone();

            // go back to the first /, should be the path point
            if let Some(slash) = baseFolder.rfind('/') {
                // got it.. terminate at the slash and register.
                baseFolder.truncate(slash);

                let useSkinName = if skinName.contains('|') {
                    // three part skin
                    format!("{}/|{}", baseFolder, skinName)
                } else {
                    format!("{}/model_{}.skin", baseFolder, skinName)
                };

                skinID = trap::R_RegisterSkin(ctx.engine, &useSkinName);
            }
        }
    }

    skinID
}

/// Raven `CG_CacheG2AnimInfo` — precaches a model's skeleton and animation set
/// by building a throwaway ghoul2 instance, parsing the `animation.cfg` beside
/// its `.gla`, and cleaning the instance up again.
/// Source: `oracle/codemp/cgame/cg_players.c:6876-6939`
pub fn CG_CacheG2AnimInfo(ctx: &mut CgContext, modelName: &str) {
    let mut useModel = modelName.to_string();

    if modelName.starts_with('$') {
        // it's a vehicle name actually, let's precache the whole vehicle
        let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
        let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());

        // both bg fns stomp the `$name` they were handed with their answer, so
        // each gets its own copy of it the way Raven's two stack buffers do
        let mut modelBuf: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        let mut skinBuf: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        Q_strncpyz(&mut modelBuf, modelName, MAX_QPATH);
        Q_strncpyz(&mut skinBuf, modelName, MAX_QPATH);

        BG_GetVehicleModelName(
            modelBuf.as_mut_ptr(),
            &mut ctx.world.bg_state,
            &traps,
            &mut callbacks,
        );
        BG_GetVehicleSkinName(
            skinBuf.as_mut_ptr(),
            &mut ctx.world.bg_state,
            &traps,
            &mut callbacks,
        );

        let vehModel = buf_to_string(&modelBuf.map(|c| c as u8));
        let useSkin = buf_to_string(&skinBuf.map(|c| c as u8));

        if !useSkin.is_empty() {
            // use a custom skin
            trap::R_RegisterSkin(
                ctx.engine,
                &format!("models/players/{vehModel}/model_{useSkin}.skin"),
            );
        } else {
            trap::R_RegisterSkin(
                ctx.engine,
                &format!("models/players/{vehModel}/model_default.skin"),
            );
        }
        useModel = format!("models/players/{vehModel}/model.glm");
    }

    let mut g2: *mut c_void = null_mut();
    trap::G2API_InitGhoul2Model(
        ctx.engine,
        &mut g2 as *mut *mut c_void,
        &useModel,
        0,
        0,
        0,
        0,
        0,
    );

    if !g2.is_null() {
        let mut animIndex: c_int = -1;
        let mut GLAName = trap::G2API_GetGLAName(ctx.engine, g2, 0, MAX_QPATH);

        if let Some(slash) = GLAName.rfind('/') {
            GLAName.truncate(slash);
            GLAName.push_str("/animation.cfg");

            let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
            let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
            let filename = cstr(&GLAName);

            animIndex = BG_ParseAnimationFile(
                &mut ctx.world.bg_state,
                &traps,
                &mut callbacks,
                filename.as_ptr(),
                null_mut(),
                qfalse,
            );
        }

        if animIndex != -1 {
            // Raven cuts `originalModelName` (its pre-GLA copy of `useModel`)
            // back to its directory with the trailing slash - `useModel` is
            // that untouched copy here
            let mut originalModelName = useModel.clone();
            if let Some(slash) = originalModelName.rfind('/') {
                originalModelName.truncate(slash + 1);
            }

            let bgNumAnimEvents = ctx.world.bg_state.bgNumAnimEvents;
            let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
            let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
            BG_ParseAnimationEvtFile(
                &mut ctx.world.bg_state,
                &traps,
                &mut callbacks,
                &originalModelName,
                animIndex,
                bgNumAnimEvents,
            );
        }

        // Now free the temp instance
        trap::G2API_CleanGhoul2Models(ctx.engine, &mut g2 as *mut *mut c_void);
    }
}

/// Raven `CG_RegisterVehicleAssets` — precaches a vehicle's effect set.
///
/// The whole body is commented out in the oracle (every `trap_FX_RegisterEffect`
/// line), so this really is a no-op; it is kept because `CG_Player`'s vehicle
/// path still calls it (`cg_players.c:7051`).
/// Source: `oracle/codemp/cgame/cg_players.c:6941-6986`
pub fn CG_RegisterVehicleAssets(_pVeh: VehicleId) {}

/// Raven `CG_CreateSurfaceDebris` — blows the named ghoul2 surface off a ship,
/// playing the debris effect at the surface's bolt (or at the ship's origin when
/// the surface has no bolt).
///
/// The five wing/nose arms read `m_pVehicle->m_pVehicleInfo->i{R,L}WingFX`/
/// `iNoseFX` through the pool for the thrown-part effect.
/// Source: `oracle/codemp/cgame/cg_players.c:7266-7361`
pub fn CG_CreateSurfaceDebris(
    ctx: &mut CgContext,
    cent: &centity_t,
    surfNum: c_int,
    fxID: c_int,
    throwPart: bool,
) {
    let mut lostPartFX: c_int = 0;
    let mut b: c_int = -1;
    let mut v: vec3_t = [0.0; 3];
    let mut d: vec3_t = [0.0; 3];
    let mut boltMatrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };
    let mut surfName: Option<&str> = None;

    if surfNum > 0 {
        surfName = bgToggleableSurfaces
            .get(surfNum as usize)
            .and_then(|s| *s)
            .map(|s| s.to_str().unwrap_or(""));
    }

    if cent.ghoul2.is_null() {
        // oh no
        return;
    }

    let engine = ctx.engine;

    // PORT-NOTE: Raven indexes `bgToggleableSurfaceDebris[surfNum]` before it
    // tests `surfNum == -1` further down, so a -1 surface reads one slot before
    // the table (§19 UB). The port reads through a bounds check: an
    // out-of-range surface matches no arm and falls into the origin path, which
    // is exactly where Raven's own `surfNum == -1` test sends it.
    let debris = bgToggleableSurfaceDebris
        .get(surfNum as usize)
        .copied()
        .unwrap_or(-1);

    // the ship's `m_pVehicleInfo` (if any) supplies the thrown-part effect.
    let vehInfoPtr = cent
        .m_pVehicle
        .map(|id| ctx.world.vehicle_pool[id.ent_num() as usize].m_pVehicleInfo);

    // let's add the surface as a bolt so we can get the base point of it
    if debris == 3 {
        // right wing flame
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, "*r_wingdamage");
        if throwPart {
            if let Some(ptr) = vehInfoPtr {
                if !ptr.is_null() {
                    lostPartFX = unsafe { (*ptr).iRWingFX };
                }
            }
        }
    } else if debris == 4 {
        // left wing flame
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, "*l_wingdamage");
        if throwPart {
            if let Some(ptr) = vehInfoPtr {
                if !ptr.is_null() {
                    lostPartFX = unsafe { (*ptr).iLWingFX };
                }
            }
        }
    } else if debris == 5 {
        // right wing flame 2
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, "*r_wingdamage");
        if throwPart {
            if let Some(ptr) = vehInfoPtr {
                if !ptr.is_null() {
                    lostPartFX = unsafe { (*ptr).iRWingFX };
                }
            }
        }
    } else if debris == 6 {
        // left wing flame 2
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, "*l_wingdamage");
        if throwPart {
            if let Some(ptr) = vehInfoPtr {
                if !ptr.is_null() {
                    lostPartFX = unsafe { (*ptr).iLWingFX };
                }
            }
        }
    } else if debris == 7 {
        // nose flame
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, "*nosedamage");
        if let Some(ptr) = vehInfoPtr {
            if !ptr.is_null() {
                lostPartFX = unsafe { (*ptr).iNoseFX };
            }
        }
    } else if let Some(surfName) = surfName {
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, surfName);
    }

    if b == -1 || surfNum == -1 {
        // couldn't find this surface apparently, so play on origin?
        _VectorCopy(cent.lerpOrigin, &mut v);
        AngleVectors(cent.lerpAngles, Some(&mut d), None, None);
        VectorNormalize(&mut d);
    } else {
        // now let's get the position and direction of this surface and make a big explosion
        let time = ctx.world.cg.time;
        trap::G2API_GetBoltMatrix(
            engine,
            cent.ghoul2,
            0,
            b,
            &mut boltMatrix,
            &cent.lerpAngles,
            &cent.lerpOrigin,
            time,
            Some(&mut ctx.world.cgs.gameModels[0]),
            &cent.modelScale,
        );
        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut v);
        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::POSITIVE_Z as c_int, &mut d);
    }

    trap::FX_PlayEffectID(engine, fxID, &v, &d, -1, -1);
    if throwPart && lostPartFX != 0 {
        // throw off a ship part, too
        let mut fxFwd: vec3_t = [0.0; 3];
        AngleVectors(cent.lerpAngles, Some(&mut fxFwd), None, None);
        trap::FX_PlayEffectID(engine, lostPartFX, &v, &fxFwd, -1, -1);
    }
}

/// Raven `CG_CreateSurfaceSmoke` — the flame/smoke trail off one of a ship's
/// four damage surfaces. An unknown surface, or one with no bolt on the model,
/// draws nothing.
/// Source: `oracle/codemp/cgame/cg_players.c:7365-7411`
pub fn CG_CreateSurfaceSmoke(ctx: &mut CgContext, cent: &centity_t, shipSurf: c_int, fxID: c_int) {
    let mut v: vec3_t = [0.0; 3];
    let mut d: vec3_t = [0.0; 3];
    let mut boltMatrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };

    if cent.ghoul2.is_null() {
        // oh no
        return;
    }

    // let's add the surface as a bolt so we can get the base point of it
    let surfName = if shipSurf == SHIPSURF_FRONT {
        // front flame/smoke
        "*nosedamage"
    } else if shipSurf == SHIPSURF_BACK {
        // back flame/smoke
        //
        // Raven: FIXME: random?  Some point in-between?
        "*exhaust1"
    } else if shipSurf == SHIPSURF_RIGHT {
        // right wing flame/smoke
        "*r_wingdamage"
    } else if shipSurf == SHIPSURF_LEFT {
        // left wing flame/smoke
        "*l_wingdamage"
    } else {
        // unknown surf!
        return;
    };

    let engine = ctx.engine;
    let b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, surfName);
    if b == -1 {
        // couldn't find this surface apparently
        return;
    }

    // now let's get the position and direction of this surface and make a big explosion
    let time = ctx.world.cg.time;
    trap::G2API_GetBoltMatrix(
        engine,
        cent.ghoul2,
        0,
        b,
        &mut boltMatrix,
        &cent.lerpAngles,
        &cent.lerpOrigin,
        time,
        Some(&mut ctx.world.cgs.gameModels[0]),
        &cent.modelScale,
    );
    BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut v);
    BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::POSITIVE_Z as c_int, &mut d);

    trap::FX_PlayEffectID(engine, fxID, &v, &d, -1, -1);
}

/// Raven `CG_VehicleShouldDrawShields` — are this ship's shields currently
/// taking damage, so the shell should be drawn?
///
/// Source: `oracle/codemp/cgame/cg_players.c:7415-7425`
pub fn CG_VehicleShouldDrawShields(world: &CgWorld, vehCent: &centity_t) -> bool {
    // ship shields currently taking damage
    vehCent.damageTime > world.cg.time
        && vehCent.currentState.NPC_class == class_t::CLASS_VEHICLE as c_int
        && vehCent.m_pVehicle.is_some_and(|id| {
            !world.vehicle_pool[id.ent_num() as usize]
                .m_pVehicleInfo
                .is_null()
        })
}

/// Raven `CG_VehicleAttachDroidUnit` — snaps an astromech onto its ship's droid
/// tag, taking the droid's whole origin and orientation off the bolt.
///
/// Source: `oracle/codemp/cgame/cg_players.c:7433-7461`
#[allow(unused_variables)]
pub fn CG_VehicleAttachDroidUnit(
    ctx: &mut CgContext,
    droidCent: &mut centity_t,
    legs: &refEntity_t,
) -> bool {
    if droidCent.currentState.owner != 0 && droidCent.currentState.clientNum >= MAX_CLIENTS_I32 {
        // the only NPCs that can ride a vehicle are droids...???
        let vehNum = droidCent.currentState.owner as usize;
        let vehGhoul2 = ctx.world.entity(vehNum).ghoul2;
        let droidUnitTag = ctx
            .world
            .entity(vehNum)
            .m_pVehicle
            .map(|id| ctx.world.vehicle_pool[id.ent_num() as usize].m_iDroidUnitTag);

        if let Some(droidUnitTag) = droidUnitTag {
            if !vehGhoul2.is_null() && droidUnitTag != -1 {
                let mut boltMatrix = mdxaBone_t {
                    matrix: [[0.0; 4]; 3],
                };
                let mut fwd: vec3_t = [0.0; 3];
                let mut rt: vec3_t = [0.0; 3];
                let mut tempAng: vec3_t = [0.0; 3];

                let vehAngles = ctx.world.entity(vehNum).lerpAngles;
                let vehOrigin = ctx.world.entity(vehNum).lerpOrigin;
                let vehScale = ctx.world.entity(vehNum).modelScale;
                let time = ctx.world.cg.time;
                trap::G2API_GetBoltMatrix(
                    ctx.engine,
                    vehGhoul2,
                    0,
                    droidUnitTag,
                    &mut boltMatrix,
                    &vehAngles,
                    &vehOrigin,
                    time,
                    Some(&mut ctx.world.cgs.gameModels[0]),
                    &vehScale,
                );
                BG_GiveMeVectorFromMatrix(
                    &boltMatrix,
                    Eorientations::ORIGIN as c_int,
                    &mut droidCent.lerpOrigin,
                );
                BG_GiveMeVectorFromMatrix(
                    &boltMatrix,
                    Eorientations::POSITIVE_X as c_int,
                    &mut fwd,
                ); //WTF???
                BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::NEGATIVE_Y as c_int, &mut rt); //WTF???
                vectoangles(fwd, &mut droidCent.lerpAngles);
                vectoangles(rt, &mut tempAng);
                droidCent.lerpAngles[ROLL] = tempAng[PITCH];

                return true;
            }
        }
    }

    false
}

/// Raven `CG_CreateNPCClient` — the per-NPC `clientInfo_t` an entity carries
/// beside the real client table. Raven pool-allocates it and every caller
/// memsets it straight after, so the block is handed back zeroed (DEC-46.2 makes
/// it owned, which is what retires Raven's "always free it, never stomp over
/// it" discipline).
/// Source: `oracle/codemp/cgame/cg_players.c:7695-7699`
pub fn CG_CreateNPCClient() -> Box<clientInfo_t> {
    Box::new(zeroed_client_info())
}

/// Raven `CG_DestroyNPCClient` — zeroes the NPC's client info and keeps the
/// block. The `trap_TrueFree` beside it is commented out in the oracle, so the
/// allocation deliberately stays alive and non-NULL: the caller's next
/// `if (!cent->npcClient)` must still be false.
/// Source: `oracle/codemp/cgame/cg_players.c:7701-7705`
pub fn CG_DestroyNPCClient(ci: &mut Option<Box<clientInfo_t>>) {
    if let Some(ci) = ci {
        **ci = zeroed_client_info();
    }
}

/// The zero fill Raven gets from its `memset(*ci, 0, sizeof(clientInfo_t))`.
/// `clientInfo_t` is too wide for a derived `Default` (its `char[64]` name
/// buffers), so the fill is spelled once here for both NPC-client fns.
pub(crate) fn zeroed_client_info() -> clientInfo_t {
    // SAFETY: `clientInfo_t` is `#[repr(C)]` scalars, arrays, `qhandle_t`s and
    // opaque ghoul2 `*mut c_void` tokens; its two enum members (`team_t`,
    // `gender_t`) both have a 0 discriminant, so all-zero is a legal value.
    unsafe { core::mem::zeroed() }
}

/// Raven `CG_InitJetpackGhoul2` — builds the one shared jetpack instance and
/// bolts it to the player's `*chestg` tag, plus the two jet-effect bolts.
/// Source: `oracle/codemp/cgame/cg_players.c:7891-7911`
pub fn CG_InitJetpackGhoul2(ctx: &mut CgContext) {
    if !ctx.world.players.cg_g2JetpackInstance.is_null() {
        debug_assert!(false, "Tried to init jetpack inst, already init'd");
        return;
    }

    let engine = ctx.engine;
    trap::G2API_InitGhoul2Model(
        engine,
        &mut ctx.world.players.cg_g2JetpackInstance as *mut *mut c_void,
        JETPACK_MODEL,
        0,
        0,
        0,
        0,
        0,
    );

    debug_assert!(!ctx.world.players.cg_g2JetpackInstance.is_null());

    // Indicate which bolt on the player we will be attached to
    // In this case bolt 0 is rhand, 1 is lhand, and 2 is the bolt
    // for the jetpack (*chestg)
    let g2 = ctx.world.players.cg_g2JetpackInstance;
    trap::G2API_SetBoltInfo(engine, g2, 0, 2);

    // Add the bolts jet effects will be played from
    trap::G2API_AddBolt(engine, g2, 0, "torso_ljet");
    trap::G2API_AddBolt(engine, g2, 0, "torso_rjet");
}

/// Raven `CG_CleanJetpackGhoul2` — drops the shared jetpack instance at
/// shutdown.
/// Source: `oracle/codemp/cgame/cg_players.c:7913-7920`
pub fn CG_CleanJetpackGhoul2(ctx: &mut CgContext) {
    if !ctx.world.players.cg_g2JetpackInstance.is_null() {
        trap::G2API_CleanGhoul2Models(
            ctx.engine,
            &mut ctx.world.players.cg_g2JetpackInstance as *mut *mut c_void,
        );
        ctx.world.players.cg_g2JetpackInstance = null_mut();
    }
}

/// Raven `CG_RadiusForCent` — the ghoul2 cull radius for an entity: an NPC
/// vehicle's `m_pVehicleInfo->g2radius` override, its own `g2radius`, or 64.
///
/// Source: `oracle/codemp/cgame/cg_players.c:8316-8336`
pub fn CG_RadiusForCent(world: &CgWorld, cent: &centity_t) -> f32 {
    if cent.currentState.eType == entityType_t::ET_NPC as c_int {
        let vehG2Radius = if cent.currentState.NPC_class == class_t::CLASS_VEHICLE as c_int {
            cent.m_pVehicle
                .map(|id| unsafe {
                    (*world.vehicle_pool[id.ent_num() as usize].m_pVehicleInfo).g2radius
                })
                .unwrap_or(0)
        } else {
            0
        };
        if vehG2Radius != 0 {
            //has override
            return vehG2Radius as f32;
        } else if cent.currentState.g2radius != 0 {
            return cent.currentState.g2radius as f32;
        }
    } else if cent.currentState.g2radius != 0 {
        return cent.currentState.g2radius as f32;
    }

    64.0
}

/// Raven `CG_ScanForExistingClientInfo` — hunts the client table for an already
/// loaded model/skin/saber set identical to `ci`'s and adopts its handles.
/// `false` means nothing matched, which is the caller's cue to defer the load.
///
/// Raven: rww - Filthy hack. If this is actually the info already belonging to
/// us, just reassign the pointer. Switching instances when not necessary
/// produces small animation glitches.
/// Source: `oracle/codemp/cgame/cg_players.c:1288-1387`
pub fn CG_ScanForExistingClientInfo(
    ctx: &mut CgContext,
    ci: &mut clientInfo_t,
    clientNum: c_int,
) -> bool {
    let ciModelName = buf_to_string(&ci.modelName.map(|c| c as u8));
    let ciSkinName = buf_to_string(&ci.skinName.map(|c| c as u8));
    let ciSaberName = buf_to_string(&ci.saberName.map(|c| c as u8));
    let ciSaber2Name = buf_to_string(&ci.saber2Name.map(|c| c as u8));

    for i in 0..ctx.world.cgs.maxclients as usize {
        // the whole match test only reads the table; the borrow dies with this
        // block so the copy below can take the entry back out mutably
        let isMatch = {
            let m = &ctx.world.cgs.clientinfo[i];
            m.infoValid != qfalse
                && m.deferred == qfalse
                && Q_stricmp(&ciModelName, &buf_to_string(&m.modelName.map(|c| c as u8))) == 0
                && Q_stricmp(&ciSkinName, &buf_to_string(&m.skinName.map(|c| c as u8))) == 0
                && Q_stricmp(&ciSaberName, &buf_to_string(&m.saberName.map(|c| c as u8))) == 0
                && Q_stricmp(&ciSaber2Name, &buf_to_string(&m.saber2Name.map(|c| c as u8))) == 0
                //			&& !Q_stricmp( ci->headModelName, match->headModelName )
                //			&& !Q_stricmp( ci->headSkinName, match->headSkinName )
                //			&& !Q_stricmp( ci->blueTeam, match->blueTeam )
                //			&& !Q_stricmp( ci->redTeam, match->redTeam )
                && (ctx.world.cgs.gametype < GT_TEAM || ci.team == m.team)
                && ci.siegeIndex == m.siegeIndex
                && !m.ghoul2Model.is_null()
                // if the bolts haven't been initialized, this "match" is useless to us
                && m.bolt_head != 0
        };

        if !isMatch {
            continue;
        }

        // this clientinfo is identical, so use it's handles
        ci.deferred = qfalse;

        if clientNum == i as c_int {
            let matchGhoul2 = ctx.world.cgs.clientinfo[i].ghoul2Model;

            //The match has a valid instance (if it didn't, we'd probably already be fudged (^_^) at this state)
            if !matchGhoul2.is_null() && trap::G2_HaveWeGhoul2Models(ctx.engine, matchGhoul2) {
                //First kill the copy we have if we have one. (but it should be null)
                if !ci.ghoul2Model.is_null()
                    && trap::G2_HaveWeGhoul2Models(ctx.engine, ci.ghoul2Model)
                {
                    trap::G2API_CleanGhoul2Models(
                        ctx.engine,
                        &mut ci.ghoul2Model as *mut *mut c_void,
                    );
                }

                let m = &ctx.world.cgs.clientinfo[i];
                _VectorCopy(m.headOffset, &mut ci.headOffset);
                // ci->footsteps = match->footsteps;
                ci.gender = m.gender;

                ci.legsModel = m.legsModel;
                ci.legsSkin = m.legsSkin;
                ci.torsoModel = m.torsoModel;
                ci.torsoSkin = m.torsoSkin;
                ci.modelIcon = m.modelIcon;

                ci.newAnims = m.newAnims;

                ci.bolt_head = m.bolt_head;
                ci.bolt_lhand = m.bolt_lhand;
                ci.bolt_rhand = m.bolt_rhand;
                ci.bolt_motion = m.bolt_motion;
                ci.bolt_llumbar = m.bolt_llumbar;
                ci.siegeIndex = m.siegeIndex;

                ci.sounds = m.sounds;
                ci.siegeSounds = m.siegeSounds;
                ci.duelSounds = m.duelSounds;

                //We can share this pointer, because it already belongs to this client.
                //The pointer itself and the ghoul2 instance is never actually changed, just passed between
                //clientinfo structures.
                ci.ghoul2Model = m.ghoul2Model;

                // Raven's saber-handle loop below this is commented out
                // (`cg_players.c:1360-1373`) — "whenever this function is called
                // the saber stuff should already be taken care of in the new info".
            }
        } else {
            // borrow split, not behavior: `CG_CopyClientInfoModel` reads the
            // match beside a live `ctx`.
            // bitwise copy-in, original left in place - a zeroed-swap is visible
            // to every ctx-reading helper inside CG_CopyClientInfoModel (C6b
            // referee catch). The copy is read-only, so it does not write back.
            // SAFETY: clientInfo_t is #[repr(C)] plain data.
            let matched = unsafe { core::ptr::read(&ctx.world.cgs.clientinfo[i]) };
            CG_CopyClientInfoModel(ctx, &matched, ci);
        }

        return true;
    }

    // nothing matches, so defer the load
    false
}

/// Raven `CG_SetLerpFrameAnimation` — starts `newAnimation` on the ghoul2
/// skeleton, picking up mid-animation where it can so a speed change doesn't
/// snap the pose.
///
/// PORT-NOTE: Raven's `lf` is `&cent->pe.legs` at every `torsoOnly == qfalse`
/// call site and `&cent->pe.torso` at every `qtrue` one
/// (`cg_players.c:3308,3326,11190,11191`), so the selector stands in for the
/// aliasing pointer parameter — nothing else was ever passed. Raven's `if (ci)`
/// guards are vestigial too: it derefs `ci` unguarded at lines 2778/2914/2943,
/// so the writes they wrap are unconditional here.
///
/// The broken-arm block after the motion bone is `#if 0`'d out in the oracle
/// (`cg_players.c:2970-3127`) and is not ported.
/// Source: `oracle/codemp/cgame/cg_players.c:2768-3129`
#[allow(clippy::too_many_arguments)]
pub fn CG_SetLerpFrameAnimation(
    ctx: &mut CgContext,
    cent: &mut centity_t,
    ci: &mut clientInfo_t,
    newAnimation: c_int,
    animSpeedMult: f32,
    torsoOnly: bool,
    flipState: bool,
) {
    let mut flags: c_int = BONE_ANIM_OVERRIDE_FREEZE;
    let mut blendTime: c_int = 100;

    if cent.localAnimIndex > 0 {
        //rockettroopers can't have broken arms, nor can anything else but humanoids
        ci.brokenLimbs = cent.currentState.brokenLimbs;
    }

    // the cent scalars the body still needs once `lf` holds the only borrow
    let ghoul2 = cent.ghoul2;
    let localAnimIndex = cent.localAnimIndex;
    let csNumber = cent.currentState.number;
    let csClientNum = cent.currentState.clientNum;
    let csFireflag = cent.currentState.fireflag;
    let csWeapon = cent.currentState.weapon;
    let csBrokenLimbs = cent.currentState.brokenLimbs;
    let csNPCclass = cent.currentState.NPC_class;
    let csTorsoAnim = cent.currentState.torsoAnim;
    let csLegsAnim = cent.currentState.legsAnim;
    let noLumbar = cent.noLumbar != qfalse;

    let lf: &mut lerpFrame_t = if torsoOnly {
        &mut cent.pe.torso
    } else {
        &mut cent.pe.legs
    };

    let oldSpeed = lf.animationSpeed;
    let oldAnim = lf.animationNumber;

    lf.animationNumber = newAnimation;

    if newAnimation < 0 || newAnimation >= animNumber_t::MAX_TOTALANIMATIONS as c_int {
        // Raven's `CG_Error` never returns; ours does, so the bad index is
        // stopped here instead of walking off the animation table.
        CG_Error(ctx, &format!("Bad animation number: {}", newAnimation));
        return;
    }

    // §F19: Raven indexes `bgAllAnims[cent->localAnimIndex]` unchecked — an
    // unparsed skeleton leaves the index at -1 and the table is a `Vec` here, so
    // an out-of-range index leaves the lerp frame exactly as it was.
    if localAnimIndex < 0 || localAnimIndex as usize >= ctx.world.bg_state.bgAllAnims.len() {
        return;
    }
    let animations = ctx.world.bg_state.bgAllAnims[localAnimIndex as usize].anims;
    if animations.is_null() {
        return;
    }

    // SAFETY: `anims` is the animation table `BG_ParseAnimationFile` allocated
    // for this skeleton; Raven indexes it unchecked and so do we.
    let anim = unsafe { animations.offset(newAnimation as isize) };
    let (animFirstFrame, animNumFrames, animFrameLerp, animLoopFrames) = unsafe {
        let a = &*anim;
        (
            a.firstFrame as c_int,
            a.numFrames as c_int,
            a.frameLerp as c_int,
            a.loopFrames as c_int,
        )
    };

    lf.animation = anim;
    lf.animationTime = lf.frameTime + animFrameLerp.abs();

    if localAnimIndex > 1 && animFirstFrame == 0 && animNumFrames == 0 {
        //We'll allow this for non-humanoids.
        return;
    }

    let cg_debugAnim = ctx.world.cvars.cg_debugAnim.integer;
    if cg_debugAnim != 0 && (cg_debugAnim < 0 || cg_debugAnim == csClientNum) {
        let time = ctx.world.cg.time;
        // `GetStringForID`'s only ported home is mp_game's `q_shared.rs`, and
        // mp_cgame does not depend on mp_game (DEC-32 keeps the one copy), so
        // its table walk is spelled out here: stop at the NULL-name sentinel,
        // answer the first id match. A miss can't happen past the range check
        // above, so the empty-string fallback is unreachable.
        let animName = animTable
            .iter()
            .take_while(|e| !e.name.is_null())
            .find(|e| e.id == newAnimation)
            // SAFETY: every non-sentinel `animTable` name is a static
            // NUL-terminated literal, the same read `bg_panimate.rs` makes.
            .map(|e| unsafe { cstr_to_str(e.name) })
            .unwrap_or_default();

        // PORT-NOTE: Raven's two labels are swapped — the legs lerp frame prints
        // "TORSO" and the torso one prints "LEGS". Kept.
        if !torsoOnly {
            CG_Printf(
                ctx,
                &format!(
                    "{}: {} TORSO Anim: {}, '{}'\n",
                    time, csClientNum, newAnimation, animName
                ),
            );
        } else {
            CG_Printf(
                ctx,
                &format!(
                    "{}: {} LEGS Anim: {}, '{}'\n",
                    time, csClientNum, newAnimation, animName
                ),
            );
        }
    }

    if ghoul2.is_null() {
        return;
    }

    let engine = ctx.engine;
    let time = ctx.world.cg.time;

    let mut resumeFrame = false;
    let mut beginFrame: c_int = -1;
    let firstFrame;
    let lastFrame;

    let mut animSpeed = 50.0f32 / animFrameLerp as f32;
    if animLoopFrames != -1 {
        flags = BONE_ANIM_OVERRIDE_LOOP;
    }

    if animSpeed < 0.0 {
        lastFrame = animFirstFrame;
        firstFrame = animFirstFrame + animNumFrames;
    } else {
        firstFrame = animFirstFrame;
        lastFrame = animFirstFrame + animNumFrames;
    }

    if ctx.world.cvars.cg_animBlend.integer != 0 {
        flags |= BONE_ANIM_BLEND;
    }

    if BG_InDeathAnim(newAnimation) != qfalse {
        flags &= !BONE_ANIM_BLEND;
    } else if oldAnim != -1 && BG_InDeathAnim(oldAnim) != qfalse {
        flags &= !BONE_ANIM_BLEND;
    }

    if flags & BONE_ANIM_BLEND != 0 {
        if BG_FlippingAnim(newAnimation) != qfalse {
            blendTime = 200;
        } else if oldAnim != -1 && BG_FlippingAnim(oldAnim) != qfalse {
            blendTime = 200;
        }
    }

    animSpeed *= animSpeedMult;

    let mut callbacks = CgGameCallbacks::new(engine, ctx.world_raw());
    // the CG_Player chain holds this entity's clientInfo swapped out of the
    // world (slot-swap/npcClient take), so my_saber must resolve the live one
    callbacks.live_ci = Some((csNumber, &raw mut *ci));
    BG_SaberStartTransAnim(
        csNumber,
        csFireflag,
        csWeapon,
        newAnimation,
        &mut animSpeed as *mut f32,
        csBrokenLimbs,
        &mut callbacks,
    );

    if torsoOnly {
        if lf.animationTorsoSpeed != animSpeedMult
            && newAnimation == oldAnim
            && flipState == (lf.lastFlip != qfalse)
        {
            //same animation, but changing speed, so we will want to resume off the frame we're on.
            resumeFrame = true;
        }
        lf.animationTorsoSpeed = animSpeedMult;
    } else {
        if lf.animationSpeed != animSpeedMult
            && newAnimation == oldAnim
            && flipState == (lf.lastFlip != qfalse)
        {
            //same animation, but changing speed, so we will want to resume off the frame we're on.
            resumeFrame = true;
        }
        lf.animationSpeed = animSpeedMult;
    }

    //vehicles may have torso etc but we only want to animate the root bone
    if csNPCclass == class_t::CLASS_VEHICLE as c_int {
        trap::G2API_SetBoneAnim(
            engine,
            ghoul2,
            0,
            "model_root",
            firstFrame,
            lastFrame,
            flags,
            animSpeed,
            time,
            beginFrame as f32,
            blendTime,
        );
        return;
    }

    // PORT-NOTE: Raven passes NULL for `trap_G2API_GetBoneFrame`'s `modelList`;
    // the engine forwards it to `G2_Get_Bone_Anim_Index`, which never
    // dereferences it (`G2_bones.cpp:872-900`), so the wrapper's required slot
    // gets a throwaway int.
    let mut modelList: c_int = 0;

    if torsoOnly && !noLumbar {
        //rww - The guesswork based on the lerp frame figures is usually BS, so I've resorted to a call to get the frame of the bone directly.
        let mut GBAcFrame: f32 = 0.0;
        if resumeFrame {
            //we already checked, and this is the same anim, same flip state, but different speed, so we want to resume with the new speed off of the same frame.
            trap::G2API_GetBoneFrame(
                engine,
                ghoul2,
                "lower_lumbar",
                time,
                &mut GBAcFrame,
                &mut modelList,
                0,
            );
            beginFrame = GBAcFrame as c_int;
        }

        //even if resuming, also be sure to check if we are running the same frame on the legs. If so, we want to use their frame no matter what.
        trap::G2API_GetBoneFrame(
            engine,
            ghoul2,
            "model_root",
            time,
            &mut GBAcFrame,
            &mut modelList,
            0,
        );

        if csTorsoAnim == csLegsAnim
            && GBAcFrame >= animFirstFrame as f32
            && GBAcFrame <= (animFirstFrame + animNumFrames) as f32
        {
            //if the legs are already running this anim, pick up on the exact same frame to avoid the "wobbly spine" problem.
            beginFrame = GBAcFrame as c_int;
        }

        if firstFrame > lastFrame || ci.torsoAnim == newAnimation {
            //don't resume on backwards playing animations.. I guess.
            beginFrame = -1;
        }

        trap::G2API_SetBoneAnim(
            engine,
            ghoul2,
            0,
            "lower_lumbar",
            firstFrame,
            lastFrame,
            flags,
            animSpeed,
            time,
            beginFrame as f32,
            blendTime,
        );

        // Update the torso frame with the new animation (this arm's `lf` IS
        // `cent->pe.torso`, so Raven's explicit write lands here)
        lf.frame = firstFrame;

        ci.torsoAnim = newAnimation;
    } else {
        if resumeFrame {
            //we already checked, and this is the same anim, same flip state, but different speed, so we want to resume with the new speed off of the same frame.
            let mut GBAcFrame: f32 = 0.0;
            trap::G2API_GetBoneFrame(
                engine,
                ghoul2,
                "model_root",
                time,
                &mut GBAcFrame,
                &mut modelList,
                0,
            );
            beginFrame = GBAcFrame as c_int;
        }

        if beginFrame < firstFrame || beginFrame > lastFrame {
            //out of range, don't use it then.
            beginFrame = -1;
        }

        if csTorsoAnim == csLegsAnim && (ci.legsAnim != newAnimation || oldSpeed != animSpeed) {
            //alright, we are starting an anim on the legs, and that same anim is already playing on the toro, so pick up the frame.
            let mut GBAcFrame: f32 = 0.0;
            let oldBeginFrame = beginFrame;

            trap::G2API_GetBoneFrame(
                engine,
                ghoul2,
                "lower_lumbar",
                time,
                &mut GBAcFrame,
                &mut modelList,
                0,
            );
            beginFrame = GBAcFrame as c_int;
            if beginFrame < firstFrame || beginFrame > lastFrame {
                //out of range, don't use it then.
                beginFrame = oldBeginFrame;
            }
        }

        trap::G2API_SetBoneAnim(
            engine,
            ghoul2,
            0,
            "model_root",
            firstFrame,
            lastFrame,
            flags,
            animSpeed,
            time,
            beginFrame as f32,
            blendTime,
        );

        ci.legsAnim = newAnimation;
    }

    if localAnimIndex <= 1 && csTorsoAnim == newAnimation && !noLumbar {
        //make sure we're humanoid before we access the motion bone
        trap::G2API_SetBoneAnim(
            engine,
            ghoul2,
            0,
            "Motion",
            firstFrame,
            lastFrame,
            flags,
            animSpeed,
            time,
            beginFrame as f32,
            blendTime,
        );
    }
}

/// Raven `CG_RagDoll` — decides whether a corpse should go limp and, once it
/// has, feeds the engine's ragdoll solver a fresh position/velocity every frame.
/// `true` means we are ragging and the caller should skip its normal animation
/// work.
/// Source: `oracle/codemp/cgame/cg_players.c:3486-3911`
pub fn CG_RagDoll(ctx: &mut CgContext, cent: &mut centity_t, forcedAngles: &vec3_t) -> bool {
    let mut usedOrg: vec3_t = [0.0; 3];
    let mut inSomething = false;

    if ctx.world.cvars.cg_ragDoll.integer == 0 {
        return false;
    }

    if cent.localAnimIndex != 0 {
        //don't rag non-humanoids
        return false;
    }

    _VectorCopy(cent.lerpOrigin, &mut usedOrg);

    if cent.isRagging == qfalse {
        //If we're not in a ragdoll state, perform the checks.
        if cent.currentState.eFlags & EF_RAG != 0 {
            //want to go into it no matter what then
            inSomething = true;
        } else if cent.currentState.groundEntityNum == ENTITYNUM_NONE {
            let mut cVel: vec3_t = [0.0; 3];

            _VectorCopy(cent.currentState.pos.trDelta, &mut cVel);

            if VectorNormalize(&mut cVel) > 400.0 {
                //if he's flying through the air at a good enough speed, switch into ragdoll
                inSomething = true;
            }
        }

        if cent.currentState.eType == entityType_t::ET_BODY as c_int {
            //just rag bodies immediately if their own was ragging on respawn
            if cent.ownerRagging != qfalse {
                cent.isRagging = qtrue;
                return false;
            }
        }

        if ctx.world.cvars.cg_ragDoll.integer > 1 {
            inSomething = true;
        }

        if !inSomething {
            let anim = cent.currentState.legsAnim;
            let mut dur: c_int = 0;

            // §F19: same unchecked `bgAllAnims` read as elsewhere in this file —
            // with no parsed skeleton the duration stays 0, which just makes the
            // death anim read as finished.
            if let Some(loaded) = ctx
                .world
                .bg_state
                .bgAllAnims
                .get(cent.localAnimIndex as usize)
            {
                if !loaded.anims.is_null() {
                    // SAFETY: Raven indexes this table unchecked and so do we.
                    let (numFrames, frameLerp) = unsafe {
                        let a = &*loaded.anims.offset(anim as isize);
                        (a.numFrames as c_int, a.frameLerp as c_int)
                    };
                    dur = ((numFrames - 1) as f64 * (frameLerp as f32 as f64).abs()) as c_int;
                }
            }

            let mut i: usize = 0;
            let mut boltChecks: [c_int; 5] = [0; 5];
            let mut boltPoints: [vec3_t; 5] = [[0.0; 3]; 5];
            let mut trStart: vec3_t = [0.0; 3];
            let mut trEnd: vec3_t = [0.0; 3];
            let mut tAng: vec3_t = [0.0; 3];
            let mut deathDone = false;
            let mut tr = trace_t::zeroed();
            let mut boltMatrix = mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            };

            VectorSet(
                &mut tAng,
                cent.turAngles[PITCH],
                cent.turAngles[YAW],
                cent.turAngles[ROLL],
            );

            if cent.pe.legs.animationTime > 50
                && (ctx.world.cg.time - cent.pe.legs.animationTime) > dur
            {
                //Looks like the death anim is done playing
                deathDone = true;
            }

            let engine = ctx.engine;
            let ghoul2 = cent.ghoul2;
            let time = ctx.world.cg.time;
            let lerpOrigin = cent.lerpOrigin;
            let modelScale = cent.modelScale;
            let csNumber = cent.currentState.number;

            if deathDone {
                //only trace from the hands if the death anim is already done.
                boltChecks[0] = trap::G2API_AddBolt(engine, ghoul2, 0, "rhand");
                boltChecks[1] = trap::G2API_AddBolt(engine, ghoul2, 0, "lhand");
            } else {
                //otherwise start the trace loop at the cranium.
                i = 2;
            }
            boltChecks[2] = trap::G2API_AddBolt(engine, ghoul2, 0, "cranium");
            //boltChecks[3] = trap_G2API_AddBolt(cent->ghoul2, 0, "rtarsal");
            //boltChecks[4] = trap_G2API_AddBolt(cent->ghoul2, 0, "ltarsal");
            boltChecks[3] = trap::G2API_AddBolt(engine, ghoul2, 0, "rtalus");
            boltChecks[4] = trap::G2API_AddBolt(engine, ghoul2, 0, "ltalus");

            //This may seem bad, but since we have a bone cache now it should manage to not be too disgustingly slow.
            //Do the head first, because the hands reference it anyway.
            trap::G2API_GetBoltMatrix(
                engine,
                ghoul2,
                0,
                boltChecks[2],
                &mut boltMatrix,
                &tAng,
                &lerpOrigin,
                time,
                Some(&mut ctx.world.cgs.gameModels[0]),
                &modelScale,
            );
            BG_GiveMeVectorFromMatrix(
                &boltMatrix,
                Eorientations::ORIGIN as c_int,
                &mut boltPoints[2],
            );

            while i < 5 {
                if i < 2 {
                    //when doing hands, trace to the head instead of origin
                    trap::G2API_GetBoltMatrix(
                        engine,
                        ghoul2,
                        0,
                        boltChecks[i],
                        &mut boltMatrix,
                        &tAng,
                        &lerpOrigin,
                        time,
                        Some(&mut ctx.world.cgs.gameModels[0]),
                        &modelScale,
                    );
                    BG_GiveMeVectorFromMatrix(
                        &boltMatrix,
                        Eorientations::ORIGIN as c_int,
                        &mut boltPoints[i],
                    );
                    _VectorCopy(boltPoints[i], &mut trStart);
                    _VectorCopy(boltPoints[2], &mut trEnd);
                } else {
                    if i > 2 {
                        //2 is the head, which already has the bolt point.
                        trap::G2API_GetBoltMatrix(
                            engine,
                            ghoul2,
                            0,
                            boltChecks[i],
                            &mut boltMatrix,
                            &tAng,
                            &lerpOrigin,
                            time,
                            Some(&mut ctx.world.cgs.gameModels[0]),
                            &modelScale,
                        );
                        BG_GiveMeVectorFromMatrix(
                            &boltMatrix,
                            Eorientations::ORIGIN as c_int,
                            &mut boltPoints[i],
                        );
                    }
                    _VectorCopy(boltPoints[i], &mut trStart);
                    _VectorCopy(lerpOrigin, &mut trEnd);
                }

                //Now that we have all that sorted out, trace between the two points we desire.
                //
                // PORT-NOTE: Raven passes NULL mins/maxs; `CM_Trace` substitutes
                // `vec3_origin` for a NULL bound (`cm_trace.cpp:1603-1610`), so
                // the zero vectors are the same point trace.
                CG_Rag_Trace(
                    ctx,
                    &mut tr,
                    &trStart,
                    &vec3_origin,
                    &vec3_origin,
                    &trEnd,
                    csNumber,
                    MASK_SOLID,
                );

                if tr.fraction != 1.0 || tr.startsolid != 0 || tr.allsolid != 0 {
                    //Hit something or start in solid, so flag it and break.
                    //This is a slight hack, but if we aren't done with the death anim, we don't really want to
                    //go into ragdoll unless our body has a relatively "flat" pitch. (the pitch test itself is
                    //`#if 0`'d out in the oracle, `cg_players.c:3603-3617`)
                    inSomething = true;
                    break;
                }

                i += 1;
            }
        }

        if inSomething {
            cent.isRagging = qtrue;
            // VectorClear(cent->lerpOriginOffset) here is `#if 0`'d in the oracle
        }
    }

    if cent.isRagging != qfalse {
        //We're in a ragdoll state, so make the call to keep our positions updated and whatnot.
        let engine = ctx.engine;
        let ghoul2 = cent.ghoul2;
        let time = ctx.world.cg.time;

        let ragAnim = CG_RagAnimForPositioning(ctx, cent);

        if cent.ikStatus != qfalse {
            //ik must be reset before ragdoll is started, or you'll get some interesting results.
            trap::G2API_SetBoneIKState(
                engine,
                ghoul2,
                time,
                None,
                sharedEIKMoveState::IKS_NONE as c_int,
                None,
            );
            cent.ikStatus = qfalse;
        }

        //these will be used as "base" frames for the ragoll settling.
        let mut startFrame: c_int = 0;
        let mut endFrame: c_int = 0;
        // §F19: the unchecked `bgAllAnims` read again — no skeleton leaves the
        // settle range at 0..0, which the solver treats as "no base pose".
        if let Some(loaded) = ctx
            .world
            .bg_state
            .bgAllAnims
            .get(cent.localAnimIndex as usize)
        {
            if !loaded.anims.is_null() {
                // SAFETY: Raven indexes this table unchecked and so do we.
                let (firstFrame, numFrames) = unsafe {
                    let a = &*loaded.anims.offset(ragAnim as isize);
                    (a.firstFrame as c_int, a.numFrames as c_int)
                };
                startFrame = firstFrame;
                endFrame = firstFrame + numFrames;
            }
        }

        // Raven: with my new method of doing things I want it to continue the anim
        {
            let mut currentFrame: f32 = 0.0;
            let mut gbaStartFrame: c_int = 0;
            let mut gbaEndFrame: c_int = 0;
            let mut gbaFlags: c_int = 0;
            let mut gbaAnimSpeed: f32 = 0.0;

            if trap::G2API_GetBoneAnim(
                engine,
                ghoul2,
                "model_root",
                time,
                &mut currentFrame,
                &mut gbaStartFrame,
                &mut gbaEndFrame,
                &mut gbaFlags,
                &mut gbaAnimSpeed,
                Some(&mut ctx.world.cgs.gameModels[0]),
                0,
            ) {
                //lock the anim on the current frame.
                let blendTime: c_int = 500;

                let mut curFirstFrame: c_int = 0;
                let mut curNumFrames: c_int = 0;
                if let Some(loaded) = ctx
                    .world
                    .bg_state
                    .bgAllAnims
                    .get(cent.localAnimIndex as usize)
                {
                    if !loaded.anims.is_null() {
                        // SAFETY: Raven indexes this table unchecked and so do we.
                        let (ff, nf) = unsafe {
                            let a = &*loaded.anims.offset(cent.currentState.legsAnim as isize);
                            (a.firstFrame as c_int, a.numFrames as c_int)
                        };
                        curFirstFrame = ff;
                        curNumFrames = nf;
                    }
                }

                if currentFrame >= (curFirstFrame + curNumFrames - 1) as f32 {
                    //this is sort of silly but it works for now.
                    currentFrame = (curFirstFrame + curNumFrames - 2) as f32;
                }

                for bone in ["lower_lumbar", "model_root", "Motion"] {
                    trap::G2API_SetBoneAnim(
                        engine,
                        ghoul2,
                        0,
                        bone,
                        currentFrame as c_int,
                        (currentFrame + 1.0) as c_int,
                        gbaFlags,
                        gbaAnimSpeed,
                        time,
                        currentFrame,
                        blendTime,
                    );
                }
            }
        }

        // PORT-NOTE: Raven hands `cgs.gameModels` to these four; the borrow can't
        // ride beside a live `ctx`, and `G2_Set_Bone_Angles` never dereferences
        // `modelList` (`G2_bones.cpp:477-535`), so `None` is the same call.
        for bone in ["upper_lumbar", "lower_lumbar", "thoracic", "cervical"] {
            CG_G2SetBoneAngles(
                ctx,
                ghoul2,
                0,
                bone,
                &vec3_origin,
                BONE_ANGLES_POSTMULT,
                Eorientations::POSITIVE_X as c_int,
                Eorientations::NEGATIVE_Y as c_int,
                Eorientations::NEGATIVE_Z as c_int,
                None,
                0,
                time,
            );
        }

        // §F19: Raven's stack `tParms` is only partly filled — the pelvis
        // offsets, impact strength, begin flag and effector mask are read back
        // out by the engine, never in, so the port sends zeros rather than
        // whatever was on the stack.
        let mut tParms = sharedRagDollParams_t {
            angles: *forcedAngles,
            position: usedOrg,
            scale: cent.modelScale,
            pelvis_angles_offset: [0.0; 3],
            pelvis_position_offset: [0.0; 3],
            f_impact_strength: 0.0,
            f_shot_strength: 4.0,
            me: cent.currentState.number,
            start_frame: startFrame,
            end_frame: endFrame,
            collision_type: 1,
            call_rag_doll_begin: qfalse,
            rag_phase: sharedERagPhase::RP_DEATH_COLLISION as c_int,
            effectors_to_turn_off: 0,
        };

        trap::G2API_SetRagDoll(engine, ghoul2, Some(&mut tParms));

        let mut tuParms = sharedRagDollUpdateParams_t {
            angles: *forcedAngles,
            position: usedOrg,
            scale: cent.modelScale,
            velocity: [0.0; 3],
            me: cent.currentState.number,
            settle_frame: tParms.end_frame - 1,
        };

        if cent.currentState.groundEntityNum != ENTITYNUM_NONE {
            VectorClear(&mut tuParms.velocity);
        } else {
            _VectorScale(cent.currentState.pos.trDelta, 2.0, &mut tuParms.velocity);
        }

        trap::G2API_AnimateG2Models(engine, ghoul2, time, &mut tuParms);

        //So if we try to get a bolt point it's still correct
        cent.turAngles[YAW] = forcedAngles[YAW];
        cent.lerpAngles[YAW] = forcedAngles[YAW];
        cent.pe.torso.yawAngle = forcedAngles[YAW];
        cent.pe.legs.yawAngle = forcedAngles[YAW];

        if cent.currentState.ragAttach != 0
            && (cent.currentState.eType != entityType_t::ET_NPC as c_int
                || cent.currentState.NPC_class != class_t::CLASS_VEHICLE as c_int)
        {
            let grabNum = if cent.currentState.ragAttach == ENTITYNUM_NONE {
                //switch cl 0 and entitynum_none, so we can operate on the "if non-0" concept
                0usize
            } else {
                cent.currentState.ragAttach as usize
            };

            let grabEnt = ctx.world.entity(grabNum);
            let grabGhoul2 = grabEnt.ghoul2;
            let grabTurAngles = grabEnt.turAngles;
            let grabLerpOrigin = grabEnt.lerpOrigin;
            let grabModelScale = grabEnt.modelScale;

            if !grabGhoul2.is_null() {
                let mut matrix = mdxaBone_t {
                    matrix: [[0.0; 4]; 3],
                };
                let mut bOrg: vec3_t = [0.0; 3];
                let mut thisHand: vec3_t = [0.0; 3];
                let mut hands: vec3_t = [0.0; 3];
                let mut pcjMin: vec3_t = [0.0; 3];
                let mut pcjMax: vec3_t = [0.0; 3];
                let mut pDif: vec3_t = [0.0; 3];
                let mut thorPoint: vec3_t = [0.0; 3];

                let turAngles = cent.turAngles;
                let lerpOrigin = cent.lerpOrigin;
                let modelScale = cent.modelScale;

                //Get the person who is holding our hand's hand location
                trap::G2API_GetBoltMatrix(
                    engine,
                    grabGhoul2,
                    0,
                    0,
                    &mut matrix,
                    &grabTurAngles,
                    &grabLerpOrigin,
                    time,
                    Some(&mut ctx.world.cgs.gameModels[0]),
                    &grabModelScale,
                );
                BG_GiveMeVectorFromMatrix(&matrix, Eorientations::ORIGIN as c_int, &mut bOrg);

                //Get our hand's location
                trap::G2API_GetBoltMatrix(
                    engine,
                    ghoul2,
                    0,
                    0,
                    &mut matrix,
                    &turAngles,
                    &lerpOrigin,
                    time,
                    Some(&mut ctx.world.cgs.gameModels[0]),
                    &modelScale,
                );
                BG_GiveMeVectorFromMatrix(&matrix, Eorientations::ORIGIN as c_int, &mut thisHand);

                //Get the position of the thoracic bone for hinting its velocity later on
                let thorBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "thoracic");
                trap::G2API_GetBoltMatrix(
                    engine,
                    ghoul2,
                    0,
                    thorBolt,
                    &mut matrix,
                    &turAngles,
                    &lerpOrigin,
                    time,
                    Some(&mut ctx.world.cgs.gameModels[0]),
                    &modelScale,
                );
                BG_GiveMeVectorFromMatrix(&matrix, Eorientations::ORIGIN as c_int, &mut thorPoint);

                _VectorSubtract(bOrg, thisHand, &mut hands);

                if VectorLength(hands) < 3.0 {
                    trap::G2API_RagForceSolve(engine, ghoul2, false);
                } else {
                    trap::G2API_RagForceSolve(engine, ghoul2, true);
                }

                //got the hand pos of him, now we want to make our hand go to it
                for bone in ["rhand", "rradius", "rradiusX", "rhumerusX", "rhumerus"] {
                    trap::G2API_RagEffectorGoal(engine, ghoul2, bone, Some(&bOrg));
                }

                //Make these two solve quickly so we can update decently
                trap::G2API_RagPCJGradientSpeed(engine, ghoul2, "rhumerus", 1.5);
                trap::G2API_RagPCJGradientSpeed(engine, ghoul2, "rradius", 1.5);

                //Break the constraints on them I suppose
                VectorSet(&mut pcjMin, -999.0, -999.0, -999.0);
                VectorSet(&mut pcjMax, 999.0, 999.0, 999.0);
                trap::G2API_RagPCJConstraint(engine, ghoul2, "rhumerus", &pcjMin, &pcjMax);
                trap::G2API_RagPCJConstraint(engine, ghoul2, "rradius", &pcjMin, &pcjMax);

                cent.overridingBones = time + 2000;

                //hit the thoracic velocity to the hand point
                _VectorSubtract(bOrg, thorPoint, &mut hands);
                VectorNormalize(&mut hands);
                let h = hands;
                _VectorScale(h, 2048.0, &mut hands);
                trap::G2API_RagEffectorKick(engine, ghoul2, "thoracic", &hands);
                trap::G2API_RagEffectorKick(engine, ghoul2, "ceyebrow", &hands);

                _VectorSubtract(cent.ragLastOrigin, cent.lerpOrigin, &mut pDif);
                let o = cent.lerpOrigin;
                _VectorCopy(o, &mut cent.ragLastOrigin);

                if cent.ragLastOriginTime >= time
                    && cent.currentState.groundEntityNum != ENTITYNUM_NONE
                {
                    //make sure it's reasonably updated
                    let difLen = VectorLength(pDif);
                    if difLen > 0.0 {
                        //if we're being dragged, then kick all the bones around a bit
                        if difLen < 12.0 {
                            let p = pDif;
                            _VectorScale(p, 12.0 / difLen, &mut pDif);
                            // Raven stores difLen = 12.0 here; nothing reads it again
                        }

                        for bone in cg_effectorStringTable {
                            let mut dVel: vec3_t = [0.0; 3];
                            let mut rVel: vec3_t = [0.0; 3];

                            _VectorCopy(pDif, &mut dVel);
                            dVel[2] = 0.0;

                            //Factor in a random velocity
                            let rx = ctx.world.bg_state.rng.flrand(-0.1, 0.1);
                            let ry = ctx.world.bg_state.rng.flrand(-0.1, 0.1);
                            let rz = ctx.world.bg_state.rng.flrand(0.1, 0.5);
                            VectorSet(&mut rVel, rx, ry, rz);
                            let r = rVel;
                            _VectorScale(r, 8.0, &mut rVel);

                            let d = dVel;
                            _VectorAdd(d, rVel, &mut dVel);
                            let d = dVel;
                            _VectorScale(d, 10.0, &mut dVel);

                            trap::G2API_RagEffectorKick(engine, ghoul2, bone, &dVel);
                        }
                    }
                }
                cent.ragLastOriginTime = time + 1000;
            }
        } else if cent.overridingBones != 0 {
            //reset things to their normal rag state
            let mut pcjMin: vec3_t = [0.0; 3];
            let mut pcjMax: vec3_t = [0.0; 3];
            let mut dVel: vec3_t = [0.0; 3];

            //got the hand pos of him, now we want to make our hand go to it
            // NULL clears the over-goal - the engine's "go back to none" arm
            // (`oracle/codemp/ghoul2/G2_API.cpp:1552-1555`)
            for bone in ["rhand", "rradius", "rradiusX", "rhumerusX", "rhumerus"] {
                trap::G2API_RagEffectorGoal(engine, ghoul2, bone, None);
            }

            VectorSet(&mut dVel, 0.0, 0.0, -64.0);
            trap::G2API_RagEffectorKick(engine, ghoul2, "rhand", &dVel);

            trap::G2API_RagPCJGradientSpeed(engine, ghoul2, "rhumerus", 0.0);
            trap::G2API_RagPCJGradientSpeed(engine, ghoul2, "rradius", 0.0);

            VectorSet(&mut pcjMin, -100.0, -40.0, -15.0);
            VectorSet(&mut pcjMax, -15.0, 80.0, 15.0);
            trap::G2API_RagPCJConstraint(engine, ghoul2, "rhumerus", &pcjMin, &pcjMax);

            VectorSet(&mut pcjMin, -25.0, -20.0, -20.0);
            VectorSet(&mut pcjMax, 90.0, 20.0, -20.0);
            trap::G2API_RagPCJConstraint(engine, ghoul2, "rradius", &pcjMin, &pcjMax);

            if cent.overridingBones < time {
                trap::G2API_RagForceSolve(engine, ghoul2, false);
                cent.overridingBones = 0;
            } else {
                trap::G2API_RagForceSolve(engine, ghoul2, true);
            }
        }

        return true;
    }

    false
}

/// Raven `CG_G2PlayerHeadAnims` — runs a humanoid's face: blinks, the odd
/// frown, and the talk pose driven by the voice channel's volume. `true` when a
/// face animation was actually started this frame.
///
/// Raven: Dead people close their eyes and don't make faces!
/// Source: `oracle/codemp/cgame/cg_players.c:4059-4189`
pub fn CG_G2PlayerHeadAnims(ctx: &mut CgContext, cent: &mut centity_t) -> bool {
    let mut anim: c_int = -1;

    if cent.localAnimIndex > 1 {
        //only do this for humanoids
        return false;
    }

    if cent.noFace != qfalse {
        // i don't have a face
        return false;
    }

    let number = cent.currentState.number;
    let isClient = number < MAX_CLIENTS_I32;

    // the three facial timers live either in the client table or in the NPC's
    // own info; they are pulled out here and written back at the bottom so the
    // blink/anim calls can hold `ctx` and `cent` at the same time
    let (mut facial_blink, mut facial_frown, mut facial_aux) = if isClient {
        let ci = &ctx.world.cgs.clientinfo[number as usize];
        (ci.facial_blink, ci.facial_frown, ci.facial_aux)
    } else {
        match cent.npcClient.as_ref() {
            Some(ci) => (ci.facial_blink, ci.facial_frown, ci.facial_aux),
            None => return false,
        }
    };

    let time = ctx.world.cg.time;

    if cent.currentState.eFlags & EF_DEAD != 0 {
        //Dead people close their eyes and don't make faces!
        anim = animNumber_t::FACE_DEAD as c_int;
        facial_blink = -1.0;
    } else {
        if facial_blink == 0.0 {
            // set the timers
            facial_blink = time as f32 + ctx.world.bg_state.rng.flrand(4000.0, 8000.0);
            facial_frown = time as f32 + ctx.world.bg_state.rng.flrand(6000.0, 10000.0);
            facial_aux = time as f32 + ctx.world.bg_state.rng.flrand(6000.0, 10000.0);
        }

        //are we blinking?
        if facial_blink < 0.0 {
            // yes, check if we are we done blinking ?
            if -facial_blink < time as f32 {
                // yes, so reset blink timer
                facial_blink = time as f32 + ctx.world.bg_state.rng.flrand(4000.0, 8000.0);
                CG_G2SetHeadBlink(ctx, cent, false); //stop the blink
            }
        } else {
            // no we aren't blinking
            if facial_blink < time as f32 {
                // but should we start ?
                CG_G2SetHeadBlink(ctx, cent, true);
                if facial_blink == 1.0 {
                    //requested to stay shut by SET_FACEEYESCLOSED
                    facial_blink = -(time as f32 + 99999999.0f32); // set blink timer
                } else {
                    facial_blink = -(time as f32 + 300.0f32); // set blink timer
                }
            }
        }

        let voiceVolume = trap::S_GetVoiceVolume(ctx.engine, number);

        if voiceVolume > 0 {
            // if we aren't talking, then it will be 0, -1 for talking but paused
            anim = animNumber_t::FACE_TALK1 as c_int + voiceVolume - 1;
        } else if voiceVolume == 0 {
            //don't do aux if in a slient part of speech
            //not talking
            if facial_aux < 0.0 {
                // are we auxing ? yes
                if -facial_aux < time as f32 {
                    // are we done auxing ? yes, reset aux timer
                    facial_aux = time as f32 + ctx.world.bg_state.rng.flrand(7000.0, 10000.0);
                } else {
                    // not yet, so choose aux
                    anim = animNumber_t::FACE_ALERT as c_int;
                }
            } else {
                // no we aren't auxing, but should we start ?
                if facial_aux < time as f32 {
                    //yes
                    anim = animNumber_t::FACE_ALERT as c_int;
                    // set aux timer
                    facial_aux = (-(time as f64 + 2000.0)) as f32;
                }
            }

            if anim != -1 {
                //we we are auxing, see if we should override with a frown
                if facial_frown < 0.0 {
                    // are we frowning ? yes,
                    if -facial_frown < time as f32 {
                        //are we done frowning ? yes, reset frown timer
                        facial_frown = time as f32 + ctx.world.bg_state.rng.flrand(7000.0, 10000.0);
                    } else {
                        // not yet, so choose frown
                        anim = animNumber_t::FACE_FROWN as c_int;
                    }
                } else {
                    // no we aren't frowning, but should we start ?
                    if facial_frown < time as f32 {
                        anim = animNumber_t::FACE_FROWN as c_int;
                        // set frown timer
                        facial_frown = (-(time as f64 + 2000.0)) as f32;
                    }
                }
            }
        } //talking
    } //dead

    if isClient {
        let ci = &mut ctx.world.cgs.clientinfo[number as usize];
        ci.facial_blink = facial_blink;
        ci.facial_frown = facial_frown;
        ci.facial_aux = facial_aux;
    } else if let Some(ci) = cent.npcClient.as_mut() {
        ci.facial_blink = facial_blink;
        ci.facial_frown = facial_frown;
        ci.facial_aux = facial_aux;
    }

    if anim != -1 {
        CG_G2SetHeadAnim(ctx, cent, anim);
        return true;
    }

    false
}

/// Raven `CG_PlayerFlag` — the CTF flag hanging off a carrier's lower lumbar,
/// canted back and to the right. Never drawn on your own body in first person.
/// Source: `oracle/codemp/cgame/cg_players.c:4367-4441`
pub fn CG_PlayerFlag(ctx: &mut CgContext, cent: &centity_t, hModel: qhandle_t) {
    let mut angles: vec3_t = [0.0; 3];
    let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];
    let mut boltOrg: vec3_t = [0.0; 3];
    let mut tAng: vec3_t = [0.0; 3];
    let mut getAng: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut boltMatrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };

    // PORT-NOTE: Raven derefs `cg.snap` unguarded; with no snapshot yet nobody
    // is "us", so the flag renders.
    let ourClientNum = ctx.world.cg.snap_ref().map_or(-1, |snap| snap.ps.clientNum);

    if cent.currentState.number == ourClientNum && ctx.world.cg.renderingThirdPerson == 0 {
        return;
    }

    if cent.ghoul2.is_null() {
        return;
    }

    let bolt_llumbar = if cent.currentState.eType == entityType_t::ET_NPC as c_int {
        match cent.npcClient.as_ref() {
            Some(ci) => ci.bolt_llumbar,
            None => {
                // §F19: Raven asserts here and then derefs the NULL anyway; a
                // flagless NPC just doesn't get a flag drawn.
                debug_assert!(false, "flag-carrying NPC with no npcClient");
                return;
            }
        }
    } else {
        ctx.world.cgs.clientinfo[cent.currentState.number as usize].bolt_llumbar
    };

    VectorSet(
        &mut tAng,
        cent.turAngles[PITCH],
        cent.turAngles[YAW],
        cent.turAngles[ROLL],
    );

    let engine = ctx.engine;
    let time = ctx.world.cg.time;
    trap::G2API_GetBoltMatrix(
        engine,
        cent.ghoul2,
        0,
        bolt_llumbar,
        &mut boltMatrix,
        &tAng,
        &cent.lerpOrigin,
        time,
        Some(&mut ctx.world.cgs.gameModels[0]),
        &cent.modelScale,
    );
    BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut boltOrg);

    BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::POSITIVE_X as c_int, &mut tAng);
    let t = tAng;
    vectoangles(t, &mut tAng);

    _VectorCopy(cent.lerpAngles, &mut angles);

    boltOrg[2] -= 12.0;
    VectorSet(&mut getAng, 0.0, cent.lerpAngles[1], 0.0);
    AngleVectors(getAng, None, Some(&mut right), None);
    boltOrg[0] += right[0] * 8.0;
    boltOrg[1] += right[1] * 8.0;
    boltOrg[2] += right[2] * 8.0;

    angles[PITCH] = -cent.lerpAngles[PITCH] / 2.0 - 30.0;
    angles[YAW] = tAng[YAW] + 270.0;

    AnglesToAxis(angles, axis.as_mut_ptr());

    let mut ent = refEntity_t::zeroed();
    _VectorMA(boltOrg, 24.0, axis[0], &mut ent.origin);

    angles[ROLL] += 20.0;
    AnglesToAxis(angles, ent.axis.as_mut_ptr());

    ent.hModel = hModel;

    ent.modelScale[0] = 0.5;
    ent.modelScale[1] = 0.5;
    ent.modelScale[2] = 0.5;
    ScaleModelAxis(&mut ent);

    // Raven's RF_FORCE_ENT_ALPHA pass for our own back is commented out
    // (`cg_players.c:4431-4437`): "Not doing this at the moment because sorting
    // totally messes up".

    trap::R_AddRefEntityToScene(engine, &ent);
}

/// Raven `CG_PlayerSprites` — the icon floating over a player's head: the
/// connection glitch beats everything, then a voice-chat balloon, then a plain
/// talk balloon. Mind-tricked entities draw nothing.
/// Source: `oracle/codemp/cgame/cg_players.c:4572-4600`
pub fn CG_PlayerSprites(ctx: &mut CgContext, cent: &centity_t) {
    if let Some(snap) = ctx.world.cg.snap_ref() {
        let ourClientNum = snap.ps.clientNum;
        if CG_IsMindTricked(
            ctx.world,
            cent.currentState.trickedentindex,
            cent.currentState.trickedentindex2,
            cent.currentState.trickedentindex3,
            cent.currentState.trickedentindex4,
            ourClientNum,
        ) {
            return; //this entity is mind-tricking the current client, so don't render it
        }
    }

    if cent.currentState.eFlags & EF_CONNECTION != 0 {
        let shader = ctx.world.cgs.media.connectionShader;
        CG_PlayerFloatSprite(ctx, cent, shader);
        return;
    }

    if cent.vChatTime > ctx.world.cg.time {
        let shader = ctx.world.cgs.media.vchatShader;
        CG_PlayerFloatSprite(ctx, cent, shader);
    } else if cent.currentState.eType != entityType_t::ET_NPC as c_int
        //don't draw talk balloons on NPCs
        && (cent.currentState.eFlags & EF_TALK) != 0
    {
        let shader = ctx.world.cgs.media.balloonShader;
        CG_PlayerFloatSprite(ctx, cent, shader);
    }
}

/// Raven `CG_AddRefEntityWithPowerups` — the one gate every player refEnt goes
/// through: an entity mind-tricking us is simply not submitted.
///
/// `team` is dead in Raven's body (the powerup shells it named are gone).
/// Source: `oracle/codemp/cgame/cg_players.c:5059-5071`
#[allow(unused_variables)]
pub fn CG_AddRefEntityWithPowerups(
    ctx: &mut CgContext,
    ent: &refEntity_t,
    state: &entityState_t,
    team: c_int,
) {
    // PORT-NOTE: Raven derefs `cg.snap` unguarded; with no snapshot there is
    // nobody to be tricked, so the entity renders.
    if let Some(snap) = ctx.world.cg.snap_ref() {
        let ourClientNum = snap.ps.clientNum;
        if CG_IsMindTricked(
            ctx.world,
            state.trickedentindex,
            state.trickedentindex2,
            state.trickedentindex3,
            state.trickedentindex4,
            ourClientNum,
        ) {
            return; //this entity is mind-tricking the current client, so don't render it
        }
    }

    trap::R_AddRefEntityToScene(ctx.engine, ent);
}

/// Raven `CG_PlayerHitFX` — the personal-shield flash on somebody who just took
/// a hit. Ships get nothing, and neither do you in first person.
///
/// Raven: only do the below fx if the cent in question is...uh...me, and it's
/// first person.
/// Source: `oracle/codemp/cgame/cg_players.c:5145-5158`
pub fn CG_PlayerHitFX(ctx: &mut CgContext, cent: &centity_t) {
    if cent.currentState.clientNum != ctx.world.cg.predictedPlayerState.clientNum
        || ctx.world.cg.renderingThirdPerson != 0
    {
        if cent.damageTime > ctx.world.cg.time
            && cent.currentState.NPC_class != class_t::CLASS_VEHICLE as c_int
        {
            CG_DrawPlayerShield(ctx, cent, &cent.lerpOrigin);
        }
    }
}

/// Raven `CG_DoSaberLight` — one dynamic light for a whole saber, averaging the
/// colour and midpoint of every lit blade and sizing the radius off how far
/// apart the tips are.
///
/// Raven: RGB combine all the colors of the sabers you're using into one
/// averaged color!
/// Source: `oracle/codemp/cgame/cg_players.c:5234-5318`
pub fn CG_DoSaberLight(ctx: &mut CgContext, saber: Option<&saberInfo_t>) {
    let mut positions: [vec3_t; MAX_BLADES * 2] = [[0.0; 3]; MAX_BLADES * 2];
    let mut mid: vec3_t = [0.0; 3];
    let mut rgbs: [vec3_t; MAX_BLADES * 2] = [[0.0; 3]; MAX_BLADES * 2];
    let mut rgb: vec3_t = [0.0; 3];
    let mut lengths: [f32; MAX_BLADES * 2] = [0.0; MAX_BLADES * 2];
    let mut totallength: f32 = 0.0;
    // Raven's counter is a float, and the average below divides by it as one
    let mut numpositions: f32 = 0.0;
    let mut diameter: f32 = 0.0;

    let saber = match saber {
        Some(saber) => saber,
        None => return,
    };

    if saber.saberFlags2 & SFL2_NO_DLIGHT != 0 {
        //no dlight!
        return;
    }

    for i in 0..saber.numBlades as usize {
        if saber.blade[i].length >= 0.5 {
            //FIXME: make RGB sabers
            CG_RGBForSaberColor(saber.blade[i].color, &mut rgbs[i]);
            lengths[i] = saber.blade[i].length;
            if saber.blade[i].length * 2.0 > diameter {
                diameter = saber.blade[i].length * 2.0;
            }
            totallength += saber.blade[i].length;
            _VectorMA(
                saber.blade[i].muzzlePoint,
                saber.blade[i].length,
                saber.blade[i].muzzleDir,
                &mut positions[i],
            );
            if numpositions == 0.0 {
                //first blade, store middle of that as midpoint
                _VectorMA(
                    saber.blade[i].muzzlePoint,
                    saber.blade[i].length * 0.5,
                    saber.blade[i].muzzleDir,
                    &mut mid,
                );
                _VectorCopy(rgbs[i], &mut rgb);
            }
            numpositions += 1.0;
        }
    }

    if totallength != 0.0 {
        //actually have something to do
        if numpositions == 1.0 {
            //only 1 blade, midpoint is already set (halfway between the start and end of that blade), rgb is already set, so it diameter
        } else {
            //multiple blades, calc averages
            VectorClear(&mut mid);
            VectorClear(&mut rgb);
            //now go through all the data and get the average RGB and middle position and the radius
            for i in 0..MAX_BLADES * 2 {
                if lengths[i] != 0.0 {
                    let r = rgb;
                    _VectorMA(r, lengths[i], rgbs[i], &mut rgb);
                    let m = mid;
                    _VectorAdd(m, positions[i], &mut mid);
                }
            }

            //get middle rgb
            let r = rgb;
            //get the average, normalized RGB
            _VectorScale(r, 1.0 / totallength, &mut rgb);
            //get mid position
            let m = mid;
            _VectorScale(m, 1.0 / numpositions, &mut mid);
            //find the farthest distance between the blade tips, this will be our diameter
            for i in 0..MAX_BLADES * 2 {
                if lengths[i] != 0.0 {
                    for j in 0..MAX_BLADES * 2 {
                        if lengths[j] != 0.0 {
                            let dist = Distance(positions[i], positions[j]);
                            if dist > diameter {
                                diameter = dist;
                            }
                        }
                    }
                }
            }
        }

        let jitter = ctx.world.bg_state.rng.random() * 8.0;
        trap::R_AddLightToScene(ctx.engine, &mid, diameter + jitter, rgb[0], rgb[1], rgb[2]);
    }
}

/// Raven `CG_DoSaber` — draws one blade: a sprite-based glow ref-ent plus the
/// white-hot line core, both radius-jittered per frame. Blades under half a unit
/// aren't worth submitting.
///
/// Raven: Jeff, I did this because I foolishly wished to have a bright halo as
/// the saber is unleashed. It's not quite what I'd hoped tho. If you have any
/// ideas, go for it! --Pat
/// Source: `oracle/codemp/cgame/cg_players.c:5320-5430`
#[allow(clippy::too_many_arguments)]
pub fn CG_DoSaber(
    ctx: &mut CgContext,
    origin: &vec3_t,
    dir: &vec3_t,
    length: f32,
    lengthMax: f32,
    radius: f32,
    color: saber_colors_t,
    rfx: c_int,
    doLight: bool,
) {
    let mut rfx = rfx;
    let mut mid: vec3_t = [0.0; 3];

    if length < 0.5 {
        // if the thing is so short, just forget even adding me.
        return;
    }

    // Find the midpoint of the saber for lighting purposes
    _VectorMA(*origin, length * 0.5, *dir, &mut mid);

    let media = &ctx.world.cgs.media;
    let (glow, blade) = match color {
        SABER_RED => (media.redSaberGlowShader, media.redSaberCoreShader),
        SABER_ORANGE => (media.orangeSaberGlowShader, media.orangeSaberCoreShader),
        SABER_YELLOW => (media.yellowSaberGlowShader, media.yellowSaberCoreShader),
        SABER_GREEN => (media.greenSaberGlowShader, media.greenSaberCoreShader),
        SABER_BLUE => (media.blueSaberGlowShader, media.blueSaberCoreShader),
        SABER_PURPLE => (media.purpleSaberGlowShader, media.purpleSaberCoreShader),
        _ => (media.blueSaberGlowShader, media.blueSaberCoreShader),
    };

    if doLight {
        // always add a light because sabers cast a nice glow before they slice you in half!!  or something...
        let mut rgb: vec3_t = [1.0, 1.0, 1.0];
        CG_RGBForSaberColor(color, &mut rgb);
        let jitter = ctx.world.bg_state.rng.random() * 3.0;
        trap::R_AddLightToScene(
            ctx.engine,
            &mid,
            (length * 1.4) + jitter,
            rgb[0],
            rgb[1],
            rgb[2],
        );
    }

    let mut saber = refEntity_t::zeroed();

    // Saber glow is it's own ref type because it uses a ton of sprites, otherwise it would eat up too many
    //	refEnts to do each glow blob individually
    saber.saberLength = length;

    let radiusmult: f32 = if length < lengthMax {
        // Note this creates a curve, and length cannot be < 0.5.
        (1.0f64 + (2.0f64 / length as f64)) as f32
    } else {
        1.0
    };

    if ctx.world.cvars.cg_saberTrail.integer == 2
        && ctx.world.cvars.cg_shadows.integer != 2
        && ctx.world.cgs.glconfig.stencilBits >= 4
    {
        //draw the blade as a post-render so it doesn't get in the cap...
        rfx |= RF_FORCEPOST;
    }

    let radiusRange = radius * 0.075;
    let mut radiusStart = radius - radiusRange;

    saber.radius = ((radiusStart as f64 + ctx.world.bg_state.rng.crandom() * radiusRange as f64)
        * radiusmult as f64) as f32;
    //saber.radius = (2.8f + crandom() * 0.2f)*radiusmult;

    _VectorCopy(*origin, &mut saber.origin);
    _VectorCopy(*dir, &mut saber.axis[0]);
    saber.reType = refEntityType_t::RT_SABER_GLOW;
    saber.customShader = glow;
    saber.shaderRGBA = [0xff; 4];
    saber.renderfx = rfx;

    trap::R_AddRefEntityToScene(ctx.engine, &saber);

    // Do the hot core
    _VectorMA(*origin, length, *dir, &mut saber.origin);
    _VectorMA(*origin, -1.0, *dir, &mut saber.oldorigin);

    saber.customShader = blade;
    saber.reType = refEntityType_t::RT_LINE;
    radiusStart = radius / 3.0;
    saber.radius = ((radiusStart as f64 + ctx.world.bg_state.rng.crandom() * radiusRange as f64)
        * radiusmult as f64) as f32;
    //	saber.radius = (1.0 + crandom() * 0.2f)*radiusmult;

    saber.shaderTexCoord[0] = 1.0;
    saber.shaderTexCoord[1] = 1.0;
    saber.shaderRGBA = [0xff; 4];

    trap::R_AddRefEntityToScene(ctx.engine, &saber);
}

/// Raven `CG_AddRandomLightning` — jitters both ends of a lightning bolt before
/// handing it to [`CG_AddLightningBeam`], so a held stream never draws the same
/// arc twice.
/// Source: `oracle/codemp/cgame/cg_players.c:6699-6740`
pub fn CG_AddRandomLightning(ctx: &mut CgContext, start: &vec3_t, end: &vec3_t) {
    let mut inOrg: vec3_t = [0.0; 3];
    let mut outOrg: vec3_t = [0.0; 3];

    _VectorCopy(*start, &mut inOrg);
    _VectorCopy(*end, &mut outOrg);

    if ctx.world.bg_state.rng.rand() & 1 != 0 {
        outOrg[0] += ctx.world.bg_state.rng.Q_irand(0, 24) as f32;
        inOrg[0] += ctx.world.bg_state.rng.Q_irand(0, 8) as f32;
    } else {
        outOrg[0] -= ctx.world.bg_state.rng.Q_irand(0, 24) as f32;
        inOrg[0] -= ctx.world.bg_state.rng.Q_irand(0, 8) as f32;
    }

    if ctx.world.bg_state.rng.rand() & 1 != 0 {
        outOrg[1] += ctx.world.bg_state.rng.Q_irand(0, 24) as f32;
        inOrg[1] += ctx.world.bg_state.rng.Q_irand(0, 8) as f32;
    } else {
        outOrg[1] -= ctx.world.bg_state.rng.Q_irand(0, 24) as f32;
        inOrg[1] -= ctx.world.bg_state.rng.Q_irand(0, 8) as f32;
    }

    if ctx.world.bg_state.rng.rand() & 1 != 0 {
        outOrg[2] += ctx.world.bg_state.rng.Q_irand(0, 50) as f32;
        inOrg[2] += ctx.world.bg_state.rng.Q_irand(0, 40) as f32;
    } else {
        outOrg[2] -= ctx.world.bg_state.rng.Q_irand(0, 64) as f32;
        inOrg[2] -= ctx.world.bg_state.rng.Q_irand(0, 40) as f32;
    }

    CG_AddLightningBeam(ctx, &inOrg, &outOrg);
}

/// Raven `CG_VehicleEffects` — a ship's per-frame effect pass: the hyperspace
/// star streak, the flyby whoosh, engine start, exhaust/trail plumes and the
/// damage smoke off broken surfaces.
///
/// The body past the hyperspace block reads `m_pVehicle->m_pVehicleInfo`
/// (`soundFlyBy`/`soundFlyBy2`, `soundEngineStart`, `type`, `surfDestruction`,
/// `iTrailFX`/`iExhaustFX`/`iTurboFX`/`iInjureFX`/`iDmgFX`) plus `m_iExhaustTag[]`
/// and `m_iLastFXTime` through the pool.
/// Source: `oracle/codemp/cgame/cg_players.c:7981-8305`
pub fn CG_VehicleEffects(ctx: &mut CgContext, cent: &centity_t) {
    if cent.currentState.eType != entityType_t::ET_NPC as c_int
        || cent.currentState.NPC_class != class_t::CLASS_VEHICLE as c_int
        || cent.m_pVehicle.is_none()
    {
        return;
    }

    if cent.currentState.clientNum == ctx.world.cg.predictedPlayerState.m_iVehicleNum //my vehicle
        && (cent.currentState.eFlags2 & EF2_HYPERSPACE) != 0
    //hyperspacing
    {
        //in hyperspace!
        let hyperSpaceTime = ctx.world.cg.predictedVehicleState.hyperSpaceTime;
        let time = ctx.world.cg.time;

        if hyperSpaceTime != 0 && (time - hyperSpaceTime) < HYPERSPACE_TIME {
            let lastEffect = ctx.world.players.cg_lastHyperSpaceEffectTime;
            if lastEffect == 0 || (time - lastEffect) > HYPERSPACE_TIME + 500 {
                //can't be from the last time we were in hyperspace, so play the effect!
                trap::FX_PlayBoltedEffectID(
                    ctx.engine,
                    ctx.world.cgs.effects.mHyperspaceStars,
                    &cent.lerpOrigin,
                    cent.ghoul2,
                    0,
                    cent.currentState.number,
                    0,
                    0,
                    true,
                );
                ctx.world.players.cg_lastHyperSpaceEffectTime = time;
            }
        }
    }

    // presence is guaranteed by the early-out above.
    let vehId = cent.m_pVehicle.expect("CG_VehicleEffects: m_pVehicle");
    let row_idx = vehId.ent_num() as usize;
    let info_ptr = ctx.world.vehicle_pool[row_idx].m_pVehicleInfo;

    let clientNum = cent.currentState.clientNum;
    let time = ctx.world.cg.time;

    //FLYBY sound
    if clientNum != ctx.world.cg.predictedPlayerState.m_iVehicleNum
        && (unsafe { (*info_ptr).soundFlyBy } != 0 || unsafe { (*info_ptr).soundFlyBy2 } != 0)
    {
        //not my vehicle
        let psSpeed = ctx.world.cg.predictedPlayerState.speed;
        if cent.currentState.speed != 0.0 && psSpeed + cent.currentState.speed > 500.0 {
            //he's moving and between the two of us, we're moving fast
            let mut diff: vec3_t = [0.0; 3];
            let psOrigin = ctx.world.cg.predictedPlayerState.origin;
            _VectorSubtract(cent.lerpOrigin, psOrigin, &mut diff);
            if VectorLength(diff) < 2048.0 {
                //close
                let mut myFwd: vec3_t = [0.0; 3];
                let mut theirFwd: vec3_t = [0.0; 3];
                let psViewangles = ctx.world.cg.predictedPlayerState.viewangles;
                AngleVectors(psViewangles, Some(&mut myFwd), None, None);
                _VectorScale(myFwd, psSpeed, &mut myFwd);
                AngleVectors(cent.lerpAngles, Some(&mut theirFwd), None, None);
                _VectorScale(theirFwd, cent.currentState.speed, &mut theirFwd);
                if ctx.world.players.lastFlyBySound[clientNum as usize] + FLYBYSOUNDTIME < time {
                    //okay to do a flyby sound on this vehicle
                    if _DotProduct(myFwd, theirFwd) < 500.0 {
                        let soundFlyBy = unsafe { (*info_ptr).soundFlyBy };
                        let soundFlyBy2 = unsafe { (*info_ptr).soundFlyBy2 };
                        let flyBySound = if soundFlyBy != 0 && soundFlyBy2 != 0 {
                            if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                                soundFlyBy
                            } else {
                                soundFlyBy2
                            }
                        } else if soundFlyBy != 0 {
                            soundFlyBy
                        } else {
                            soundFlyBy2
                        };
                        trap::S_StartSound(
                            ctx.engine,
                            None,
                            clientNum,
                            CHAN_LESS_ATTEN,
                            flyBySound,
                        );
                        ctx.world.players.lastFlyBySound[clientNum as usize] = time;
                    }
                }
            }
        }
    }

    if cent.currentState.speed == 0.0 //was stopped
        && cent.nextState.speed > 0.0 //now moving forward
        && unsafe { (*info_ptr).soundEngineStart } != 0
    {
        //engines rev up for the first time
        let soundEngineStart = unsafe { (*info_ptr).soundEngineStart };
        trap::S_StartSound(
            ctx.engine,
            None,
            clientNum,
            CHAN_LESS_ATTEN,
            soundEngineStart,
        );
    }

    // Animals don't exude any effects...
    if unsafe { (*info_ptr).r#type } != vehicleType_t::VH_ANIMAL {
        let mut didFireTrail = false;
        if unsafe { (*info_ptr).surfDestruction } != 0 && !cent.ghoul2.is_null() {
            //see if anything has been blown off
            let mut surfDmg = false;

            for i in 0..BG_NUM_TOGGLEABLE_SURFACES {
                if bgToggleableSurfaceDebris.get(i).copied().unwrap_or(0) > 1 {
                    //this is decidedly a destroyable surface, let's check its status
                    let surfName = bgToggleableSurfaces
                        .get(i)
                        .and_then(|s| *s)
                        .map(|s| s.to_str().unwrap_or(""))
                        .unwrap_or("");
                    let surfTest =
                        trap::G2API_GetSurfaceRenderStatus(ctx.engine, cent.ghoul2, 0, surfName);

                    if surfTest != -1 && (surfTest & TURN_OFF) != 0 {
                        //it exists, but it's off...
                        surfDmg = true;

                        //create some flames
                        let burning = ctx.world.cgs.effects.mShipDestBurning;
                        CG_CreateSurfaceDebris(ctx, cent, i as c_int, burning, false);
                        didFireTrail = true;
                    }
                }
            }

            if surfDmg {
                //if any surface are damaged, neglect exhaust etc effects (so we don't have exhaust trails coming out of invisible surfaces)
                return;
            }
        }
        if !didFireTrail && (cent.currentState.eFlags & EF_DEAD) != 0 {
            //spiralling out of control anyway
            let burning = ctx.world.cgs.effects.mShipDestBurning;
            CG_CreateSurfaceDebris(ctx, cent, -1, burning, false);
        }

        if ctx.world.vehicle_pool[row_idx].m_iLastFXTime <= time {
            //until we attach it, we need to debounce this
            let mut fwd: vec3_t = [0.0; 3];
            let mut rt: vec3_t = [0.0; 3];
            let mut up: vec3_t = [0.0; 3];
            let mut flat: vec3_t = [0.0; 3];
            let nextFXDelay: f32 = 50.0;
            VectorSet(&mut flat, 0.0, cent.lerpAngles[1], cent.lerpAngles[2]);
            AngleVectors(flat, Some(&mut fwd), Some(&mut rt), Some(&mut up));
            if cent.currentState.speed > 0.0 {
                //FIXME: only do this when accelerator is being pressed! (must have a driver?)
                let mut org: vec3_t = [0.0; 3];
                let doExhaust;
                _VectorMA(cent.lerpOrigin, -16.0, up, &mut org);
                let orgTmp = org;
                _VectorMA(orgTmp, -42.0, fwd, &mut org);
                // Play damage effects.
                //if ( pVehNPC->m_iArmor <= 75 )
                if false {
                    //hurt
                    let blackSmoke = ctx.world.cgs.effects.mBlackSmoke;
                    trap::FX_PlayEffectID(ctx.engine, blackSmoke, &org, &fwd, -1, -1);
                } else if unsafe { (*info_ptr).iTrailFX } != 0 {
                    //okay, do normal trail
                    let iTrailFX = unsafe { (*info_ptr).iTrailFX };
                    trap::FX_PlayEffectID(ctx.engine, iTrailFX, &org, &fwd, -1, -1);
                }
                //=====================================================================
                //EXHAUST FX
                //=====================================================================
                //do exhaust
                if (cent.currentState.eFlags & EF_JETPACK_ACTIVE) != 0 {
                    //cheap way of telling us the vehicle is in "turbo" mode
                    doExhaust = unsafe { (*info_ptr).iTurboFX } != 0;
                } else {
                    doExhaust = unsafe { (*info_ptr).iExhaustFX } != 0;
                }
                if doExhaust && !cent.ghoul2.is_null() {
                    for i in 0..MAX_VEHICLE_EXHAUSTS {
                        // We hit an invalid tag, we quit (they should be created in order so tough luck if not).
                        let exhaustTag = ctx.world.vehicle_pool[row_idx].m_iExhaustTag[i];
                        if exhaustTag == -1 {
                            break;
                        }

                        if (cent.currentState.brokenLimbs & (1 << SHIPSURF_DAMAGE_BACK_HEAVY)) != 0
                        {
                            //engine has taken heavy damage
                            if ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
                                //50% chance of not drawing this engine glow this frame
                                continue;
                            }
                        } else if (cent.currentState.brokenLimbs
                            & (1 << SHIPSURF_DAMAGE_BACK_LIGHT))
                            != 0
                        {
                            //engine has taken light damage
                            if ctx.world.bg_state.rng.Q_irand(0, 4) == 0 {
                                //20% chance of not drawing this engine glow this frame
                                continue;
                            }
                        }

                        let fx = if (cent.currentState.eFlags & EF_JETPACK_ACTIVE) != 0 //cheap way of telling us the vehicle is in "turbo" mode
                            && unsafe { (*info_ptr).iTurboFX } != 0
                        //they have a valid turbo exhaust effect to play
                        {
                            unsafe { (*info_ptr).iTurboFX }
                        } else {
                            //play the normal one
                            unsafe { (*info_ptr).iExhaustFX }
                        };

                        if unsafe { (*info_ptr).r#type } == vehicleType_t::VH_FIGHTER {
                            trap::FX_PlayBoltedEffectID(
                                ctx.engine,
                                fx,
                                &cent.lerpOrigin,
                                cent.ghoul2,
                                exhaustTag,
                                cent.currentState.number,
                                0,
                                0,
                                true,
                            );
                        } else {
                            //fixme: bolt these too
                            let mut boltMatrix = mdxaBone_t {
                                matrix: [[0.0; 4]; 3],
                            };
                            let mut boltOrg: vec3_t = [0.0; 3];

                            trap::G2API_GetBoltMatrix(
                                ctx.engine,
                                cent.ghoul2,
                                0,
                                exhaustTag,
                                &mut boltMatrix,
                                &flat,
                                &cent.lerpOrigin,
                                time,
                                Some(&mut ctx.world.cgs.gameModels[0]),
                                &cent.modelScale,
                            );

                            BG_GiveMeVectorFromMatrix(
                                &boltMatrix,
                                Eorientations::ORIGIN as c_int,
                                &mut boltOrg,
                            );
                            let boltDir = fwd; //fixme?

                            trap::FX_PlayEffectID(ctx.engine, fx, &boltOrg, &boltDir, -1, -1);
                        }
                    }
                }
                //=====================================================================
                //WING TRAIL FX
                //=====================================================================
                //do trail
                //FIXME: not in space!!!
                if unsafe { (*info_ptr).iTrailFX } != 0 && !cent.ghoul2.is_null() {
                    let mut getBoltAngles: vec3_t = cent.lerpAngles;
                    if unsafe { (*info_ptr).r#type } != vehicleType_t::VH_FIGHTER {
                        //only fighters use pitch/roll in refent axis
                        getBoltAngles[PITCH] = 0.0;
                        getBoltAngles[ROLL] = 0.0;
                    }

                    for i in 1..5 {
                        let trailBolt = trap::G2API_AddBolt(
                            ctx.engine,
                            cent.ghoul2,
                            0,
                            &format!("*trail{}", i),
                        );
                        // We hit an invalid tag, we quit (they should be created in order so tough luck if not).
                        if trailBolt == -1 {
                            break;
                        }

                        let mut boltMatrix = mdxaBone_t {
                            matrix: [[0.0; 4]; 3],
                        };
                        let mut boltOrg: vec3_t = [0.0; 3];
                        trap::G2API_GetBoltMatrix(
                            ctx.engine,
                            cent.ghoul2,
                            0,
                            trailBolt,
                            &mut boltMatrix,
                            &getBoltAngles,
                            &cent.lerpOrigin,
                            time,
                            Some(&mut ctx.world.cgs.gameModels[0]),
                            &cent.modelScale,
                        );

                        BG_GiveMeVectorFromMatrix(
                            &boltMatrix,
                            Eorientations::ORIGIN as c_int,
                            &mut boltOrg,
                        );
                        let boltDir = fwd; //fixme?

                        let iTrailFX = unsafe { (*info_ptr).iTrailFX };
                        trap::FX_PlayEffectID(ctx.engine, iTrailFX, &boltOrg, &boltDir, -1, -1);
                    }
                }
            }
            //FIXME armor needs to be sent over network
            if (cent.currentState.eFlags & EF_DEAD) != 0 {
                //just plain dead, use flames
                let up: vec3_t = [0.0, 0.0, 1.0];
                //doh!  no tag
                let boltOrg = cent.lerpOrigin;
                let burning = ctx.world.cgs.effects.mShipDestBurning;
                trap::FX_PlayEffectID(ctx.engine, burning, &boltOrg, &up, -1, -1);
            }
            if cent.currentState.brokenLimbs != 0 {
                if ctx.world.bg_state.rng.Q_irand(0, 5) == 0 {
                    for i in SHIPSURF_FRONT..=SHIPSURF_LEFT {
                        if (cent.currentState.brokenLimbs
                            & (1 << ((i - SHIPSURF_FRONT) + SHIPSURF_DAMAGE_FRONT_HEAVY)))
                            != 0
                        {
                            //heavy damage, do both effects
                            if unsafe { (*info_ptr).iInjureFX } != 0 {
                                let iInjureFX = unsafe { (*info_ptr).iInjureFX };
                                CG_CreateSurfaceSmoke(ctx, cent, i, iInjureFX);
                            }
                            if unsafe { (*info_ptr).iDmgFX } != 0 {
                                let iDmgFX = unsafe { (*info_ptr).iDmgFX };
                                CG_CreateSurfaceSmoke(ctx, cent, i, iDmgFX);
                            }
                        } else if (cent.currentState.brokenLimbs
                            & (1 << ((i - SHIPSURF_FRONT) + SHIPSURF_DAMAGE_FRONT_LIGHT)))
                            != 0
                        {
                            //only light damage
                            if unsafe { (*info_ptr).iInjureFX } != 0 {
                                let iInjureFX = unsafe { (*info_ptr).iInjureFX };
                                CG_CreateSurfaceSmoke(ctx, cent, i, iInjureFX);
                            }
                        }
                    }
                }
            }
            ctx.world.vehicle_pool[row_idx].m_iLastFXTime = time + nextFXDelay as c_int;
        }
    }
}

/// Raven `CG_CustomSound` — resolves a `*`-prefixed custom sound reference
/// against a client's (or NPC's) per-model sound table, falling back to a
/// straight `trap_S_RegisterSound` for anything that isn't a custom
/// reference.
/// Source: `oracle/codemp/cgame/cg_players.c:170-304`
pub fn CG_CustomSound(ctx: &mut CgContext, clientNum: c_int, soundName: &str) -> sfxHandle_t {
    if !soundName.starts_with('*') {
        return trap::S_RegisterSound(ctx.engine, soundName);
    }

    let lSoundName = COM_StripExtension(soundName);

    let clientNum = if clientNum < 0 { 0 } else { clientNum };
    let isNpc = clientNum as usize >= MAX_CLIENTS;

    let ci = if isNpc {
        ctx.world.entities[clientNum as usize].npcClient.as_deref()
    } else {
        Some(&ctx.world.cgs.clientinfo[clientNum as usize])
    };

    let ci = match ci {
        Some(ci) => ci,
        None => return 0,
    };

    let numCSounds = cg_customSoundNames
        .iter()
        .position(Option::is_none)
        .unwrap_or(MAX_CUSTOM_SOUNDS);

    let mut numCComSounds = 0;
    let mut numCExSounds = 0;
    let mut numCJediSounds = 0;

    if isNpc {
        // these are only for npc's
        numCComSounds = cg_customCombatSoundNames
            .iter()
            .position(Option::is_none)
            .unwrap_or(MAX_CUSTOM_COMBAT_SOUNDS);
        numCExSounds = cg_customExtraSoundNames
            .iter()
            .position(Option::is_none)
            .unwrap_or(MAX_CUSTOM_EXTRA_SOUNDS);
        numCJediSounds = cg_customJediSoundNames
            .iter()
            .position(Option::is_none)
            .unwrap_or(MAX_CUSTOM_JEDI_SOUNDS);
    }

    let siege = ctx.world.cgs.gametype >= GT_TEAM || ctx.world.cvars.cg_buildScript.integer != 0;
    // PORT-NOTE: Raven's counting loop scans `bg_customSiegeSoundNames` up to
    // `MAX_CUSTOM_SOUNDS` (40), but the array itself is `MAX_CUSTOM_SIEGE_SOUNDS`
    // (30) long (own comment: "for now these must all be the same" — they
    // aren't) — an OOB C read past the real 30 entries. `.position()` only ever
    // walks the array's real length, so it can't reproduce that read; every
    // table has its `None` sentinel well inside 30, so the count is identical
    // either way.
    let mut numCSiegeSounds = 0;
    if siege {
        // siege only
        numCSiegeSounds = bg_customSiegeSoundNames
            .iter()
            .position(Option::is_none)
            .unwrap_or(bg_customSiegeSoundNames.len());
    }

    let duel = ctx.world.cgs.gametype == GT_DUEL
        || ctx.world.cgs.gametype == GT_POWERDUEL
        || ctx.world.cvars.cg_buildScript.integer != 0;
    let mut numCDuelSounds = 0;
    if duel {
        // Duel only
        numCDuelSounds = cg_customDuelSoundNames
            .iter()
            .position(Option::is_none)
            .unwrap_or(MAX_CUSTOM_DUEL_SOUNDS);
    }

    for i in 0..MAX_CUSTOM_SOUNDS {
        if i < numCSounds && cg_customSoundNames[i] == Some(lSoundName.as_str()) {
            return ci.sounds[i];
        } else if siege
            && i < numCSiegeSounds
            && bg_customSiegeSoundNames[i].and_then(|s| s.to_str().ok())
                == Some(lSoundName.as_str())
        {
            // siege only
            return ci.siegeSounds[i];
        } else if duel
            && i < numCDuelSounds
            && cg_customDuelSoundNames[i] == Some(lSoundName.as_str())
        {
            // siege only
            return ci.duelSounds[i];
        } else if isNpc
            && i < numCComSounds
            && cg_customCombatSoundNames[i] == Some(lSoundName.as_str())
        {
            // npc only
            return ci.combatSounds[i];
        } else if isNpc
            && i < numCExSounds
            && cg_customExtraSoundNames[i] == Some(lSoundName.as_str())
        {
            // npc only
            return ci.extraSounds[i];
        } else if isNpc
            && i < numCJediSounds
            && cg_customJediSoundNames[i] == Some(lSoundName.as_str())
        {
            // npc only
            return ci.jediSounds[i];
        }
    }

    //CG_Error( "Unknown custom sound: %s", lSoundName );
    //
    // PORT-NOTE: FINAL_BUILD is never defined for this port (release parity),
    // so Raven's `#ifndef FINAL_BUILD` diagnostic always fires.
    Com_Printf(ctx, &format!("Unknown custom sound: {}", lSoundName));
    0
}

/// Raven `CG_ParseSurfsFile` — loads `models/players/<model>/model_<skin>.surf`
/// and folds its `surfOff`/`surfOn` directives into two comma-joined lists.
///
/// Out-params become the return value: `Some((surfOff, surfOn))` for Raven's
/// `qtrue`, `None` for `qfalse`. Raven's
/// `memset( (char *)surfOff, 0, sizeof(surfOff) )` only clears
/// `sizeof(char*)` bytes of the caller's buffer (a sizeof-on-a-pointer-param
/// bug) — but that's enough to zero byte 0, the only byte either
/// `if (surfOff[0])` check ever reads, so the accumulators behave exactly as
/// if freshly cleared regardless of what a caller passed in. The port starts
/// both empty and hands them back instead of writing through caller buffers.
/// Source: `oracle/codemp/cgame/cg_players.c:314-409`
pub fn CG_ParseSurfsFile(
    ctx: &mut CgContext,
    modelName: &str,
    skinName: &str,
) -> Option<(String, String)> {
    // this is a multi-part skin, said skins do not support .surf files
    if skinName.contains('|') {
        return None;
    }

    // Load and parse .surf file
    let sfilename = format!("models/players/{}/model_{}.surf", modelName, skinName);

    // load the file
    let mut f: fileHandle_t = 0;
    let len = trap::FS_FOpenFile(ctx.engine, &sfilename, &mut f, FS_READ);
    if len <= 0 {
        // no file
        return None;
    }
    if len >= 20000 - 1 {
        Com_Printf(ctx, &format!("File {} too long\n", sfilename));
        return None;
    }

    let mut text = vec![0u8; len as usize];
    trap::FS_Read(ctx.engine, &mut text, f);
    trap::FS_FCloseFile(ctx.engine, f);

    // parse the text
    let mut text_p: Option<&[u8]> = Some(&text[..]);

    let mut surfOff = String::new();
    let mut surfOn = String::new();
    let mut qs = QSharedScratch::zeroed();

    // read information for surfOff and surfOn
    loop {
        let (token, rest) = COM_ParseExt(&mut qs, text_p, true);
        text_p = rest;
        if token.is_empty() {
            break;
        }

        // surfOff
        if Q_stricmp(&token, "surfOff") == 0 {
            // Raven's `COM_ParseString` guard tests the (always non-NULL)
            // token pointer, never the parsed value, so its `continue`-on-error
            // arm is dead (see `COM_ParseString`'s own doc); dropped here too.
            let (value, rest2) = COM_ParseString(&mut qs, text_p);
            text_p = rest2;

            if !surfOff.is_empty() {
                strcat_string(&mut surfOff, MAX_SURF_LIST_SIZE, ",");
                strcat_string(&mut surfOff, MAX_SURF_LIST_SIZE, &value);
            } else {
                surfOff = strncpyz_string(value.as_bytes(), MAX_SURF_LIST_SIZE);
            }
            continue;
        }

        // surfOn
        if Q_stricmp(&token, "surfOn") == 0 {
            let (value, rest2) = COM_ParseString(&mut qs, text_p);
            text_p = rest2;

            if !surfOn.is_empty() {
                strcat_string(&mut surfOn, MAX_SURF_LIST_SIZE, ",");
                strcat_string(&mut surfOn, MAX_SURF_LIST_SIZE, &value);
            } else {
                surfOn = strncpyz_string(value.as_bytes(), MAX_SURF_LIST_SIZE);
            }
            continue;
        }
    }

    Some((surfOff, surfOn))
}

/// Raven `CG_RunLerpFrame` — advances one lerp-frame's animation state
/// (torso or legs, chosen by `torsoOnly`) by one client render frame.
///
/// The port derives `lf` from `cent.pe.torso`/`cent.pe.legs` by `torsoOnly`
/// rather than threading Raven's `lerpFrame_t *lf` param — every oracle call
/// site passes exactly one of those two fields (`cg_players.c:3308,3326`),
/// matching the already-ported `CG_SetLerpFrameAnimation`'s own shape.
/// Source: `oracle/codemp/cgame/cg_players.c:3185-3235`
pub fn CG_RunLerpFrame(
    ctx: &mut CgContext,
    cent: &mut centity_t,
    ci: &mut clientInfo_t,
    flipState: bool,
    newAnimation: c_int,
    speedScale: f32,
    torsoOnly: bool,
) {
    // debugging tool to get no animations
    if ctx.world.cvars.cg_animSpeed.integer == 0 {
        let lf = if torsoOnly {
            &mut cent.pe.torso
        } else {
            &mut cent.pe.legs
        };
        lf.oldFrame = 0;
        lf.frame = 0;
        lf.backlerp = 0.0;
        return;
    }

    let ghoul2 = cent.ghoul2;
    let forceFrame = cent.currentState.forceFrame;
    let csBrokenLimbs = cent.currentState.brokenLimbs;
    let time = ctx.world.cg.time;

    // see if the animation sequence is switching
    if forceFrame != 0 {
        let lf = if torsoOnly {
            &mut cent.pe.torso
        } else {
            &mut cent.pe.legs
        };

        if lf.lastForcedFrame != forceFrame {
            let flags = BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND;
            let animSpeed = 1.0f32;
            trap::G2API_SetBoneAnim(
                ctx.engine,
                ghoul2,
                0,
                "lower_lumbar",
                forceFrame,
                forceFrame + 1,
                flags,
                animSpeed,
                time,
                -1.0,
                150,
            );
            trap::G2API_SetBoneAnim(
                ctx.engine,
                ghoul2,
                0,
                "model_root",
                forceFrame,
                forceFrame + 1,
                flags,
                animSpeed,
                time,
                -1.0,
                150,
            );
            trap::G2API_SetBoneAnim(
                ctx.engine,
                ghoul2,
                0,
                "Motion",
                forceFrame,
                forceFrame + 1,
                flags,
                animSpeed,
                time,
                -1.0,
                150,
            );
        }

        lf.lastForcedFrame = forceFrame;
        lf.animationNumber = 0;
    } else {
        let needsNewAnim = {
            let lf = if torsoOnly {
                &mut cent.pe.torso
            } else {
                &mut cent.pe.legs
            };
            lf.lastForcedFrame = -1;

            newAnimation != lf.animationNumber
                || csBrokenLimbs != ci.brokenLimbs
                || flipState != (lf.lastFlip != qfalse)
                || lf.animation.is_null()
                || CG_FirstAnimFrame(lf, torsoOnly, speedScale)
        };

        if needsNewAnim {
            CG_SetLerpFrameAnimation(
                ctx,
                cent,
                ci,
                newAnimation,
                speedScale,
                torsoOnly,
                flipState,
            );
        }
    }

    let lf = if torsoOnly {
        &mut cent.pe.torso
    } else {
        &mut cent.pe.legs
    };
    lf.lastFlip = if flipState { qtrue } else { qfalse };

    if lf.frameTime > time + 200 {
        lf.frameTime = time;
    }

    if lf.oldFrameTime > time {
        lf.oldFrameTime = time;
    }

    // calculate current lerp value
    if lf.frameTime == lf.oldFrameTime {
        lf.backlerp = 0.0;
    } else {
        lf.backlerp =
            1.0 - (time - lf.oldFrameTime) as f32 / (lf.frameTime - lf.oldFrameTime) as f32;
    }
}

/// Raven `CG_ClearLerpFrame` — snaps one lerp-frame (torso or legs, chosen by
/// `torsoOnly`) straight onto `animationNumber`'s first frame, no blend.
///
/// `lf` is derived from `cent.pe.torso`/`cent.pe.legs` the same way
/// `CG_RunLerpFrame` derives it — see that fn's doc.
/// Source: `oracle/codemp/cgame/cg_players.c:3243-3255`
pub fn CG_ClearLerpFrame(
    ctx: &mut CgContext,
    cent: &mut centity_t,
    ci: &mut clientInfo_t,
    animationNumber: c_int,
    torsoOnly: bool,
) {
    let time = ctx.world.cg.time;
    {
        let lf = if torsoOnly {
            &mut cent.pe.torso
        } else {
            &mut cent.pe.legs
        };
        lf.frameTime = time;
        lf.oldFrameTime = time;
    }

    CG_SetLerpFrameAnimation(ctx, cent, ci, animationNumber, 1.0, torsoOnly, false);

    let lf = if torsoOnly {
        &mut cent.pe.torso
    } else {
        &mut cent.pe.legs
    };

    if lf.animation.is_null() {
        // §F19: `CG_SetLerpFrameAnimation`'s ported early-return (unparsed
        // skeleton / out-of-range `localAnimIndex`) can leave `animation`
        // unset; Raven trusted `bgAllAnims[cent->localAnimIndex]` was always
        // valid and dereferenced it unconditionally. Leave the frame exactly
        // as `CG_SetLerpFrameAnimation` left it rather than deref null.
        return;
    }

    // SAFETY: `animation` was just resolved by `CG_SetLerpFrameAnimation` (or
    // is a still-valid earlier resolution); Raven dereferences it unchecked
    // and so do we.
    let (frameLerp, firstFrame, numFrames) = unsafe {
        let a = &*lf.animation;
        (a.frameLerp, a.firstFrame, a.numFrames)
    };

    if frameLerp < 0 {
        //Plays backwards
        let frame = firstFrame as c_int + numFrames as c_int;
        lf.oldFrame = frame;
        lf.frame = frame;
    } else {
        lf.oldFrame = firstFrame as c_int;
        lf.frame = firstFrame as c_int;
    }
}

/// Raven `CG_G2ServerBoneAngles` — replays the server's four packed
/// `boneIndex`/`boneAngles` overrides from `entityState_t` onto the client's
/// ghoul2 instance.
/// Source: `oracle/codemp/cgame/cg_players.c:3914-3962`
pub fn CG_G2ServerBoneAngles(ctx: &mut CgContext, cent: &centity_t) {
    let mut bone = cent.currentState.boneIndex1;
    let mut boneAngles: vec3_t = [0.0; 3];
    _VectorCopy(cent.currentState.boneAngles1, &mut boneAngles);

    let boneOrient = cent.currentState.boneOrient;
    let ghoul2 = cent.ghoul2;
    let engine = ctx.engine;
    let time = ctx.world.cg.time;

    for i in 0..4 {
        // cycle through the 4 bone index values on the entstate
        if bone != 0 {
            // if it's non-0 then it could have something in it.
            let boneName = CG_ConfigString(ctx, CS_G2BONES + bone);

            if !boneName.is_empty() {
                // got the bone, now set the angles from the corresponding
                // entitystate boneangles value.
                let flags = BONE_ANGLES_POSTMULT;

                // get the orientation out of our bit field
                let forward = boneOrient & 7; // 3 bits from bit 0
                let right = (boneOrient >> 3) & 7; // 3 bits from bit 3
                let up = (boneOrient >> 6) & 7; // 3 bits from bit 6

                trap::G2API_SetBoneAngles(
                    engine,
                    ghoul2,
                    0,
                    &boneName,
                    &boneAngles,
                    flags,
                    up,
                    right,
                    forward,
                    Some(&mut ctx.world.cgs.gameModels[0]),
                    100,
                    time,
                );
            }
        }

        match i {
            0 => {
                bone = cent.currentState.boneIndex2;
                _VectorCopy(cent.currentState.boneAngles2, &mut boneAngles);
            }
            1 => {
                bone = cent.currentState.boneIndex3;
                _VectorCopy(cent.currentState.boneAngles3, &mut boneAngles);
            }
            2 => {
                bone = cent.currentState.boneIndex4;
                _VectorCopy(cent.currentState.boneAngles4, &mut boneAngles);
            }
            _ => {}
        }
    }
}

/// Raven `CG_PlayerPowerups` — the powerup-driven dlights (quad/redflag/
/// blueflag/neutralflag) plus the flag models themselves.
///
/// `torso` is unused in Raven's own body (kept for signature parity with the
/// caller). `ci` resolves the owning `clientInfo_t` purely to mirror Raven's
/// `assert(ci)` NPC-sanity check — nothing downstream reads it, matching the
/// oracle body exactly.
/// Source: `oracle/codemp/cgame/cg_players.c:4449-4495`
pub fn CG_PlayerPowerups(ctx: &mut CgContext, cent: &centity_t, _torso: &refEntity_t) {
    let powerups = cent.currentState.powerups;
    if powerups == 0 {
        return;
    }

    // quad gives a dlight
    if powerups & (1 << PW_QUAD) != 0 {
        let intensity = 200 + (ctx.world.bg_state.rng.rand() & 31);
        trap::R_AddLightToScene(
            ctx.engine,
            &cent.lerpOrigin,
            intensity as f32,
            0.2,
            0.2,
            1.0,
        );
    }

    let ci = if cent.currentState.eType == entityType_t::ET_NPC as c_int {
        cent.npcClient.as_deref()
    } else {
        Some(&ctx.world.cgs.clientinfo[cent.currentState.clientNum as usize])
    };
    debug_assert!(ci.is_some(), "CG_PlayerPowerups: NPC with no npcClient");

    // redflag
    if powerups & (1 << PW_REDFLAG) != 0 {
        let hModel = ctx.world.cgs.media.redFlagModel;
        CG_PlayerFlag(ctx, cent, hModel);
        let intensity = 200 + (ctx.world.bg_state.rng.rand() & 31);
        trap::R_AddLightToScene(
            ctx.engine,
            &cent.lerpOrigin,
            intensity as f32,
            1.0,
            0.2,
            0.2,
        );
    }

    // blueflag
    if powerups & (1 << PW_BLUEFLAG) != 0 {
        let hModel = ctx.world.cgs.media.blueFlagModel;
        CG_PlayerFlag(ctx, cent, hModel);
        let intensity = 200 + (ctx.world.bg_state.rng.rand() & 31);
        trap::R_AddLightToScene(
            ctx.engine,
            &cent.lerpOrigin,
            intensity as f32,
            0.2,
            0.2,
            1.0,
        );
    }

    // neutralflag
    if powerups & (1 << PW_NEUTRALFLAG) != 0 {
        let intensity = 200 + (ctx.world.bg_state.rng.rand() & 31);
        trap::R_AddLightToScene(
            ctx.engine,
            &cent.lerpOrigin,
            intensity as f32,
            1.0,
            1.0,
            1.0,
        );
    }

    // haste leaves smoke trails
    //
    // if ( powerups & ( 1 << PW_HASTE ) ) {
    //     CG_HasteTrail( cent );
    // }
}

/// The zero fill Raven gets from its `memset( &cent->pe.legs, 0, … )`.
/// `lerpFrame_t` carries a raw `*mut animation_t`, so it has no derived
/// `Default`; the fill is spelled once here the way `zeroed_client_info` is.
fn zeroed_lerp_frame() -> lerpFrame_t {
    // SAFETY: every field is a POD scalar and `animation` is a raw pointer,
    // whose all-zero value is the NULL Raven's `memset` leaves there.
    unsafe { core::mem::zeroed() }
}

/// Raven `CG_RegisterClientModelname` — loads a player model, its skin, its
/// animation set and every bolt the rest of the module indexes by name. A model
/// that fails any of those falls back to kyle/default and is retried; `false`
/// means even the retry could not be set up.
///
/// Raven's `retryModel:` label is the head of the loop below and its two
/// `goto`s are `continue`s.
/// Source: `oracle/codemp/cgame/cg_players.c:421-694`
pub fn CG_RegisterClientModelname(
    ctx: &mut CgContext,
    ci: &mut clientInfo_t,
    modelName: &str,
    skinName: &str,
    teamName: &str,
    clientNum: c_int,
) -> bool {
    let mut modelName = modelName.to_string();
    let mut skinName = skinName.to_string();
    let tempVec: vec3_t = [0.0, 0.0, 0.0];
    let mut badModel = false;

    loop {
        // retryModel:
        if badModel {
            if !modelName.is_empty() {
                Com_Printf(
                    ctx,
                    &format!(
                        "WARNING: Attempted to load an unsupported multiplayer model {}! (bad or missing bone, or missing animation sequence)\n",
                        modelName
                    ),
                );
            }

            modelName = "kyle".to_string();
            skinName = "default".to_string();

            badModel = false;
        }

        // First things first.  If this is a ghoul2 model, then let's make sure we demolish this first.
        if !ci.ghoul2Model.is_null() && trap::G2_HaveWeGhoul2Models(ctx.engine, ci.ghoul2Model) {
            trap::G2API_CleanGhoul2Models(ctx.engine, &mut ci.ghoul2Model as *mut *mut c_void);
        }

        let modelName_c = cstr(&modelName);
        let skinName_c = cstr(&skinName);
        if BG_IsValidCharacterModel(modelName_c.as_ptr(), skinName_c.as_ptr()) == qfalse {
            modelName = "kyle".to_string();
            skinName = "default".to_string();
        }

        if ctx.world.cgs.gametype >= GT_TEAM
            && ctx.world.cgs.jediVmerc == qfalse
            && ctx.world.cgs.gametype != GT_SIEGE
        {
            //We won't force colors for siege.
            let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
            let ciModelName = ci.modelName.as_ptr();
            let ciSkinName = ci.skinName.as_mut_ptr();
            let colors = ci.colorOverride.as_mut_ptr();
            BG_ValidateSkinForTeam(
                ciModelName,
                ciSkinName,
                ci.team,
                colors,
                &ctx.world.bg_state,
                &traps,
            );
            skinName = buf_to_string(&ci.skinName.map(|c| c as u8));
        } else {
            ci.colorOverride[0] = 0.0;
            ci.colorOverride[1] = 0.0;
            ci.colorOverride[2] = 0.0;
        }

        let useSkinName = if skinName.contains('|') {
            //three part skin
            format!("models/players/{}/|{}", modelName, skinName)
        } else {
            format!("models/players/{}/model_{}.skin", modelName, skinName)
        };

        let checkSkin = trap::R_RegisterSkin(ctx.engine, &useSkinName);

        if checkSkin != 0 {
            ci.torsoSkin = checkSkin;
        } else {
            //fallback to the default skin
            //
            // PORT-NOTE: Raven's format string has one `%s` but is handed two
            // arguments, so the skin name is dropped on the floor. Kept.
            ci.torsoSkin = trap::R_RegisterSkin(
                ctx.engine,
                &format!("models/players/{}/model_default.skin", modelName),
            );
        }

        let afilename = format!("models/players/{}/model.glm", modelName);
        let handle = trap::G2API_InitGhoul2Model(
            ctx.engine,
            &mut ci.ghoul2Model as *mut *mut c_void,
            &afilename,
            0,
            ci.torsoSkin,
            0,
            0,
            0,
        );

        if handle < 0 {
            return false;
        }

        // The model is now loaded.

        trap::G2API_SetSkin(ctx.engine, ci.ghoul2Model, 0, ci.torsoSkin, ci.torsoSkin);

        let GLAName = trap::G2API_GetGLAName(ctx.engine, ci.ghoul2Model, 0, MAX_QPATH);
        if !GLAName.is_empty() {
            // Raven's rockettrooper-in-siege alternative is commented out in the
            // oracle, so only the humanoid path is allowed.
            if !GLAName.contains("players/_humanoid/") {
                //Bad!
                badModel = true;
                continue;
            }
        }

        if ctx.world.bg_state.BGPAFtextLoaded == qfalse {
            if GLAName.is_empty() {
                badModel = true;
                continue;
            }

            // Now afilename holds just the path to the animation.cfg
            let mut afilename = GLAName.clone();
            match afilename.rfind('/') {
                Some(slash) => {
                    afilename.truncate(slash);
                    afilename.push_str("/animation.cfg");
                }
                None => {
                    // Didn't find any slashes, this is a raw filename right in base (whish isn't a good thing)
                    return false;
                }
            }

            //rww - All player models must use humanoid, no matter what.
            if Q_stricmp(&afilename, "models/players/_humanoid/animation.cfg") != 0 {
                Com_Printf(ctx, "Model does not use supported animation config.\n");
                return false;
            }

            let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
            let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
            let humanoidAnims = ctx.world.bg_state.bgHumanoidAnimations.as_mut_ptr();
            let humanoidCfg = cstr("models/players/_humanoid/animation.cfg");
            if BG_ParseAnimationFile(
                &mut ctx.world.bg_state,
                &traps,
                &mut callbacks,
                humanoidCfg.as_ptr(),
                humanoidAnims,
                qtrue,
            ) == -1
            {
                Com_Printf(
                    ctx,
                    "Failed to load animation file models/players/_humanoid/animation.cfg\n",
                );
                return false;
            }

            //get the sounds for the humanoid anims
            {
                let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
                let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
                BG_ParseAnimationEvtFile(
                    &mut ctx.world.bg_state,
                    &traps,
                    &mut callbacks,
                    "models/players/_humanoid/",
                    0,
                    -1,
                );
            }
            //For the time being, we're going to have all real players use the generic humanoid soundset and that's it.
            //Only npc's will use model-specific soundsets.
        } else if ctx.world.bg_state.bgAllEvents[0].eventsParsed == qfalse {
            //make sure the player anim sounds are loaded even if the anims already are
            let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
            let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
            BG_ParseAnimationEvtFile(
                &mut ctx.world.bg_state,
                &traps,
                &mut callbacks,
                "models/players/_humanoid/",
                0,
                -1,
            );
        }

        if let Some((surfOff, surfOn)) = CG_ParseSurfsFile(ctx, &modelName, &skinName) {
            //turn on/off any surfs
            let mut qs = QSharedScratch::zeroed();

            //Now turn on/off any surfaces
            if !surfOff.is_empty() {
                let mut p: Option<&[u8]> = Some(surfOff.as_bytes());
                loop {
                    let (token, rest) = COM_ParseExt(&mut qs, p, true);
                    p = rest;
                    if token.is_empty() {
                        //reached end of list
                        break;
                    }
                    //turn off this surf
                    trap::G2API_SetSurfaceOnOff(
                        ctx.engine,
                        ci.ghoul2Model,
                        &token,
                        0x00000002, /*G2SURFACEFLAG_OFF*/
                    );
                }
            }
            if !surfOn.is_empty() {
                let mut p: Option<&[u8]> = Some(surfOn.as_bytes());
                loop {
                    let (token, rest) = COM_ParseExt(&mut qs, p, true);
                    p = rest;
                    if token.is_empty() {
                        //reached end of list
                        break;
                    }
                    //turn on this surf
                    trap::G2API_SetSurfaceOnOff(ctx.engine, ci.ghoul2Model, &token, 0);
                }
            }
        }

        let g2 = ci.ghoul2Model;
        let time = ctx.world.cg.time;

        ci.bolt_rhand = trap::G2API_AddBolt(ctx.engine, g2, 0, "*r_hand");

        if !trap::G2API_SetBoneAnim(
            ctx.engine,
            g2,
            0,
            "model_root",
            0,
            12,
            BONE_ANIM_OVERRIDE_LOOP,
            1.0,
            time,
            -1.0,
            -1,
        ) {
            badModel = true;
        }

        if !trap::G2API_SetBoneAngles(
            ctx.engine,
            g2,
            0,
            "upper_lumbar",
            &tempVec,
            BONE_ANGLES_POSTMULT,
            Eorientations::POSITIVE_X as c_int,
            Eorientations::NEGATIVE_Y as c_int,
            Eorientations::NEGATIVE_Z as c_int,
            None,
            0,
            time,
        ) {
            badModel = true;
        }

        if !trap::G2API_SetBoneAngles(
            ctx.engine,
            g2,
            0,
            "cranium",
            &tempVec,
            BONE_ANGLES_POSTMULT,
            Eorientations::POSITIVE_Z as c_int,
            Eorientations::NEGATIVE_Y as c_int,
            Eorientations::POSITIVE_X as c_int,
            None,
            0,
            time,
        ) {
            badModel = true;
        }

        ci.bolt_lhand = trap::G2API_AddBolt(ctx.engine, g2, 0, "*l_hand");

        //rhand must always be first bolt. lhand always second. Whichever you want the
        //jetpack bolted to must always be third.
        trap::G2API_AddBolt(ctx.engine, g2, 0, "*chestg");

        //claw bolts
        trap::G2API_AddBolt(ctx.engine, g2, 0, "*r_hand_cap_r_arm");
        trap::G2API_AddBolt(ctx.engine, g2, 0, "*l_hand_cap_l_arm");

        ci.bolt_head = trap::G2API_AddBolt(ctx.engine, g2, 0, "*head_top");
        if ci.bolt_head == -1 {
            ci.bolt_head = trap::G2API_AddBolt(ctx.engine, g2, 0, "ceyebrow");
        }

        ci.bolt_motion = trap::G2API_AddBolt(ctx.engine, g2, 0, "Motion");

        //We need a lower lumbar bolt for footsteps
        ci.bolt_llumbar = trap::G2API_AddBolt(ctx.engine, g2, 0, "lower_lumbar");

        if ci.bolt_rhand == -1
            || ci.bolt_lhand == -1
            || ci.bolt_head == -1
            || ci.bolt_motion == -1
            || ci.bolt_llumbar == -1
        {
            badModel = true;
        }

        if badModel {
            continue;
        }

        if Q_stricmp(&modelName, "boba_fett") == 0 {
            //special case, turn off the jetpack surfs
            trap::G2API_SetSurfaceOnOff(ctx.engine, g2, "torso_rjet", TURN_OFF);
            trap::G2API_SetSurfaceOnOff(ctx.engine, g2, "torso_cjet", TURN_OFF);
            trap::G2API_SetSurfaceOnOff(ctx.engine, g2, "torso_ljet", TURN_OFF);
        }

        if clientNum != -1 {
            // Raven's clean-and-duplicate of the client's own instance is
            // commented out in the oracle; only the weapon-instance reset runs.
            ctx.world.entity_mut(clientNum as usize).ghoul2weapon = null_mut();
        }

        Q_strncpyz(&mut ci.teamName, teamName, MAX_TEAMNAME);

        // Model icon for drawing the portrait on screen
        ci.modelIcon = trap::R_RegisterShaderNoMip(
            ctx.engine,
            &format!("models/players/{}/icon_{}", modelName, skinName),
        );
        if ci.modelIcon == 0 {
            let sk = skinName.as_bytes();
            let mut i = 0usize;
            let mut iconName: Vec<u8> = b"icon_".to_vec();

            while i < sk.len() && sk[i] != b'|' && iconName.len() < 1024 {
                iconName.push(sk[i]);
                i += 1;
            }

            if i < sk.len() && sk[i] == b'|' {
                //looks like it actually may be a custom model skin, let's try getting the icon...
                let iconName = buf_to_string(&iconName);
                ci.modelIcon = trap::R_RegisterShaderNoMip(
                    ctx.engine,
                    &format!("models/players/{}/{}", modelName, iconName),
                );
            }
        }

        return true;
    }
}

/// Raven `CG_PlayerAnimation` — runs an entity's legs and torso lerp frames for
/// this render frame and hands back the frame pair each one landed on. Rage and
/// speed scale the legs; only rage scales the torso; vehicles have no torso anim
/// at all, so the three torso out-params are left untouched for them (Raven's
/// caller reads uninitialised stack in that case — §F19, kept by writing
/// nothing).
///
/// ESCALATION (shape): Raven picks `ci` itself from `cgs.clientinfo[clientNum]`;
/// the port takes it as a parameter because wave 2's `CG_RunLerpFrame` wants
/// `cent` and `ci` as `&mut` borrows *disjoint from* `ctx.world`, so neither can
/// be re-derived from `ctx` here. The `ET_NPC` half of Raven's pick still
/// happens below, off `cent.npcClient`.
/// Source: `oracle/codemp/cgame/cg_players.c:3267-3332`
#[allow(clippy::too_many_arguments)]
pub fn CG_PlayerAnimation(
    ctx: &mut CgContext,
    cent: &mut centity_t,
    ci: &mut clientInfo_t,
    legsOld: &mut c_int,
    legs: &mut c_int,
    legsBackLerp: &mut f32,
    torsoOld: &mut c_int,
    torso: &mut c_int,
    torsoBackLerp: &mut f32,
) {
    if ctx.world.cvars.cg_noPlayerAnims.integer != 0 {
        *legsOld = 0;
        *legs = 0;
        *torsoOld = 0;
        *torso = 0;
        return;
    }

    let legsAnim = cent.currentState.legsAnim;
    let forcePowersActive = cent.currentState.forcePowersActive;

    let mut speedScale;
    if PM_RunningAnim(legsAnim) == qfalse && PM_WalkingAnim(legsAnim) == qfalse {
        //if legs are not in a walking/running anim then just animate at standard speed
        speedScale = 1.0f32;
    } else if forcePowersActive & (1 << FP_RAGE) != 0 {
        speedScale = 1.3f32;
    } else if forcePowersActive & (1 << FP_SPEED) != 0 {
        speedScale = 1.7f32;
    } else {
        speedScale = 1.0f32;
    }

    // an NPC's clientinfo lives on the entity; taking the box out leaves `cent`
    // usable beside it and it goes straight back at the end.
    let mut npcCi = if cent.currentState.eType == entityType_t::ET_NPC as c_int {
        cent.npcClient.take()
    } else {
        None
    };
    debug_assert!(
        cent.currentState.eType != entityType_t::ET_NPC as c_int || npcCi.is_some(),
        "CG_PlayerAnimation: NPC with no npcClient"
    );
    let ci: &mut clientInfo_t = match npcCi.as_deref_mut() {
        Some(c) => c,
        None => &mut *ci,
    };

    let legsFlip = cent.currentState.legsFlip != qfalse;
    CG_RunLerpFrame(ctx, cent, ci, legsFlip, legsAnim, speedScale, false);

    if cent.currentState.forcePowersActive & (1 << FP_RAGE) == 0 {
        //don't affect torso anim speed unless raged
        speedScale = 1.0;
    } else {
        speedScale = 1.7;
    }

    *legsOld = cent.pe.legs.oldFrame;
    *legs = cent.pe.legs.frame;
    *legsBackLerp = cent.pe.legs.backlerp;

    // If this is not a vehicle, you may lerm the frame (since vehicles never have a torso anim). -AReis
    if cent.currentState.NPC_class != class_t::CLASS_VEHICLE as c_int {
        let torsoFlip = cent.currentState.torsoFlip != qfalse;
        let torsoAnim = cent.currentState.torsoAnim;
        CG_RunLerpFrame(ctx, cent, ci, torsoFlip, torsoAnim, speedScale, true);

        *torsoOld = cent.pe.torso.oldFrame;
        *torso = cent.pe.torso.frame;
        *torsoBackLerp = cent.pe.torso.backlerp;
    }

    if let Some(b) = npcCi {
        cent.npcClient = Some(b);
    }
}

/// Raven `CG_G2PlayerAngles` — builds an entity's leg axis for this frame:
/// ragdoll angles when it is limp, the full bg torso/legs/head solve for
/// humanoids, and a plain yaw-only axis for everything else.
///
/// ESCALATION (shape): `cent` is a parameter for the same reason
/// [`CG_PlayerAnimation`]'s is — wave 2's `CG_RagDoll` wants `cent` as a `&mut`
/// borrow disjoint from `ctx.world`. The `ci` pick happens inside, like Raven's:
/// the entity's own `npcClient` for NPCs, the live `clientinfo` row otherwise —
/// and it happens after the ragdoll block so `CG_RagDoll` sees the real row.
///
/// The walker/fighter/speeder arms read `m_pVehicle->m_pVehicleInfo->type`
/// through the pool (resolved once as `cent_veh_type`).
/// Source: `oracle/codemp/cgame/cg_players.c:4192-4331`
pub fn CG_G2PlayerAngles(
    ctx: &mut CgContext,
    cent: &mut centity_t,
    legs: &mut [vec3_t; 3],
    legsAngles: &mut vec3_t,
) {
    // Raven reads `cent->m_pVehicle->m_pVehicleInfo->type` in the vehicle arms
    // below; resolve it once through the pool up front (the referent can't
    // change while we're in here).
    let cent_veh_type: Option<vehicleType_t> = cent.m_pVehicle.map(|id| unsafe {
        (*ctx.world.vehicle_pool[id.ent_num() as usize].m_pVehicleInfo).r#type
    });

    //rww - now do ragdoll stuff
    if (cent.currentState.eFlags & EF_DEAD) != 0 || (cent.currentState.eFlags & EF_RAG) != 0 {
        let mut forcedAngles: vec3_t = [0.0; 3];

        VectorClear(&mut forcedAngles);
        forcedAngles[YAW] = cent.lerpAngles[YAW];

        if CG_RagDoll(ctx, cent, &forcedAngles) {
            //if we managed to go into the rag state, give our ent axis the forced angles and return.
            AnglesToAxis(forcedAngles, legs.as_mut_ptr());
            _VectorCopy(forcedAngles, legsAngles);
            return;
        }
    } else if cent.isRagging != qfalse {
        cent.isRagging = qfalse;
        //calling with null parms resets to no ragdoll.
        trap::G2API_SetRagDoll(ctx.engine, cent.ghoul2, None);
    }

    //rww - Quite possibly the most arguments for a function ever.
    if cent.localAnimIndex <= 1 {
        //don't do these things on non-humanoids

        // Raven picks ci right at the top of the fn: npcClient for NPCs, the
        // clientinfo row (by currentState.number) for players. We take it out
        // here instead - after the ragdoll block, which never touches ci - so
        // it can be borrowed beside ctx; it goes straight back at the arm end.
        // Source: oracle/codemp/cgame/cg_players.c:4217-4225
        let mut npcCi = if cent.currentState.eType == entityType_t::ET_NPC as c_int {
            cent.npcClient.take()
        } else {
            None
        };
        debug_assert!(
            cent.currentState.eType != entityType_t::ET_NPC as c_int || npcCi.is_some(),
            "CG_G2PlayerAngles: NPC with no npcClient"
        );
        let ciNumber = cent.currentState.number as usize;
        // bitwise copy-in, original left in place - a zeroed-swap is visible to
        // every ctx-reading helper that reads clientinfo[ciNumber] below (C6b
        // referee catch). The copy writes back at the tail if it is Some.
        // SAFETY: clientInfo_t is #[repr(C)] plain data; the copy is written back whole.
        let mut ci_slot = if npcCi.is_some() {
            None
        } else {
            Some(unsafe { core::ptr::read(&ctx.world.cgs.clientinfo[ciNumber]) })
        };
        let ci: &mut clientInfo_t = match npcCi.as_deref_mut() {
            Some(c) => c,
            None => ci_slot.as_mut().unwrap(),
        };

        let mut lookAngles: vec3_t = [0.0; 3];

        if cent.currentState.hasLookTarget != qfalse {
            let target = ctx
                .world
                .entity(cent.currentState.lookTarget as usize)
                .lerpOrigin;
            _VectorSubtract(target, cent.lerpOrigin, &mut lookAngles);
            let dir = lookAngles;
            vectoangles(dir, &mut lookAngles);
            ci.lookTime = ctx.world.cg.time + 1000;
        } else {
            _VectorCopy(cent.lerpAngles, &mut lookAngles);
        }
        lookAngles[PITCH] = 0.0;

        let emplaced: *mut entityState_t = if cent.currentState.otherEntityNum2 != 0 {
            &mut ctx
                .world
                .entity_mut(cent.currentState.otherEntityNum2 as usize)
                .currentState
        } else {
            null_mut()
        };

        let ghoul2 = cent.ghoul2;
        let lerpOrigin = cent.lerpOrigin;
        let lerpAngles = cent.lerpAngles;
        let modelScale = cent.modelScale;
        let time = ctx.world.cg.time;
        let frametime = ctx.world.cg.frametime;

        let centState: *mut entityState_t = &mut cent.currentState;
        let tYawing: *mut qboolean = &mut cent.pe.torso.yawing;
        let tPitching: *mut qboolean = &mut cent.pe.torso.pitching;
        let lYawing: *mut qboolean = &mut cent.pe.legs.yawing;
        let tYawAngle: *mut f32 = &mut cent.pe.torso.yawAngle;
        let tPitchAngle: *mut f32 = &mut cent.pe.torso.pitchAngle;
        let lYawAngle: *mut f32 = &mut cent.pe.legs.yawAngle;
        let corrTime: *mut c_int = &mut ci.corrTime;
        let superSmoothTime: *mut c_int = &mut ci.superSmoothTime;
        let ciLegs = ci.legsAnim;
        let ciTorso = ci.torsoAnim;
        let lookTime = ci.lookTime;
        let boltMotion = ci.bolt_motion;
        let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());

        BG_G2PlayerAngles(
            ghoul2,
            boltMotion,
            centState,
            time,
            lerpOrigin,
            lerpAngles,
            legs.as_mut_ptr(),
            legsAngles,
            tYawing,
            tPitching,
            lYawing,
            tYawAngle,
            tPitchAngle,
            lYawAngle,
            frametime,
            &mut cent.turAngles,
            modelScale,
            ciLegs,
            ciTorso,
            corrTime,
            lookAngles,
            &mut ci.lastHeadAngles,
            lookTime,
            emplaced,
            superSmoothTime,
            &traps,
        );

        let heldByClient = cent.currentState.heldByClient;
        if heldByClient != 0 && heldByClient <= MAX_CLIENTS_I32 {
            //then put our arm in this client's hand
            //is index+1 because index 0 is valid.
            let heldByIndex = (heldByClient - 1) as usize;

            // Raven's `other` is an array element, so its NULL test is dead.
            let otherGhoul2 = ctx.world.entity(heldByIndex).ghoul2;
            if !otherGhoul2.is_null() && ci.bolt_lhand != 0 {
                let mut boltMatrix = mdxaBone_t {
                    matrix: [[0.0; 4]; 3],
                };
                let mut boltOrg: vec3_t = [0.0; 3];

                let otherTurAngles = ctx.world.entity(heldByIndex).turAngles;
                let otherLerpOrigin = ctx.world.entity(heldByIndex).lerpOrigin;
                let otherModelScale = ctx.world.entity(heldByIndex).modelScale;

                trap::G2API_GetBoltMatrix(
                    ctx.engine,
                    otherGhoul2,
                    0,
                    ci.bolt_lhand,
                    &mut boltMatrix,
                    &otherTurAngles,
                    &otherLerpOrigin,
                    time,
                    Some(&mut ctx.world.cgs.gameModels[0]),
                    &otherModelScale,
                );
                BG_GiveMeVectorFromMatrix(
                    &boltMatrix,
                    Eorientations::ORIGIN as c_int,
                    &mut boltOrg,
                );

                let centState: *mut entityState_t = &mut cent.currentState;
                let ikStatus: *mut qboolean = &mut cent.ikStatus;
                let torsoAnim = cent.currentState.torsoAnim;
                BG_IK_MoveArm(
                    cent.ghoul2,
                    ci.bolt_lhand,
                    time,
                    centState,
                    torsoAnim, /*BOTH_DEAD1*/
                    boltOrg,
                    ikStatus,
                    cent.lerpOrigin,
                    cent.lerpAngles,
                    cent.modelScale,
                    500,
                    qfalse,
                    &ctx.world.bg_state,
                    &traps,
                );
            }
        } else if cent.ikStatus != qfalse {
            //make sure we aren't IKing if we don't have anyone to hold onto us.
            let centState: *mut entityState_t = &mut cent.currentState;
            let ikStatus: *mut qboolean = &mut cent.ikStatus;
            let torsoAnim = cent.currentState.torsoAnim;
            BG_IK_MoveArm(
                cent.ghoul2,
                ci.bolt_lhand,
                time,
                centState,
                torsoAnim, /*BOTH_DEAD1*/
                vec3_origin,
                ikStatus,
                cent.lerpOrigin,
                cent.lerpAngles,
                cent.modelScale,
                500,
                qtrue,
                &ctx.world.bg_state,
                &traps,
            );
        }

        if let Some(b) = npcCi {
            cent.npcClient = Some(b);
        }
        if let Some(s) = ci_slot {
            ctx.world.cgs.clientinfo[ciNumber] = s;
        }
    }
    // a walker (AT-ST): zero the legs pitch, then drive the torso through
    // `BG_G2ATSTAngles` with a yaw/roll-free look vector.
    // Source: oracle/codemp/cgame/cg_players.c:4280-4292
    else if cent.m_pVehicle.is_some() && cent_veh_type == Some(vehicleType_t::VH_WALKER) {
        let mut lookAngles: vec3_t = [0.0; 3];

        _VectorCopy(cent.lerpAngles, legsAngles);
        legsAngles[PITCH] = 0.0;
        AnglesToAxis(*legsAngles, legs.as_mut_ptr());

        _VectorCopy(cent.lerpAngles, &mut lookAngles);
        lookAngles[YAW] = 0.0;
        lookAngles[ROLL] = 0.0;

        let time = ctx.world.cg.time;
        let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
        BG_G2ATSTAngles(cent.ghoul2, time, lookAngles, &traps);
    } else if cent.currentState.eType == entityType_t::ET_NPC as c_int
        && cent.currentState.NPC_class == class_t::CLASS_VEHICLE as c_int
        && cent.m_pVehicle.is_some()
        && cent_veh_type == Some(vehicleType_t::VH_FIGHTER)
    {
        //fighters actually want to take pitch and roll into account for the axial angles
        _VectorCopy(cent.lerpAngles, legsAngles);
        AnglesToAxis(*legsAngles, legs.as_mut_ptr());
    } else if cent.currentState.eType == entityType_t::ET_NPC as c_int
        && cent.currentState.m_iVehicleNum != 0
        && cent.currentState.NPC_class != class_t::CLASS_VEHICLE as c_int
    {
        //an NPC bolted to a vehicle should use the full angles
        _VectorCopy(cent.lerpAngles, legsAngles);
        AnglesToAxis(*legsAngles, legs.as_mut_ptr());
    } else {
        let mut nhAngles: vec3_t = [0.0; 3];

        if cent.currentState.eType == entityType_t::ET_NPC as c_int
            && cent.currentState.NPC_class == class_t::CLASS_VEHICLE as c_int
            && cent.m_pVehicle.is_some()
            && cent_veh_type == Some(vehicleType_t::VH_SPEEDER)
        {
            //yeah, a hack, sorry.
            VectorSet(
                &mut nhAngles,
                0.0,
                cent.lerpAngles[YAW],
                cent.lerpAngles[ROLL],
            );
        } else {
            VectorSet(&mut nhAngles, 0.0, cent.lerpAngles[YAW], 0.0);
        }
        AnglesToAxis(nhAngles, legs.as_mut_ptr());
    }

    //See if we have any bone angles sent from the server
    CG_G2ServerBoneAngles(ctx, cent);
}

/// Raven `CG_ForcePushBlur` — the blur behind a force push/pull. With
/// render-to-texture off (or no pusher at all) it is two counter-moving sprites;
/// with it on, a distortion shell that expands and fades over
/// [`REFRACT_EFFECT_DURATION`].
///
/// `cent` is `Option` because Raven's first test is `!cent` — the sprite arm is
/// exactly the no-entity arm.
/// Source: `oracle/codemp/cgame/cg_players.c:4798-4943`
pub fn CG_ForcePushBlur(ctx: &mut CgContext, org: &vec3_t, centNum: Option<usize>) {
    if centNum.is_none() || ctx.world.cvars.cg_renderToTextureFX.integer == 0 {
        let time = ctx.world.cg.time;
        let viewaxis1 = ctx.world.cg.refdef.viewaxis[1];
        // Raven registers the shader once per sprite. The demo referee
        // byte-diffs the trap stream, so we keep both calls.
        let pushShader = trap::R_RegisterShader(ctx.engine, "gfx/effects/forcePush");

        let handle = CG_AllocLocalEntity(ctx.world);
        {
            let ex = ctx
                .world
                .cg_localEntities
                .get_mut(handle)
                .expect("CG_ForcePushBlur: fresh slot");
            ex.leType = leType_t::LE_PUFF;
            ex.refEntity.reType = refEntityType_t::RT_SPRITE;
            ex.radius = 2.0;
            ex.startTime = time;
            ex.endTime = ex.startTime + 120;
            _VectorCopy(*org, &mut ex.pos.trBase);
            ex.pos.trTime = time;
            ex.pos.trType = trType_t::TR_LINEAR;
            _VectorScale(viewaxis1, 55.0, &mut ex.pos.trDelta);

            ex.color[0] = 24.0;
            ex.color[1] = 32.0;
            ex.color[2] = 40.0;
            ex.refEntity.customShader = pushShader;
        }

        let pushShader = trap::R_RegisterShader(ctx.engine, "gfx/effects/forcePush");

        let handle = CG_AllocLocalEntity(ctx.world);
        let ex = ctx
            .world
            .cg_localEntities
            .get_mut(handle)
            .expect("CG_ForcePushBlur: fresh slot");
        ex.leType = leType_t::LE_PUFF;
        ex.refEntity.reType = refEntityType_t::RT_SPRITE;
        ex.refEntity.rotation = 180.0;
        ex.radius = 2.0;
        ex.startTime = time;
        ex.endTime = ex.startTime + 120;
        _VectorCopy(*org, &mut ex.pos.trBase);
        ex.pos.trTime = time;
        ex.pos.trType = trType_t::TR_LINEAR;
        _VectorScale(viewaxis1, -55.0, &mut ex.pos.trDelta);

        ex.color[0] = 24.0;
        ex.color[1] = 32.0;
        ex.color[2] = 40.0;
        ex.refEntity.customShader = pushShader;
    } else {
        //superkewl "refraction" (well sort of) effect -rww
        let centNum = centNum.expect("CG_ForcePushBlur: refraction arm needs a pusher");
        let time = ctx.world.cg.time;

        if ctx.world.entity(centNum).bodyFadeTime == 0 {
            //the duration for the expansion and fade
            ctx.world.entity_mut(centNum).bodyFadeTime = time + REFRACT_EFFECT_DURATION;
        }

        //closer tDif is to 0, the closer we are to
        //being "done"
        let bodyFadeTime = ctx.world.entity(centNum).bodyFadeTime;
        let tDif = bodyFadeTime - time;

        if (REFRACT_EFFECT_DURATION - tDif) < 200 {
            //stop following the hand after a little and stay in a fixed spot
            //save the initial spot of the effect
            _VectorCopy(*org, &mut ctx.world.entity_mut(centNum).pushEffectOrigin);
        }

        //scale from 1.0f to 0.1f then hold at 0.1 for the rest of the duration
        let mut scale = if ctx.world.entity(centNum).currentState.powerups & (1 << PW_PULL) != 0 {
            (REFRACT_EFFECT_DURATION - tDif) as f32 * 0.003
        } else {
            tDif as f32 * 0.003
        };

        if scale > 1.0 {
            scale = 1.0;
        } else if scale < 0.2 {
            scale = 0.2;
        }

        //start alpha at 244, fade to 10
        let mut alpha = tDif as f32 * 0.488;

        if alpha > 244.0 {
            alpha = 244.0;
        } else if alpha < 10.0 {
            alpha = 10.0;
        }

        let mut ent = refEntity_t::zeroed();
        let mut ang: vec3_t = [0.0; 3];
        ent.shaderTime = (bodyFadeTime - REFRACT_EFFECT_DURATION) as f32 / 1000.0;

        _VectorCopy(ctx.world.entity(centNum).pushEffectOrigin, &mut ent.origin);

        let vieworg = ctx.world.cg.refdef.vieworg;
        let origin = ent.origin;
        _VectorSubtract(origin, vieworg, &mut ent.axis[0]);
        let vLen = VectorLength(ent.axis[0]);
        if vLen <= 0.1 {
            // Entity is right on vieworg.  quit.
            return;
        }

        vectoangles(ent.axis[0], &mut ang);
        ang[ROLL] += 180.0;
        AnglesToAxis(ang, ent.axis.as_mut_ptr());

        //radius must be a power of 2, and is the actual captured texture size
        if vLen < 128.0 {
            ent.radius = 256.0;
        } else if vLen < 256.0 {
            ent.radius = 128.0;
        } else if vLen < 512.0 {
            ent.radius = 64.0;
        } else {
            ent.radius = 32.0;
        }

        let axis0 = ent.axis[0];
        _VectorScale(axis0, scale, &mut ent.axis[0]);
        let axis1 = ent.axis[1];
        _VectorScale(axis1, scale, &mut ent.axis[1]);
        let axis2 = ent.axis[2];
        _VectorScale(axis2, scale, &mut ent.axis[2]);

        ent.hModel = ctx.world.cgs.media.halfShieldModel;
        ent.customShader = ctx.world.cgs.media.refractionShader; //cgs.media.cloakedShader;
        ent.nonNormalizedAxes = qtrue;

        //make it partially transparent so it blends with the background
        ent.renderfx = RF_DISTORTION | RF_FORCE_ENT_ALPHA;
        ent.shaderRGBA[0] = 255.0f32 as i32 as u8;
        ent.shaderRGBA[1] = 255.0f32 as i32 as u8;
        ent.shaderRGBA[2] = 255.0f32 as i32 as u8;
        ent.shaderRGBA[3] = alpha as i32 as u8;

        trap::R_AddRefEntityToScene(ctx.engine, &ent);
    }
}

/// Raven `CG_ForceGripEffect` — the two red sprites that pulse at a gripped
/// player's throat. The second one deliberately ignores the wave value Raven
/// computed for the first (its colour block is commented out in the oracle).
/// Source: `oracle/codemp/cgame/cg_players.c:5000-5048`
pub fn CG_ForceGripEffect(ctx: &mut CgContext, org: &vec3_t) {
    let time = ctx.world.cg.time;
    // `sin` is a double call in C, so the whole product is a double before it
    // narrows back into the float `wv`.
    let wv = ((time as f32 * 0.004f32) as f64).sin() * 0.08f32 as f64 + 0.1f32 as f64;
    let wv = wv as f32;

    let viewaxis1 = ctx.world.cg.refdef.viewaxis[1];
    let pushShader = trap::R_RegisterShader(ctx.engine, "gfx/effects/forcePush");
    let glowShader = ctx.world.cgs.media.redSaberGlowShader;

    let handle = CG_AllocLocalEntity(ctx.world);
    {
        let ex = ctx
            .world
            .cg_localEntities
            .get_mut(handle)
            .expect("CG_ForceGripEffect: fresh slot");
        ex.leType = leType_t::LE_PUFF;
        ex.refEntity.reType = refEntityType_t::RT_SPRITE;
        ex.radius = 2.0;
        ex.startTime = time;
        ex.endTime = ex.startTime + 120;
        _VectorCopy(*org, &mut ex.pos.trBase);
        ex.pos.trTime = time;
        ex.pos.trType = trType_t::TR_LINEAR;
        _VectorScale(viewaxis1, 55.0, &mut ex.pos.trDelta);

        ex.color[0] = 200.0 + wv * 255.0;
        if ex.color[0] > 255.0 {
            ex.color[0] = 255.0;
        }
        ex.color[1] = 0.0;
        ex.color[2] = 0.0;
        ex.refEntity.customShader = pushShader;
    }

    let handle = CG_AllocLocalEntity(ctx.world);
    let ex = ctx
        .world
        .cg_localEntities
        .get_mut(handle)
        .expect("CG_ForceGripEffect: fresh slot");
    ex.leType = leType_t::LE_PUFF;
    ex.refEntity.reType = refEntityType_t::RT_SPRITE;
    ex.refEntity.rotation = 180.0;
    ex.radius = 2.0;
    ex.startTime = time;
    ex.endTime = ex.startTime + 120;
    _VectorCopy(*org, &mut ex.pos.trBase);
    ex.pos.trTime = time;
    ex.pos.trType = trType_t::TR_LINEAR;
    _VectorScale(viewaxis1, -55.0, &mut ex.pos.trDelta);

    // Raven's wave-tinted red for this one is commented out in the oracle.
    ex.color[0] = 255.0;
    ex.color[1] = 255.0;
    ex.color[2] = 255.0;
    ex.refEntity.customShader = glowShader; //trap_R_RegisterShader( "gfx/effects/forcePush" );
}

/// Raven `CG_CreateSaberMarks` — projects the saber's swing quad onto the world
/// and lays a burn plus a glow pass down for every fragment it comes back with.
/// `cg_saberDynamicMarks` routes the same fragments through the FX system
/// instead of the persistent mark pool.
/// Source: `oracle/codemp/cgame/cg_players.c:5462-5630`
pub fn CG_CreateSaberMarks(ctx: &mut CgContext, start: &vec3_t, end: &vec3_t, normal: &vec3_t) {
    let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];
    let mut originalPoints: [vec3_t; 4] = [[0.0; 3]; 4];
    let mut mid: vec3_t = [0.0; 3];
    let mut markPoints: [vec3_t; MAX_MARK_POINTS] = [[0.0; 3]; MAX_MARK_POINTS];
    let mut projection: vec3_t = [0.0; 3];
    let mut verts: [polyVert_t; MAX_VERTS_ON_POLY] = [polyVert_t {
        xyz: [0.0; 3],
        st: [0.0; 2],
        modulate: [0; 4],
    }; MAX_VERTS_ON_POLY];
    let mut markFragments: [markFragment_t; MAX_MARK_FRAGMENTS] = [markFragment_t {
        firstPoint: 0,
        numPoints: 0,
    }; MAX_MARK_FRAGMENTS];

    let radius: f32 = 0.65;

    if ctx.world.cvars.cg_addMarks.integer == 0 {
        return;
    }

    _VectorSubtract(*end, *start, &mut axis[1]);
    VectorNormalize(&mut axis[1]);

    // create the texture axis
    _VectorCopy(*normal, &mut axis[0]);
    let (a1, a0) = (axis[1], axis[0]);
    CrossProduct(a1, a0, &mut axis[2]);

    // create the full polygon that we'll project
    for i in 0..3 {
        // stretch a bit more in the direction that we are traveling in...  debateable as to whether this makes things better or worse
        originalPoints[0][i] = start[i] - radius * axis[1][i] - radius * axis[2][i];
        originalPoints[1][i] = end[i] + radius * axis[1][i] - radius * axis[2][i];
        originalPoints[2][i] = end[i] + radius * axis[1][i] + radius * axis[2][i];
        originalPoints[3][i] = start[i] - radius * axis[1][i] + radius * axis[2][i];
    }

    _VectorScale(*normal, -1.0, &mut projection);

    // get the fragments
    let numFragments = trap::CM_MarkFragments(
        ctx.engine,
        &originalPoints,
        &projection,
        &mut markPoints,
        &mut markFragments,
    );

    for i in 0..numFragments as usize {
        let mf = &mut markFragments[i];

        // we have an upper limit on the complexity of polygons that we store persistantly
        if mf.numPoints > MAX_VERTS_ON_POLY as c_int {
            mf.numPoints = MAX_VERTS_ON_POLY as c_int;
        }
        let numPoints = mf.numPoints;
        let firstPoint = mf.firstPoint;

        for j in 0..numPoints as usize {
            let mut delta: vec3_t = [0.0; 3];

            // Set up our texture coords, this may need some work
            _VectorCopy(markPoints[firstPoint as usize + j], &mut verts[j].xyz);
            _VectorAdd(*end, *start, &mut mid);
            let m = mid;
            _VectorScale(m, 0.5, &mut mid);
            _VectorSubtract(verts[j].xyz, mid, &mut delta);

            let r0 = ctx.world.bg_state.rng.random();
            let r1 = ctx.world.bg_state.rng.random();
            verts[j].st[0] =
                (0.5f64 + (_DotProduct(delta, axis[1]) * (0.05 + r0 * 0.03)) as f64) as f32;
            verts[j].st[1] =
                (0.5f64 + (_DotProduct(delta, axis[2]) * (0.15 + r1 * 0.05)) as f64) as f32;
        }

        if ctx.world.cvars.cg_saberDynamicMarks.integer != 0 {
            let mut apArgs = addpolyArgStruct_t {
                p: [[0.0; 3]; 4],
                ev: [[0.0; 2]; 4],
                numVerts: 0,
                vel: [0.0; 3],
                accel: [0.0; 3],
                alpha1: 0.0,
                alpha2: 0.0,
                alphaParm: 0.0,
                rgb1: [0.0; 3],
                rgb2: [0.0; 3],
                rgbParm: 0.0,
                rotationDelta: [0.0; 3],
                bounce: 0.0,
                motionDelay: 0,
                killTime: 0,
                shader: 0,
                flags: 0,
            };
            let mut x: vec3_t = [0.0; 3];

            for i in 0..4 {
                for i_2 in 0..3 {
                    apArgs.p[i][i_2] = verts[i].xyz[i_2];
                }
            }

            for i in 0..4 {
                for i_2 in 0..2 {
                    apArgs.ev[i][i_2] = verts[i].st[i_2];
                }
            }

            //When using addpoly, having a situation like this tends to cause bad results.
            //(I assume it doesn't like trying to draw a polygon over two planes and extends
            //the vertex out to some odd value)
            _VectorSubtract(apArgs.p[0], apArgs.p[3], &mut x);
            if VectorLength(x) > 3.0 {
                return;
            }

            apArgs.numVerts = numPoints;
            _VectorCopy(vec3_origin, &mut apArgs.vel);
            _VectorCopy(vec3_origin, &mut apArgs.accel);

            apArgs.alpha1 = 1.0;
            apArgs.alpha2 = 0.0;
            apArgs.alphaParm = 255.0;

            VectorSet(&mut apArgs.rgb1, 0.0, 0.0, 0.0);
            VectorSet(&mut apArgs.rgb2, 0.0, 0.0, 0.0);

            apArgs.rgbParm = 0.0;

            apArgs.bounce = 0.0;
            apArgs.motionDelay = 0;
            apArgs.killTime = ctx.world.cvars.cg_saberDynamicMarkTime.integer;
            apArgs.shader = ctx.world.cgs.media.rivetMarkShader;
            apArgs.flags = 0x08000000 | 0x00000004;

            trap::FX_AddPoly(ctx.engine, &mut apArgs);

            apArgs.shader = ctx.world.cgs.media.mSaberDamageGlow;
            apArgs.rgb1[0] = 215.0 + ctx.world.bg_state.rng.random() * 40.0;
            apArgs.rgb1[1] = 96.0 + ctx.world.bg_state.rng.random() * 32.0;
            apArgs.alphaParm = ctx.world.bg_state.rng.random() * 15.0;
            apArgs.rgb1[2] = apArgs.alphaParm;

            apArgs.rgb1[0] /= 255.0;
            apArgs.rgb1[1] /= 255.0;
            apArgs.rgb1[2] /= 255.0;
            let rgb1 = apArgs.rgb1;
            _VectorCopy(rgb1, &mut apArgs.rgb2);

            apArgs.killTime = 100;

            trap::FX_AddPoly(ctx.engine, &mut apArgs);
        } else {
            // save it persistantly, do burn first
            let time = ctx.world.cg.time;
            let rivetMarkShader = ctx.world.cgs.media.rivetMarkShader;
            let saberDamageGlow = ctx.world.cgs.media.mSaberDamageGlow;

            let handle = CG_AllocMark(ctx.world);
            {
                let mark = ctx
                    .world
                    .cg_markPolys
                    .get_mut(handle)
                    .expect("CG_CreateSaberMarks: fresh slot");
                mark.time = time;
                mark.alphaFade = qtrue;
                mark.markShader = rivetMarkShader;
                mark.poly.numVerts = numPoints;
                mark.color[0] = 255.0;
                mark.color[1] = 255.0;
                mark.color[2] = 255.0;
                mark.color[3] = 255.0;
                mark.verts[..numPoints as usize].copy_from_slice(&verts[..numPoints as usize]);
            }

            // And now do a glow pass
            // by moving the start time back, we can hack it to fade out way before the burn does
            let c0 = 215.0 + ctx.world.bg_state.rng.random() * 40.0;
            let c1 = 96.0 + ctx.world.bg_state.rng.random() * 32.0;
            let c2 = ctx.world.bg_state.rng.random() * 15.0;
            let handle = CG_AllocMark(ctx.world);
            let mark = ctx
                .world
                .cg_markPolys
                .get_mut(handle)
                .expect("CG_CreateSaberMarks: fresh slot");
            mark.time = time - 8500;
            mark.alphaFade = qfalse;
            mark.markShader = saberDamageGlow;
            mark.poly.numVerts = numPoints;
            mark.color[0] = c0;
            mark.color[1] = c1;
            mark.color[2] = c2;
            mark.color[3] = c2;
            mark.verts[..numPoints as usize].copy_from_slice(&verts[..numPoints as usize]);
        }
    }
}

/// Raven `CG_G2SaberEffects` — sweeps the blade segment both ways and, when it
/// lands on a client's ghoul2 skeleton, throws blood sparks and a saber-hit
/// sound at the collision point.
/// Source: `oracle/codemp/cgame/cg_players.c:5691-5734`
pub fn CG_G2SaberEffects(ctx: &mut CgContext, start: &vec3_t, end: &vec3_t, ownerNum: usize) {
    let mut backWards = false;
    let mut doneWithTraces = false;

    let ownerEnt = ctx.world.entity(ownerNum).currentState.number;

    while !doneWithTraces {
        let mut trace = trace_t::zeroed();
        let mut startTr: vec3_t = [0.0; 3];
        let mut endTr: vec3_t = [0.0; 3];

        if !backWards {
            _VectorCopy(*start, &mut startTr);
            _VectorCopy(*end, &mut endTr);
        } else {
            _VectorCopy(*end, &mut startTr);
            _VectorCopy(*start, &mut endTr);
        }

        CG_Trace(
            ctx,
            &mut trace,
            &startTr,
            None,
            None,
            &endTr,
            ownerEnt,
            MASK_PLAYERSOLID,
        );

        if (trace.entityNum as c_int) < MAX_CLIENTS_I32 {
            //hit a client..
            CG_G2TraceCollide(ctx, &mut trace, None, None, &startTr, &endTr);

            if trace.entityNum as c_int != ENTITYNUM_NONE {
                //it succeeded with the ghoul2 trace
                let bloodSparks = ctx.world.cgs.effects.mSaberBloodSparks;
                trap::FX_PlayEffectID(
                    ctx.engine,
                    bloodSparks,
                    &trace.endpos,
                    &trace.plane.normal,
                    -1,
                    -1,
                );
                let which = ctx.world.bg_state.rng.Q_irand(1, 3);
                let sfx = trap::S_RegisterSound(
                    ctx.engine,
                    &format!("sound/weapons/saber/saberhit{}.wav", which),
                );
                trap::S_StartSound(
                    ctx.engine,
                    Some(&trace.endpos),
                    trace.entityNum as c_int,
                    CHAN_AUTO,
                    sfx,
                );
            }
        }

        if !backWards {
            backWards = true;
        } else {
            doneWithTraces = true;
        }
    }
}

/// Raven `CG_SaberCompWork` — the clientside half of a server-reported saber
/// hit: re-traces the blade, and if it lands on the entity the server named,
/// lays a ghoul2 mark on it and plays the flesh or cut effect.
///
/// Raven's backwards pass is switched off by its own `doneWithTraces = qtrue`
/// at the bottom of the loop (the flip block is commented out — "sometimes it
/// just makes too many effects"), so this is the single forward trace.
///
/// The mark is skipped for `VH_FIGHTER` vehicles ("they have crazy full axial
/// angles"), read through the pool.
/// Source: `oracle/codemp/cgame/cg_players.c:5798-5984`
pub fn CG_SaberCompWork(
    ctx: &mut CgContext,
    start: &vec3_t,
    end: &vec3_t,
    ownerNum: usize,
    saberNum: usize,
    bladeNum: c_int,
) {
    let mut trace = trace_t::zeroed();
    let mut doEffect = false;

    let time = ctx.world.cg.time;
    let serverSaberHitTime = ctx.world.entity(ownerNum).serverSaberHitTime;

    if (time - serverSaberHitTime) > CG_MAX_SABER_COMP_TIME {
        return;
    }

    if time == serverSaberHitTime {
        //don't want to do it the same frame as the server hit, to avoid burst effect concentrations every x ms.
        return;
    }

    let mut startTr: vec3_t = [0.0; 3];
    let mut endTr: vec3_t = [0.0; 3];
    _VectorCopy(*start, &mut startTr);
    _VectorCopy(*end, &mut endTr);

    let ownerEnt = ctx.world.entity(ownerNum).currentState.number;
    CG_Trace(
        ctx,
        &mut trace,
        &startTr,
        None,
        None,
        &endTr,
        ownerEnt,
        MASK_PLAYERSOLID,
    );

    if trace.entityNum as c_int != ctx.world.entity(ownerNum).serverSaberHitIndex {
        return;
    }

    //this is the guy the server says we last hit, so continue.
    if !ctx.world.entity(trace.entityNum as usize).ghoul2.is_null() {
        //If it has a g2 instance, do the proper ghoul2 checks
        CG_G2TraceCollide(ctx, &mut trace, None, None, &startTr, &endTr);

        if trace.entityNum as c_int != ENTITYNUM_NONE {
            //it succeeded with the ghoul2 trace
            doEffect = true;

            if ctx.world.cvars.cg_ghoul2Marks.integer != 0 {
                let trEntNum = trace.entityNum as usize;
                let trGhoul2 = ctx.world.entity(trEntNum).ghoul2;

                let trVehType: Option<vehicleType_t> =
                    ctx.world.entity(trEntNum).m_pVehicle.map(|id| unsafe {
                        (*ctx.world.vehicle_pool[id.ent_num() as usize].m_pVehicleInfo).r#type
                    });
                //don't do on fighters cause they have crazy full axial angles
                let isVehicleFighter = ctx.world.entity(trEntNum).currentState.eType
                    == entityType_t::ET_NPC as c_int
                    && ctx.world.entity(trEntNum).currentState.NPC_class
                        == class_t::CLASS_VEHICLE as c_int
                    && trVehType == Some(vehicleType_t::VH_FIGHTER);

                if !trGhoul2.is_null() && !isVehicleFighter {
                    let mut ePos: vec3_t = [0.0; 3];
                    let mut weaponMarkShader: c_int = 0;
                    let mut markShader: c_int = ctx.world.cgs.media.bdecal_saberglow;

                    _VectorSubtract(endTr, trace.endpos, &mut ePos);
                    VectorNormalize(&mut ePos);
                    let dir = ePos;
                    _VectorMA(trace.endpos, 4.0, dir, &mut ePos);

                    // `client` is the owner's clientinfo — the NPC copy lives on
                    // the entity, the player copy in the client table.
                    let isNpc = ctx.world.entity(ownerNum).currentState.eType
                        == entityType_t::ET_NPC as c_int;
                    let ownerClientNum = ctx.world.entity(ownerNum).currentState.clientNum as usize;

                    let clientValid = if isNpc {
                        ctx.world
                            .entity(ownerNum)
                            .npcClient
                            .as_deref()
                            .map(|c| c.infoValid != qfalse)
                            .unwrap_or(false)
                    } else {
                        ctx.world.cgs.clientinfo[ownerClientNum].infoValid != qfalse
                    };

                    if clientValid {
                        let saberPtr: &mut saberInfo_t = if isNpc {
                            &mut ctx
                                .world
                                .entity_mut(ownerNum)
                                .npcClient
                                .as_deref_mut()
                                .expect("CG_SaberCompWork: NPC clientinfo")
                                .saber[saberNum]
                        } else {
                            &mut ctx.world.cgs.clientinfo[ownerClientNum].saber[saberNum]
                        };

                        let useSecond =
                            WP_SaberBladeUseSecondBladeStyle(saberPtr, bladeNum) != qfalse;

                        let saber: &saberInfo_t = if isNpc {
                            &ctx.world
                                .entity(ownerNum)
                                .npcClient
                                .as_deref()
                                .expect("CG_SaberCompWork: NPC clientinfo")
                                .saber[saberNum]
                        } else {
                            &ctx.world.cgs.clientinfo[ownerClientNum].saber[saberNum]
                        };

                        if useSecond {
                            if saber.g2MarksShader2 != 0 {
                                //we have a shader to use instead of the standard mark shader
                                markShader = saber.g2MarksShader2;
                            }
                            if saber.g2WeaponMarkShader2 != 0 {
                                //we have a shader to use as a splashback onto the weapon model
                                weaponMarkShader = saber.g2WeaponMarkShader2;
                            }
                        } else {
                            if saber.g2MarksShader != 0 {
                                //we have a shader to use instead of the standard mark shader
                                markShader = saber.g2MarksShader;
                            }
                            if saber.g2WeaponMarkShader != 0 {
                                //we have a shader to use as a splashback onto the weapon model
                                weaponMarkShader = saber.g2WeaponMarkShader;
                            }
                        }
                    }

                    let size = ctx.world.bg_state.rng.flrand(3.0, 4.0);
                    let lifeTime = ctx.world.bg_state.rng.Q_irand(5000, 10000);
                    let trLerpOrigin = ctx.world.entity(trEntNum).lerpOrigin;
                    let trYaw = ctx.world.entity(trEntNum).lerpAngles[YAW];
                    let mut trScale = ctx.world.entity(trEntNum).modelScale;
                    let endpos = trace.endpos;
                    CG_AddGhoul2Mark(
                        ctx,
                        markShader,
                        size,
                        &endpos,
                        &ePos,
                        trace.entityNum as c_int,
                        &trLerpOrigin,
                        trYaw,
                        trGhoul2,
                        &mut trScale,
                        lifeTime,
                    );

                    if weaponMarkShader != 0 {
                        let mut splashBackDir: vec3_t = [0.0; 3];
                        _VectorScale(ePos, -1.0, &mut splashBackDir);
                        let size = ctx.world.bg_state.rng.flrand(0.5, 2.0);
                        let lifeTime = ctx.world.bg_state.rng.Q_irand(5000, 10000);
                        let ownerClient = ctx.world.entity(ownerNum).currentState.clientNum;
                        let ownerOrigin = ctx.world.entity(ownerNum).lerpOrigin;
                        let ownerYaw = ctx.world.entity(ownerNum).lerpAngles[YAW];
                        let ownerGhoul2 = ctx.world.entity(ownerNum).ghoul2;
                        let mut ownerScale = ctx.world.entity(ownerNum).modelScale;
                        CG_AddGhoul2Mark(
                            ctx,
                            weaponMarkShader,
                            size,
                            &endpos,
                            &splashBackDir,
                            ownerClient,
                            &ownerOrigin,
                            ownerYaw,
                            ownerGhoul2,
                            &mut ownerScale,
                            lifeTime,
                        );
                    }
                }
            }
        }
    } else {
        //otherwise, we're all set.
        doEffect = true;
    }

    if doEffect {
        let mut hitPersonFxID = ctx.world.cgs.effects.mSaberBloodSparks;
        let mut hitOtherFxID = ctx.world.cgs.effects.mSaberCut;

        let isNpc = ctx.world.entity(ownerNum).currentState.eType == entityType_t::ET_NPC as c_int;
        let ownerClientNum = ctx.world.entity(ownerNum).currentState.clientNum as usize;

        let clientValid = if isNpc {
            ctx.world
                .entity(ownerNum)
                .npcClient
                .as_deref()
                .map(|c| c.infoValid != qfalse)
                .unwrap_or(false)
        } else {
            ctx.world.cgs.clientinfo[ownerClientNum].infoValid != qfalse
        };

        if clientValid {
            let saberPtr: &mut saberInfo_t = if isNpc {
                &mut ctx
                    .world
                    .entity_mut(ownerNum)
                    .npcClient
                    .as_deref_mut()
                    .expect("CG_SaberCompWork: NPC clientinfo")
                    .saber[saberNum]
            } else {
                &mut ctx.world.cgs.clientinfo[ownerClientNum].saber[saberNum]
            };

            let useSecond = WP_SaberBladeUseSecondBladeStyle(saberPtr, bladeNum) != qfalse;

            let saber: &saberInfo_t = if isNpc {
                &ctx.world
                    .entity(ownerNum)
                    .npcClient
                    .as_deref()
                    .expect("CG_SaberCompWork: NPC clientinfo")
                    .saber[saberNum]
            } else {
                &ctx.world.cgs.clientinfo[ownerClientNum].saber[saberNum]
            };

            if useSecond {
                //use second blade style values
                if saber.hitPersonEffect2 != 0 {
                    hitPersonFxID = saber.hitPersonEffect2;
                }
                if saber.hitOtherEffect2 != 0 {
                    //custom hit other effect
                    hitOtherFxID = saber.hitOtherEffect2;
                }
            } else {
                //use first blade style values
                if saber.hitPersonEffect != 0 {
                    hitPersonFxID = saber.hitPersonEffect;
                }
                if saber.hitOtherEffect != 0 {
                    //custom hit other effect
                    hitOtherFxID = saber.hitOtherEffect;
                }
            }
        }

        if trace.plane.normal[0] == 0.0
            && trace.plane.normal[1] == 0.0
            && trace.plane.normal[2] == 0.0
        {
            //who cares, just shoot it somewhere.
            trace.plane.normal[1] = 1.0;
        }

        if ctx.world.entity(ownerNum).serverSaberFleshImpact != qfalse {
            //do standard player/live ent hit sparks
            trap::FX_PlayEffectID(
                ctx.engine,
                hitPersonFxID,
                &trace.endpos,
                &trace.plane.normal,
                -1,
                -1,
            );
        } else {
            //do the cut effect
            trap::FX_PlayEffectID(
                ctx.engine,
                hitOtherFxID,
                &trace.endpos,
                &trace.plane.normal,
                -1,
                -1,
            );
        }
    }
}

/// Raven `CG_G2AnimEntModelLoad` — (re)builds an NPC entity's ghoul2 instance
/// from its configstring model name, bolts the humanoid tags it needs, resolves
/// its animation set and hands its sabers and sounds over.
///
/// The vehicle path creates the clientside `Vehicle_t` out of
/// `CgWorld::vehicle_pool` via the CGAME `G_Create*NPC` arms in
/// [`crate::vehicle_npc`] (DEC-47.3).
/// Source: `oracle/codemp/cgame/cg_players.c:6998-7262`
pub fn CG_G2AnimEntModelLoad(ctx: &mut CgContext, centNum: usize) {
    let modelindex = ctx.world.entity(centNum).currentState.modelindex;
    let cModelName = CG_ConfigString(ctx, CS_MODELS + modelindex);

    if ctx.world.entity(centNum).npcClient.is_none() {
        //have not init'd client yet
        return;
    }

    if !cModelName.is_empty() {
        let mut modelName = cModelName.clone();
        let skinID;

        if ctx.world.entity(centNum).currentState.NPC_class == class_t::CLASS_VEHICLE as c_int
            && modelName.starts_with('$')
        {
            //vehicles pass their veh names over as model names, then we get the model name from the veh type
            //create a vehicle object clientside for this type
            let vehType = modelName[1..].to_string();
            let vehType_c = cstr(&vehType);
            let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
            let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
            let iVehIndex = BG_VehicleGetIndex(
                vehType_c.as_ptr(),
                &mut ctx.world.bg_state,
                &traps,
                &mut callbacks,
            );
            match ctx.world.bg_state.g_vehicleInfo[iVehIndex as usize].r#type {
                vehicleType_t::VH_ANIMAL => {
                    // Create the animal (making sure all it's data is initialized).
                    G_CreateAnimalNPC(ctx, centNum, &vehType);
                }
                vehicleType_t::VH_SPEEDER => {
                    // Create the speeder (making sure all it's data is initialized).
                    G_CreateSpeederNPC(ctx, centNum, &vehType);
                }
                vehicleType_t::VH_FIGHTER => {
                    // Create the fighter (making sure all it's data is initialized).
                    G_CreateFighterNPC(ctx, centNum, &vehType);
                }
                vehicleType_t::VH_WALKER => {
                    // Create the walker (making sure all it's data is initialized).
                    G_CreateWalkerNPC(ctx, centNum, &vehType);
                }
                _ => {
                    // Raven: assert(!"vehicle with an unknown type - couldn't create vehicle_t")
                    CG_Printf(
                        ctx,
                        "vehicle with an unknown type - couldn't create vehicle_t\n",
                    );
                }
            }

            // Raven derefs `cent->m_pVehicle` with no null check - on the
            // unknown-type arm above that's a release-build null deref, so the
            // defined behavior here is to skip the block instead (§19)
            if let Some(id) = ctx.world.entity(centNum).m_pVehicle {
                let sn = ctx.world.entity(centNum).currentState.number as usize;
                let row = &mut ctx.world.vehicle_pool[id.ent_num() as usize];

                //set up my happy prediction hack
                row.m_vOrientation = &raw mut ctx.world.cgSendPSPool[sn].vehOrientation[0];

                row.m_pParentEntity = &raw mut ctx.world.bg_ents[centNum];

                //attach the handles for fx cgame-side
                CG_RegisterVehicleAssets(id);
            }

            let mut modelBuf: [c_char; MAX_QPATH] = [0; MAX_QPATH];
            Q_strncpyz(&mut modelBuf, &modelName, MAX_QPATH);
            let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
            let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
            BG_GetVehicleModelName(
                modelBuf.as_mut_ptr(),
                &mut ctx.world.bg_state,
                &traps,
                &mut callbacks,
            );
            modelName = buf_to_string(&modelBuf.map(|c| c as u8));

            let skin_ptr = match ctx.world.entity(centNum).m_pVehicle {
                Some(id) => unsafe {
                    (*ctx.world.vehicle_pool[id.ent_num() as usize].m_pVehicleInfo).skin
                },
                None => null_mut(),
            };
            if !skin_ptr.is_null() && unsafe { *skin_ptr } != 0 {
                //use a custom skin
                let skin_str = unsafe { cstr_to_str(skin_ptr) };
                skinID = trap::R_RegisterSkin(
                    ctx.engine,
                    &format!("models/players/{}/model_{}.skin", modelName, skin_str),
                );
            } else {
                skinID = trap::R_RegisterSkin(
                    ctx.engine,
                    &format!("models/players/{}/model_default.skin", modelName),
                );
            }
            modelName = format!("models/players/{}/model.glm", modelName);

            //this sound is *only* used for vehicles now
            ctx.world.cgs.media.noAmmoSound =
                trap::S_RegisterSound(ctx.engine, "sound/weapons/noammo.wav");
        } else {
            skinID = CG_HandleAppendedSkin(ctx, &mut modelName); //get the skin if there is one.
        }

        if !ctx.world.entity(centNum).ghoul2.is_null() {
            //clean it first!
            trap::G2API_CleanGhoul2Models(
                ctx.engine,
                &mut ctx.world.entity_mut(centNum).ghoul2 as *mut *mut c_void,
            );
        }

        trap::G2API_InitGhoul2Model(
            ctx.engine,
            &mut ctx.world.entity_mut(centNum).ghoul2 as *mut *mut c_void,
            &modelName,
            0,
            skinID,
            0,
            0,
            0,
        );

        if !ctx.world.entity(centNum).ghoul2.is_null() {
            let ghoul2 = ctx.world.entity(centNum).ghoul2;
            let number = ctx.world.entity(centNum).currentState.number;
            let npcClass = ctx.world.entity(centNum).currentState.NPC_class;

            if npcClass == class_t::CLASS_VEHICLE as c_int {
                if let Some(vehId) = ctx.world.entity(centNum).m_pVehicle {
                    //do special vehicle stuff
                    let row_idx = vehId.ent_num() as usize;

                    // Setup the default first bolt
                    trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "model_root");

                    // Setup the droid unit.
                    let tag = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*droidunit");
                    ctx.world.vehicle_pool[row_idx].m_iDroidUnitTag = tag;

                    // Setup the Exhausts.
                    for i in 0..MAX_VEHICLE_EXHAUSTS {
                        let strTemp = format!("*exhaust{}", i + 1);
                        let tag = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, &strTemp);
                        ctx.world.vehicle_pool[row_idx].m_iExhaustTag[i] = tag;
                    }

                    // Setup the Muzzles.
                    for i in 0..MAX_VEHICLE_MUZZLES {
                        let strTemp = format!("*muzzle{}", i + 1);
                        let tag = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, &strTemp);
                        ctx.world.vehicle_pool[row_idx].m_iMuzzleTag[i] = tag;
                        if ctx.world.vehicle_pool[row_idx].m_iMuzzleTag[i] == -1 {
                            //ergh, try *flash?
                            let strTemp = format!("*flash{}", i + 1);
                            let tag = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, &strTemp);
                            ctx.world.vehicle_pool[row_idx].m_iMuzzleTag[i] = tag;
                        }
                    }

                    // Setup the Turrets.
                    for i in 0..MAX_VEHICLE_TURRETS {
                        let gunnerViewTag = unsafe {
                            (*ctx.world.vehicle_pool[row_idx].m_pVehicleInfo).turret[i]
                                .gunnerViewTag
                        };
                        if !gunnerViewTag.is_null() {
                            let name = unsafe { cstr_to_str(gunnerViewTag) };
                            let tag = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, &name);
                            ctx.world.vehicle_pool[row_idx].m_iGunnerViewTag[i] = tag;
                        } else {
                            ctx.world.vehicle_pool[row_idx].m_iGunnerViewTag[i] = -1;
                        }
                    }
                }
            }

            // the NPC's clientinfo lives on the entity; taking the box out lets
            // it be handed to bg and to `CG_InitG2SaberData` beside `ctx`.
            let mut npcCi = ctx.world.entity_mut(centNum).npcClient.take();

            let npcSaber1 = ctx.world.entity(centNum).currentState.npcSaber1;
            if npcSaber1 != 0 {
                let saber = CG_ConfigString(ctx, CS_MODELS + npcSaber1);
                //valid saber names should always start with '@' for NPCs
                debug_assert!(saber.is_empty() || saber.starts_with('@'));

                if !saber.is_empty() {
                    let mut saber = saber;
                    saber.remove(0); //skip over the @
                    let saber_c = cstr(&saber);
                    if let Some(ci) = npcCi.as_deref_mut() {
                        let sabers = ci.saber.as_mut_ptr();
                        let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
                        let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
                        WP_SetSaber(
                            number,
                            sabers,
                            0,
                            saber_c.as_ptr(),
                            &mut ctx.world.bg_state,
                            &traps,
                            &mut callbacks,
                        );
                    }
                }
            }
            let npcSaber2 = ctx.world.entity(centNum).currentState.npcSaber2;
            if npcSaber2 != 0 {
                let saber = CG_ConfigString(ctx, CS_MODELS + npcSaber2);
                //valid saber names should always start with '@' for NPCs
                debug_assert!(saber.is_empty() || saber.starts_with('@'));

                if !saber.is_empty() {
                    let mut saber = saber;
                    saber.remove(0); //skip over the @
                    let saber_c = cstr(&saber);
                    if let Some(ci) = npcCi.as_deref_mut() {
                        let sabers = ci.saber.as_mut_ptr();
                        let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
                        let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
                        WP_SetSaber(
                            number,
                            sabers,
                            1,
                            saber_c.as_ptr(),
                            &mut ctx.world.bg_state,
                            &traps,
                            &mut callbacks,
                        );
                    }
                }
            }

            // If this is a not vehicle, give it saber stuff...
            if npcClass != class_t::CLASS_VEHICLE as c_int {
                for j in 0..MAX_SABERS {
                    let hasModel = npcCi
                        .as_deref()
                        .map(|ci| ci.saber[j].model[0] != 0)
                        .unwrap_or(false);
                    if hasModel {
                        let oldInstance = npcCi
                            .as_deref()
                            .map(|ci| ci.ghoul2Weapons[j])
                            .unwrap_or(null_mut());
                        if !oldInstance.is_null() {
                            //free the old instance(s)
                            if let Some(ci) = npcCi.as_deref_mut() {
                                trap::G2API_CleanGhoul2Models(
                                    ctx.engine,
                                    &mut ci.ghoul2Weapons[j] as *mut *mut c_void,
                                );
                                ci.ghoul2Weapons[j] = null_mut();
                            }
                        }

                        if let Some(ci) = npcCi.as_deref_mut() {
                            CG_InitG2SaberData(ctx, j, ci);
                        }
                    }
                }
            }

            ctx.world.entity_mut(centNum).npcClient = npcCi;

            trap::G2API_SetSkin(ctx.engine, ghoul2, 0, skinID, skinID);

            ctx.world.entity_mut(centNum).localAnimIndex = -1;

            let GLAName = trap::G2API_GetGLAName(ctx.engine, ghoul2, 0, MAX_QPATH);

            let originalModelName = modelName.clone();

            // Raven's rockettrooper alternative is commented out in the oracle.
            if !GLAName.is_empty() && !GLAName.contains("players/_humanoid/") {
                //it doesn't use humanoid anims.
                let mut GLAName = GLAName.clone();
                if let Some(slash) = GLAName.rfind('/') {
                    GLAName.truncate(slash);
                    GLAName.push_str("/animation.cfg");

                    let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
                    let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
                    let filename = cstr(&GLAName);
                    let animIndex = BG_ParseAnimationFile(
                        &mut ctx.world.bg_state,
                        &traps,
                        &mut callbacks,
                        filename.as_ptr(),
                        null_mut(),
                        qfalse,
                    );
                    ctx.world.entity_mut(centNum).localAnimIndex = animIndex;
                }
            } else {
                //humanoid index.
                trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*r_hand");
                trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*l_hand");

                //rhand must always be first bolt. lhand always second. Whichever you want the
                //jetpack bolted to must always be third.
                trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*chestg");

                //claw bolts
                trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*r_hand_cap_r_arm");
                trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*l_hand_cap_l_arm");

                ctx.world.entity_mut(centNum).localAnimIndex =
                    if GLAName.contains("players/rockettrooper/") {
                        1
                    } else {
                        0
                    };

                if trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*head_top") == -1 {
                    trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "ceyebrow");
                }
                trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "Motion");
            }

            // If this is a not vehicle...
            if npcClass != class_t::CLASS_VEHICLE as c_int {
                if trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "lower_lumbar") == -1 {
                    //check now to see if we have this bone for setting anims and such
                    ctx.world.entity_mut(centNum).noLumbar = qtrue;
                }

                if trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "face") == -1 {
                    //check now to see if we have this bone for setting anims and such
                    ctx.world.entity_mut(centNum).noFace = qtrue;
                }
            } else {
                ctx.world.entity_mut(centNum).noLumbar = qtrue;
                ctx.world.entity_mut(centNum).noFace = qtrue;
            }

            if ctx.world.entity(centNum).localAnimIndex != -1 {
                let mut originalModelName = originalModelName;
                if let Some(slash) = originalModelName.rfind('/') {
                    originalModelName.truncate(slash + 1);
                }

                let localAnimIndex = ctx.world.entity(centNum).localAnimIndex;
                let bgNumAnimEvents = ctx.world.bg_state.bgNumAnimEvents;
                let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
                let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
                let eventAnimIndex = BG_ParseAnimationEvtFile(
                    &mut ctx.world.bg_state,
                    &traps,
                    &mut callbacks,
                    &originalModelName,
                    localAnimIndex,
                    bgNumAnimEvents,
                );
                ctx.world.entity_mut(centNum).eventAnimIndex = eventAnimIndex;
            }
        }
    }

    trap::S_ShutUp(ctx.engine, true);
    CG_HandleNPCSounds(ctx, centNum); //handle sound loading here as well.
    trap::S_ShutUp(ctx.engine, false);
}

/// Raven `CG_ForceElectrocution` — one arc of the lightning that plays over a
/// shocked body: picks a random bolt, fires a short trace off it in a fudged
/// direction, and hands the segment to the FX system.
///
/// Raven: Undoing for now, at least this code should compile if I ( or anyone
/// else ) decides to work on this effect.
/// Source: `oracle/codemp/cgame/cg_players.c:7707-7885`
pub fn CG_ForceElectrocution(
    ctx: &mut CgContext,
    centNum: usize,
    origin: &vec3_t,
    tempAngles: &vec3_t,
    shader: qhandle_t,
    alwaysDo: bool,
) {
    let mut found = false;
    let mut fxOrg: vec3_t = [0.0; 3];
    let mut fxOrg2: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let mut rgb: vec3_t = [0.0; 3];
    let mut boltMatrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };
    let mut tr = trace_t::zeroed();
    let mut bolt: c_int = -1;
    let mut iter: c_int = 0;
    let mut torsoBolt: c_int = -1;
    let mut crotchBolt: c_int = -1;
    let mut elbowLBolt: c_int = -1;
    let mut elbowRBolt: c_int = -1;
    let mut handLBolt: c_int = -1;
    let mut handRBolt: c_int = -1;
    let mut kneeLBolt: c_int = -1;
    let mut kneeRBolt: c_int = -1;
    let mut footLBolt: c_int = -1;
    let mut footRBolt: c_int = -1;

    VectorSet(&mut rgb, 1.0, 1.0, 1.0);

    let ghoul2 = ctx.world.entity(centNum).ghoul2;
    let engine = ctx.engine;

    if ctx.world.entity(centNum).localAnimIndex <= 1 {
        //humanoid
        torsoBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "lower_lumbar");
        crotchBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "pelvis");
        elbowLBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "*l_arm_elbow");
        elbowRBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "*r_arm_elbow");
        handLBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "*l_hand");
        handRBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "*r_hand");
        kneeLBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "*hips_l_knee");
        kneeRBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "*hips_r_knee");
        footLBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "*l_leg_foot");
        footRBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "*r_leg_foot");
    } else if ctx.world.entity(centNum).currentState.NPC_class == class_t::CLASS_PROTOCOL as c_int {
        //any others that can use these bolts too?
        torsoBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "lower_lumbar");
        crotchBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "pelvis");
        elbowLBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "*bicep_lg");
        elbowRBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "*bicep_rg");
        handLBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "*hand_l");
        handRBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "*weapon");
        kneeLBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "*thigh_lg");
        kneeRBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "*thigh_rg");
        footLBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "*foot_lg");
        footRBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "*foot_rg");
    }
    // `crotchBolt`/`kneeLBolt`/`kneeRBolt` are never read - Raven bolts them
    // and then leaves them out of the pick below.
    let _ = (crotchBolt, kneeLBolt, kneeRBolt);

    // Pick a random start point
    while bolt < 0 {
        let test = if iter > 5 {
            iter - 5
        } else {
            ctx.world.bg_state.rng.Q_irand(0, 6)
        };
        bolt = match test {
            // Right Elbow
            0 => elbowRBolt,
            // Left Hand
            1 => handLBolt,
            // Right hand
            2 => handRBolt,
            // Left Foot
            3 => footLBolt,
            // Right foot
            4 => footRBolt,
            // Torso
            5 => torsoBolt,
            // Left Elbow
            _ => elbowLBolt,
        };
        iter += 1;
        if iter == 20 {
            break;
        }
    }
    if bolt >= 0 {
        let time = ctx.world.cg.time;
        let modelScale = ctx.world.entity(centNum).modelScale;
        found = trap::G2API_GetBoltMatrix(
            ctx.engine,
            ghoul2,
            0,
            bolt,
            &mut boltMatrix,
            tempAngles,
            origin,
            time,
            Some(&mut ctx.world.cgs.gameModels[0]),
            &modelScale,
        );
    }

    // Make sure that it's safe to even try and get these values out of the Matrix, otherwise the values could be garbage
    if found {
        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut fxOrg);
        if ctx.world.bg_state.rng.random() > 0.5 {
            BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::NEGATIVE_X as c_int, &mut dir);
        } else {
            BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::NEGATIVE_Y as c_int, &mut dir);
        }

        // Add some fudge, makes us not normalized, but that isn't really important
        for i in 0..3 {
            let fudge = ctx.world.bg_state.rng.crandom() * 0.4f32 as f64;
            dir[i] = (dir[i] as f64 + fudge) as f32;
        }
    } else {
        // Just use the lerp Origin and a random direction
        _VectorCopy(ctx.world.entity(centNum).lerpOrigin, &mut fxOrg);
        // Not normalized, but who cares.
        let x = ctx.world.bg_state.rng.crandom() as f32;
        let y = ctx.world.bg_state.rng.crandom() as f32;
        let z = ctx.world.bg_state.rng.crandom() as f32;
        VectorSet(&mut dir, x, y, z);

        let npcClass = ctx.world.entity(centNum).currentState.NPC_class;
        if npcClass == class_t::CLASS_PROBE as c_int {
            fxOrg[2] += 50.0;
        } else if npcClass == class_t::CLASS_MARK1 as c_int {
            fxOrg[2] += 50.0;
        } else if npcClass == class_t::CLASS_ATST as c_int {
            fxOrg[2] += 120.0;
        }
    }

    // Raven's `VectorMA` is a macro, so the `random() * 40 + 40` scale argument
    // is evaluated once per component - three draws, a different reach per axis.
    // Source: `oracle/codemp/game/q_shared.h:1365`, `cg_players.c:7850`
    for k in 0..3 {
        let reach = ctx.world.bg_state.rng.random() * 40.0 + 40.0;
        fxOrg2[k] = fxOrg[k] + dir[k] * reach;
    }

    CG_Trace(
        ctx,
        &mut tr,
        &fxOrg,
        None,
        None,
        &fxOrg2,
        -1,
        CONTENTS_SOLID,
    );

    if tr.fraction < 1.0 || ctx.world.bg_state.rng.random() > 0.94 || alwaysDo {
        let killTime = ctx.world.bg_state.rng.random() * 50.0 + 100.0;
        let mut p = addElectricityArgStruct_t {
            start: [0.0; 3],
            end: [0.0; 3],
            size1: 1.5,
            size2: 4.0,
            sizeParm: 0.0,
            alpha1: 1.0,
            alpha2: 0.5,
            alphaParm: 0.0,
            sRGB: [0.0; 3],
            eRGB: [0.0; 3],
            rgbParm: 0.0,
            chaos: 5.0,
            killTime: killTime as c_int,
            shader,
            flags: 0x00000001 | 0x00000100 | 0x02000000 | 0x04000000 | 0x01000000,
        };

        _VectorCopy(fxOrg, &mut p.start);
        _VectorCopy(tr.endpos, &mut p.end);
        _VectorCopy(rgb, &mut p.sRGB);
        _VectorCopy(rgb, &mut p.eRGB);

        trap::FX_AddElectricity(ctx.engine, &mut p);

        //In other words:
        //FX_AddElectricity( fxOrg, tr.endpos, 1.5f, 4.0f, 0.0f, 1.0f, 0.5f, 0.0f,
        //	rgb, rgb, 0.0f, 5.5f, random() * 50 + 100, shader,
        //	FX_ALPHA_LINEAR | FX_SIZE_LINEAR | FX_BRANCH | FX_GROW | FX_TAPER );
    }
}

/// Raven `CG_CheckThirdPersonAlpha` — fades your own body (or the ship you are
/// flying) out when the third-person camera is inside it.
///
/// Source: `oracle/codemp/cgame/cg_players.c:8341-8421`
pub fn CG_CheckThirdPersonAlpha(ctx: &mut CgContext, centNum: usize, legs: &mut refEntity_t) {
    let mut alpha: f32 = 1.0;
    let mut setFlags: c_int = 0;

    let clientNum = ctx.world.entity(centNum).currentState.clientNum;

    if let Some(vehId) = ctx.world.entity(centNum).m_pVehicle {
        //a vehicle
        let info_ptr = ctx.world.vehicle_pool[vehId.ent_num() as usize].m_pVehicleInfo;
        let hasAlpha = ctx.world.cg.predictedPlayerState.m_iVehicleNum != clientNum //not mine
            && !info_ptr.is_null()
            && unsafe { (*info_ptr).cameraOverride != qfalse } //has an override
            && unsafe { (*info_ptr).cameraAlpha != 0.0 }; //it has alpha
        if hasAlpha {
            //make sure it's not using any alpha
            legs.renderfx |= RF_FORCE_ENT_ALPHA;
            legs.shaderRGBA[3] = 255;
            return;
        }
    }

    if ctx.world.cg.renderingThirdPerson == qfalse {
        return;
    }

    if ctx.world.cg.predictedPlayerState.m_iVehicleNum != 0 {
        //in a vehicle
        if ctx.world.cg.predictedPlayerState.m_iVehicleNum == clientNum {
            //this is my vehicle
            let vehInfo = ctx.world.entity(centNum).m_pVehicle.and_then(|vehId| {
                let info_ptr = ctx.world.vehicle_pool[vehId.ent_num() as usize].m_pVehicleInfo;
                if info_ptr.is_null() {
                    None
                } else {
                    let info = unsafe { &*info_ptr };
                    Some((
                        info.cameraOverride != qfalse,
                        info.cameraAlpha,
                        info.cameraRange,
                    ))
                }
            });

            if let Some((cameraOverride, cameraAlpha, cameraRange)) = vehInfo {
                if cameraOverride && cameraAlpha != 0.0 {
                    //vehicle has auto third-person alpha on
                    let mut trace = trace_t::zeroed();
                    let mut dir2Crosshair: vec3_t = [0.0; 3];
                    let mut end: vec3_t = [0.0; 3];
                    let cameraCurLoc = ctx.world.view.cameraCurLoc;
                    let cg_crosshairPos = ctx.world.draw.cg_crosshairPos;
                    _VectorSubtract(cg_crosshairPos, cameraCurLoc, &mut dir2Crosshair);
                    VectorNormalize(&mut dir2Crosshair);
                    _VectorMA(cameraCurLoc, cameraRange * 2.0, dir2Crosshair, &mut end);
                    CG_G2Trace(
                        ctx,
                        &mut trace,
                        &cameraCurLoc,
                        Some(&vec3_origin),
                        Some(&vec3_origin),
                        &end,
                        ENTITYNUM_NONE,
                        CONTENTS_BODY,
                    );
                    if trace.entityNum as c_int == clientNum
                        || trace.entityNum as c_int == ctx.world.cg.predictedPlayerState.clientNum
                    {
                        //hit me or the vehicle I'm in
                        ctx.world.players.cg_vehThirdPersonAlpha -=
                            0.1 * ctx.world.cg.frametime as f32 / 50.0;
                        if ctx.world.players.cg_vehThirdPersonAlpha < cameraAlpha {
                            ctx.world.players.cg_vehThirdPersonAlpha = cameraAlpha;
                        }
                    } else {
                        ctx.world.players.cg_vehThirdPersonAlpha +=
                            0.1 * ctx.world.cg.frametime as f32 / 50.0;
                        if ctx.world.players.cg_vehThirdPersonAlpha > 1.0 {
                            ctx.world.players.cg_vehThirdPersonAlpha = 1.0;
                        }
                    }
                    alpha = ctx.world.players.cg_vehThirdPersonAlpha;
                } else {
                    //use the cvar
                    //reset this
                    ctx.world.players.cg_vehThirdPersonAlpha = 1.0;
                    //use the cvar
                    alpha = ctx.world.cvars.cg_thirdPersonAlpha.value;
                }
            } else {
                //use the cvar
                //reset this
                ctx.world.players.cg_vehThirdPersonAlpha = 1.0;
                //use the cvar
                alpha = ctx.world.cvars.cg_thirdPersonAlpha.value;
            }
        }
    } else if ctx.world.cg.predictedPlayerState.clientNum == clientNum {
        //it's me
        //reset this
        ctx.world.players.cg_vehThirdPersonAlpha = 1.0;
        //use the cvar
        setFlags = RF_FORCE_ENT_ALPHA;
        alpha = ctx.world.cvars.cg_thirdPersonAlpha.value;
    }

    if alpha < 1.0 {
        legs.renderfx |= setFlags;
        legs.shaderRGBA[3] = (alpha * 255.0) as i32 as u8;
    }
}

/// Raven `CG_ResetPlayerEntity` — wipes an entity's clientside animation,
/// interpolation and saber state when it (re)enters the PVS, and duplicates the
/// clientinfo's ghoul2 instance onto it if it has none.
///
/// ESCALATION (shape): `cent` and `ci` are parameters for the same reason
/// [`CG_PlayerAnimation`]'s are — wave 2's `CG_ClearLerpFrame` wants them as
/// `&mut` borrows disjoint from `ctx.world`. Raven picks `ci` itself, and the
/// `ET_NPC` half of that pick (including the allocation) still happens below.
///
/// The fighter-in-my-own-ship early-out reads
/// `m_pVehicle->m_pVehicleInfo->type == VH_FIGHTER` through the pool.
/// Source: `oracle/codemp/cgame/cg_players.c:11119-11273`
pub fn CG_ResetPlayerEntity(ctx: &mut CgContext, cent: &mut centity_t, ci: &mut clientInfo_t) {
    // Raven's `cent->errorTime`/`extrapolated` resets are commented out in the
    // oracle.

    let isNpc = cent.currentState.eType == entityType_t::ET_NPC as c_int;

    if isNpc {
        if cent.currentState.NPC_class == class_t::CLASS_VEHICLE as c_int
            && cent.m_pVehicle.is_some_and(|id| unsafe {
                (*ctx.world.vehicle_pool[id.ent_num() as usize].m_pVehicleInfo).r#type
                    == vehicleType_t::VH_FIGHTER
            })
            && ctx.world.cg.predictedPlayerState.m_iVehicleNum != 0
            && cent.currentState.number == ctx.world.cg.predictedPlayerState.m_iVehicleNum
        {
            //holy hackery, batman!
            //I don't think this will break anything. But really, do I ever?
            return;
        }

        if cent.npcClient.is_none() {
            //allocate memory for it, already zeroed (Raven memsets right after
            //the alloc and NULLs `ghoul2Model`, which the zero fill covers). The
            //alloc cannot fail, so Raven's `assert(0); return;` arm is dropped.
            cent.npcClient = Some(CG_CreateNPCClient());
        }

        //just force these guys to be set again, it won't hurt anything if they're
        //already set.
        cent.npcLocalSurfOff = 0;
        cent.npcLocalSurfOn = 0;
    }

    // the NPC arm's `ci` is the entity's own block; taking it out leaves `cent`
    // usable beside it and it goes back before the weapon block needs it.
    let mut npcCi = if isNpc { cent.npcClient.take() } else { None };
    debug_assert!(!isNpc || npcCi.is_some());

    {
        let ci: &mut clientInfo_t = match npcCi.as_deref_mut() {
            Some(c) => c,
            None => &mut *ci,
        };

        for i in 0..MAX_SABERS {
            for j in 0..ci.saber[i].numBlades as usize {
                ci.saber[i].blade[j].trail.lastTime = -20000;
            }
        }

        ci.facial_blink = -1.0;
        ci.facial_frown = 0.0;
        ci.facial_aux = 0.0;
        ci.superSmoothTime = 0;

        //reset lerp origin smooth point
        let lerpOrigin = cent.lerpOrigin;
        _VectorCopy(lerpOrigin, &mut cent.beamEnd);

        if !isNpc || (cent.currentState.eFlags & EF_DEAD) == 0 {
            let legsAnim = cent.currentState.legsAnim;
            let torsoAnim = cent.currentState.torsoAnim;
            CG_ClearLerpFrame(ctx, cent, ci, legsAnim, false);
            CG_ClearLerpFrame(ctx, cent, ci, torsoAnim, true);

            let time = ctx.world.cg.time;
            let posPtr: *const trajectory_t = &cent.currentState.pos;
            BG_EvaluateTrajectory(posPtr, time, &mut cent.lerpOrigin);
            let aposPtr: *const trajectory_t = &cent.currentState.apos;
            BG_EvaluateTrajectory(aposPtr, time, &mut cent.lerpAngles);

            let lerpAngles = cent.lerpAngles;
            _VectorCopy(lerpAngles, &mut cent.rawAngles);

            cent.pe.legs = zeroed_lerp_frame();
            cent.pe.legs.yawAngle = cent.rawAngles[YAW];
            cent.pe.legs.yawing = qfalse;
            cent.pe.legs.pitchAngle = 0.0;
            cent.pe.legs.pitching = qfalse;

            // PORT-NOTE: Raven's second memset passes `sizeof(cent->pe.legs)` —
            // same type as the torso, so the wrong-looking sizeof is harmless.
            cent.pe.torso = zeroed_lerp_frame();
            cent.pe.torso.yawAngle = cent.rawAngles[YAW];
            cent.pe.torso.yawing = qfalse;
            cent.pe.torso.pitchAngle = cent.rawAngles[PITCH];
            cent.pe.torso.pitching = qfalse;

            if isNpc {
                //just start them off at 0 pitch
                cent.pe.torso.pitchAngle = 0.0;
            }

            if cent.ghoul2.is_null()
                && !ci.ghoul2Model.is_null()
                && trap::G2_HaveWeGhoul2Models(ctx.engine, ci.ghoul2Model)
            {
                trap::G2API_DuplicateGhoul2Instance(
                    ctx.engine,
                    ci.ghoul2Model,
                    &mut cent.ghoul2 as *mut *mut c_void,
                );
                cent.weapon = 0;
                cent.ghoul2weapon = null_mut();

                //Attach the instance to this entity num so we can make use of client-server
                //shared operations if possible.
                trap::G2API_AttachInstanceToEntNum(
                    ctx.engine,
                    cent.ghoul2,
                    cent.currentState.number,
                    false,
                );

                if trap::G2API_AddBolt(ctx.engine, cent.ghoul2, 0, "face") == -1 {
                    //check now to see if we have this bone for setting anims and such
                    cent.noFace = qtrue;
                }

                cent.localAnimIndex = CG_G2SkelForModel(ctx, cent.ghoul2);
                cent.eventAnimIndex = CG_G2EvIndexForModel(ctx, cent.ghoul2, cent.localAnimIndex);

                // Raven's weapon-instance copy here is commented out in the oracle.
            }
        }
    }

    if let Some(b) = npcCi.take() {
        cent.npcClient = Some(b);
    }

    //do this to prevent us from making a saber unholster sound the first time we enter the pvs
    let number = cent.currentState.number;
    let weapon = cent.currentState.weapon;
    if number != ctx.world.cg.predictedPlayerState.clientNum
        && weapon == WP_SABER
        && cent.weapon != weapon
    {
        cent.weapon = weapon;

        let ciGhoul2Model = if isNpc {
            cent.npcClient
                .as_deref()
                .map(|c| c.ghoul2Model)
                .unwrap_or(null_mut())
        } else {
            ci.ghoul2Model
        };

        if !cent.ghoul2.is_null() && !ciGhoul2Model.is_null() {
            CG_CopyG2WeaponInstance(ctx, number as usize, weapon, cent.ghoul2);
            cent.ghoul2weapon = CG_G2WeaponInstance(ctx.world, cent, weapon);
        }

        if cent.currentState.saberHolstered == 0 {
            //if not holstered set length and desired length for both blades to full right now.
            let mut npcCi = if isNpc { cent.npcClient.take() } else { None };
            {
                let ci: &mut clientInfo_t = match npcCi.as_deref_mut() {
                    Some(c) => c,
                    None => &mut *ci,
                };

                let saber0: *mut saberInfo_t = &mut ci.saber[0];
                BG_SI_SetDesiredLength(saber0, 0.0, -1);
                let saber1: *mut saberInfo_t = &mut ci.saber[1];
                BG_SI_SetDesiredLength(saber1, 0.0, -1);

                for i in 0..MAX_SABERS {
                    for j in 0..ci.saber[i].numBlades as usize {
                        ci.saber[i].blade[j].length = ci.saber[i].blade[j].lengthMax;
                    }
                }
            }
            if let Some(b) = npcCi {
                cent.npcClient = Some(b);
            }
        }
    }

    if ctx.world.cvars.cg_debugPosition.integer != 0 {
        // §F19: Raven prints `cent->pe.torso.yawAngle` (a float) through `%i`,
        // so the oracle shows garbage; we print the truncated angle instead.
        let yaw = cent.pe.torso.yawAngle as c_int;
        CG_Printf(ctx, &format!("{} ResetPlayerEntity yaw={}\n", number, yaw));
    }
}

/// Raven `CG_LoadClientInfo` — thin wrapper over `CG_LoadClientInfoFor` for the
/// callers that hold a real slot index (`CG_ActualLoadDeferredPlayers`).
///
/// Raven's `CG_LoadClientInfo(&cgs.clientinfo[i])` derives `clientNum = ci -
/// cgs.clientinfo` = i (in range, so unclamped == clamped). We hand the record
/// out of the slot, run the body against it with `gateNum = clientNum`, and put
/// it back — the real-slot semantics Raven had when passed a real `&ci`.
/// Source: `oracle/codemp/cgame/cg_players.c:1006-1147`
pub fn CG_LoadClientInfo(ctx: &mut CgContext, clientNum: c_int) {
    // Raven clamps the derived index to -1 when out of range; a real slot is
    // always in range, so this is a no-op for these callers.
    let mut gateNum = clientNum;
    if gateNum < 0 || gateNum >= MAX_CLIENTS_I32 {
        gateNum = -1;
    }

    // bitwise copy-in/copy-back, original left in place - a zeroed-swap is
    // visible to every ctx-reading helper inside CG_LoadClientInfoFor (C6b
    // referee catch). The copy is written back whole.
    // SAFETY: clientInfo_t is #[repr(C)] plain data; the copy is written back whole.
    let mut ci = unsafe { core::ptr::read(&ctx.world.cgs.clientinfo[clientNum as usize]) };
    CG_LoadClientInfoFor(ctx, &mut ci, gateNum);
    ctx.world.cgs.clientinfo[clientNum as usize] = ci;
}

/// Raven `CG_LoadClientInfo`'s body, operating on the caller's `clientInfo_t`
/// record directly — Raven's stack-local dataflow made explicit.
///
/// `gateNum` is Raven's clamped `clientNum` (the `ci - cgs.clientinfo` value,
/// -1 when the record isn't inside `cgs.clientinfo` — which is every call
/// forwarding a stack-local `&newInfo`). It gates `ClearAttachedInstance`, the
/// ghoul2 duplicate/attach/face-bolt block, the `clientNum` passed through to
/// `CG_RegisterClientModelname`, and the tail entity-reset sweep. With
/// `gateNum == -1` all of those skip — Raven's unclamped re-derivation at
/// `:1140` yields an address that matches no entity, so nothing resets.
/// Source: `oracle/codemp/cgame/cg_players.c:1006-1147`
pub(crate) fn CG_LoadClientInfoFor(ctx: &mut CgContext, ci: &mut clientInfo_t, gateNum: c_int) {
    let engine = ctx.engine;

    ci.deferred = qfalse;

    let mut teamname = String::new();
    if ctx.world.cgs.gametype >= GT_TEAM {
        if ci.team == TEAM_BLUE {
            teamname = DEFAULT_BLUETEAM_NAME.to_string();
        } else {
            teamname = DEFAULT_REDTEAM_NAME.to_string();
        }
    }
    if !teamname.is_empty() {
        teamname.push('/');
    }

    let mut modelloaded = true;
    if ctx.world.cgs.gametype == GT_SIEGE && (ci.team == TEAM_SPECTATOR || ci.siegeIndex == -1) {
        // yeah.. kind of a hack. Don't care until they're actually ingame with a valid class.
        if !CG_RegisterClientModelname(ctx, ci, DEFAULT_MODEL, "default", &teamname, -1) {
            CG_Error(
                ctx,
                &format!("DEFAULT_MODEL ({DEFAULT_MODEL}) failed to register"),
            );
            return;
        }
    } else {
        let modelName = buf_to_string(&ci.modelName.map(|c| c as u8));
        let skinName = buf_to_string(&ci.skinName.map(|c| c as u8));
        // Raven hands the *clamped* clientNum here, so -1 flows through on the
        // out-of-range arm
        if !CG_RegisterClientModelname(ctx, ci, &modelName, &skinName, &teamname, gateNum) {
            // rww - DO NOT error out here! A nonsense model name would crash
            // everyone's client; give it a shot at the default model instead.

            // fall back to default team name
            if ctx.world.cgs.gametype >= GT_TEAM {
                // keep skin name
                if ci.team == TEAM_BLUE {
                    teamname = DEFAULT_BLUETEAM_NAME.to_string();
                } else {
                    teamname = DEFAULT_REDTEAM_NAME.to_string();
                }
                let skinName = buf_to_string(&ci.skinName.map(|c| c as u8));
                if !CG_RegisterClientModelname(ctx, ci, DEFAULT_MODEL, &skinName, &teamname, -1) {
                    CG_Error(
                        ctx,
                        &format!(
                            "DEFAULT_MODEL / skin ({DEFAULT_MODEL}/{skinName}) failed to register"
                        ),
                    );
                    return;
                }
            } else if !CG_RegisterClientModelname(ctx, ci, DEFAULT_MODEL, "default", &teamname, -1)
            {
                CG_Error(
                    ctx,
                    &format!("DEFAULT_MODEL ({DEFAULT_MODEL}) failed to register"),
                );
                return;
            }
            modelloaded = false;
        }
    }

    if gateNum != -1 {
        trap::G2API_ClearAttachedInstance(engine, gateNum);
    }

    if gateNum != -1
        && !ci.ghoul2Model.is_null()
        && trap::G2_HaveWeGhoul2Models(engine, ci.ghoul2Model)
    {
        let existingGhoul2 = ctx.world.entity(gateNum as usize).ghoul2;
        if !existingGhoul2.is_null() && trap::G2_HaveWeGhoul2Models(engine, existingGhoul2) {
            let ghoul2Ptr = &mut ctx.world.entity_mut(gateNum as usize).ghoul2 as *mut *mut c_void;
            trap::G2API_CleanGhoul2Models(engine, ghoul2Ptr);
        }
        let ghoul2Ptr = &mut ctx.world.entity_mut(gateNum as usize).ghoul2 as *mut *mut c_void;
        trap::G2API_DuplicateGhoul2Instance(engine, ci.ghoul2Model, ghoul2Ptr);

        // Attach the instance to this entity num so we can make use of
        // client-server shared operations if possible.
        let dupGhoul2 = ctx.world.entity(gateNum as usize).ghoul2;
        trap::G2API_AttachInstanceToEntNum(engine, dupGhoul2, gateNum, false);

        if trap::G2API_AddBolt(engine, dupGhoul2, 0, "face") == -1 {
            // check now to see if we have this bone for setting anims and such
            ctx.world.entity_mut(gateNum as usize).noFace = qtrue;
        }

        let dupGhoul2 = ctx.world.entity(gateNum as usize).ghoul2;
        let localAnimIndex = CG_G2SkelForModel(ctx, dupGhoul2);
        ctx.world.entity_mut(gateNum as usize).localAnimIndex = localAnimIndex;
        let dupGhoul2 = ctx.world.entity(gateNum as usize).ghoul2;
        let eventAnimIndex = CG_G2EvIndexForModel(ctx, dupGhoul2, localAnimIndex);
        ctx.world.entity_mut(gateNum as usize).eventAnimIndex = eventAnimIndex;
    }

    ci.newAnims = qfalse;
    if ci.torsoModel != 0 {
        let mut tag = orientation_t {
            origin: [0.0; 3],
            axis: [[0.0; 3]; 3],
        };
        // if the torso model has the "tag_flag"
        if trap::R_LerpTag(engine, &mut tag, ci.torsoModel, 0, 0, 1.0, "tag_flag") != 0 {
            ci.newAnims = qtrue;
        }
    }

    // sounds
    if ctx.world.cgs.gametype == GT_SIEGE && (ci.team == TEAM_SPECTATOR || ci.siegeIndex == -1) {
        // don't need to load sounds
    } else {
        CG_LoadCISounds(ctx, ci, modelloaded);
    }

    ci.deferred = qfalse;

    // reset any existing players and bodies, because they might be in bad
    // frames for this new model. Raven re-derives `clientNum = ci -
    // cgs.clientinfo` unclamped here; with a stack-local record (gateNum == -1)
    // that address matches no entity, so the sweep is skipped entirely.
    if gateNum != -1 {
        for i in 0..MAX_GENTITIES {
            if ctx.world.entity(i).currentState.clientNum == gateNum
                && ctx.world.entity(i).currentState.eType == entityType_t::ET_PLAYER as c_int
            {
                // take/put-back: CG_ResetPlayerEntity wants (&mut cent, &mut ci);
                // `ci` is a live &mut param, so only `cent` needs lifting.
                // copy-in/copy-back with the original left in place (see the
                // cg_snapshot.rs caller note - zeroed swap-outs alias wrong)
                // SAFETY: `npcClient` is an owned Box, so the copy never drops and the
                // ptr::write write-back does not drop the original.
                let mut cent = ManuallyDrop::new(unsafe { core::ptr::read(ctx.world.entity(i)) });
                CG_ResetPlayerEntity(ctx, &mut cent, ci);
                unsafe {
                    core::ptr::write(ctx.world.entity_mut(i), ManuallyDrop::into_inner(cent))
                };
            }
        }
    }
}

/// Raven `_PlayerFootStep` — traces to the ground under a foot, plays the
/// material's footstep sound, and (when `cg_footsteps` is turned up) throws the
/// material effect and stamps a footprint mark.
///
/// `centNum` stands in for Raven's `cent` pointer — only the owner's clientNum
/// (for the sound entity) is read off it.
/// Source: `oracle/codemp/cgame/cg_players.c:1989-2181`
pub fn _PlayerFootStep(
    ctx: &mut CgContext,
    origin: &vec3_t,
    orientation: f32,
    radius: f32,
    centNum: usize,
    footStepType: footstepType_t,
) {
    let engine = ctx.engine;
    let mins: vec3_t = [-7.0, -7.0, 0.0];
    let maxs: vec3_t = [7.0, 7.0, 2.0];
    let mut trace = trace_t::zeroed();
    // Raven inits soundType to FOOTSTEP_TOTAL, but every arm below overwrites
    // it before the read - dead store dropped (difLen precedent)
    let soundType: c_int;
    let mut bMark = false;
    let footMarkShader: qhandle_t;
    let mut effectID: c_int = -1;

    // send a trace down from the player to the ground
    let mut end: vec3_t = [0.0; 3];
    _VectorCopy(*origin, &mut end);
    end[2] -= FOOTSTEP_DISTANCE;

    trap::CM_BoxTrace(
        engine,
        &mut trace,
        origin,
        &end,
        Some(&mins),
        Some(&maxs),
        0,
        MASK_PLAYERSOLID,
    );

    // no shadow if too high
    if trace.fraction >= 1.0 {
        return;
    }

    let heavy = footStepType == footstepType_t::FOOTSTEP_HEAVY_R
        || footStepType == footstepType_t::FOOTSTEP_HEAVY_L;

    // check for foot-steppable surface flag
    match trace.surfaceFlags & MATERIAL_MASK {
        MATERIAL_MUD => {
            bMark = true;
            soundType = (if heavy {
                footstep_t::FOOTSTEP_MUDRUN
            } else {
                footstep_t::FOOTSTEP_MUDWALK
            }) as c_int;
            effectID = ctx.world.cgs.effects.footstepMud;
        }
        MATERIAL_DIRT => {
            bMark = true;
            soundType = (if heavy {
                footstep_t::FOOTSTEP_DIRTRUN
            } else {
                footstep_t::FOOTSTEP_DIRTWALK
            }) as c_int;
            effectID = ctx.world.cgs.effects.footstepSand;
        }
        MATERIAL_SAND => {
            bMark = true;
            soundType = (if heavy {
                footstep_t::FOOTSTEP_SANDRUN
            } else {
                footstep_t::FOOTSTEP_SANDWALK
            }) as c_int;
            effectID = ctx.world.cgs.effects.footstepSand;
        }
        MATERIAL_SNOW => {
            bMark = true;
            soundType = (if heavy {
                footstep_t::FOOTSTEP_SNOWRUN
            } else {
                footstep_t::FOOTSTEP_SNOWWALK
            }) as c_int;
            effectID = ctx.world.cgs.effects.footstepSnow;
        }
        MATERIAL_SHORTGRASS | MATERIAL_LONGGRASS => {
            soundType = (if heavy {
                footstep_t::FOOTSTEP_GRASSRUN
            } else {
                footstep_t::FOOTSTEP_GRASSWALK
            }) as c_int;
        }
        MATERIAL_SOLIDMETAL => {
            soundType = (if heavy {
                footstep_t::FOOTSTEP_METALRUN
            } else {
                footstep_t::FOOTSTEP_METALWALK
            }) as c_int;
        }
        MATERIAL_HOLLOWMETAL => {
            soundType = (if heavy {
                footstep_t::FOOTSTEP_PIPERUN
            } else {
                footstep_t::FOOTSTEP_PIPEWALK
            }) as c_int;
        }
        MATERIAL_GRAVEL => {
            soundType = (if heavy {
                footstep_t::FOOTSTEP_GRAVELRUN
            } else {
                footstep_t::FOOTSTEP_GRAVELWALK
            }) as c_int;
            effectID = ctx.world.cgs.effects.footstepGravel;
        }
        MATERIAL_CARPET | MATERIAL_FABRIC | MATERIAL_CANVAS | MATERIAL_RUBBER
        | MATERIAL_PLASTIC => {
            soundType = (if heavy {
                footstep_t::FOOTSTEP_RUGRUN
            } else {
                footstep_t::FOOTSTEP_RUGWALK
            }) as c_int;
        }
        MATERIAL_SOLIDWOOD | MATERIAL_HOLLOWWOOD => {
            soundType = (if heavy {
                footstep_t::FOOTSTEP_WOODRUN
            } else {
                footstep_t::FOOTSTEP_WOODWALK
            }) as c_int;
        }
        // default: glass/water/flesh/concrete/rock/ice/marble/... all fall
        // through to the "stone" footstep in Raven's switch.
        _ => {
            soundType = (if heavy {
                footstep_t::FOOTSTEP_STONERUN
            } else {
                footstep_t::FOOTSTEP_STONEWALK
            }) as c_int;
        }
    }

    if soundType < footstep_t::FOOTSTEP_TOTAL as c_int {
        let idx = (ctx.world.bg_state.rng.rand() & 3) as usize;
        let sfx = ctx.world.cgs.media.footsteps[soundType as usize][idx];
        let clientNum = ctx.world.entity(centNum).currentState.clientNum;
        trap::S_StartSound(engine, None, clientNum, CHAN_BODY, sfx);
    }

    if ctx.world.cvars.cg_footsteps.integer < 4 {
        // debugging - 4 always does footstep effect
        if ctx.world.cvars.cg_footsteps.integer < 2 {
            // 1 for sounds, 2 for effects, 3 for marks
            return;
        }
    }

    if effectID != -1 {
        trap::FX_PlayEffectID(engine, effectID, &trace.endpos, &trace.plane.normal, -1, -1);
    }

    if ctx.world.cvars.cg_footsteps.integer < 4 {
        // debugging - 4 always does footstep effect
        if !bMark || ctx.world.cvars.cg_footsteps.integer < 3 {
            // 1 for sounds, 2 for effects, 3 for marks
            return;
        }
    }

    match footStepType {
        footstepType_t::FOOTSTEP_HEAVY_R => {
            footMarkShader = ctx.world.cgs.media.fshrMarkShader;
        }
        footstepType_t::FOOTSTEP_HEAVY_L => {
            footMarkShader = ctx.world.cgs.media.fshlMarkShader;
        }
        footstepType_t::FOOTSTEP_R => {
            footMarkShader = ctx.world.cgs.media.fsrMarkShader;
        }
        // default / FOOTSTEP_L
        _ => {
            footMarkShader = ctx.world.cgs.media.fslMarkShader;
        }
    }

    // add the mark as a temporary, so it goes directly to the renderer without
    // taking a spot in the cg_marks array
    if trace.plane.normal[0] != 0.0 || trace.plane.normal[1] != 0.0 || trace.plane.normal[2] != 0.0
    {
        CG_ImpactMark(
            ctx,
            footMarkShader,
            trace.endpos,
            trace.plane.normal,
            orientation,
            1.0,
            1.0,
            1.0,
            1.0,
            false,
            radius,
            false,
        );
    }
}

/// Raven `CG_PlayerShadow` — traces down for the player's shadow, sets
/// `shadowPlane`, and (dropshadow mode only) stamps the shadow mark. Returns
/// whether the caller should still draw its own shadow geometry.
/// Source: `oracle/codemp/cgame/cg_players.c:4612-4707`
pub fn CG_PlayerShadow(ctx: &mut CgContext, centNum: usize, shadowPlane: &mut f32) -> bool {
    let engine = ctx.engine;
    let mins: vec3_t = [-15.0, -15.0, 0.0];
    let maxs: vec3_t = [15.0, 15.0, 2.0];
    let mut trace = trace_t::zeroed();
    let mut radius = 24.0f32;

    *shadowPlane = 0.0;

    if ctx.world.cvars.cg_shadows.integer == 0 {
        return false;
    }

    // no shadows when cloaked
    if ctx.world.entity(centNum).currentState.powerups & (1 << PW_CLOAKED) != 0 {
        return false;
    }

    if ctx.world.entity(centNum).currentState.eFlags & EF_DEAD != 0 {
        return false;
    }

    let ti1 = ctx.world.entity(centNum).currentState.trickedentindex;
    let ti2 = ctx.world.entity(centNum).currentState.trickedentindex2;
    let ti3 = ctx.world.entity(centNum).currentState.trickedentindex3;
    let ti4 = ctx.world.entity(centNum).currentState.trickedentindex4;
    // §F19: Raven derefs `cg.snap->ps` unconditionally; a null snapshot has no
    // local client to be tricked, so we read -1.
    let snapClient = ctx.world.cg.snap_ref().map_or(-1, |snap| snap.ps.clientNum);
    if CG_IsMindTricked(ctx.world, ti1, ti2, ti3, ti4, snapClient) {
        // this entity is mind-tricking the current client, so don't render it
        return false;
    }

    if ctx.world.cvars.cg_shadows.integer == 1 {
        // dropshadow
        if ctx.world.entity(centNum).currentState.m_iVehicleNum != 0
            && ctx.world.entity(centNum).currentState.NPC_class != class_t::CLASS_VEHICLE as c_int
        {
            // riding a vehicle, no dropshadow
            return false;
        }
    }

    // send a trace down from the player to the ground
    let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
    let mut end: vec3_t = [0.0; 3];
    _VectorCopy(lerpOrigin, &mut end);
    if ctx.world.cvars.cg_shadows.integer == 2 {
        // stencil
        end[2] -= 4096.0;

        trap::CM_BoxTrace(
            engine,
            &mut trace,
            &lerpOrigin,
            &end,
            Some(&mins),
            Some(&maxs),
            0,
            MASK_PLAYERSOLID,
        );

        if trace.fraction == 1.0 || trace.startsolid != 0 || trace.allsolid != 0 {
            trace.endpos[2] = lerpOrigin[2] - 25.0;
        }
    } else {
        end[2] -= SHADOW_DISTANCE;

        trap::CM_BoxTrace(
            engine,
            &mut trace,
            &lerpOrigin,
            &end,
            Some(&mins),
            Some(&maxs),
            0,
            MASK_PLAYERSOLID,
        );

        // no shadow if too high
        if trace.fraction == 1.0 || trace.startsolid != 0 || trace.allsolid != 0 {
            return false;
        }
    }

    if ctx.world.cvars.cg_shadows.integer == 2 {
        // stencil shadows need plane to be on ground
        *shadowPlane = trace.endpos[2];
    } else {
        *shadowPlane = trace.endpos[2] + 1.0;
    }

    if ctx.world.cvars.cg_shadows.integer != 1 {
        // no mark for stencil or projection shadows
        return true;
    }

    // fade the shadow out with height (Raven's `1.0` is a double, so the whole
    // subtract is done wide then narrowed)
    let alpha = (1.0f64 - trace.fraction as f64) as f32;

    // add the mark as a temporary, so it goes directly to the renderer without
    // taking a spot in the cg_marks array
    let npcClass = ctx.world.entity(centNum).currentState.NPC_class;
    if npcClass == class_t::CLASS_REMOTE as c_int || npcClass == class_t::CLASS_SEEKER as c_int {
        radius = 8.0;
    }
    let yawAngle = ctx.world.entity(centNum).pe.legs.yawAngle;
    let shadowMarkShader = ctx.world.cgs.media.shadowMarkShader;
    CG_ImpactMark(
        ctx,
        shadowMarkShader,
        trace.endpos,
        trace.plane.normal,
        yawAngle,
        alpha,
        alpha,
        alpha,
        1.0,
        false,
        radius,
        true,
    );

    true
}

/// Raven `CG_ForcePushBodyBlur` — bolts a force-push blur onto each of the
/// humanoid's push bones. Non-humanoid models (`localAnimIndex > 1`) and
/// mind-tricking entities are skipped.
/// Source: `oracle/codemp/cgame/cg_players.c:4958-4998`
pub fn CG_ForcePushBodyBlur(ctx: &mut CgContext, centNum: usize) {
    let engine = ctx.engine;

    if ctx.world.entity(centNum).localAnimIndex > 1 {
        // Sorry, the humanoid IS IN ANOTHER CASTLE.
        return;
    }

    if ctx.world.cg.snap_ref().is_some() {
        let ti1 = ctx.world.entity(centNum).currentState.trickedentindex;
        let ti2 = ctx.world.entity(centNum).currentState.trickedentindex2;
        let ti3 = ctx.world.entity(centNum).currentState.trickedentindex3;
        let ti4 = ctx.world.entity(centNum).currentState.trickedentindex4;
        let snapClient = ctx.world.cg.snap_ref().map_or(-1, |snap| snap.ps.clientNum);
        if CG_IsMindTricked(ctx.world, ti1, ti2, ti3, ti4, snapClient) {
            // this entity is mind-tricking the current client, so don't render it
            return;
        }
    }

    debug_assert!(!ctx.world.entity(centNum).ghoul2.is_null());

    let time = ctx.world.cg.time;
    for boneName in cg_pushBoneNames {
        // go through all the bones we want to put a blur effect on
        let boneName = match boneName {
            Some(name) => name,
            None => break,
        };
        let ghoul2 = ctx.world.entity(centNum).ghoul2;
        let bolt = trap::G2API_AddBolt(engine, ghoul2, 0, boneName);

        if bolt == -1 {
            debug_assert!(
                false,
                "You've got an invalid bone/bolt name in cg_pushBoneNames"
            );
            continue;
        }

        let mut boltMatrix = mdxaBone_t {
            matrix: [[0.0; 4]; 3],
        };
        let turAngles = ctx.world.entity(centNum).turAngles;
        let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
        let modelScale = ctx.world.entity(centNum).modelScale;
        trap::G2API_GetBoltMatrix(
            engine,
            ghoul2,
            0,
            bolt,
            &mut boltMatrix,
            &turAngles,
            &lerpOrigin,
            time,
            Some(&mut ctx.world.cgs.gameModels[0]),
            &modelScale,
        );
        let mut fxOrg: vec3_t = [0.0; 3];
        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut fxOrg);

        // standard effect, don't be refractive (for now)
        CG_ForcePushBlur(ctx, &fxOrg, None);
    }
}

/// Raven `CG_AddSaberBlade` — bolts, traces, marks, trails and finally draws one
/// saber blade. `centNum`/`scentNum` stand in for Raven's `cent`/`scent`
/// pointers; `saber` and `modelIndex` ride the signature but aren't read here.
/// Source: `oracle/codemp/cgame/cg_players.c:5993-6480`
#[allow(clippy::too_many_arguments)]
pub fn CG_AddSaberBlade(
    ctx: &mut CgContext,
    centNum: usize,
    scentNum: usize,
    saber: &mut refEntity_t,
    renderfx: c_int,
    modelIndex: c_int,
    saberNum: usize,
    bladeNum: c_int,
    origin: &vec3_t,
    angles: &vec3_t,
    fromSaber: bool,
    dontDraw: bool,
) {
    let engine = ctx.engine;
    // `saber` and `modelIndex` are part of Raven's signature but unread in this body.
    let _ = (saber, modelIndex);

    let isNpc = ctx.world.entity(centNum).currentState.eType == entityType_t::ET_NPC as c_int;
    let number = ctx.world.entity(centNum).currentState.number as usize;

    // §F19: Raven indexes cgs.clientinfo with the raw entity number and reads
    // OOB garbage for a non-client ent; the port skips the blade instead.
    if !isNpc && number >= MAX_CLIENTS {
        return;
    }

    // Raven's `client` is either the NPC's owned clientinfo or the player's slot
    // in cgs.clientinfo; hand it out (take/put-back) so we can mutate its saber
    // trail alongside ctx-taking calls. Both arms box up to one binding.
    let mut client: Box<clientInfo_t> = if isNpc {
        ctx.world
            .entity_mut(centNum)
            .npcClient
            .take()
            .unwrap_or_else(|| {
                debug_assert!(false, "CG_AddSaberBlade: NPC with no npcClient");
                Box::new(zeroed_client_info())
            })
    } else {
        Box::new(core::mem::replace(
            &mut ctx.world.cgs.clientinfo[number],
            zeroed_client_info(),
        ))
    };

    let saberEntNum = ctx.world.entity(centNum).currentState.saberEntityNum;
    let mut saberLen = client.saber[saberNum].blade[bladeNum as usize].length;

    'blade: {
        if saberLen <= 0.0 && !dontDraw {
            // don't bother then.
            break 'blade;
        }

        let mut org_: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let mut v: vec3_t = [0.0; 3];
        // "shut the compiler up" - Raven zero-inits the axis
        let mut axis_: [vec3_t; 3] = [[0.0; 3]; 3];
        let mut trace = trace_t::zeroed();

        let mut futureAngles: vec3_t = [0.0; 3];
        futureAngles[YAW] = angles[YAW];
        futureAngles[PITCH] = angles[PITCH];
        futureAngles[ROLL] = angles[ROLL];

        let useModelIndex: c_int = if fromSaber { 0 } else { saberNum as c_int + 1 };

        // Assume bladeNum equals the bolt index (bolts added in blade order).
        // If there's an effect on this blade, play it.
        let scentGhoul2 = ctx.world.entity(scentNum).ghoul2;
        let scentLerpOrigin = ctx.world.entity(scentNum).lerpOrigin;
        let scentNumber = ctx.world.entity(scentNum).currentState.number;
        let saberPtr: &mut saberInfo_t = &mut client.saber[saberNum];
        let useSecond = WP_SaberBladeUseSecondBladeStyle(saberPtr, bladeNum) != qfalse;
        if !useSecond && client.saber[saberNum].bladeEffect != 0 {
            trap::FX_PlayBoltedEffectID(
                engine,
                client.saber[saberNum].bladeEffect,
                &scentLerpOrigin,
                scentGhoul2,
                bladeNum,
                scentNumber,
                useModelIndex,
                -1,
                false,
            );
        } else if useSecond && client.saber[saberNum].bladeEffect2 != 0 {
            trap::FX_PlayBoltedEffectID(
                engine,
                client.saber[saberNum].bladeEffect2,
                &scentLerpOrigin,
                scentGhoul2,
                bladeNum,
                scentNumber,
                useModelIndex,
                -1,
                false,
            );
        }

        // get the boltMatrix
        let mut boltMatrix = mdxaBone_t {
            matrix: [[0.0; 4]; 3],
        };
        let scentModelScale = ctx.world.entity(scentNum).modelScale;
        let time = ctx.world.cg.time;
        trap::G2API_GetBoltMatrix(
            engine,
            scentGhoul2,
            useModelIndex,
            bladeNum,
            &mut boltMatrix,
            &futureAngles,
            origin,
            time,
            Some(&mut ctx.world.cgs.gameModels[0]),
            &scentModelScale,
        );

        // work the matrix axis stuff into the original axis and origins used.
        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut org_);
        BG_GiveMeVectorFromMatrix(
            &boltMatrix,
            Eorientations::NEGATIVE_Y as c_int,
            &mut axis_[0],
        );

        let saberInFlight = ctx.world.entity(centNum).currentState.saberInFlight;
        if !fromSaber && (saberEntNum as usize) < MAX_GENTITIES && saberInFlight == qfalse {
            // §F19: saberEntityNum is server-supplied; guard the index rather
            // than let a bad number panic. Raven's `saberEnt` is never null.
            let sEnt = ctx.world.entity_mut(saberEntNum as usize);
            _VectorCopy(org_, &mut sEnt.currentState.pos.trBase);
            _VectorCopy(axis_[0], &mut sEnt.currentState.apos.trBase);
        }

        _VectorMA(org_, saberLen, axis_[0], &mut end);
        _VectorAdd(end, axis_[0], &mut end);

        let mut scolor: c_int = if isNpc {
            client.saber[saberNum].blade[bladeNum as usize].color
        } else if saberNum == 0 {
            client.icolor1
        } else {
            client.icolor2
        };

        if ctx.world.cgs.gametype >= GT_TEAM
            && ctx.world.cgs.gametype != GT_SIEGE
            && ctx.world.cgs.jediVmerc == qfalse
            && !isNpc
        {
            if client.team == TEAM_RED {
                scolor = SABER_RED;
            } else if client.team == TEAM_BLUE {
                scolor = SABER_BLUE;
            }
        }

        // if we don't have saber contact enabled, just add the blade and don't
        // care what it's touching (Raven's `goto CheckTrail`)
        if ctx.world.cvars.cg_saberContact.integer != 0 && !dontDraw {
            if ctx.world.cvars.cg_saberModelTraceEffect.integer != 0 {
                CG_G2SaberEffects(ctx, &org_, &end, centNum);
            } else if ctx.world.cvars.cg_saberClientVisualCompensation.integer != 0 {
                CG_Trace(
                    ctx,
                    &mut trace,
                    &org_,
                    None,
                    None,
                    &end,
                    ENTITYNUM_NONE,
                    MASK_SOLID,
                );

                if trace.fraction != 1.0 {
                    // nudge the endpos a hair from base->end so the backwards
                    // comp trace lands at the end and sparks on each side.
                    let mut seDif: vec3_t = [0.0; 3];
                    _VectorSubtract(trace.endpos, org_, &mut seDif);
                    VectorNormalize(&mut seDif);
                    trace.endpos[0] += seDif[0] * 0.1;
                    trace.endpos[1] += seDif[1] * 0.1;
                    trace.endpos[2] += seDif[2] * 0.1;
                }

                let time = ctx.world.cg.time;
                // debounce for absurd framerates - storageTime is otherwise unused clientside
                if client.saber[saberNum].blade[bladeNum as usize].storageTime < time {
                    // CG_SaberCompWork reads the owner's clientinfo, so our
                    // working copy has to be in place: put back, call, take out
                    // again (its writes are captured on the re-take).
                    if isNpc {
                        ctx.world.entity_mut(centNum).npcClient = Some(client);
                    } else {
                        ctx.world.cgs.clientinfo[number] = *client;
                    }
                    let endpos = trace.endpos;
                    CG_SaberCompWork(ctx, &org_, &endpos, centNum, saberNum, bladeNum);
                    client = if isNpc {
                        ctx.world
                            .entity_mut(centNum)
                            .npcClient
                            .take()
                            .unwrap_or_else(|| Box::new(zeroed_client_info()))
                    } else {
                        Box::new(core::mem::replace(
                            &mut ctx.world.cgs.clientinfo[number],
                            zeroed_client_info(),
                        ))
                    };
                    client.saber[saberNum].blade[bladeNum as usize].storageTime = time + 5;
                }
            }

            // was 2 (both directions) but leaves saber trails on either side of
            // architecture and still looks bad on corners, so just base->end.
            for i in 0..1usize {
                if i != 0 {
                    // tracing from end to base
                    CG_Trace(
                        ctx,
                        &mut trace,
                        &end,
                        None,
                        None,
                        &org_,
                        ENTITYNUM_NONE,
                        MASK_SOLID,
                    );
                } else {
                    // tracing from base to end
                    CG_Trace(
                        ctx,
                        &mut trace,
                        &org_,
                        None,
                        None,
                        &end,
                        ENTITYNUM_NONE,
                        MASK_SOLID,
                    );
                }

                if trace.fraction < 1.0 {
                    let mut trDir: vec3_t = [0.0; 3];
                    _VectorCopy(trace.plane.normal, &mut trDir);
                    if trDir[0] == 0.0 && trDir[1] == 0.0 && trDir[2] == 0.0 {
                        trDir[1] = 1.0;
                    }

                    if client.saber[saberNum].saberFlags2 & SFL2_NO_WALL_MARKS != 0 {
                        // don't actually draw the marks/impact effects
                    } else if trace.surfaceFlags & SURF_NOIMPACT == 0 {
                        // never spark on sky
                        let mSparks = ctx.world.cgs.effects.mSparks;
                        trap::FX_PlayEffectID(engine, mSparks, &trace.endpos, &trDir, -1, -1);
                    }

                    // Stop saber? (it wouldn't look right stuck through a thin wall)
                    _VectorSubtract(org_, trace.endpos, &mut v);
                    saberLen = VectorLength(v);

                    _VectorCopy(trace.endpos, &mut end);

                    if client.saber[saberNum].saberFlags2 & SFL2_NO_WALL_MARKS != 0 {
                        // don't actually draw the marks
                    } else {
                        // draw marks if we hit a wall. All I need is a bool for
                        // whether I have a previous point to work with.
                        if client.saber[saberNum].blade[bladeNum as usize]
                            .trail
                            .haveOldPos[i]
                            != qfalse
                        {
                            let entNum = trace.entityNum as usize;
                            let hitEType = ctx.world.entity(entNum).currentState.eType;
                            let hitEFlags = ctx.world.entity(entNum).currentState.eFlags;
                            if trace.entityNum as c_int == ENTITYNUM_WORLD
                                || hitEType == entityType_t::ET_TERRAIN as c_int
                                || hitEFlags & EF_PERMANENT != 0
                            {
                                // only put marks on architecture - cool burn/glow bits
                                let oldPos =
                                    client.saber[saberNum].blade[bladeNum as usize].trail.oldPos[i];
                                let normal = trace.plane.normal;
                                let endpos = trace.endpos;
                                CG_CreateSaberMarks(ctx, &oldPos, &endpos, &normal);

                                // make a sound
                                let time = ctx.world.cg.time;
                                if time
                                    - client.saber[saberNum].blade[bladeNum as usize]
                                        .hitWallDebounceTime
                                    >= 100
                                {
                                    // ugh, need a real sound debouncer... or do this game-side
                                    client.saber[saberNum].blade[bladeNum as usize]
                                        .hitWallDebounceTime = time;
                                    let n = ctx.world.bg_state.rng.Q_irand(1, 3);
                                    let sfx = trap::S_RegisterSound(
                                        engine,
                                        &format!("sound/weapons/saber/saberhitwall{n}"),
                                    );
                                    trap::S_StartSound(
                                        engine,
                                        Some(&trace.endpos),
                                        -1,
                                        CHAN_WEAPON,
                                        sfx,
                                    );
                                }
                            }
                        } else {
                            // if we impact next frame, we'll mark a slash mark
                            client.saber[saberNum].blade[bladeNum as usize]
                                .trail
                                .haveOldPos[i] = qtrue;
                        }
                    }

                    // stash point so we can connect-the-dots later
                    let endpos = trace.endpos;
                    let normal = trace.plane.normal;
                    _VectorCopy(
                        endpos,
                        &mut client.saber[saberNum].blade[bladeNum as usize].trail.oldPos[i],
                    );
                    _VectorCopy(
                        normal,
                        &mut client.saber[saberNum].blade[bladeNum as usize]
                            .trail
                            .oldNormal[i],
                    );
                } else {
                    // no impact this frame, so turn off our mark tracking mechanism
                    client.saber[saberNum].blade[bladeNum as usize]
                        .trail
                        .haveOldPos[i] = qfalse;
                }
            }
        }

        // CheckTrail:
        let cg_saberTrail = ctx.world.cvars.cg_saberTrail.integer;
        let saberPtr: &mut saberInfo_t = &mut client.saber[saberNum];
        let useSecondTrail = WP_SaberBladeUseSecondBladeStyle(saberPtr, bladeNum) != qfalse;
        // don't do the trail (Raven's two `goto JustDoIt`s folded into one skip)
        let skipTrail = cg_saberTrail == 0
            || (!useSecondTrail && client.saber[saberNum].trailStyle > 1)
            || (useSecondTrail && client.saber[saberNum].trailStyle2 > 1);
        if !skipTrail {
            // FIXME: if trailStyle is 1, use the motion blur instead
            let saberMove = ctx.world.entity(centNum).currentState.saberMove;
            let torsoAnim = ctx.world.entity(centNum).currentState.torsoAnim;
            client.saber[saberNum].blade[bladeNum as usize]
                .trail
                .duration = saberMoveData[saberMove as usize].trailLength;

            // duration is int but Raven divides by 5.0f then truncates back
            let mut trailDur = (client.saber[saberNum].blade[bladeNum as usize]
                .trail
                .duration as f32
                / 5.0f32) as c_int;
            if trailDur == 0 {
                // hmm.. ok, default
                if BG_SuperBreakWinAnim(torsoAnim) != qfalse {
                    trailDur = 150;
                } else {
                    trailDur = SABER_TRAIL_TIME as c_int;
                }
            }

            // if timescaled or high framerate, don't flood the system with very
            // small trail slices...perhaps by distance would be better?
            let time = ctx.world.cg.time;
            let lastTime = client.saber[saberNum].blade[bladeNum as usize]
                .trail
                .lastTime;
            if time > lastTime + 2 || cg_saberTrail == 2 {
                // 2ms
                if !dontDraw {
                    let powerups = ctx.world.entity(centNum).currentState.powerups;
                    let saberInFlight = ctx.world.entity(centNum).currentState.saberInFlight;
                    let cg_speedTrail = ctx.world.cvars.cg_speedTrail.integer;
                    // if we have a stale segment, don't draw until we have a fresh one
                    if (BG_SuperBreakWinAnim(torsoAnim) != qfalse
                        || saberMoveData[saberMove as usize].trailLength > 0
                        || (powerups & (1 << PW_SPEED) != 0 && cg_speedTrail != 0)
                        || (saberInFlight != qfalse && saberNum == 0))
                        && time < lastTime + 2000
                    {
                        // (Raven's `#if 0` stencil-poly branch is compiled out;
                        // only the trail-quad `else` survives.)
                        let mut rgb1: vec3_t = [255.0, 255.0, 255.0];
                        match scolor {
                            SABER_RED => VectorSet(&mut rgb1, 255.0, 0.0, 0.0),
                            SABER_ORANGE => VectorSet(&mut rgb1, 255.0, 64.0, 0.0),
                            SABER_YELLOW => VectorSet(&mut rgb1, 255.0, 255.0, 0.0),
                            SABER_GREEN => VectorSet(&mut rgb1, 0.0, 255.0, 0.0),
                            SABER_BLUE => VectorSet(&mut rgb1, 0.0, 64.0, 255.0),
                            SABER_PURPLE => VectorSet(&mut rgb1, 220.0, 0.0, 255.0),
                            _ => VectorSet(&mut rgb1, 0.0, 64.0, 255.0),
                        }

                        // Fill the arg struct and hand it to the effects system,
                        // which turns it into a CTrail once it lands there.
                        // SAFETY: effectTrailArgStruct_t is a #[repr(C)] POD of
                        // f32/i32/arrays, so all-zero is a valid value. Raven
                        // leaves it uninitialized and fills only what it uses
                        // (§19: we pick the defined all-zero for the rest).
                        let mut fx: effectTrailArgStruct_t = unsafe { core::mem::zeroed() };

                        // new muzzle -> new end -> old end -> old muzzle -> back
                        _VectorCopy(org_, &mut fx.mVerts[0].origin);
                        _VectorMA(end, 3.0, axis_[0], &mut fx.mVerts[1].origin);
                        let tip = client.saber[saberNum].blade[bladeNum as usize].trail.tip;
                        let base = client.saber[saberNum].blade[bladeNum as usize].trail.base;
                        _VectorCopy(tip, &mut fx.mVerts[2].origin);
                        _VectorCopy(base, &mut fx.mVerts[3].origin);

                        let diff = (time - lastTime) as f32;

                        // not sure clipping this is best; prevents the trail from
                        // showing at all in low framerate situations.
                        if diff <= 10000.0 {
                            // don't draw it if the last time is way out of date
                            let oldAlpha = 1.0 - (diff / trailDur as f32);

                            let cg_shadows = ctx.world.cvars.cg_shadows.integer;
                            let stencilBits = ctx.world.cgs.glconfig.stencilBits;
                            if cg_saberTrail == 2 && cg_shadows != 2 && stencilBits >= 4 {
                                // does other stuff below
                            } else if (!useSecondTrail && client.saber[saberNum].trailStyle == 1)
                                || (useSecondTrail && client.saber[saberNum].trailStyle2 == 1)
                            {
                                // motion trail
                                fx.mShader = ctx.world.cgs.media.swordTrailShader;
                                VectorSet(&mut rgb1, 32.0, 32.0, 32.0); // faint sith sword trail
                                trailDur = (trailDur as f32 * 2.0) as c_int; // stay around twice as long?
                                fx.mKillTime = trailDur;
                                fx.mSetFlags = FX_USE_ALPHA;
                            } else {
                                fx.mShader = ctx.world.cgs.media.saberBlurShader;
                                fx.mKillTime = trailDur;
                                fx.mSetFlags = FX_USE_ALPHA;
                            }

                            // New muzzle
                            _VectorCopy(rgb1, &mut fx.mVerts[0].rgb);
                            fx.mVerts[0].alpha = 255.0;
                            fx.mVerts[0].ST[0] = 0.0;
                            fx.mVerts[0].ST[1] = 1.0;
                            fx.mVerts[0].destST[0] = 1.0;
                            fx.mVerts[0].destST[1] = 1.0;

                            // new tip
                            _VectorCopy(rgb1, &mut fx.mVerts[1].rgb);
                            fx.mVerts[1].alpha = 255.0;
                            fx.mVerts[1].ST[0] = 0.0;
                            fx.mVerts[1].ST[1] = 0.0;
                            fx.mVerts[1].destST[0] = 1.0;
                            fx.mVerts[1].destST[1] = 0.0;

                            // old tip
                            _VectorCopy(rgb1, &mut fx.mVerts[2].rgb);
                            fx.mVerts[2].alpha = 255.0;
                            // NOTE: this just happens to contain the value I want
                            fx.mVerts[2].ST[0] = 1.0 - oldAlpha;
                            fx.mVerts[2].ST[1] = 0.0;
                            fx.mVerts[2].destST[0] = 1.0 + fx.mVerts[2].ST[0];
                            fx.mVerts[2].destST[1] = 0.0;

                            // old muzzle
                            _VectorCopy(rgb1, &mut fx.mVerts[3].rgb);
                            fx.mVerts[3].alpha = 255.0;
                            fx.mVerts[3].ST[0] = 1.0 - oldAlpha;
                            fx.mVerts[3].ST[1] = 1.0;
                            fx.mVerts[3].destST[0] = 1.0 + fx.mVerts[2].ST[0];
                            fx.mVerts[3].destST[1] = 1.0;

                            if cg_saberTrail == 2 && cg_shadows != 2 && stencilBits >= 4 {
                                // don't need to do this every frame.. but..
                                trap::R_SetRefractProp(engine, 1.0, 0.0, true, true);

                                let saberMove2 = ctx.world.entity(centNum).currentState.saberMove;
                                let torsoAnim2 = ctx.world.entity(centNum).currentState.torsoAnim;
                                if BG_SaberInAttack(saberMove2) != qfalse
                                    || BG_SuperBreakWinAnim(torsoAnim2) != qfalse
                                {
                                    // in attack, strong trail
                                    fx.mKillTime = 300;
                                } else {
                                    // faded trail
                                    fx.mKillTime = 40;
                                }
                                fx.mShader = 2; // 2 is always refractive shader
                                fx.mSetFlags = FX_USE_ALPHA;
                            }

                            trap::FX_AddPrimitive(engine, &mut fx);
                        }
                    }
                }

                // we must always do this, even if we aren't active..otherwise we
                // won't know where to pick up from
                _VectorCopy(
                    org_,
                    &mut client.saber[saberNum].blade[bladeNum as usize].trail.base,
                );
                _VectorMA(
                    end,
                    3.0,
                    axis_[0],
                    &mut client.saber[saberNum].blade[bladeNum as usize].trail.tip,
                );
                client.saber[saberNum].blade[bladeNum as usize]
                    .trail
                    .lastTime = time;
            }
        }

        // JustDoIt:
        if dontDraw {
            break 'blade;
        }

        if client.saber[saberNum].saberFlags2 & SFL2_NO_BLADE != 0 {
            // don't actually draw the blade at all
            if client.saber[saberNum].numBlades < 3
                && client.saber[saberNum].saberFlags2 & SFL2_NO_DLIGHT == 0
            {
                // hmm, but still add the dlight
                CG_DoSaberLight(ctx, Some(&client.saber[saberNum]));
            }
            break 'blade;
        }

        // Pass the renderfx flags attached to the saber weapon model so saber
        // glows render properly in a mirror...not sure this is necessary??
        let doLight = client.saber[saberNum].numBlades < 3
            && client.saber[saberNum].saberFlags2 & SFL2_NO_DLIGHT == 0;
        let lengthMax = client.saber[saberNum].blade[bladeNum as usize].lengthMax;
        let bradius = client.saber[saberNum].blade[bladeNum as usize].radius;
        CG_DoSaber(
            ctx, &org_, &axis_[0], saberLen, lengthMax, bradius, scolor, renderfx, doLight,
        );
    }

    // put our working copy back
    if isNpc {
        ctx.world.entity_mut(centNum).npcClient = Some(client);
    } else {
        ctx.world.cgs.clientinfo[number] = *client;
    }
}

/// Raven `CG_SetDeferredClientInfo` — a new client wants a model/skin that isn't
/// registered yet; hunts the client table for one that's close enough to reuse
/// (exact match loads for real, a same-skin teammate donates its handles),
/// falling back to a real load when nobody matches.
///
/// Raven passes a stack-local `&newInfo` here (cg_players.c:1807) while the old
/// occupant still sits in `cgs.clientinfo[clientNum]`, so we take `ci` as a
/// detached `&mut clientInfo_t` — the three scans read `cgs.clientinfo` directly
/// (no aliasing; the old occupant IS a legitimate donor candidate, exactly as in
/// Raven), and the loader hand-off forwards `gateNum = -1` (Raven's `ci -
/// cgs.clientinfo` on a stack address). CG_NewClientInfo cleans the old ghoul2
/// from its captured `oldGhoul2` and writes `ci` into the slot afterward.
/// Source: `oracle/codemp/cgame/cg_players.c:1397-1492`
pub fn CG_SetDeferredClientInfo(ctx: &mut CgContext, ci: &mut clientInfo_t) {
    let maxclients = ctx.world.cgs.maxclients;
    let gametype = ctx.world.cgs.gametype;

    let ciSkinName = buf_to_string(&ci.skinName.map(|c| c as u8));
    let ciModelName = buf_to_string(&ci.modelName.map(|c| c as u8));

    // if someone else is already the same models and skins we
    // can just load the client info
    for i in 0..maxclients {
        let m = &ctx.world.cgs.clientinfo[i as usize];
        if m.infoValid == qfalse || m.deferred != qfalse {
            continue;
        }
        let mSkinName = buf_to_string(&m.skinName.map(|c| c as u8));
        let mModelName = buf_to_string(&m.modelName.map(|c| c as u8));
        if Q_stricmp(&ciSkinName, &mSkinName) != 0
            || Q_stricmp(&ciModelName, &mModelName) != 0
            || (gametype >= GT_TEAM && ci.team != m.team && ci.team != TEAM_SPECTATOR)
        {
            continue;
        }

        // just load the real info cause it uses the same models and skins
        CG_LoadClientInfoFor(ctx, ci, -1);
        return;
    }

    // if we are in teamplay, only grab a model if the skin is correct
    if gametype >= GT_TEAM {
        for i in 0..maxclients {
            let iu = i as usize;
            let isMatch = {
                let m = &ctx.world.cgs.clientinfo[iu];
                if m.infoValid == qfalse || m.deferred != qfalse {
                    false
                } else {
                    let mSkinName = buf_to_string(&m.skinName.map(|c| c as u8));
                    !(ci.team != TEAM_SPECTATOR
                        && (Q_stricmp(&ciSkinName, &mSkinName) != 0
                            || (gametype >= GT_TEAM && ci.team != m.team)))
                }
            };
            if !isMatch {
                continue;
            }

            ci.deferred = qtrue;
            // borrow split: `CG_CopyClientInfoModel` reads the match beside `ctx`.
            // bitwise copy-in, original left in place - a zeroed-swap is visible
            // to every ctx-reading helper inside CG_CopyClientInfoModel (C6b
            // referee catch). The copy is read-only, so it does not write back.
            // SAFETY: clientInfo_t is #[repr(C)] plain data.
            let matchCi = unsafe { core::ptr::read(&ctx.world.cgs.clientinfo[iu]) };
            CG_CopyClientInfoModel(ctx, &matchCi, ci);
            return;
        }
        // load the full model, because we don't ever want to show
        // an improper team skin.  This will cause a hitch for the first
        // player, when the second enters.  Combat shouldn't be going on
        // yet, so it shouldn't matter
        CG_LoadClientInfoFor(ctx, ci, -1);
        return;
    }

    // find the first valid clientinfo and grab its stuff
    for i in 0..maxclients {
        let iu = i as usize;
        // no deferring off of deferred info. Because I said so.
        let isMatch = {
            let m = &ctx.world.cgs.clientinfo[iu];
            m.infoValid != qfalse && m.deferred == qfalse
        };
        if !isMatch {
            continue;
        }

        ci.deferred = qtrue;
        // bitwise copy-in, original left in place - a zeroed-swap is visible to
        // every ctx-reading helper inside CG_CopyClientInfoModel (C6b referee
        // catch). The copy is read-only, so it does not write back.
        // SAFETY: clientInfo_t is #[repr(C)] plain data.
        let matchCi = unsafe { core::ptr::read(&ctx.world.cgs.clientinfo[iu]) };
        CG_CopyClientInfoModel(ctx, &matchCi, ci);
        return;
    }

    // we should never get here...
    // Actually it is possible now because of the unique sabers.
    CG_LoadClientInfoFor(ctx, ci, -1);
}

/// Raven `CG_ActualLoadDeferredPlayers` — scans every client slot for one still
/// waiting on a deferred model load and loads it for real.
/// Source: `oracle/codemp/cgame/cg_players.c:1953-1965`
pub fn CG_ActualLoadDeferredPlayers(ctx: &mut CgContext) {
    // scan for a deferred player to load
    for i in 0..ctx.world.cgs.maxclients {
        let shouldLoad = {
            let ci = &ctx.world.cgs.clientinfo[i as usize];
            ci.infoValid != qfalse && ci.deferred != qfalse
        };
        if shouldLoad {
            CG_LoadClientInfo(ctx, i);
            // break; - Raven leaves the early-out commented, so every deferred
            // slot gets picked up in one pass
        }
    }
}

/// Raven `CG_PlayerFootsteps` — bolts a matrix at the stepping foot and hands
/// off to `_PlayerFootStep` for the ground trace, sound, and decal work.
/// Source: `oracle/codemp/cgame/cg_players.c:2183-2235`
pub fn CG_PlayerFootsteps(ctx: &mut CgContext, centNum: usize, footStepType: footstepType_t) {
    if ctx.world.cvars.cg_footsteps.integer == 0 {
        return;
    }

    let cent = ctx.world.entity(centNum);
    let npcClass = cent.currentState.NPC_class;

    // FIXME: make this a feature of NPCs in the NPCs.cfg? Specify a footstep shader, if any?
    if npcClass == class_t::CLASS_ATST as c_int
        || npcClass == class_t::CLASS_CLAW as c_int
        || npcClass == class_t::CLASS_FISH as c_int
        || npcClass == class_t::CLASS_FLIER2 as c_int
        || npcClass == class_t::CLASS_GLIDER as c_int
        || npcClass == class_t::CLASS_INTERROGATOR as c_int
        || npcClass == class_t::CLASS_MURJJ as c_int
        || npcClass == class_t::CLASS_PROBE as c_int
        || npcClass == class_t::CLASS_R2D2 as c_int
        || npcClass == class_t::CLASS_R5D2 as c_int
        || npcClass == class_t::CLASS_REMOTE as c_int
        || npcClass == class_t::CLASS_SEEKER as c_int
        || npcClass == class_t::CLASS_SENTRY as c_int
        || npcClass == class_t::CLASS_SWAMP as c_int
    {
        return;
    }

    let yawAngle = cent.pe.legs.yawAngle;
    let ghoul2 = cent.ghoul2;
    let lerpOrigin = cent.lerpOrigin;
    let modelScale = cent.modelScale;

    let engine = ctx.engine;
    let mut boltMatrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };
    let mut tempAngles: vec3_t = [0.0; 3];
    let mut sideOrigin: vec3_t = [0.0; 3];

    tempAngles[PITCH] = 0.0;
    tempAngles[YAW] = yawAngle;
    tempAngles[ROLL] = 0.0;

    let footBolt = match footStepType {
        footstepType_t::FOOTSTEP_R | footstepType_t::FOOTSTEP_HEAVY_R => {
            trap::G2API_AddBolt(engine, ghoul2, 0, "*r_leg_foot") //cent->gent->footRBolt;
        }
        _ => trap::G2API_AddBolt(engine, ghoul2, 0, "*l_leg_foot"), //cent->gent->footLBolt;
    };

    // FIXME: get yaw orientation of the foot and use on decal
    let time = ctx.world.cg.time;
    trap::G2API_GetBoltMatrix(
        engine,
        ghoul2,
        0,
        footBolt,
        &mut boltMatrix,
        &tempAngles,
        &lerpOrigin,
        time,
        Some(&mut ctx.world.cgs.gameModels[0]),
        &modelScale,
    );
    BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut sideOrigin);
    sideOrigin[2] += 15.0; // fudge up a bit for coplanar
    _PlayerFootStep(ctx, &sideOrigin, yawAngle, 6.0, centNum, footStepType);
}

/// Raven `CG_NewClientInfo` — rebuilds a client's cached model/skin/saber/force
/// info from its `CS_PLAYERS` configstring, reusing existing ghoul2 handles where
/// it can and re-attaching the entity's instance when the model actually changed.
/// An empty configstring means the player just left, so we clean up and clear the
/// slot.
///
/// Raven builds a stack-local `newInfo`, runs the scan/load/defer helpers against
/// it while `cgs.clientinfo[clientNum]` still holds the old occupant, then copies
/// it into the slot last. We keep `newInfo` detached the whole way through —
/// `CG_ScanForExistingClientInfo`, `CG_LoadClientInfoFor(.., -1)` and
/// `CG_SetDeferredClientInfo` all take it as a `&mut clientInfo_t` and read the
/// old occupant straight from the slot, so the loader sees `gateNum == -1` (Raven's
/// `ci - cgs.clientinfo` on a stack address). `oldGhoul2` captured up front is the
/// pre-copy slot handle Raven cleans at :1815, then `newInfo` writes into the slot.
/// Source: `oracle/codemp/cgame/cg_players.c:1503-1942`
pub fn CG_NewClientInfo(ctx: &mut CgContext, clientNum: c_int, entitiesInitialized: bool) {
    let cn = clientNum as usize;

    let oldGhoul2 = ctx.world.cgs.clientinfo[cn].ghoul2Model;

    let mut oldG2Weapons: [*mut c_void; MAX_SABERS] = [null_mut(); MAX_SABERS];
    let mut k = 0;
    while k < MAX_SABERS {
        oldG2Weapons[k] = ctx.world.cgs.clientinfo[cn].ghoul2Weapons[k];
        k += 1;
    }

    let configstring = CG_ConfigString(ctx, clientNum + CS_PLAYERS);
    if configstring.is_empty() {
        let engine = ctx.engine;
        // clean this stuff up first
        let ghoul2Model = ctx.world.cgs.clientinfo[cn].ghoul2Model;
        if !ghoul2Model.is_null() && trap::G2_HaveWeGhoul2Models(engine, ghoul2Model) {
            trap::G2API_CleanGhoul2Models(
                engine,
                &mut ctx.world.cgs.clientinfo[cn].ghoul2Model as *mut *mut c_void,
            );
        }
        let mut k = 0;
        while k < MAX_SABERS {
            let w = ctx.world.cgs.clientinfo[cn].ghoul2Weapons[k];
            if !w.is_null() && trap::G2_HaveWeGhoul2Models(engine, w) {
                trap::G2API_CleanGhoul2Models(
                    engine,
                    &mut ctx.world.cgs.clientinfo[cn].ghoul2Weapons[k] as *mut *mut c_void,
                );
            }
            k += 1;
        }

        ctx.world.cgs.clientinfo[cn] = zeroed_client_info();
        return; // player just left
    }

    // build into a temp buffer so the defer checks can use the old value
    let mut newInfo = zeroed_client_info();

    // isolate the player's name
    let v = Info_ValueForKey(&configstring, "n");
    Q_strncpyz(&mut newInfo.name, &v, MAX_QPATH);

    // colors
    let v = Info_ValueForKey(&configstring, "c1");
    CG_ColorFromString(&v, &mut newInfo.color1);
    newInfo.icolor1 = atoi(&v);

    let v = Info_ValueForKey(&configstring, "c2");
    CG_ColorFromString(&v, &mut newInfo.color2);
    newInfo.icolor2 = atoi(&v);

    // bot skill
    let v = Info_ValueForKey(&configstring, "skill");
    newInfo.botSkill = atoi(&v);

    // handicap
    let v = Info_ValueForKey(&configstring, "hc");
    newInfo.handicap = atoi(&v);

    // wins
    let v = Info_ValueForKey(&configstring, "w");
    newInfo.wins = atoi(&v);

    // losses
    let v = Info_ValueForKey(&configstring, "l");
    newInfo.losses = atoi(&v);

    // team
    let v = Info_ValueForKey(&configstring, "t");
    newInfo.team = atoi(&v);

    // copy team info out to menu
    if clientNum == ctx.world.cg.clientNum {
        // this is me
        trap::Cvar_Set(ctx.engine, "ui_team", &v);
    }

    // team task
    let v = Info_ValueForKey(&configstring, "tt");
    newInfo.teamTask = atoi(&v);

    // team leader
    let v = Info_ValueForKey(&configstring, "tl");
    newInfo.teamLeader = atoi(&v);

    // model
    let v = Info_ValueForKey(&configstring, "model");
    if ctx.world.cvars.cg_forceModel.integer != 0 {
        // forcemodel makes everyone use a single model to prevent load hitches
        let modelStr = trap::Cvar_VariableStringBuffer(ctx.engine, "model", MAX_QPATH);
        // strchr splits modelStr at the first '/': before it is the model, after
        // it is the skin; no '/' means the default skin
        let (modelStr, skin): (String, String) = match modelStr.find('/') {
            None => (modelStr, "default".to_string()),
            Some(pos) => (modelStr[..pos].to_string(), modelStr[pos + 1..].to_string()),
        };
        Q_strncpyz(&mut newInfo.skinName, &skin, MAX_QPATH);
        Q_strncpyz(&mut newInfo.modelName, &modelStr, MAX_QPATH);

        if ctx.world.cgs.gametype >= GT_TEAM {
            // keep skin name
            if let Some(pos) = v.find('/') {
                Q_strncpyz(&mut newInfo.skinName, &v[pos + 1..], MAX_QPATH);
            }
        }
    } else {
        Q_strncpyz(&mut newInfo.modelName, &v, MAX_QPATH);

        // strchr runs over the just-copied (and possibly truncated) buffer
        let modelName = buf_to_string(&newInfo.modelName.map(|c| c as u8));
        match modelName.find('/') {
            None => {
                // modelName didn not include a skin name
                Q_strncpyz(&mut newInfo.skinName, "default", MAX_QPATH);
            }
            Some(pos) => {
                Q_strncpyz(&mut newInfo.skinName, &modelName[pos + 1..], MAX_QPATH);
                // truncate modelName
                Q_strncpyz(&mut newInfo.modelName, &modelName[..pos], MAX_QPATH);
            }
        }
    }

    if ctx.world.cgs.gametype == GT_SIEGE {
        // entries only sent in siege mode

        // siege desired team
        let v = Info_ValueForKey(&configstring, "sdt");
        if !v.is_empty() {
            newInfo.siegeDesiredTeam = atoi(&v);
        } else {
            newInfo.siegeDesiredTeam = 0;
        }

        // siege classname
        let v = Info_ValueForKey(&configstring, "siegeclass");
        newInfo.siegeIndex = -1;

        // Info_ValueForKey never hands back NULL (Raven's `if (v)` is always
        // taken), so we always run the lookup.
        let siegeClass = BG_SiegeFindClassByName(&v, &ctx.world.bg_state);
        if !siegeClass.is_null() {
            // See if this class forces a model, if so, then use it. Same for skin.
            newInfo.siegeIndex = BG_SiegeFindClassIndexByName(&v, &ctx.world.bg_state);

            // `BG_SiegeFindClassByName` hands back a raw pointer module code
            // never derefs (§D11); the index it just found (>= 0 since the name
            // matched) names the owning `bgSiegeClasses` slot for the reads.
            let idx = newInfo.siegeIndex as usize;
            let (forcedModel, forcedSkin, hasSaberColor, saberColor, hasSaber2Color, saber2Color) = {
                let sc = &ctx.world.bg_state.bgSiegeClasses[idx];
                (
                    sc.forcedModel.clone(),
                    sc.forcedSkin.clone(),
                    sc.hasForcedSaberColor,
                    sc.forcedSaberColor,
                    sc.hasForcedSaber2Color,
                    sc.forcedSaber2Color,
                )
            };

            if !forcedModel.is_empty() {
                Q_strncpyz(&mut newInfo.modelName, &forcedModel, MAX_QPATH);
            }

            if !forcedSkin.is_empty() {
                Q_strncpyz(&mut newInfo.skinName, &forcedSkin, MAX_QPATH);
            }

            if hasSaberColor != qfalse {
                newInfo.icolor1 = saberColor;
                CG_ColorFromInt(newInfo.icolor1, &mut newInfo.color1);
            }
            if hasSaber2Color != qfalse {
                newInfo.icolor2 = saber2Color;
                CG_ColorFromInt(newInfo.icolor2, &mut newInfo.color2);
            }
        }
    }

    let mut saberUpdate = [false; MAX_SABERS];

    // saber being used (Raven's `if (v && ...)` — `v` is never NULL)
    let v = Info_ValueForKey(&configstring, "st");
    let ciSaberName = buf_to_string(&ctx.world.cgs.clientinfo[cn].saberName.map(|c| c as u8));
    if Q_stricmp(&v, &ciSaberName) != 0 {
        Q_strncpyz(&mut newInfo.saberName, &v, 64);
        let sabers = newInfo.saber.as_mut_ptr();
        let saberName_ptr = newInfo.saberName.as_ptr();
        let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
        let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
        WP_SetSaber(
            clientNum,
            sabers,
            0,
            saberName_ptr,
            &mut ctx.world.bg_state,
            &traps,
            &mut callbacks,
        );
        saberUpdate[0] = true;
    } else {
        let ciSaberNameBuf = ctx.world.cgs.clientinfo[cn].saberName.map(|c| c as u8);
        Q_strncpyzBytes(&mut newInfo.saberName, &ciSaberNameBuf, 64);
        newInfo.saber[0] = ctx.world.cgs.clientinfo[cn].saber[0];
        newInfo.ghoul2Weapons[0] = ctx.world.cgs.clientinfo[cn].ghoul2Weapons[0];
    }

    let v = Info_ValueForKey(&configstring, "st2");
    let ciSaber2Name = buf_to_string(&ctx.world.cgs.clientinfo[cn].saber2Name.map(|c| c as u8));
    if Q_stricmp(&v, &ciSaber2Name) != 0 {
        Q_strncpyz(&mut newInfo.saber2Name, &v, 64);
        let sabers = newInfo.saber.as_mut_ptr();
        let saber2Name_ptr = newInfo.saber2Name.as_ptr();
        let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
        let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
        WP_SetSaber(
            clientNum,
            sabers,
            1,
            saber2Name_ptr,
            &mut ctx.world.bg_state,
            &traps,
            &mut callbacks,
        );
        saberUpdate[1] = true;
    } else {
        let ciSaber2NameBuf = ctx.world.cgs.clientinfo[cn].saber2Name.map(|c| c as u8);
        Q_strncpyzBytes(&mut newInfo.saber2Name, &ciSaber2NameBuf, 64);
        newInfo.saber[1] = ctx.world.cgs.clientinfo[cn].saber[1];
        newInfo.ghoul2Weapons[1] = ctx.world.cgs.clientinfo[cn].ghoul2Weapons[1];
    }

    if saberUpdate[0] || saberUpdate[1] {
        let mut j = 0;
        while j < MAX_SABERS {
            if saberUpdate[j] {
                if newInfo.saber[j].model[0] != 0 {
                    if !oldG2Weapons[j].is_null() {
                        // free the old instance(s)
                        trap::G2API_CleanGhoul2Models(
                            ctx.engine,
                            &mut oldG2Weapons[j] as *mut *mut c_void,
                        );
                        oldG2Weapons[j] = null_mut();
                    }

                    CG_InitG2SaberData(ctx, j, &mut newInfo);
                } else if !oldG2Weapons[j].is_null() {
                    // free the old instance(s)
                    trap::G2API_CleanGhoul2Models(
                        ctx.engine,
                        &mut oldG2Weapons[j] as *mut *mut c_void,
                    );
                    oldG2Weapons[j] = null_mut();
                }

                ctx.world.entity_mut(cn).weapon = 0;
                ctx.world.entity_mut(cn).ghoul2weapon = null_mut(); // force a refresh
            }
            j += 1;
        }
    }

    // Check for any sabers that didn't get set again, if they didn't, then
    // reassign the pointers for the new ci
    let mut k = 0;
    while k < MAX_SABERS {
        if !oldG2Weapons[k].is_null() {
            newInfo.ghoul2Weapons[k] = oldG2Weapons[k];
        }
        k += 1;
    }

    // duel team (Raven's `if (v)` — never NULL, so always atoi)
    let v = Info_ValueForKey(&configstring, "dt");
    newInfo.duelTeam = atoi(&v);

    // force powers
    let v = Info_ValueForKey(&configstring, "forcepowers");
    Q_strncpyz(&mut newInfo.forcePowers, &v, MAX_QPATH);

    if ctx.world.cgs.gametype >= GT_TEAM
        && ctx.world.cgs.jediVmerc == qfalse
        && ctx.world.cgs.gametype != GT_SIEGE
    {
        // We won't force colors for siege.
        let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
        let modelName_ptr = newInfo.modelName.as_ptr();
        let skinName_ptr = newInfo.skinName.as_mut_ptr();
        let team = newInfo.team;
        let colors_ptr = newInfo.colorOverride.as_mut_ptr();
        BG_ValidateSkinForTeam(
            modelName_ptr,
            skinName_ptr,
            team,
            colors_ptr,
            &ctx.world.bg_state,
            &traps,
        );
    } else {
        newInfo.colorOverride[0] = 0.0;
        newInfo.colorOverride[1] = 0.0;
        newInfo.colorOverride[2] = 0.0;
    }

    // scan for an existing clientinfo that matches this modelname so we can
    // avoid loading checks if possible
    if !CG_ScanForExistingClientInfo(ctx, &mut newInfo, clientNum) {
        // newInfo stays detached — the loaders read the old occupant straight
        // from the slot and forward gateNum -1 (Raven's stack-local `&newInfo`).

        // if we are defering loads, just have it pick the first valid
        if ctx
            .world
            .cg
            .snap_ref()
            .is_some_and(|snap| snap.ps.clientNum == clientNum)
        {
            // rww - don't defer your own client info ever
            CG_LoadClientInfoFor(ctx, &mut newInfo, -1);
        } else if ctx.world.cvars.cg_deferPlayers.integer != 0
            && ctx.world.cgs.gametype != GT_SIEGE
            && ctx.world.cvars.cg_buildScript.integer == 0
            && ctx.world.cg.loading == qfalse
        {
            // keep whatever they had if it won't violate team skins
            CG_SetDeferredClientInfo(ctx, &mut newInfo);
        } else {
            CG_LoadClientInfoFor(ctx, &mut newInfo, -1);
        }
    }

    // replace whatever was there with the new one
    newInfo.infoValid = qtrue;
    let engine = ctx.engine;
    if !oldGhoul2.is_null()
        && oldGhoul2 != newInfo.ghoul2Model
        && trap::G2_HaveWeGhoul2Models(engine, oldGhoul2)
    {
        // We must kill this instance before we remove our only pointer to it from
        // the cgame. Otherwise we will end up with extra instances all over the
        // place, I think.
        let mut g = oldGhoul2;
        trap::G2API_CleanGhoul2Models(engine, &mut g as *mut *mut c_void);
    }
    ctx.world.cgs.clientinfo[cn] = newInfo;

    // force a weapon change anyway, for all clients being rendered to the current
    // client
    let mut i = 0;
    while i < MAX_CLIENTS {
        ctx.world.entity_mut(i).ghoul2weapon = null_mut();
        i += 1;
    }

    if clientNum != -1 {
        // don't want it using an invalid pointer to share
        trap::G2API_ClearAttachedInstance(engine, clientNum);
    }

    // Check if the ghoul2 model changed in any way. This is safer than assuming
    // we have a legal cent shile loading info.
    let ciGhoul2Model = ctx.world.cgs.clientinfo[cn].ghoul2Model;
    if entitiesInitialized && !ciGhoul2Model.is_null() && oldGhoul2 != ciGhoul2Model {
        // Copy the new ghoul2 model to the centity.
        let time = ctx.world.cg.time;

        // §F19: Raven indexes `bgHumanoidAnimations[legsAnim]` unchecked and its
        // `if (anim)` never fails (address of an array slot); legsAnim is
        // server-supplied, so an OOB index draws nothing here instead of reading
        // garbage.
        let legsAnim = ctx.world.entity(cn).currentState.legsAnim as usize;
        if legsAnim < ctx.world.bg_state.bgHumanoidAnimations.len() {
            let anim = ctx.world.bg_state.bgHumanoidAnimations[legsAnim];
            let mut flags = BONE_ANIM_OVERRIDE_FREEZE;
            let firstFrame = anim.firstFrame as c_int;
            let mut setFrame: c_int = -1;
            let animSpeed = 50.0f32 / anim.frameLerp as f32;

            if anim.loopFrames != -1 {
                flags = BONE_ANIM_OVERRIDE_LOOP;
            }

            let legsFrame = ctx.world.entity(cn).pe.legs.frame;
            if legsFrame >= anim.firstFrame as c_int
                && legsFrame <= (anim.firstFrame as c_int + anim.numFrames as c_int)
            {
                setFrame = legsFrame;
            }

            // rww - Set the animation again because it just got reset due to the
            // model change
            trap::G2API_SetBoneAnim(
                engine,
                ciGhoul2Model,
                0,
                "model_root",
                firstFrame,
                anim.firstFrame as c_int + anim.numFrames as c_int,
                flags,
                animSpeed,
                time,
                setFrame as f32,
                150,
            );
        }
        // Raven's clear runs regardless (his `if (anim)` never fails), so it
        // stays outside the OOB guard
        ctx.world.entity_mut(cn).currentState.legsAnim = 0;

        let torsoAnim = ctx.world.entity(cn).currentState.torsoAnim as usize;
        if torsoAnim < ctx.world.bg_state.bgHumanoidAnimations.len() {
            let anim = ctx.world.bg_state.bgHumanoidAnimations[torsoAnim];
            let mut flags = BONE_ANIM_OVERRIDE_FREEZE;
            let firstFrame = anim.firstFrame as c_int;
            let mut setFrame: c_int = -1;
            let animSpeed = 50.0f32 / anim.frameLerp as f32;

            if anim.loopFrames != -1 {
                flags = BONE_ANIM_OVERRIDE_LOOP;
            }

            let torsoFrame = ctx.world.entity(cn).pe.torso.frame;
            if torsoFrame >= anim.firstFrame as c_int
                && torsoFrame <= (anim.firstFrame as c_int + anim.numFrames as c_int)
            {
                setFrame = torsoFrame;
            }

            // rww - Set the animation again because it just got reset due to the
            // model change
            trap::G2API_SetBoneAnim(
                engine,
                ciGhoul2Model,
                0,
                "lower_lumbar",
                firstFrame,
                anim.firstFrame as c_int + anim.numFrames as c_int,
                flags,
                animSpeed,
                time,
                setFrame as f32,
                150,
            );
        }
        // same always-run clear as legsAnim above
        ctx.world.entity_mut(cn).currentState.torsoAnim = 0;

        let entGhoul2 = ctx.world.entity(cn).ghoul2;
        if !entGhoul2.is_null() && trap::G2_HaveWeGhoul2Models(engine, entGhoul2) {
            trap::G2API_CleanGhoul2Models(
                engine,
                &mut ctx.world.entity_mut(cn).ghoul2 as *mut *mut c_void,
            );
        }
        trap::G2API_DuplicateGhoul2Instance(
            engine,
            ciGhoul2Model,
            &mut ctx.world.entity_mut(cn).ghoul2 as *mut *mut c_void,
        );

        if clientNum != -1 {
            // Attach the instance to this entity num so we can make use of
            // client-server shared operations if possible.
            let g2 = ctx.world.entity(cn).ghoul2;
            trap::G2API_AttachInstanceToEntNum(engine, g2, clientNum, false);
        }

        // check now to see if we have this bone for setting anims and such
        let g2 = ctx.world.entity(cn).ghoul2;
        if trap::G2API_AddBolt(engine, g2, 0, "face") == -1 {
            ctx.world.entity_mut(cn).noFace = qtrue;
        }

        let g2 = ctx.world.entity(cn).ghoul2;
        let localAnimIndex = CG_G2SkelForModel(ctx, g2);
        ctx.world.entity_mut(cn).localAnimIndex = localAnimIndex;

        let g2 = ctx.world.entity(cn).ghoul2;
        let eventAnimIndex = CG_G2EvIndexForModel(ctx, g2, localAnimIndex);
        ctx.world.entity_mut(cn).eventAnimIndex = eventAnimIndex;

        let stateNumber = ctx.world.entity(cn).currentState.number;
        let stateWeapon = ctx.world.entity(cn).currentState.weapon;
        if stateNumber != ctx.world.cg.predictedPlayerState.clientNum && stateWeapon == WP_SABER {
            ctx.world.entity_mut(cn).weapon = stateWeapon;

            let entGhoul2 = ctx.world.entity(cn).ghoul2;
            if !entGhoul2.is_null() && !ciGhoul2Model.is_null() {
                CG_CopyG2WeaponInstance(ctx, cn, stateWeapon, entGhoul2);
                let world = &*ctx.world;
                let gw = CG_G2WeaponInstance(world, world.entity(cn), stateWeapon);
                ctx.world.entity_mut(cn).ghoul2weapon = gw;
            }

            if ctx.world.entity(cn).currentState.saberHolstered == 0 {
                // if not holstered set length and desired length for both blades
                // to full right now.
                let saber0: *mut saberInfo_t = &mut ctx.world.cgs.clientinfo[cn].saber[0];
                BG_SI_SetDesiredLength(saber0, 0.0, -1);
                let saber1: *mut saberInfo_t = &mut ctx.world.cgs.clientinfo[cn].saber[1];
                BG_SI_SetDesiredLength(saber1, 0.0, -1);

                let mut i = 0;
                while i < MAX_SABERS {
                    let numBlades = ctx.world.cgs.clientinfo[cn].saber[i].numBlades as usize;
                    let mut j = 0;
                    while j < numBlades {
                        ctx.world.cgs.clientinfo[cn].saber[i].blade[j].length =
                            ctx.world.cgs.clientinfo[cn].saber[i].blade[j].lengthMax;
                        j += 1;
                    }
                    i += 1;
                }
            }
        }
    }
}

/// Raven `CG_PlayerAnimEventDo` — fires the one animation event that just came
/// due on a player/NPC: a sound, a saber swing/spin whoosh, a footstep, or a
/// bolt-anchored effect. `AEV_FIRE`/`AEV_MOVE` are server-side only and do
/// nothing here.
///
/// `cent` is read-only so it threads as `centNum`; `animEvent` is mutated in
/// place (the effect arm caches its resolved bolt/model index and NULs its
/// `stringData` so the lookup only runs once), so it stays `&mut`.
/// Source: `oracle/codemp/cgame/cg_players.c:2237-2453`
pub fn CG_PlayerAnimEventDo(ctx: &mut CgContext, centNum: usize, animEvent: &mut animevent_t) {
    // Raven's `if (!cent || !animEvent) return;` null guard drops - both are
    // references here, unreachable by construction
    let engine = ctx.engine;

    match animEvent.eventType {
        // AEV_SOUNDCHAN falls through into AEV_SOUND in Raven; the only difference
        // is that it first reads the channel out of the event data.
        animEventType_t::AEV_SOUNDCHAN | animEventType_t::AEV_SOUND => {
            let mut channel: c_int = CHAN_AUTO;
            if animEvent.eventType == animEventType_t::AEV_SOUNDCHAN {
                channel = animEvent.eventData[AED_SOUNDCHANNEL] as c_int;
            }
            // are there variations on the sound?
            let pick = ctx
                .world
                .bg_state
                .rng
                .Q_irand(0, animEvent.eventData[AED_SOUND_NUMRANDOMSNDS] as c_int)
                as usize;
            let holdSnd = animEvent.eventData[AED_SOUNDINDEX_START + pick] as c_int;
            if holdSnd > 0 {
                let num = ctx.world.entity(centNum).currentState.number;
                trap::S_StartSound(engine, None, num, channel, holdSnd);
            }
        }

        animEventType_t::AEV_SABER_SWING => {
            let eType = ctx.world.entity(centNum).currentState.eType;
            let stateClientNum = ctx.world.entity(centNum).currentState.clientNum;
            let saberNum = animEvent.eventData[AED_SABER_SWING_SABERNUM] as usize;

            // custom swing sound off the client's saber, if it has one
            let customSwing: Option<[qhandle_t; 3]> = {
                // §F19: `clientNum` is server-supplied; an OOB one yields no
                // client (random fallback) instead of indexing past the table.
                let client: Option<&clientInfo_t> = if eType == entityType_t::ET_NPC as c_int {
                    let c = ctx.world.entity(centNum).npcClient.as_deref();
                    debug_assert!(c.is_some());
                    c
                } else {
                    ctx.world.cgs.clientinfo.get(stateClientNum as usize)
                };
                match client {
                    // §F19: `saberNum` comes from event data; an OOB saber index
                    // yields no custom sound (the random fallback below) rather
                    // than reading past the array.
                    Some(cl)
                        if cl.infoValid != qfalse
                            && cl.saber.get(saberNum).is_some_and(|s| s.swingSound[0] != 0) =>
                    {
                        // custom swing sound
                        Some(cl.saber[0].swingSound)
                    }
                    _ => None,
                }
            };

            let swingSound: qhandle_t = if let Some(sounds) = customSwing {
                sounds[ctx.world.bg_state.rng.Q_irand(0, 2) as usize]
            } else {
                let randomSwing = match animEvent.eventData[AED_SABER_SWING_TYPE] {
                    1 => ctx.world.bg_state.rng.Q_irand(4, 6), // SWING_MEDIUM
                    2 => ctx.world.bg_state.rng.Q_irand(7, 9), // SWING_STRONG
                    // default / 0 == SWING_FAST
                    _ => ctx.world.bg_state.rng.Q_irand(1, 3),
                };
                trap::S_RegisterSound(
                    engine,
                    &format!("sound/weapons/saber/saberhup{randomSwing}.wav"),
                )
            };

            let pos = ctx.world.entity(centNum).currentState.pos.trBase;
            let num = ctx.world.entity(centNum).currentState.number;
            trap::S_StartSound(engine, Some(&pos), num, CHAN_AUTO, swingSound);
        }

        animEventType_t::AEV_SABER_SPIN => {
            let eType = ctx.world.entity(centNum).currentState.eType;
            let stateClientNum = ctx.world.entity(centNum).currentState.clientNum;

            // override spin sound off the client's saber, if it has one
            let overrideSpin: Option<qhandle_t> = {
                // §F19: `clientNum` is server-supplied; an OOB one yields no
                // client (default spin sound) instead of indexing past the table.
                let client: Option<&clientInfo_t> = if eType == entityType_t::ET_NPC as c_int {
                    let c = ctx.world.entity(centNum).npcClient.as_deref();
                    debug_assert!(c.is_some());
                    c
                } else {
                    ctx.world.cgs.clientinfo.get(stateClientNum as usize)
                };
                match client {
                    Some(cl)
                        if cl.infoValid != qfalse
                            && cl.saber[AED_SABER_SPIN_SABERNUM].spinSound != 0 =>
                    {
                        // use override
                        Some(cl.saber[AED_SABER_SPIN_SABERNUM].spinSound)
                    }
                    _ => None,
                }
            };

            let spinSound: qhandle_t = if let Some(s) = overrideSpin {
                s
            } else {
                match animEvent.eventData[AED_SABER_SPIN_TYPE] {
                    0 => trap::S_RegisterSound(engine, "sound/weapons/saber/saberspinoff.wav"),
                    1 => trap::S_RegisterSound(engine, "sound/weapons/saber/saberspin.wav"),
                    2 => trap::S_RegisterSound(engine, "sound/weapons/saber/saberspin1.wav"),
                    3 => trap::S_RegisterSound(engine, "sound/weapons/saber/saberspin2.wav"),
                    4 => trap::S_RegisterSound(engine, "sound/weapons/saber/saberspin3.wav"),
                    // random saberspin1-3
                    _ => trap::S_RegisterSound(
                        engine,
                        &format!(
                            "sound/weapons/saber/saberspin{}.wav",
                            ctx.world.bg_state.rng.Q_irand(1, 3)
                        ),
                    ),
                }
            };

            if spinSound != 0 {
                trap::S_StartSound(engine, None, stateClientNum, CHAN_AUTO, spinSound);
            }
        }

        animEventType_t::AEV_FOOTSTEP => {
            let footStepType = match animEvent.eventData[AED_FOOTSTEP_TYPE] {
                0 => footstepType_t::FOOTSTEP_R,
                1 => footstepType_t::FOOTSTEP_L,
                2 => footstepType_t::FOOTSTEP_HEAVY_R,
                3 => footstepType_t::FOOTSTEP_HEAVY_L,
                // §F19: config-supplied type; an out-of-range value can't form a
                // footstepType_t, so it falls to the left-foot step (which is
                // CG_PlayerFootsteps' own default arm) rather than reinterpreting
                // garbage.
                _ => footstepType_t::FOOTSTEP_L,
            };
            CG_PlayerFootsteps(ctx, centNum, footStepType);
        }

        animEventType_t::AEV_EFFECT => {
            // my method (Raven's `#if 0` SP branch is dropped)
            let ghoul2 = ctx.world.entity(centNum).ghoul2;
            let stringData = animEvent.stringData;

            // SAFETY: `stringData`, when non-NULL, is the anim event's own C string
            // buffer; we read the bone name out of it and, mirroring Raven, NUL its
            // first byte so the bolt lookup only runs once (the engine reads that
            // back on later events, so the write must land).
            if !stringData.is_null() && unsafe { *stringData } != 0 && !ghoul2.is_null() {
                let boneName = unsafe { cstr_to_str(stringData) };
                animEvent.eventData[AED_MODELINDEX] = 0;
                if Q_stricmpn("*blade", &boneName, 6) == 0 || Q_stricmp("*flash", &boneName) == 0 {
                    // must be a weapon, try weapon 0?
                    animEvent.eventData[AED_BOLTINDEX] =
                        trap::G2API_AddBolt(engine, ghoul2, 1, &boneName) as c_short;
                    if animEvent.eventData[AED_BOLTINDEX] != -1 {
                        // found it!
                        animEvent.eventData[AED_MODELINDEX] = 1;
                    } else {
                        // hmm, just try on the player model, then?
                        animEvent.eventData[AED_BOLTINDEX] =
                            trap::G2API_AddBolt(engine, ghoul2, 0, &boneName) as c_short;
                    }
                } else {
                    animEvent.eventData[AED_BOLTINDEX] =
                        trap::G2API_AddBolt(engine, ghoul2, 0, &boneName) as c_short;
                }
                unsafe {
                    *stringData = 0;
                }
            }

            if animEvent.eventData[AED_BOLTINDEX] != -1 {
                let mut lAngles: vec3_t = [0.0; 3];
                let mut bPoint: vec3_t = [0.0; 3];
                let mut bAngle: vec3_t = [0.0; 3];
                let mut matrix = mdxaBone_t {
                    matrix: [[0.0; 4]; 3],
                };

                let lerpAngles = ctx.world.entity(centNum).lerpAngles;
                let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
                let modelScale = ctx.world.entity(centNum).modelScale;
                let time = ctx.world.cg.time;

                VectorSet(&mut lAngles, 0.0, lerpAngles[YAW], 0.0);

                trap::G2API_GetBoltMatrix(
                    engine,
                    ghoul2,
                    animEvent.eventData[AED_MODELINDEX] as c_int,
                    animEvent.eventData[AED_BOLTINDEX] as c_int,
                    &mut matrix,
                    &lAngles,
                    &lerpOrigin,
                    time,
                    Some(&mut ctx.world.cgs.gameModels[0]),
                    &modelScale,
                );
                BG_GiveMeVectorFromMatrix(&matrix, Eorientations::ORIGIN as c_int, &mut bPoint);
                VectorSet(&mut bAngle, 0.0, 1.0, 0.0);

                trap::FX_PlayEffectID(
                    engine,
                    animEvent.eventData[AED_EFFECTINDEX] as c_int,
                    &bPoint,
                    &bAngle,
                    -1,
                    -1,
                );
            } else {
                let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
                let mut bAngle: vec3_t = [0.0; 3];

                VectorSet(&mut bAngle, 0.0, 1.0, 0.0);
                trap::FX_PlayEffectID(
                    engine,
                    animEvent.eventData[AED_EFFECTINDEX] as c_int,
                    &lerpOrigin,
                    &bAngle,
                    -1,
                    -1,
                );
            }
        }

        // Would have to keep track of this on server to for these, it's not worth
        // it. (Raven's AEV_FIRE/AEV_MOVE bodies are commented out.)
        animEventType_t::AEV_FIRE | animEventType_t::AEV_MOVE => {}

        // Raven's `default: return;` — AEV_NONE and the AEV_NUM_AEV sentinel.
        _ => {}
    }
}

/// Raven `CG_PlayerAnimEvents` — walks a parsed legs/torso event list looking
/// for one whose keyframe the legs/torso lerp just crossed this frame, firing
/// it through `CG_PlayerAnimEventDo`. Frame steps bigger than 1 (a lerp that
/// skipped frames, or a new anim starting) fall back to a same-anim/looped
/// range check instead of an exact match.
///
/// `CG_PlayerAnimEventDo` caches its resolved bolt/model index back into the
/// event and NULs `stringData` so the effect lookup only runs once — Raven
/// mutates the entry in the live `bgAllEvents` cache in place, so each hit is
/// copied out, mutated, and written back rather than transcribed as a
/// borrowed reference (which the `ctx` reborrow for the `Do` call rules out).
/// Source: `oracle/codemp/cgame/cg_players.c:2461-2674`
pub fn CG_PlayerAnimEvents(
    ctx: &mut CgContext,
    animFileIndex: c_int,
    eventFileIndex: c_int,
    torso: bool,
    oldFrame: c_int,
    frame: c_int,
    centNum: usize,
) {
    let mut firstFrame: c_int = 0;
    let mut lastFrame: c_int = 0;
    let mut inSameAnim = false;
    let mut loopAnim = false;
    let mut animBackward = false;
    // Raven declares doEvent at fn scope and never resets it in the loop, so
    // once any event fires, every later matching event fires too - probability
    // roll or not. Accidental, but §20 keeps it (the roll still draws).
    let mut doEvent = false;

    //given a range, see if keyFrame falls in that range
    if ((oldFrame - frame) as f32).abs() as f64 > 1.0 {
        let (oldAnim, anim) = if torso {
            let cent = ctx.world.entity(centNum);
            (cent.currentState.torsoAnim, cent.nextState.torsoAnim)
        } else {
            let cent = ctx.world.entity(centNum);
            (cent.currentState.legsAnim, cent.nextState.legsAnim)
        };
        if anim != oldAnim {
            // not in same anim
            inSameAnim = false;
            //FIXME: we *could* see if the oldFrame was *just about* to play the keyframed sound...
        } else {
            // still in same anim, check for looping anim
            inSameAnim = true;

            // `bgAllAnims[..].anims` is still `mp_bg`'s raw `animation_t*` table; the
            // whole workspace reads it through a pointer (see `CG_G2SetHeadAnim`).
            let animations = ctx.world.bg_state.bgAllAnims[animFileIndex as usize].anims;
            // SAFETY: same raw-ptr pattern as `CG_G2SetHeadAnim` — Raven indexes it
            // unchecked and so do we.
            let (frameLerp, loopFrames, animFirstFrame, numFrames) = unsafe {
                let a = &*animations.offset(anim as isize);
                (
                    a.frameLerp,
                    a.loopFrames,
                    a.firstFrame as c_int,
                    a.numFrames as c_int,
                )
            };
            animBackward = frameLerp < 0;
            if loopFrames != -1 {
                // a looping anim!
                loopAnim = true;
                firstFrame = animFirstFrame;
                lastFrame = animFirstFrame + numFrames;
            }
        }
    }

    // Check for anim sound
    for i in 0..MAX_ANIM_EVENTS {
        let animEvent = if torso {
            ctx.world.bg_state.bgAllEvents[eventFileIndex as usize].torsoAnimEvents[i]
        } else {
            ctx.world.bg_state.bgAllEvents[eventFileIndex as usize].legsAnimEvents[i]
        };

        if animEvent.eventType == animEventType_t::AEV_NONE {
            // No event, end of list
            break;
        }

        let mut matched = false;
        if animEvent.keyFrame as c_int == frame {
            // exact match
            matched = true;
        } else if ((oldFrame - frame) as f32).abs() as f64 > 1.0 {
            //given a range, see if keyFrame falls in that range
            if inSameAnim {
                // if changed anims altogether, sorry, the sound is lost
                let keyFrame = animEvent.keyFrame as c_int;
                if ((oldFrame - keyFrame) as f32).abs() as f64 <= 3.0
                    || ((frame - keyFrame) as f32).abs() as f64 <= 3.0
                {
                    //must be at least close to the keyframe
                    if animBackward {
                        // animation plays backwards
                        if oldFrame > keyFrame && frame < keyFrame {
                            // old to new passed through keyframe
                            matched = true;
                        } else if loopAnim {
                            //hmm, didn't pass through it linearally, see if we looped
                            if keyFrame >= firstFrame && keyFrame < lastFrame {
                                //keyframe is in this anim
                                if oldFrame > keyFrame && frame > oldFrame {
                                    // old to new passed through keyframe
                                    matched = true;
                                }
                            }
                        }
                    } else {
                        // anim plays forwards
                        if oldFrame < keyFrame && frame > keyFrame {
                            // old to new passed through keyframe
                            matched = true;
                        } else if loopAnim {
                            //hmm, didn't pass through it linearally, see if we looped
                            if keyFrame >= firstFrame && keyFrame < lastFrame {
                                //keyframe is in this anim
                                if oldFrame < keyFrame && frame < oldFrame {
                                    // old to new passed through keyframe
                                    matched = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        if matched {
            match animEvent.eventType {
                animEventType_t::AEV_SOUND | animEventType_t::AEV_SOUNDCHAN => {
                    // Determine probability of playing sound
                    if animEvent.eventData[AED_SOUND_PROBABILITY] == 0 {
                        // 100%
                        doEvent = true;
                    } else if animEvent.eventData[AED_SOUND_PROBABILITY] as c_int
                        > ctx.world.bg_state.rng.Q_irand(0, 99)
                    {
                        doEvent = true;
                    }
                }
                animEventType_t::AEV_SABER_SWING => {
                    // Determine probability of playing sound
                    if animEvent.eventData[AED_SABER_SWING_PROBABILITY] == 0 {
                        // 100%
                        doEvent = true;
                    } else if animEvent.eventData[AED_SABER_SWING_PROBABILITY] as c_int
                        > ctx.world.bg_state.rng.Q_irand(0, 99)
                    {
                        doEvent = true;
                    }
                }
                animEventType_t::AEV_SABER_SPIN => {
                    // Determine probability of playing sound
                    if animEvent.eventData[AED_SABER_SPIN_PROBABILITY] == 0 {
                        // 100%
                        doEvent = true;
                    } else if animEvent.eventData[AED_SABER_SPIN_PROBABILITY] as c_int
                        > ctx.world.bg_state.rng.Q_irand(0, 99)
                    {
                        doEvent = true;
                    }
                }
                animEventType_t::AEV_FOOTSTEP => {
                    // Determine probability of playing sound
                    if animEvent.eventData[AED_FOOTSTEP_PROBABILITY] == 0 {
                        // 100%
                        doEvent = true;
                    } else if animEvent.eventData[AED_FOOTSTEP_PROBABILITY] as c_int
                        > ctx.world.bg_state.rng.Q_irand(0, 99)
                    {
                        doEvent = true;
                    }
                }
                animEventType_t::AEV_EFFECT => {
                    // Determine probability of playing sound
                    if animEvent.eventData[AED_EFFECT_PROBABILITY] == 0 {
                        // 100%
                        doEvent = true;
                    } else if animEvent.eventData[AED_EFFECT_PROBABILITY] as c_int
                        > ctx.world.bg_state.rng.Q_irand(0, 99)
                    {
                        doEvent = true;
                    }
                }
                animEventType_t::AEV_FIRE => {
                    // Determine probability of playing sound
                    if animEvent.eventData[AED_FIRE_PROBABILITY] == 0 {
                        // 100%
                        doEvent = true;
                    } else if animEvent.eventData[AED_FIRE_PROBABILITY] as c_int
                        > ctx.world.bg_state.rng.Q_irand(0, 99)
                    {
                        doEvent = true;
                    }
                }
                animEventType_t::AEV_MOVE => {
                    doEvent = true;
                }
                _ => {
                    //doEvent = qfalse;//implicit
                }
            }

            // do event
            if doEvent {
                let mut ev = animEvent;
                CG_PlayerAnimEventDo(ctx, centNum, &mut ev);
                // `CG_PlayerAnimEventDo` caches its resolved index into the event
                // and NULs `stringData` so the lookup only runs once - write the
                // mutated copy back into the live parsed-event cache.
                if torso {
                    ctx.world.bg_state.bgAllEvents[eventFileIndex as usize].torsoAnimEvents[i] = ev;
                } else {
                    ctx.world.bg_state.bgAllEvents[eventFileIndex as usize].legsAnimEvents[i] = ev;
                }
            }
        }
    }
}

/// Raven `CG_TriggerAnimSounds` - this also sets the lerp frames, so I suggest you keep calling
/// it regardless of if you want anim sounds.
/// Source: `oracle/codemp/cgame/cg_players.c:2676-2716`
pub fn CG_TriggerAnimSounds(ctx: &mut CgContext, cent: &mut centity_t) {
    let mut curFrame: c_int = 0;
    let mut currentFrame: f32 = 0.0;

    // Raven's assert compiles out in release; debug_assert matches (a
    // non-humanoid GLA with no '/' in its name leaves this -1)
    debug_assert!(cent.localAnimIndex >= 0);

    let sFileIndex = cent.eventAnimIndex;

    let engine = ctx.engine;
    let time = ctx.world.cg.time;

    if trap::G2API_GetBoneFrame(
        engine,
        cent.ghoul2,
        "model_root",
        time,
        &mut currentFrame,
        &mut ctx.world.cgs.gameModels[0],
        0,
    ) {
        // the above may have failed, not sure what to do about it, current frame will be zero in that case
        curFrame = currentFrame.floor() as c_int;
    }
    if curFrame != cent.pe.legs.frame {
        CG_PlayerAnimEvents(
            ctx,
            cent.localAnimIndex,
            sFileIndex,
            false,
            cent.pe.legs.frame,
            curFrame,
            cent.currentState.number as usize,
        );
    }
    cent.pe.legs.oldFrame = cent.pe.torso.frame;
    cent.pe.legs.frame = curFrame;

    if cent.noLumbar != qfalse {
        //probably a droid or something.
        cent.pe.torso.oldFrame = cent.pe.legs.oldFrame;
        cent.pe.torso.frame = cent.pe.legs.frame;
        return;
    }

    if trap::G2API_GetBoneFrame(
        engine,
        cent.ghoul2,
        "lower_lumbar",
        time,
        &mut currentFrame,
        &mut ctx.world.cgs.gameModels[0],
        0,
    ) {
        curFrame = currentFrame.floor() as c_int;
    }
    if curFrame != cent.pe.torso.frame {
        CG_PlayerAnimEvents(
            ctx,
            cent.localAnimIndex,
            sFileIndex,
            true,
            cent.pe.torso.frame,
            curFrame,
            cent.currentState.number as usize,
        );
    }
    cent.pe.torso.oldFrame = cent.pe.torso.frame;
    cent.pe.torso.frame = curFrame;
    cent.pe.torso.backlerp = 1.0 - (currentFrame - curFrame as f32);
}

/// Raven `CG_G2Animated` — the per-frame update for a ghoul2-animated,
/// non-player `ET_NPC`: it loads the model the first frame, reflects the
/// server's surface on/off toggles into the local ghoul2 instance (throwing off
/// debris where a surface just came off), reattaches a severed limb, drives the
/// ragdoll while dead/ragging, smooths the yaw, then hands the whole thing to
/// [`CG_Player`] to actually render.
///
/// The `SMOOTH_G2ANIM_LERPANGLES` block is compiled in (Raven defines it right
/// above the function); the commented-out weapon-copy and `cg_showVehBounds`
/// debug blocks are dropped.
/// Source: `oracle/codemp/cgame/cg_players.c:7463-7578`
pub fn CG_G2Animated(ctx: &mut CgContext, centNum: usize) {
    // SMOOTH_G2ANIM_LERPANGLES
    let angSmoothFactor = 0.7f32;

    if ctx.world.entity(centNum).ghoul2.is_null() {
        //Initialize this g2 anim ent, then return (will start rendering next frame)
        CG_G2AnimEntModelLoad(ctx, centNum);
        ctx.world.entity_mut(centNum).npcLocalSurfOff = 0;
        ctx.world.entity_mut(centNum).npcLocalSurfOn = 0;
        return;
    }

    let surfacesOff = ctx.world.entity(centNum).currentState.surfacesOff;
    let surfacesOn = ctx.world.entity(centNum).currentState.surfacesOn;

    if ctx.world.entity(centNum).npcLocalSurfOff != surfacesOff
        || ctx.world.entity(centNum).npcLocalSurfOn != surfacesOn
    {
        //looks like it's time for an update.
        // PORT-NOTE: Raven's `BG_NUM_TOGGLEABLE_SURFACES` bound equals the length
        // of the `bgToggleableSurfaces` table (31); the `.is_some()` sentinel
        // ends the walk at the same slot Raven's non-NULL test does.
        let mut i: c_int = 0;
        while (i as usize) < bgToggleableSurfaces.len()
            && bgToggleableSurfaces[i as usize].is_some()
        {
            let localSurfOff = ctx.world.entity(centNum).npcLocalSurfOff;
            if (localSurfOff & (1 << i)) == 0 && (surfacesOff & (1 << i)) != 0 {
                //it wasn't off before but it's off now, so reflect this change in the g2 instance.
                if bgToggleableSurfaceDebris[i as usize] > 0 {
                    //make some local debris of this thing?
                    //FIXME: throw off the proper model effect, too
                    let fxID = ctx.world.cgs.effects.mShipDestDestroyed;
                    // CG_CreateSurfaceDebris reads `&centity_t` beside `ctx`.
                    // bitwise copy-in, original left in place - a zeroed-swap is
                    // visible to every ctx-reading helper inside
                    // CG_CreateSurfaceDebris (C6b referee catch). The copy is
                    // read-only, so it does not write back.
                    // SAFETY: `npcClient` is an owned Box, so the ManuallyDrop copy never
                    // drops. The original keeps the one live Box.
                    let cent_tmp =
                        ManuallyDrop::new(unsafe { core::ptr::read(ctx.world.entity(centNum)) });
                    CG_CreateSurfaceDebris(ctx, &cent_tmp, i, fxID, true);
                }

                let surfName = bgToggleableSurfaces[i as usize]
                    .unwrap()
                    .to_str()
                    .unwrap_or("");
                let g2 = ctx.world.entity(centNum).ghoul2;
                trap::G2API_SetSurfaceOnOff(ctx.engine, g2, surfName, TURN_OFF);
            }

            let localSurfOn = ctx.world.entity(centNum).npcLocalSurfOn;
            if (localSurfOn & (1 << i)) == 0 && (surfacesOn & (1 << i)) != 0 {
                //same as above, but on instead of off.
                let surfName = bgToggleableSurfaces[i as usize]
                    .unwrap()
                    .to_str()
                    .unwrap_or("");
                let g2 = ctx.world.entity(centNum).ghoul2;
                trap::G2API_SetSurfaceOnOff(ctx.engine, g2, surfName, TURN_ON);
            }

            i += 1;
        }

        ctx.world.entity_mut(centNum).npcLocalSurfOff = surfacesOff;
        ctx.world.entity_mut(centNum).npcLocalSurfOn = surfacesOn;
    }

    // Raven's commented-out weapon-copy block here is dropped.

    if ctx.world.entity(centNum).torsoBolt != 0
        && (ctx.world.entity(centNum).currentState.eFlags & EF_DEAD) == 0
    {
        //he's alive and has a limb missing still, reattach it and reset the weapon
        CG_ReattachLimb(ctx, centNum);
    }

    let eFlags = ctx.world.entity(centNum).currentState.eFlags;
    if ((eFlags & EF_DEAD) != 0 || (eFlags & EF_RAG) != 0)
        && ctx.world.entity(centNum).localAnimIndex == 0
    {
        let mut forcedAngles: vec3_t = [0.0; 3];

        VectorClear(&mut forcedAngles);
        forcedAngles[YAW] = ctx.world.entity(centNum).lerpAngles[YAW];

        // CG_RagDoll wants `cent` as a `&mut` disjoint from `ctx.world`.
        // bitwise copy-in/copy-back, original left in place - a zeroed-swap is
        // visible to every ctx-reading helper inside CG_RagDoll (C6b referee
        // catch). The copy is written back whole.
        // SAFETY: `npcClient` is an owned Box, so the copy never drops and the
        // ptr::write write-back does not drop the original.
        let mut cent_tmp = ManuallyDrop::new(unsafe { core::ptr::read(ctx.world.entity(centNum)) });
        CG_RagDoll(ctx, &mut cent_tmp, &forcedAngles);
        unsafe {
            core::ptr::write(
                ctx.world.entity_mut(centNum),
                ManuallyDrop::into_inner(cent_tmp),
            )
        };
    }

    // SMOOTH_G2ANIM_LERPANGLES
    let yaw = ctx.world.entity(centNum).lerpAngles[YAW];
    let smoothYaw = ctx.world.entity(centNum).smoothYaw;
    if (yaw > 0.0 && smoothYaw < 0.0) || (yaw < 0.0 && smoothYaw > 0.0) {
        //keep it from snapping around on the threshold
        ctx.world.entity_mut(centNum).smoothYaw = -smoothYaw;
    }
    let smoothYaw = ctx.world.entity(centNum).smoothYaw;
    let yaw = ctx.world.entity(centNum).lerpAngles[YAW];
    let newYaw = smoothYaw + (yaw - smoothYaw) * angSmoothFactor;
    ctx.world.entity_mut(centNum).lerpAngles[YAW] = newYaw;
    ctx.world.entity_mut(centNum).smoothYaw = newYaw;

    //now just render as a player
    CG_Player(ctx, centNum);

    // Raven's commented-out `cg_showVehBounds` bbox debug block here is dropped.
}

/// Resolve the `clientInfo_t` Raven's `ci` local aims at: an `ET_NPC`'s owned
/// `npcClient`, otherwise the shared `cgs.clientinfo` slot. Taken as a
/// short-lived borrow at each use so it never fights the surrounding `ctx.world`
/// accesses, and so the live slot is never swapped out from under callees like
/// `CG_AddSaberBlade` that re-read it by client number (DEC-46.2).
fn player_ci(world: &CgWorld, is_npc: bool, centNum: usize, clientNum: c_int) -> &clientInfo_t {
    if is_npc {
        world.entity(centNum).npcClient.as_deref().unwrap()
    } else {
        &world.cgs.clientinfo[clientNum as usize]
    }
}

/// Mutable twin of [`player_ci`].
fn player_ci_mut(
    world: &mut CgWorld,
    is_npc: bool,
    centNum: usize,
    clientNum: c_int,
) -> &mut clientInfo_t {
    if is_npc {
        world.entity_mut(centNum).npcClient.as_deref_mut().unwrap()
    } else {
        &mut world.cgs.clientinfo[clientNum as usize]
    }
}

/// [`CG_PlayerFloatSprite`] reads `&centity_t` beside `ctx`; the entity stays
/// authoritative for callees that re-read it by number.
fn cg_player_float_sprite(ctx: &mut CgContext, centNum: usize, shader: qhandle_t) {
    // bitwise copy-in, original left in place - a zeroed-swap is visible to
    // every ctx-reading helper inside CG_PlayerFloatSprite (C6b referee catch).
    // The copy is read-only, so it does not write back.
    // SAFETY: `npcClient` is an owned Box, so the ManuallyDrop copy never
    // drops. The original keeps the one live Box.
    let cent_tmp = ManuallyDrop::new(unsafe { core::ptr::read(ctx.world.entity(centNum)) });
    CG_PlayerFloatSprite(ctx, &cent_tmp, shader);
}

/// Same copy for [`CG_DrawPlayerSphere`], which reads `&centity_t`.
fn cg_draw_player_sphere(
    ctx: &mut CgContext,
    centNum: usize,
    origin: &vec3_t,
    scale: f32,
    shader: c_int,
) {
    // bitwise copy-in, original left in place - a zeroed-swap is visible to
    // every ctx-reading helper inside CG_DrawPlayerSphere (C6b referee catch).
    // The copy is read-only, so it does not write back.
    // SAFETY: `npcClient` is an owned Box, so the ManuallyDrop copy never
    // drops. The original keeps the one live Box.
    let cent_tmp = ManuallyDrop::new(unsafe { core::ptr::read(ctx.world.entity(centNum)) });
    CG_DrawPlayerSphere(ctx, &cent_tmp, origin, scale, shader);
}

/// Raven `CG_Player` — the whole client-entity render: smoothing, vehicle
/// attach, weapon-model swaps, team/duel sprites, shadow, force-power effects,
/// the saber(s), cloak, speed trails and every powerup shell, all funnelled into
/// `trap_R_AddRefEntityToScene` calls.
///
/// Threading (DEC-46.1/.2): the entity stays in the arena, reached inline as
/// `ctx.world.entity(centNum)`; callees that want a `&mut centity_t` disjoint
/// from `ctx.world` (the anim/angles/headanim/trigger/attach family) get it via
/// a take/put-back swap with [`centity_t::zeroed`], and `ci` is resolved in
/// place through [`player_ci`]/[`player_ci_mut`].
///
/// The vehicle arms (rider attach, smoothing, shield shader) read the
/// DEC-47.3 pool directly; the rider-attach branch calls bg's
/// `AttachRidersGeneric`, the fn Raven's CGAME build binds to the retired
/// `AttachRiders` slot.
/// Source: `oracle/codemp/cgame/cg_players.c:8423-11107`
#[allow(clippy::needless_range_loop)]
pub fn CG_Player(ctx: &mut CgContext, centNum: usize) {
    let engine = ctx.engine;

    let mut legs = refEntity_t::zeroed();
    let mut torso: refEntity_t;
    let mut rootAngles: vec3_t = [0.0; 3];
    let mut renderfx: c_int;
    let shadow: bool;
    let mut shadowPlane: f32 = 0.0;
    let mut dead = false;
    let mut angle: f32;
    let mut angles: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let mut elevated: vec3_t = [0.0; 3];
    let mut seekorg: vec3_t = [0.0; 3];
    let mut iwantout: c_int = 0;
    let mut successchange: c_int;
    let team: c_int;
    let mut boltMatrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };
    let mut lHandMatrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };
    let mut doAlpha: c_int = 0;
    let mut gotLHandMatrix: bool;
    let mut g2HasWeapon: bool;
    let mut drawPlayerSaber = false;
    let mut checkDroidShields = false;

    //first if we are not an npc and we are using an emplaced gun then make sure our
    //angles are visually capped to the constraints (otherwise it's possible to lerp
    //a little outside and look kind of twitchy)
    if ctx.world.entity(centNum).currentState.weapon == WP_EMPLACED_GUN
        && ctx.world.entity(centNum).currentState.otherEntityNum2 != 0
    {
        let otherNum = ctx.world.entity(centNum).currentState.otherEntityNum2 as usize;
        let baseAngles = ctx.world.entity(centNum).lerpAngles;
        let otherAngles = ctx.world.entity(otherNum).currentState.angles;
        let constraint = ctx.world.entity(otherNum).currentState.origin2[0];
        let mut empYaw: f32 = 0.0;
        if BG_EmplacedView(baseAngles, otherAngles, &mut empYaw, constraint) != 0 {
            ctx.world.entity_mut(centNum).lerpAngles[YAW] = empYaw;
        }
    }

    if ctx.world.entity(centNum).currentState.iModelScale != 0 {
        //if the server says we have a custom scale then set it now.
        let s = ctx.world.entity(centNum).currentState.iModelScale as f32 / 100.0;
        ctx.world.entity_mut(centNum).modelScale = [s, s, s];
        if ctx.world.entity(centNum).currentState.NPC_class != class_t::CLASS_VEHICLE as c_int {
            let ms2 = ctx.world.entity(centNum).modelScale[2];
            if ms2 != 0.0 && ms2 != 1.0 {
                ctx.world.entity_mut(centNum).lerpOrigin[2] += 24.0 * (ms2 - 1.0);
            }
        }
    } else {
        VectorClear(&mut ctx.world.entity_mut(centNum).modelScale);
    }

    let smoothClients = ctx.world.cvars.cg_smoothClients.integer != 0;
    let heldByClient = ctx.world.entity(centNum).currentState.heldByClient;
    let groundEntityNum = ctx.world.entity(centNum).currentState.groundEntityNum;
    let eType0 = ctx.world.entity(centNum).currentState.eType;
    let eFlags2_0 = ctx.world.entity(centNum).currentState.eFlags2;
    let number0 = ctx.world.entity(centNum).currentState.number;
    if (smoothClients || heldByClient != 0)
        && (groundEntityNum >= ENTITYNUM_WORLD || eType0 == entityType_t::ET_TERRAIN as c_int)
        && (eFlags2_0 & EF2_HYPERSPACE) == 0
        && ctx.world.cg.predictedPlayerState.m_iVehicleNum != number0
    {
        //always smooth when being thrown
        let mut fTolerance = 20000.0f32;
        let smoothFactor: f32;

        if heldByClient != 0 {
            //smooth the origin more when in this state, because movement is origin-based on server.
            smoothFactor = 0.2;
        } else if (ctx.world.entity(centNum).currentState.powerups & (1 << PW_SPEED)) != 0
            || (ctx.world.entity(centNum).currentState.forcePowersActive & (1 << FP_RAGE)) != 0
        {
            //we're moving fast so don't smooth as much
            smoothFactor = 0.6;
        } else if eType0 == entityType_t::ET_NPC as c_int
            && ctx.world.entity(centNum).currentState.NPC_class == class_t::CLASS_VEHICLE as c_int
            && ctx
                .world
                .entity(centNum)
                .m_pVehicle
                .is_some_and(|id| unsafe {
                    (*ctx.world.vehicle_pool[id.ent_num() as usize].m_pVehicleInfo).r#type
                        == vehicleType_t::VH_FIGHTER
                })
        {
            //greater smoothing for flying vehicles, since they move so fast
            fTolerance = 6000000.0; //500000.0f; //yeah, this is so wrong..but..
            smoothFactor = 0.5;
        } else {
            smoothFactor = 0.5;
        }

        let beamEnd = ctx.world.entity(centNum).beamEnd;
        let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
        if DistanceSquared(beamEnd, lerpOrigin) > smoothFactor * fTolerance {
            //10000
            ctx.world.entity_mut(centNum).beamEnd = lerpOrigin;
        }

        let mut posDif: vec3_t = [0.0; 3];
        let beamEnd = ctx.world.entity(centNum).beamEnd;
        let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
        _VectorSubtract(lerpOrigin, beamEnd, &mut posDif);

        for k in 0..3 {
            let nv = ctx.world.entity(centNum).beamEnd[k] + posDif[k] * smoothFactor;
            ctx.world.entity_mut(centNum).beamEnd[k] = nv;
            ctx.world.entity_mut(centNum).lerpOrigin[k] = nv;
        }
    } else {
        let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
        ctx.world.entity_mut(centNum).beamEnd = lerpOrigin;
    }

    if ctx.world.entity(centNum).currentState.m_iVehicleNum != 0
        && ctx.world.entity(centNum).currentState.NPC_class != class_t::CLASS_VEHICLE as c_int
    {
        //this player is riding a vehicle
        let vehNum = ctx.world.entity(centNum).currentState.m_iVehicleNum as usize;

        let vehYaw = ctx.world.entity(vehNum).lerpAngles[YAW];
        ctx.world.entity_mut(centNum).lerpAngles[YAW] = vehYaw;

        //Attach ourself to the vehicle
        let vehHasVehicle = ctx.world.entity(vehNum).m_pVehicle.is_some();
        let centHasPs = ctx.world.entity(centNum).playerState != PlayerStateRef::None;
        let vehHasPs = ctx.world.entity(vehNum).playerState != PlayerStateRef::None;
        let centGhoul2 = ctx.world.entity(centNum).ghoul2;
        let vehGhoul2 = ctx.world.entity(vehNum).ghoul2;
        if vehHasVehicle && centHasPs && vehHasPs && !centGhoul2.is_null() && !vehGhoul2.is_null() {
            let owner = ctx.world.entity(vehNum).currentState.owner;
            let centClientNum = ctx.world.entity(centNum).currentState.clientNum;
            if owner != centClientNum {
                //FIXME: what about visible passengers?
                // bitwise copy-in/copy-back, original left in place - a
                // zeroed-swap is visible to every ctx-reading helper inside
                // CG_VehicleAttachDroidUnit (C6b referee catch). The copy is
                // written back whole.
                // SAFETY: `npcClient` is an owned Box, so the copy never drops and the
                // ptr::write write-back does not drop the original.
                let mut cent_tmp =
                    ManuallyDrop::new(unsafe { core::ptr::read(ctx.world.entity(centNum)) });
                let attached = CG_VehicleAttachDroidUnit(ctx, &mut cent_tmp, &legs);
                unsafe {
                    core::ptr::write(
                        ctx.world.entity_mut(centNum),
                        ManuallyDrop::into_inner(cent_tmp),
                    )
                };
                if attached {
                    checkDroidShields = true;
                }
            } else if owner != ENTITYNUM_NONE {
                //has a pilot...???

                // Raven calls the `m_pVehicleInfo->AttachRiders` fn-ptr slot,
                // which the Rust `vehicleInfo_t` retired - every CGAME-side
                // setter binds it to bg's `AttachRidersGeneric` (the `#ifndef
                // QAGAME` arm, e.g. AnimalNPC.c:879-881), so the call is
                // direct.
                // Source: oracle/codemp/cgame/cg_players.c:8544-8562
                if let Some(vehId) = ctx.world.entity(vehNum).m_pVehicle {
                    let row_idx = vehId.ent_num() as usize;
                    let (pVeh, veh_ps, cent_ps, vehLerpOrigin, vehLerpAngles, centLerpOrigin) = {
                        let world = &mut *ctx.world;

                        // re-sync the two bg rows first - the predict-time
                        // sync can be a model-reload stale by draw time, and
                        // the attach reads ghoul2/modelScale through them
                        for i in [vehNum, centNum] {
                            let m_pVehicle = match world.entities[i].m_pVehicle {
                                Some(id) => &raw mut world.vehicle_pool[id.ent_num() as usize],
                                None => null_mut(),
                            };
                            let ent = &world.entities[i];
                            let row = &mut world.bg_ents[i];
                            row.s = ent.currentState;
                            row.ghoul2 = ent.ghoul2;
                            row.localAnimIndex = ent.localAnimIndex;
                            row.modelScale = ent.modelScale;
                            row.m_pVehicle = m_pVehicle;
                        }

                        //make sure it has its pilot and parent set
                        world.vehicle_pool[row_idx].m_pPilot =
                            &raw mut world.bg_ents[owner as usize];
                        world.vehicle_pool[row_idx].m_pParentEntity =
                            &raw mut world.bg_ents[vehNum];

                        (
                            &raw mut world.vehicle_pool[row_idx],
                            world.bg_ents[vehNum].playerState,
                            world.bg_ents[centNum].playerState,
                            world.entities[vehNum].lerpOrigin,
                            world.entities[vehNum].lerpAngles,
                            world.entities[centNum].lerpOrigin,
                        )
                    };
                    let time = ctx.world.cg.time;

                    // SAFETY: both rows' playerState pointers are wired
                    // (CG_PmoveClientPointerUpdate + the predict repoints) and
                    // gated non-None above; no Rust borrow of their targets is
                    // live across the bg call
                    unsafe {
                        let oldPSOrg = (*veh_ps).origin;

                        //update the veh's playerstate org for getting the bolt
                        (*veh_ps).origin = vehLerpOrigin;
                        (*cent_ps).origin = centLerpOrigin;

                        //Now do the attach
                        (*veh_ps).viewangles = vehLerpAngles;
                        {
                            let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
                            AttachRidersGeneric(pVeh, &traps, time);
                        }

                        //copy the "playerstate origin" to the lerpOrigin since that's what we use to display
                        ctx.world.entity_mut(centNum).lerpOrigin = (*cent_ps).origin;

                        (*veh_ps).origin = oldPSOrg;
                    }
                }
            }
        }
    }

    // the client number is stored in clientNum.  It can't be derived
    // from the entity number, because a single client may have
    // multiple corpses on the level using the same clientinfo
    let is_npc = ctx.world.entity(centNum).currentState.eType == entityType_t::ET_NPC as c_int;
    let clientNum: c_int;
    if !is_npc {
        clientNum = ctx.world.entity(centNum).currentState.clientNum;
        if clientNum < 0 || clientNum >= MAX_CLIENTS_I32 {
            CG_Error(ctx, "Bad clientNum on player entity");
            return;
        }
    } else {
        clientNum = 0; // unused for NPCs — ci lives on the entity
        if ctx.world.entity(centNum).npcClient.is_none() {
            //allocate memory for it
            // Raven's `assert(0); return;` on a NULL alloc is unreachable: the box
            // allocation panics on OOM rather than returning NULL, and
            // CG_CreateNPCClient hands back a zeroed block (the memset +
            // ghoul2Model = NULL that follow in Raven are already satisfied).
            ctx.world.entity_mut(centNum).npcClient = Some(CG_CreateNPCClient());
        }

        let centGhoul2 = ctx.world.entity(centNum).ghoul2;
        let localAnimIndex = ctx.world.entity(centNum).localAnimIndex;
        let ciGhoul2Model = ctx
            .world
            .entity(centNum)
            .npcClient
            .as_deref()
            .unwrap()
            .ghoul2Model;
        if ciGhoul2Model != centGhoul2 && !centGhoul2.is_null() {
            {
                let ci = ctx
                    .world
                    .entity_mut(centNum)
                    .npcClient
                    .as_deref_mut()
                    .unwrap();
                ci.ghoul2Model = centGhoul2;
            }
            if localAnimIndex <= 1 {
                let g2 = centGhoul2;
                let rhand = trap::G2API_AddBolt(engine, g2, 0, "*r_hand");
                let lhand = trap::G2API_AddBolt(engine, g2, 0, "*l_hand");

                //rhand must always be first bolt. lhand always second. Whichever you want the
                //jetpack bolted to must always be third.
                trap::G2API_AddBolt(engine, g2, 0, "*chestg");

                //claw bolts
                trap::G2API_AddBolt(engine, g2, 0, "*r_hand_cap_r_arm");
                trap::G2API_AddBolt(engine, g2, 0, "*l_hand_cap_l_arm");

                let mut head = trap::G2API_AddBolt(engine, g2, 0, "*head_top");
                if head == -1 {
                    head = trap::G2API_AddBolt(engine, g2, 0, "ceyebrow");
                }
                let motion = trap::G2API_AddBolt(engine, g2, 0, "Motion");
                let llumbar = trap::G2API_AddBolt(engine, g2, 0, "lower_lumbar");

                let ci = ctx
                    .world
                    .entity_mut(centNum)
                    .npcClient
                    .as_deref_mut()
                    .unwrap();
                ci.bolt_rhand = rhand;
                ci.bolt_lhand = lhand;
                ci.bolt_head = head;
                ci.bolt_motion = motion;
                ci.bolt_llumbar = llumbar;
            } else {
                let ci = ctx
                    .world
                    .entity_mut(centNum)
                    .npcClient
                    .as_deref_mut()
                    .unwrap();
                ci.bolt_rhand = -1;
                ci.bolt_lhand = -1;
                ci.bolt_head = -1;
                ci.bolt_motion = -1;
                ci.bolt_llumbar = -1;
            }
            let ci = ctx
                .world
                .entity_mut(centNum)
                .npcClient
                .as_deref_mut()
                .unwrap();
            ci.team = TEAM_FREE;
            ci.infoValid = qtrue;
        }
    }

    // it is possible to see corpses from disconnected players that may
    // not have valid clientinfo
    if player_ci(ctx.world, is_npc, centNum, clientNum).infoValid == qfalse {
        return;
    }

    // cg.snap anchors every viewer-relative test below. Raven dereferences it
    // unconditionally in render; §F19: with no snapshot there is nothing to draw
    // against, so skip the whole entity.
    let (
        snapClientNum,
        snapPersTeam,
        snapPmFlags,
        snapDuelInProgress,
        snapDuelIndex,
        snapIsJediMaster,
        snapOrigin,
        snapMt1,
        snapMt2,
        snapMt3,
        snapMt4,
        snapFdForceActive,
        snapSeeLevel,
    ) = match ctx.world.cg.snap_ref() {
        Some(s) => (
            s.ps.clientNum,
            s.ps.persistant[PERS_TEAM as usize],
            s.ps.pm_flags,
            s.ps.duelInProgress,
            s.ps.duelIndex,
            s.ps.isJediMaster,
            s.ps.origin,
            s.ps.fd.forceMindtrickTargetIndex,
            s.ps.fd.forceMindtrickTargetIndex2,
            s.ps.fd.forceMindtrickTargetIndex3,
            s.ps.fd.forceMindtrickTargetIndex4,
            s.ps.fd.forcePowersActive,
            s.ps.fd.forcePowerLevel[FP_SEE as usize],
        ),
        None => return,
    };

    // Add the player to the radar if on the same team and its a team game
    if ctx.world.cgs.gametype >= GT_TEAM {
        let eType = ctx.world.entity(centNum).currentState.eType;
        let number = ctx.world.entity(centNum).currentState.number;
        let ciTeam = player_ci(ctx.world, is_npc, centNum, clientNum).team;
        if eType != entityType_t::ET_NPC as c_int
            && snapClientNum != number
            && ciTeam == snapPersTeam
        {
            CG_AddRadarEnt(ctx.world, centNum);
        }
    }

    if ctx.world.entity(centNum).currentState.eType == entityType_t::ET_NPC as c_int
        && ctx.world.entity(centNum).currentState.NPC_class == class_t::CLASS_VEHICLE as c_int
    {
        //add vehicles
        CG_AddRadarEnt(ctx.world, centNum);
        if CG_InFighter(ctx.world) {
            //this is a vehicle, bracket it
            let clientNumState = ctx.world.entity(centNum).currentState.clientNum;
            if ctx.world.cg.predictedPlayerState.m_iVehicleNum != clientNumState {
                //don't add the vehicle I'm in... :)
                CG_AddBracketedEnt(ctx.world, centNum);
            }
        }
    }

    if ctx.world.entity(centNum).ghoul2.is_null() {
        //not ready yet?
        // (Raven's _DEBUG Com_Printf dropped — release parity.)
        let number = ctx.world.entity(centNum).currentState.number;
        trap::G2API_ClearAttachedInstance(engine, number);

        let ciGhoul2Model = player_ci(ctx.world, is_npc, centNum, clientNum).ghoul2Model;
        if !ciGhoul2Model.is_null() && trap::G2_HaveWeGhoul2Models(engine, ciGhoul2Model) {
            let mut newG2: *mut c_void = null_mut();
            trap::G2API_DuplicateGhoul2Instance(engine, ciGhoul2Model, &mut newG2);
            ctx.world.entity_mut(centNum).ghoul2 = newG2;

            //Attach the instance to this entity num so we can make use of client-server
            //shared operations if possible.
            trap::G2API_AttachInstanceToEntNum(engine, newG2, number, false);

            if trap::G2API_AddBolt(engine, newG2, 0, "face") == -1 {
                //check now to see if we have this bone for setting anims and such
                ctx.world.entity_mut(centNum).noFace = qtrue;
            }

            let localAnimIndex = CG_G2SkelForModel(ctx, newG2);
            ctx.world.entity_mut(centNum).localAnimIndex = localAnimIndex;
            let eventAnimIndex = CG_G2EvIndexForModel(ctx, newG2, localAnimIndex);
            ctx.world.entity_mut(centNum).eventAnimIndex = eventAnimIndex;
        }
        return;
    }

    if player_ci(ctx.world, is_npc, centNum, clientNum).superSmoothTime != 0 {
        //do crazy smoothing
        let superSmoothTime = player_ci(ctx.world, is_npc, centNum, clientNum).superSmoothTime;
        let g2 = ctx.world.entity(centNum).ghoul2;
        if superSmoothTime > ctx.world.cg.time {
            //do it
            trap::G2API_AbsurdSmoothing(engine, g2, true);
        } else {
            //turn it off
            player_ci_mut(ctx.world, is_npc, centNum, clientNum).superSmoothTime = 0;
            trap::G2API_AbsurdSmoothing(engine, g2, false);
        }
    }

    if ctx.world.cg.predictedPlayerState.pm_type == PM_INTERMISSION as c_int {
        //don't show all this shit during intermission
        let eType = ctx.world.entity(centNum).currentState.eType;
        let npcClass = ctx.world.entity(centNum).currentState.NPC_class;
        if eType == entityType_t::ET_NPC as c_int && npcClass != class_t::CLASS_VEHICLE as c_int {
            //NPC in intermission
        } else {
            //don't render players or vehicles in intermissions, allow other NPCs for scripts
            return;
        }
    }

    {
        // CG_VehicleEffects reads `&centity_t`.
        // bitwise copy-in, original left in place - a zeroed-swap is visible to
        // every ctx-reading helper inside CG_VehicleEffects (C6b referee catch).
        // The copy is read-only, so it does not write back.
        // SAFETY: `npcClient` is an owned Box, so the ManuallyDrop copy never
        // drops. The original keeps the one live Box.
        let cent_tmp = ManuallyDrop::new(unsafe { core::ptr::read(ctx.world.entity(centNum)) });
        CG_VehicleEffects(ctx, &cent_tmp);
    }

    let eFlags = ctx.world.entity(centNum).currentState.eFlags;
    if (eFlags & EF_JETPACK) != 0
        && (eFlags & EF_DEAD) == 0
        && !ctx.world.players.cg_g2JetpackInstance.is_null()
    {
        //should have a jetpack attached
        //1 is rhand weap, 2 is lhand weap (akimbo sabs), 3 is jetpack
        let g2 = ctx.world.entity(centNum).ghoul2;
        if !trap::G2API_HasGhoul2ModelOnIndex(
            engine,
            &raw mut ctx.world.entity_mut(centNum).ghoul2,
            3,
        ) {
            let jet = ctx.world.players.cg_g2JetpackInstance;
            trap::G2API_CopySpecificGhoul2Model(engine, jet, 0, g2, 3);
        }

        if (eFlags & EF_JETPACK_ACTIVE) != 0 {
            let mut n = 0;
            while n < 2 {
                //Get the position/dir of the flame bolt on the jetpack model bolted to the player
                let mut mat = mdxaBone_t {
                    matrix: [[0.0; 4]; 3],
                };
                let g2 = ctx.world.entity(centNum).ghoul2;
                let turAngles = ctx.world.entity(centNum).turAngles;
                let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
                let modelScale = ctx.world.entity(centNum).modelScale;
                let time = ctx.world.cg.time;
                trap::G2API_GetBoltMatrix(
                    engine,
                    g2,
                    3,
                    n,
                    &mut mat,
                    &turAngles,
                    &lerpOrigin,
                    time,
                    Some(&mut ctx.world.cgs.gameModels[0]),
                    &modelScale,
                );
                let mut flamePos: vec3_t = [0.0; 3];
                let mut flameDir: vec3_t = [0.0; 3];
                BG_GiveMeVectorFromMatrix(&mat, Eorientations::ORIGIN as c_int, &mut flamePos);

                if n == 0 {
                    BG_GiveMeVectorFromMatrix(
                        &mat,
                        Eorientations::NEGATIVE_Y as c_int,
                        &mut flameDir,
                    );
                    let base = flamePos;
                    _VectorMA(base, -9.5, flameDir, &mut flamePos);
                    BG_GiveMeVectorFromMatrix(
                        &mat,
                        Eorientations::POSITIVE_X as c_int,
                        &mut flameDir,
                    );
                    let base = flamePos;
                    _VectorMA(base, -13.5, flameDir, &mut flamePos);
                } else {
                    BG_GiveMeVectorFromMatrix(
                        &mat,
                        Eorientations::POSITIVE_X as c_int,
                        &mut flameDir,
                    );
                    let base = flamePos;
                    _VectorMA(base, -9.5, flameDir, &mut flamePos);
                    BG_GiveMeVectorFromMatrix(
                        &mat,
                        Eorientations::NEGATIVE_Y as c_int,
                        &mut flameDir,
                    );
                    let base = flamePos;
                    _VectorMA(base, -13.5, flameDir, &mut flamePos);
                }

                let jetFx = ctx.world.cgs.effects.mBobaJet;
                if (ctx.world.entity(centNum).currentState.eFlags & EF_JETPACK_FLAMING) != 0 {
                    //create effects
                    //FIXME: Just one big effect
                    //Play the effect
                    trap::FX_PlayEffectID(engine, jetFx, &flamePos, &flameDir, -1, -1);
                    trap::FX_PlayEffectID(engine, jetFx, &flamePos, &flameDir, -1, -1);

                    //Keep the jet fire sound looping
                    let number = ctx.world.entity(centNum).currentState.number;
                    let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
                    let sfx = trap::S_RegisterSound(engine, "sound/effects/fire_lp");
                    trap::S_AddLoopingSound(engine, number, &lerpOrigin, &vec3_origin, sfx);
                } else {
                    //just idling
                    //FIXME: Different smaller effect for idle
                    //Play the effect
                    trap::FX_PlayEffectID(engine, jetFx, &flamePos, &flameDir, -1, -1);
                }

                n += 1;
            }

            let number = ctx.world.entity(centNum).currentState.number;
            let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
            let sfx = trap::S_RegisterSound(engine, "sound/boba/JETHOVER");
            trap::S_AddLoopingSound(engine, number, &lerpOrigin, &vec3_origin, sfx);
        }
    } else if trap::G2API_HasGhoul2ModelOnIndex(
        engine,
        &raw mut ctx.world.entity_mut(centNum).ghoul2,
        3,
    ) {
        //fixme: would be good if this could be done not every frame
        trap::G2API_RemoveGhoul2Model(engine, &raw mut ctx.world.entity_mut(centNum).ghoul2, 3);
    }

    g2HasWeapon =
        trap::G2API_HasGhoul2ModelOnIndex(engine, &raw mut ctx.world.entity_mut(centNum).ghoul2, 1);

    if !g2HasWeapon {
        //force a redup of the weapon instance onto the client instance
        ctx.world.entity_mut(centNum).ghoul2weapon = null_mut();
        ctx.world.entity_mut(centNum).weapon = 0;
    }

    if ctx.world.entity(centNum).torsoBolt != 0
        && (ctx.world.entity(centNum).currentState.eFlags & EF_DEAD) == 0
    {
        //he's alive and has a limb missing still, reattach it and reset the weapon
        CG_ReattachLimb(ctx, centNum);
    }

    if ctx.world.entity(centNum).isRagging != qfalse
        && (ctx.world.entity(centNum).currentState.eFlags & EF_DEAD) == 0
        && (ctx.world.entity(centNum).currentState.eFlags & EF_RAG) == 0
    {
        //make sure we don't ragdoll ever while alive unless directly told to with eFlags
        ctx.world.entity_mut(centNum).isRagging = qfalse;
        let g2 = ctx.world.entity(centNum).ghoul2;
        trap::G2API_SetRagDoll(engine, g2, None); //calling with null parms resets to no ragdoll.
    }

    let torsoBolt = ctx.world.entity(centNum).torsoBolt;
    if !ctx.world.entity(centNum).ghoul2.is_null()
        && torsoBolt != 0
        && ((torsoBolt & RARMBIT) != 0
            || (torsoBolt & RHANDBIT) != 0
            || (torsoBolt & WAISTBIT) != 0)
        && g2HasWeapon
    {
        //kill the weapon if the limb holding it is no longer on the model
        trap::G2API_RemoveGhoul2Model(engine, &raw mut ctx.world.entity_mut(centNum).ghoul2, 1);
        g2HasWeapon = false;
    }

    let cgTime = ctx.world.cg.time;
    if ctx.world.entity(centNum).trickAlphaTime == 0
        || (cgTime - ctx.world.entity(centNum).trickAlphaTime) > 1000
    {
        //things got out of sync, perhaps a new client is trying to fill in this slot
        ctx.world.entity_mut(centNum).trickAlpha = 255;
        ctx.world.entity_mut(centNum).trickAlphaTime = cgTime;
    }

    if (ctx.world.entity(centNum).currentState.eFlags & EF_NODRAW) != 0 {
        //If nodraw, return here
        return;
    } else if (ctx.world.entity(centNum).currentState.eFlags2 & EF2_SHIP_DEATH) != 0 {
        //died in ship, don't draw, we were "obliterated"
        return;
    }

    //If this client has tricked you.
    let ti1 = ctx.world.entity(centNum).currentState.trickedentindex;
    let ti2 = ctx.world.entity(centNum).currentState.trickedentindex2;
    let ti3 = ctx.world.entity(centNum).currentState.trickedentindex3;
    let ti4 = ctx.world.entity(centNum).currentState.trickedentindex4;
    if CG_IsMindTricked(ctx.world, ti1, ti2, ti3, ti4, snapClientNum) {
        if ctx.world.entity(centNum).trickAlpha > 1 {
            let cgTime = ctx.world.cg.time;
            let trickAlphaTime = ctx.world.entity(centNum).trickAlphaTime;
            // Raven computes `trickAlpha - (dt)*0.5` in double, then truncates.
            let newAlpha = (ctx.world.entity(centNum).trickAlpha as f64
                - (cgTime - trickAlphaTime) as f64 * 0.5) as c_int;
            ctx.world.entity_mut(centNum).trickAlpha = newAlpha;
            ctx.world.entity_mut(centNum).trickAlphaTime = cgTime;

            if ctx.world.entity(centNum).trickAlpha < 0 {
                ctx.world.entity_mut(centNum).trickAlpha = 0;
            }

            doAlpha = 1;
        } else {
            doAlpha = 1;
            ctx.world.entity_mut(centNum).trickAlpha = 1;
            ctx.world.entity_mut(centNum).trickAlphaTime = ctx.world.cg.time;
            iwantout = 1;
        }
    } else if ctx.world.entity(centNum).trickAlpha < 255 {
        let cgTime = ctx.world.cg.time;
        let trickAlphaTime = ctx.world.entity(centNum).trickAlphaTime;
        let newAlpha = ctx.world.entity(centNum).trickAlpha + (cgTime - trickAlphaTime);
        ctx.world.entity_mut(centNum).trickAlpha = newAlpha;
        ctx.world.entity_mut(centNum).trickAlphaTime = cgTime;

        if ctx.world.entity(centNum).trickAlpha > 255 {
            ctx.world.entity_mut(centNum).trickAlpha = 255;
        }

        doAlpha = 1;
    } else {
        ctx.world.entity_mut(centNum).trickAlpha = 255;
        ctx.world.entity_mut(centNum).trickAlphaTime = ctx.world.cg.time;
    }

    // get the player model information
    renderfx = 0;
    if ctx.world.entity(centNum).currentState.number == snapClientNum {
        if ctx.world.cg.renderingThirdPerson == qfalse {
            if ctx.world.entity(centNum).currentState.weapon != WP_SABER {
                renderfx = RF_THIRD_PERSON; // only draw in mirrors
            }
        } else if ctx.world.cvars.cg_cameraMode.integer != 0 {
            // Raven sets `iwantout = 1` here but returns immediately (goto minimal_add
            // commented out) - dead store dropped.
            // goto minimal_add;
            // NOTENOTE Temporary
            return;
        }
    }

    // Update the player's client entity information regarding weapons.
    // rww - Make sure weapons don't get set BEFORE cent->ghoul2 is initialized or else we'll have no
    // weapon bolted on
    if ctx.world.entity(centNum).currentState.saberInFlight != qfalse {
        let world = &*ctx.world;
        let gw = CG_G2WeaponInstance(world, world.entity(centNum), WP_SABER);
        ctx.world.entity_mut(centNum).ghoul2weapon = gw;
    }

    let stateWeapon = ctx.world.entity(centNum).currentState.weapon;
    let curWeapInstance = {
        let world = &*ctx.world;
        CG_G2WeaponInstance(world, world.entity(centNum), stateWeapon)
    };
    let npcClass = ctx.world.entity(centNum).currentState.NPC_class;
    let eType = ctx.world.entity(centNum).currentState.eType;
    let eFlags = ctx.world.entity(centNum).currentState.eFlags;
    let number = ctx.world.entity(centNum).currentState.number;
    if !ctx.world.entity(centNum).ghoul2.is_null()
        && (eType != entityType_t::ET_NPC as c_int
            || (npcClass != class_t::CLASS_VEHICLE as c_int
                && npcClass != class_t::CLASS_REMOTE as c_int
                && npcClass != class_t::CLASS_SEEKER as c_int)) //don't add weapon models to NPCs that have no bolt for them!
        && ctx.world.entity(centNum).ghoul2weapon != curWeapInstance
        && (eFlags & EF_DEAD) == 0
        && ctx.world.entity(centNum).torsoBolt == 0
        && (number != snapClientNum || (snapPmFlags & PMF_FOLLOW) != 0)
    {
        if player_ci(ctx.world, is_npc, centNum, clientNum).team == TEAM_SPECTATOR {
            ctx.world.entity_mut(centNum).ghoul2weapon = null_mut();
            ctx.world.entity_mut(centNum).weapon = 0;
        } else {
            let g2 = ctx.world.entity(centNum).ghoul2;
            CG_CopyG2WeaponInstance(ctx, centNum, stateWeapon, g2);

            if ctx.world.entity(centNum).currentState.eType != entityType_t::ET_NPC as c_int {
                let centWeapon = ctx.world.entity(centNum).weapon;
                let saberHolstered = ctx.world.entity(centNum).currentState.saberHolstered;
                if centWeapon == WP_SABER && centWeapon != stateWeapon && saberHolstered == 0 {
                    //switching away from the saber
                    let s0Off = player_ci(ctx.world, is_npc, centNum, clientNum).saber[0].soundOff;
                    let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
                    if s0Off != 0 && saberHolstered == 0 {
                        trap::S_StartSound(engine, Some(&lerpOrigin), number, CHAN_AUTO, s0Off);
                    }

                    let s1Off = player_ci(ctx.world, is_npc, centNum, clientNum).saber[1].soundOff;
                    let s1Model0 =
                        player_ci(ctx.world, is_npc, centNum, clientNum).saber[1].model[0];
                    if s1Off != 0 && s1Model0 != 0 && saberHolstered == 0 {
                        trap::S_StartSound(engine, Some(&lerpOrigin), number, CHAN_AUTO, s1Off);
                    }
                } else if stateWeapon == WP_SABER
                    && centWeapon != stateWeapon
                    && ctx.world.entity(centNum).saberWasInFlight == qfalse
                {
                    //switching to the saber
                    let s0On = player_ci(ctx.world, is_npc, centNum, clientNum).saber[0].soundOn;
                    let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
                    if s0On != 0 {
                        trap::S_StartSound(engine, Some(&lerpOrigin), number, CHAN_AUTO, s0On);
                    }

                    let s1On = player_ci(ctx.world, is_npc, centNum, clientNum).saber[1].soundOn;
                    if s1On != 0 {
                        trap::S_StartSound(engine, Some(&lerpOrigin), number, CHAN_AUTO, s1On);
                    }

                    let s0: *mut saberInfo_t =
                        &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[0];
                    BG_SI_SetDesiredLength(s0, 0.0, -1);
                    let s1: *mut saberInfo_t =
                        &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[1];
                    BG_SI_SetDesiredLength(s1, 0.0, -1);
                }
            }

            ctx.world.entity_mut(centNum).weapon = stateWeapon;
            let world = &*ctx.world;
            let gw = CG_G2WeaponInstance(world, world.entity(centNum), stateWeapon);
            ctx.world.entity_mut(centNum).ghoul2weapon = gw;
        }
    } else if (ctx.world.entity(centNum).currentState.eFlags & EF_DEAD) != 0
        || ctx.world.entity(centNum).torsoBolt != 0
    {
        ctx.world.entity_mut(centNum).ghoul2weapon = null_mut(); //be sure to update after respawning/getting limb regrown
    }

    if ctx.world.entity(centNum).saberWasInFlight != qfalse && g2HasWeapon {
        ctx.world.entity_mut(centNum).saberWasInFlight = qfalse;
    }

    legs = refEntity_t::zeroed();

    CG_SetGhoul2Info(&mut legs, ctx.world.entity(centNum));

    _VectorCopy(ctx.world.entity(centNum).modelScale, &mut legs.modelScale);
    legs.radius = CG_RadiusForCent(ctx.world, ctx.world.entity(centNum));
    VectorClear(&mut legs.angles);

    let colorOverride = player_ci(ctx.world, is_npc, centNum, clientNum).colorOverride;
    if colorOverride[0] != 0.0 || colorOverride[1] != 0.0 || colorOverride[2] != 0.0 {
        legs.shaderRGBA[0] = (colorOverride[0] * 255.0) as i32 as u8;
        legs.shaderRGBA[1] = (colorOverride[1] * 255.0) as i32 as u8;
        legs.shaderRGBA[2] = (colorOverride[2] * 255.0) as i32 as u8;
        legs.shaderRGBA[3] = ctx.world.entity(centNum).currentState.customRGBA[3] as u8;
    } else {
        let customRGBA = ctx.world.entity(centNum).currentState.customRGBA;
        legs.shaderRGBA[0] = customRGBA[0] as u8;
        legs.shaderRGBA[1] = customRGBA[1] as u8;
        legs.shaderRGBA[2] = customRGBA[2] as u8;
        legs.shaderRGBA[3] = customRGBA[3] as u8;
    }

    // minimal_add: (label dead — the only goto to it is commented out)

    team = player_ci(ctx.world, is_npc, centNum, clientNum).team;

    let number = ctx.world.entity(centNum).currentState.number;
    let eType = ctx.world.entity(centNum).currentState.eType;
    if ctx.world.cgs.gametype >= GT_TEAM
        && ctx.world.cvars.cg_drawFriend.integer != 0
        && number != snapClientNum
        && eType != entityType_t::ET_NPC as c_int
    {
        // If the view is either a spectator or on the same team as this character, show a symbol above their head.
        if (snapPersTeam == TEAM_SPECTATOR || snapPersTeam == team)
            && (ctx.world.entity(centNum).currentState.eFlags & EF_DEAD) == 0
        {
            if ctx.world.cgs.gametype == GT_SIEGE {
                //check for per-map team shaders
                if team == SIEGETEAM_TEAM1 {
                    let plShader = ctx.world.saga.cgSiegeTeam1PlShader;
                    if plShader != 0 {
                        cg_player_float_sprite(ctx, centNum, plShader);
                    } else {
                        //if there isn't one fallback to default
                        let sh = ctx.world.cgs.media.teamRedShader;
                        cg_player_float_sprite(ctx, centNum, sh);
                    }
                } else {
                    let plShader = ctx.world.saga.cgSiegeTeam2PlShader;
                    if plShader != 0 {
                        cg_player_float_sprite(ctx, centNum, plShader);
                    } else {
                        //if there isn't one fallback to default
                        let sh = ctx.world.cgs.media.teamBlueShader;
                        cg_player_float_sprite(ctx, centNum, sh);
                    }
                }
            } else {
                //generic teamplay
                if team == TEAM_RED {
                    let sh = ctx.world.cgs.media.teamRedShader;
                    cg_player_float_sprite(ctx, centNum, sh);
                } else {
                    // if (team == TEAM_BLUE)
                    let sh = ctx.world.cgs.media.teamBlueShader;
                    cg_player_float_sprite(ctx, centNum, sh);
                }
            }
        }
    } else if ctx.world.cgs.gametype == GT_POWERDUEL
        && ctx.world.cvars.cg_drawFriend.integer != 0
        && number != snapClientNum
    {
        let ciDuelTeam = player_ci(ctx.world, is_npc, centNum, clientNum).duelTeam;
        let viewerDuelTeam = ctx.world.cgs.clientinfo[snapClientNum as usize].duelTeam;
        if ctx.world.cg.predictedPlayerState.persistant[PERS_TEAM as usize] != TEAM_SPECTATOR
            && number < MAX_CLIENTS_I32
            && (ctx.world.entity(centNum).currentState.eFlags & EF_DEAD) == 0
            && viewerDuelTeam == ciDuelTeam
        {
            //ally in powerduel, so draw the icon
            let sh = ctx.world.cgs.media.powerDuelAllyShader;
            cg_player_float_sprite(ctx, centNum, sh);
        } else if ctx.world.cg.predictedPlayerState.persistant[PERS_TEAM as usize] == TEAM_SPECTATOR
            && number < MAX_CLIENTS_I32
            && (ctx.world.entity(centNum).currentState.eFlags & EF_DEAD) == 0
            && ciDuelTeam == DUELTEAM_DOUBLE as c_int
        {
            let sh = ctx.world.cgs.media.powerDuelAllyShader;
            cg_player_float_sprite(ctx, centNum, sh);
        }
    }

    if ctx.world.cgs.gametype == GT_JEDIMASTER
        && ctx.world.cvars.cg_drawFriend.integer != 0
        && number != snapClientNum
    {
        // Don't show a sprite above a player's own head in 3rd person.
        // If the view is either a spectator or on the same team as this character, show a symbol above their head.
        if (snapPersTeam == TEAM_SPECTATOR || snapPersTeam == team)
            && (ctx.world.entity(centNum).currentState.eFlags & EF_DEAD) == 0
            && CG_ThereIsAMaster(ctx.world)
            && snapIsJediMaster == qfalse
            && ctx.world.entity(centNum).currentState.isJediMaster == qfalse
        {
            let sh = ctx.world.cgs.media.teamRedShader;
            cg_player_float_sprite(ctx, centNum, sh);
        }
    }

    // add the shadow
    shadow = CG_PlayerShadow(ctx, centNum, &mut shadowPlane);

    let eFlags = ctx.world.entity(centNum).currentState.eFlags;
    let genericenemyindex = ctx.world.entity(centNum).currentState.genericenemyindex;
    let eType = ctx.world.entity(centNum).currentState.eType;
    if ((eFlags & EF_SEEKERDRONE) != 0 || genericenemyindex != -1)
        && eType != entityType_t::ET_NPC as c_int
    {
        let mut seeker = refEntity_t::zeroed();

        let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
        _VectorCopy(lerpOrigin, &mut elevated);
        elevated[2] += 40.0;

        _VectorCopy(elevated, &mut seeker.lightingOrigin);
        seeker.shadowPlane = shadowPlane;
        seeker.renderfx = 0; //renderfx;
                             //don't show in first person?

        let cgTime = ctx.world.cg.time;
        angle = (((cgTime / 12) & 255) as f64 * (PI * 2.0) / 255.0) as f32;
        dir[0] = ((angle as f64).cos() * 20.0) as f32;
        dir[1] = ((angle as f64).sin() * 20.0) as f32;
        dir[2] = ((angle as f64).cos() * 5.0) as f32;
        _VectorAdd(elevated, dir, &mut seeker.origin);

        _VectorCopy(seeker.origin, &mut seekorg);

        successchange = 0;
        if genericenemyindex > MAX_GENTITIES as c_int {
            let mut prefig = ((genericenemyindex - cgTime) / 80) as f32;

            if prefig > 55.0 {
                prefig = 55.0;
            } else if prefig < 1.0 {
                prefig = 1.0;
            }

            elevated[2] -= 55.0 - prefig;

            angle = (((cgTime / 12) & 255) as f64 * (PI * 2.0) / 255.0) as f32;
            dir[0] = ((angle as f64).cos() * 20.0) as f32;
            dir[1] = ((angle as f64).sin() * 20.0) as f32;
            dir[2] = ((angle as f64).cos() * 5.0) as f32;
            _VectorAdd(elevated, dir, &mut seeker.origin);
        } else if genericenemyindex != ENTITYNUM_NONE
            && genericenemyindex != -1
            // wire-sourced 32-bit field (doubles as a seeker time offset), so
            // exactly MAX_GENTITIES or a stray negative reaches this arm; Raven
            // reads garbage there, we keep the no-enemy wobble instead (§F19).
            && (0..MAX_GENTITIES as c_int).contains(&genericenemyindex)
        {
            let enentOrigin = ctx.world.entity(genericenemyindex as usize).lerpOrigin;
            let mut enang: vec3_t = [0.0; 3];
            _VectorSubtract(enentOrigin, seekorg, &mut enang);
            VectorNormalize(&mut enang);
            vectoangles(enang, &mut angles);
            successchange = 1;
        }

        if successchange == 0 {
            angles[0] = ((angle as f64).sin() * 30.0) as f32;
            angles[1] = (angle as f64 * 180.0 / PI + 90.0) as f32;
            if angles[1] > 360.0 {
                angles[1] -= 360.0;
            }
            angles[2] = 0.0;
        }

        AnglesToAxis(angles, seeker.axis.as_mut_ptr());

        seeker.hModel = trap::R_RegisterModel(engine, "models/items/remote.md3");
        trap::R_AddRefEntityToScene(engine, &seeker);
    }

    // add a water splash if partially in and out of water
    {
        // bitwise copy-in, original left in place - a zeroed-swap is visible to
        // every ctx-reading helper inside CG_PlayerSplash (C6b referee catch).
        // The copy is read-only, so it does not write back.
        // SAFETY: `npcClient` is an owned Box, so the ManuallyDrop copy never
        // drops. The original keeps the one live Box.
        let cent_tmp = ManuallyDrop::new(unsafe { core::ptr::read(ctx.world.entity(centNum)) });
        CG_PlayerSplash(ctx, &cent_tmp);
    }

    let cgShadows = ctx.world.cvars.cg_shadows.integer;
    if (cgShadows == 3 || cgShadows == 2) && shadow {
        renderfx |= RF_SHADOW_PLANE;
    }
    renderfx |= RF_LIGHTING_ORIGIN; // use the same origin for all

    // if we've been hit, display proper fullscreen fx
    {
        // bitwise copy-in, original left in place - a zeroed-swap is visible to
        // every ctx-reading helper inside CG_PlayerHitFX (C6b referee catch).
        // The copy is read-only, so it does not write back.
        // SAFETY: `npcClient` is an owned Box, so the ManuallyDrop copy never
        // drops. The original keeps the one live Box.
        let cent_tmp = ManuallyDrop::new(unsafe { core::ptr::read(ctx.world.entity(centNum)) });
        CG_PlayerHitFX(ctx, &cent_tmp);
    }

    let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
    _VectorCopy(lerpOrigin, &mut legs.origin);
    _VectorCopy(lerpOrigin, &mut legs.lightingOrigin);
    legs.shadowPlane = shadowPlane;
    legs.renderfx = renderfx;
    if ctx.world.cvars.cg_shadows.integer == 2 && (renderfx & RF_THIRD_PERSON) != 0 {
        //can see own shadow
        legs.renderfx |= RF_SHADOW_ONLY;
    }
    let legsOrigin = legs.origin;
    _VectorCopy(legsOrigin, &mut legs.oldorigin); // don't positionally lerp at all

    // CG_G2PlayerAngles + head anims want `cent` disjoint from ctx.world; the
    // ci pick happens inside CG_G2PlayerAngles like Raven's.
    {
        // bitwise copy-in/copy-back, original left in place - a zeroed-swap is
        // visible to every ctx-reading helper inside these calls (C6b referee
        // catch). The copy is written back whole.
        // SAFETY: `npcClient` is an owned Box, so the copy never drops and the
        // ptr::write write-back does not drop the original.
        let mut cent_tmp = ManuallyDrop::new(unsafe { core::ptr::read(ctx.world.entity(centNum)) });
        CG_G2PlayerAngles(ctx, &mut cent_tmp, &mut legs.axis, &mut rootAngles);
        CG_G2PlayerHeadAnims(ctx, &mut cent_tmp);
        unsafe {
            core::ptr::write(
                ctx.world.entity_mut(centNum),
                ManuallyDrop::into_inner(cent_tmp),
            )
        };
    }

    if (ctx.world.entity(centNum).currentState.eFlags2 & EF2_HELD_BY_MONSTER) != 0
        && ctx.world.entity(centNum).currentState.hasLookTarget != qfalse
    {
        //NOTE: lookTarget is an entity number, so this presumes that client 0 is NOT a Rancor...
        let rancorNum = ctx.world.entity(centNum).currentState.lookTarget as usize;
        // Raven guards `if (rancor)`; a reference is always non-NULL, so the body always runs.
        let rancYaw = ctx.world.entity(rancorNum).lerpAngles[YAW];
        let rancOrigin = ctx.world.entity(rancorNum).lerpOrigin;
        let rancModelScale = ctx.world.entity(rancorNum).modelScale;
        let rancGhoul2 = ctx.world.entity(rancorNum).ghoul2;
        let inMouth = ctx.world.entity(rancorNum).currentState.eFlags2 & EF2_GENERIC_NPC_FLAG;
        let cgTime = ctx.world.cg.time;
        // `modelList` is a read-only registry lookup for the callee; copy it to a
        // local so the mutable `*mut` doesn't alias the shared `&bg_state` borrow.
        let mut gameModelsCopy = ctx.world.cgs.gameModels;
        let traps = CgBgTraps::new(engine, ctx.world_raw());
        BG_AttachToRancor(
            rancGhoul2,
            rancYaw,
            rancOrigin,
            cgTime,
            gameModelsCopy.as_mut_ptr(),
            rancModelScale,
            inMouth,
            &mut legs.origin,
            &mut legs.angles,
            null_mut(), // Raven passes NULL - the axis out-param must stay untouched
            &ctx.world.bg_state,
            &traps,
        );

        if ctx.world.entity(centNum).isRagging != qfalse {
            //hack, ragdoll has you way at bottom of bounding box
            let legsAxis2 = legs.axis[2];
            let base = legs.origin;
            _VectorMA(base, 32.0, legsAxis2, &mut legs.origin);
        }
        let legsOrigin = legs.origin;
        _VectorCopy(legsOrigin, &mut legs.oldorigin);
        _VectorCopy(legsOrigin, &mut legs.lightingOrigin);

        let legsAngles = legs.angles;
        _VectorCopy(legsAngles, &mut ctx.world.entity_mut(centNum).lerpAngles);
        let lerpAngles = ctx.world.entity(centNum).lerpAngles;
        _VectorCopy(lerpAngles, &mut rootAngles); //??? tempAngles);//tempAngles is needed a lot below
        _VectorCopy(lerpAngles, &mut ctx.world.entity_mut(centNum).turAngles);
        _VectorCopy(legs.origin, &mut ctx.world.entity_mut(centNum).lerpOrigin);
    }

    //This call is mainly just to reconstruct the skeleton. But we'll get the left hand matrix while we're at it.
    {
        let g2 = ctx.world.entity(centNum).ghoul2;
        let boltLhand = player_ci(ctx.world, is_npc, centNum, clientNum).bolt_lhand;
        let turAngles = ctx.world.entity(centNum).turAngles;
        let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
        let modelScale = ctx.world.entity(centNum).modelScale;
        let cgTime = ctx.world.cg.time;
        trap::G2API_GetBoltMatrix(
            engine,
            g2,
            0,
            boltLhand,
            &mut lHandMatrix,
            &turAngles,
            &lerpOrigin,
            cgTime,
            Some(&mut ctx.world.cgs.gameModels[0]),
            &modelScale,
        );
    }
    gotLHandMatrix = true;

    if (ctx.world.entity(centNum).currentState.eFlags & EF_DEAD) != 0 {
        dead = true;
        //rww - since our angles are fixed when we're dead this shouldn't be an issue anyway
    }
    let _ = dead;

    ScaleModelAxis(&mut legs);

    torso = refEntity_t::zeroed();

    //rww - force speed "trail" effect
    if (ctx.world.entity(centNum).currentState.powerups & (1 << PW_SPEED)) == 0
        || doAlpha != 0
        || ctx.world.cvars.cg_speedTrail.integer == 0
    {
        ctx.world.entity_mut(centNum).frame_minus1_refreshed = 0;
        ctx.world.entity_mut(centNum).frame_minus2_refreshed = 0;
    }

    if ctx.world.entity(centNum).frame_minus1_refreshed != 0
        || ctx.world.entity(centNum).frame_minus2_refreshed != 0
    {
        let mut tDir = ctx.world.entity(centNum).currentState.pos.trDelta;
        // Raven's distVelBase is an int - the double-valued product truncates on assignment.
        let distVelBase =
            (SPEED_TRAIL_DISTANCE as f64 * (VectorNormalize(&mut tDir) as f64 * 0.004)) as c_int;

        if ctx.world.entity(centNum).frame_minus1_refreshed != 0 {
            let mut reframe_minus1 = legs;
            reframe_minus1.renderfx |= RF_FORCE_ENT_ALPHA;
            reframe_minus1.shaderRGBA[0] = legs.shaderRGBA[0];
            reframe_minus1.shaderRGBA[1] = legs.shaderRGBA[1];
            reframe_minus1.shaderRGBA[2] = legs.shaderRGBA[2];
            reframe_minus1.shaderRGBA[3] = 100;

            let mut tDir: vec3_t = [0.0; 3];
            let frame_minus1 = ctx.world.entity(centNum).frame_minus1;
            _VectorSubtract(frame_minus1, legs.origin, &mut tDir);
            VectorNormalize(&mut tDir);

            let nx = legs.origin[0] + tDir[0] * distVelBase as f32;
            let ny = legs.origin[1] + tDir[1] * distVelBase as f32;
            let nz = legs.origin[2] + tDir[2] * distVelBase as f32;
            ctx.world.entity_mut(centNum).frame_minus1 = [nx, ny, nz];

            _VectorCopy(
                ctx.world.entity(centNum).frame_minus1,
                &mut reframe_minus1.origin,
            );

            trap::R_AddRefEntityToScene(engine, &reframe_minus1);
        }

        if ctx.world.entity(centNum).frame_minus2_refreshed != 0 {
            let mut reframe_minus2 = legs;

            reframe_minus2.renderfx |= RF_FORCE_ENT_ALPHA;
            reframe_minus2.shaderRGBA[0] = legs.shaderRGBA[0];
            reframe_minus2.shaderRGBA[1] = legs.shaderRGBA[1];
            reframe_minus2.shaderRGBA[2] = legs.shaderRGBA[2];
            reframe_minus2.shaderRGBA[3] = 50;

            let mut tDir: vec3_t = [0.0; 3];
            let frame_minus2 = ctx.world.entity(centNum).frame_minus2;
            let frame_minus1 = ctx.world.entity(centNum).frame_minus1;
            _VectorSubtract(frame_minus2, frame_minus1, &mut tDir);
            VectorNormalize(&mut tDir);

            let f1 = ctx.world.entity(centNum).frame_minus1;
            let nx = f1[0] + tDir[0] * distVelBase as f32;
            let ny = f1[1] + tDir[1] * distVelBase as f32;
            let nz = f1[2] + tDir[2] * distVelBase as f32;
            ctx.world.entity_mut(centNum).frame_minus2 = [nx, ny, nz];

            _VectorCopy(
                ctx.world.entity(centNum).frame_minus2,
                &mut reframe_minus2.origin,
            );

            trap::R_AddRefEntityToScene(engine, &reframe_minus2);
        }
    }

    //trigger animation-based sounds, done before next lerp frame.
    {
        // bitwise copy-in/copy-back, original left in place - a zeroed-swap is
        // visible to every ctx-reading helper inside CG_TriggerAnimSounds (C6b
        // referee catch). The copy is written back whole.
        // SAFETY: `npcClient` is an owned Box, so the copy never drops and the
        // ptr::write write-back does not drop the original.
        let mut cent_tmp = ManuallyDrop::new(unsafe { core::ptr::read(ctx.world.entity(centNum)) });
        CG_TriggerAnimSounds(ctx, &mut cent_tmp);
        unsafe {
            core::ptr::write(
                ctx.world.entity_mut(centNum),
                ManuallyDrop::into_inner(cent_tmp),
            )
        };
    }

    // get the animation state (after rotation, to allow feet shuffle)
    {
        // bitwise copy-in/copy-back, original left in place - a zeroed-swap is
        // visible to every ctx-reading helper inside CG_PlayerAnimation (C6b
        // referee catch). The copy is written back whole.
        // SAFETY: `npcClient` is an owned Box, so the copy never drops and the
        // ptr::write write-back does not drop the original.
        let mut cent_tmp = ManuallyDrop::new(unsafe { core::ptr::read(ctx.world.entity(centNum)) });
        if is_npc {
            let mut dummy = zeroed_client_info();
            CG_PlayerAnimation(
                ctx,
                &mut cent_tmp,
                &mut dummy,
                &mut legs.oldframe,
                &mut legs.frame,
                &mut legs.backlerp,
                &mut torso.oldframe,
                &mut torso.frame,
                &mut torso.backlerp,
            );
        } else {
            // same copy-in/copy-back for the clientinfo row (C6b referee catch).
            // SAFETY: clientInfo_t is #[repr(C)] plain data; the copy is written back whole.
            let mut ci_slot =
                unsafe { core::ptr::read(&ctx.world.cgs.clientinfo[clientNum as usize]) };
            CG_PlayerAnimation(
                ctx,
                &mut cent_tmp,
                &mut ci_slot,
                &mut legs.oldframe,
                &mut legs.frame,
                &mut legs.backlerp,
                &mut torso.oldframe,
                &mut torso.frame,
                &mut torso.backlerp,
            );
            ctx.world.cgs.clientinfo[clientNum as usize] = ci_slot;
        }
        unsafe {
            core::ptr::write(
                ctx.world.entity_mut(centNum),
                ManuallyDrop::into_inner(cent_tmp),
            )
        };
    }

    // add the talk baloon or disconnect icon
    {
        // bitwise copy-in, original left in place - a zeroed-swap is visible to
        // every ctx-reading helper inside CG_PlayerSprites (C6b referee catch).
        // The copy is read-only, so it does not write back.
        // SAFETY: `npcClient` is an owned Box, so the ManuallyDrop copy never
        // drops. The original keeps the one live Box.
        let cent_tmp = ManuallyDrop::new(unsafe { core::ptr::read(ctx.world.entity(centNum)) });
        CG_PlayerSprites(ctx, &cent_tmp);
    }

    if (ctx.world.entity(centNum).currentState.eFlags & EF_DEAD) != 0 {
        //keep track of death anim frame for when we copy off the bodyqueue
        let torsoFrame = ctx.world.entity(centNum).pe.torso.frame;
        player_ci_mut(ctx.world, is_npc, centNum, clientNum).frame = torsoFrame;
    }

    let activeForcePass = ctx.world.entity(centNum).currentState.activeForcePass;
    let npcClass = ctx.world.entity(centNum).currentState.NPC_class;
    if activeForcePass > FORCE_LEVEL_3 && npcClass != class_t::CLASS_VEHICLE as c_int {
        let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];
        let mut fAng: vec3_t = [0.0; 3];
        let mut fxDir: vec3_t = [0.0; 3];
        let mut efOrg: vec3_t = [0.0; 3];

        let realForceLev = activeForcePass - FORCE_LEVEL_3;

        // Raven's `tAng` here is a dead store (VectorSet then unused); dropped.
        let torsoPitch = ctx.world.entity(centNum).pe.torso.pitchAngle;
        let torsoYaw = ctx.world.entity(centNum).pe.torso.yawAngle;
        VectorSet(&mut fAng, torsoPitch, torsoYaw, 0.0);

        AngleVectors(fAng, Some(&mut fxDir), None, None);
        let _ = fxDir; // computed by Raven, read only by commented-out code

        let torsoAnim = ctx.world.entity(centNum).currentState.torsoAnim;
        if torsoAnim == animNumber_t::BOTH_FORCE_2HANDEDLIGHTNING_HOLD as c_int
            && ctx.world.bg_state.rng.Q_irand(0, 1) != 0
        {
            //alternate back and forth between left and right
            let mut rHandMatrix = mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            };
            let g2 = ctx.world.entity(centNum).ghoul2;
            let boltRhand = player_ci(ctx.world, is_npc, centNum, clientNum).bolt_rhand;
            let turAngles = ctx.world.entity(centNum).turAngles;
            let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
            let modelScale = ctx.world.entity(centNum).modelScale;
            let cgTime = ctx.world.cg.time;
            trap::G2API_GetBoltMatrix(
                engine,
                g2,
                0,
                boltRhand,
                &mut rHandMatrix,
                &turAngles,
                &lerpOrigin,
                cgTime,
                Some(&mut ctx.world.cgs.gameModels[0]),
                &modelScale,
            );
            efOrg[0] = rHandMatrix.matrix[0][3];
            efOrg[1] = rHandMatrix.matrix[1][3];
            efOrg[2] = rHandMatrix.matrix[2][3];
        } else {
            if !gotLHandMatrix {
                let g2 = ctx.world.entity(centNum).ghoul2;
                let boltLhand = player_ci(ctx.world, is_npc, centNum, clientNum).bolt_lhand;
                let turAngles = ctx.world.entity(centNum).turAngles;
                let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
                let modelScale = ctx.world.entity(centNum).modelScale;
                let cgTime = ctx.world.cg.time;
                trap::G2API_GetBoltMatrix(
                    engine,
                    g2,
                    0,
                    boltLhand,
                    &mut lHandMatrix,
                    &turAngles,
                    &lerpOrigin,
                    cgTime,
                    Some(&mut ctx.world.cgs.gameModels[0]),
                    &modelScale,
                );
                gotLHandMatrix = true;
            }
            efOrg[0] = lHandMatrix.matrix[0][3];
            efOrg[1] = lHandMatrix.matrix[1][3];
            efOrg[2] = lHandMatrix.matrix[2][3];
        }

        AnglesToAxis(fAng, axis.as_mut_ptr());

        if realForceLev > FORCE_LEVEL_2 {
            //arc
            let fx = ctx.world.cgs.effects.forceDrainWide;
            trap::FX_PlayEntityEffectID(engine, fx, &efOrg, &axis, -1, -1, -1, -1);
        } else {
            //line
            let fx = ctx.world.cgs.effects.forceDrain;
            trap::FX_PlayEntityEffectID(engine, fx, &efOrg, &axis, -1, -1, -1, -1);
        }
    } else if activeForcePass != 0 && npcClass != class_t::CLASS_VEHICLE as c_int {
        //doing the electrocuting
        let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];
        let mut fAng: vec3_t = [0.0; 3];
        let mut fxDir: vec3_t = [0.0; 3];
        let mut efOrg: vec3_t = [0.0; 3];

        let torsoPitch = ctx.world.entity(centNum).pe.torso.pitchAngle;
        let torsoYaw = ctx.world.entity(centNum).pe.torso.yawAngle;
        VectorSet(&mut fAng, torsoPitch, torsoYaw, 0.0);

        AngleVectors(fAng, Some(&mut fxDir), None, None);
        let _ = fxDir;

        if !gotLHandMatrix {
            let g2 = ctx.world.entity(centNum).ghoul2;
            let boltLhand = player_ci(ctx.world, is_npc, centNum, clientNum).bolt_lhand;
            let turAngles = ctx.world.entity(centNum).turAngles;
            let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
            let modelScale = ctx.world.entity(centNum).modelScale;
            let cgTime = ctx.world.cg.time;
            trap::G2API_GetBoltMatrix(
                engine,
                g2,
                0,
                boltLhand,
                &mut lHandMatrix,
                &turAngles,
                &lerpOrigin,
                cgTime,
                Some(&mut ctx.world.cgs.gameModels[0]),
                &modelScale,
            );
            gotLHandMatrix = true;
        }

        efOrg[0] = lHandMatrix.matrix[0][3];
        efOrg[1] = lHandMatrix.matrix[1][3];
        efOrg[2] = lHandMatrix.matrix[2][3];

        AnglesToAxis(fAng, axis.as_mut_ptr());

        if activeForcePass > FORCE_LEVEL_2 {
            //arc
            let fx = ctx.world.cgs.effects.forceLightningWide;
            trap::FX_PlayEntityEffectID(engine, fx, &efOrg, &axis, -1, -1, -1, -1);
        } else {
            //line
            let fx = ctx.world.cgs.effects.forceLightning;
            trap::FX_PlayEntityEffectID(engine, fx, &efOrg, &axis, -1, -1, -1, -1);
        }
    }

    //fullbody push effect
    if (ctx.world.entity(centNum).currentState.eFlags & EF_BODYPUSH) != 0 {
        CG_ForcePushBodyBlur(ctx, centNum);
    }

    if (ctx.world.entity(centNum).currentState.powerups & (1 << PW_DISINT_4)) != 0 {
        let mut efOrg: vec3_t = [0.0; 3];

        // Raven's `tAng` VectorSet here is a dead store (unused); dropped.
        if !gotLHandMatrix {
            let g2 = ctx.world.entity(centNum).ghoul2;
            let boltLhand = player_ci(ctx.world, is_npc, centNum, clientNum).bolt_lhand;
            let turAngles = ctx.world.entity(centNum).turAngles;
            let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
            let modelScale = ctx.world.entity(centNum).modelScale;
            let cgTime = ctx.world.cg.time;
            trap::G2API_GetBoltMatrix(
                engine,
                g2,
                0,
                boltLhand,
                &mut lHandMatrix,
                &turAngles,
                &lerpOrigin,
                cgTime,
                Some(&mut ctx.world.cgs.gameModels[0]),
                &modelScale,
            );
            // Raven sets gotLHandMatrix here too; nothing reads it again - dead store dropped.
        }

        efOrg[0] = lHandMatrix.matrix[0][3];
        efOrg[1] = lHandMatrix.matrix[1][3];
        efOrg[2] = lHandMatrix.matrix[2][3];

        let forcePowersActive = ctx.world.entity(centNum).currentState.forcePowersActive;
        let number = ctx.world.entity(centNum).currentState.number;
        if (forcePowersActive & (1 << FP_GRIP)) != 0
            && (ctx.world.cg.renderingThirdPerson != qfalse || number != snapClientNum)
        {
            // Raven's origBolt/boltDir (+ its BG_GiveMeVectorFromMatrix) feed only
            // the commented-out regrip-arm render; dropped as a pure dead store.
            CG_ForceGripEffect(ctx, &efOrg);
            CG_ForceGripEffect(ctx, &efOrg);
        } else if (forcePowersActive & (1 << FP_GRIP)) == 0 {
            //use refractive effect
            CG_ForcePushBlur(ctx, &efOrg, Some(centNum));
        }
    } else if ctx.world.entity(centNum).bodyFadeTime != 0 {
        //reset the counter for keeping track of push refraction effect state
        ctx.world.entity_mut(centNum).bodyFadeTime = 0;
    }

    if ctx.world.entity(centNum).currentState.weapon == WP_STUN_BATON
        && ctx.world.entity(centNum).currentState.number == snapClientNum
    {
        let number = ctx.world.entity(centNum).currentState.number;
        let vieworg = ctx.world.cg.refdef.vieworg;
        let sfx = trap::S_RegisterSound(engine, "sound/weapons/baton/idle.wav");
        trap::S_AddLoopingSound(engine, number, &vieworg, &vec3_origin, sfx);
    }

    //NOTE: All effects that should be visible during mindtrick should go above here

    // Raven `if (iwantout) goto stillDoSaber;` — everything below up to the saber
    // handling is the skipped-when-`iwantout` span.
    if iwantout == 0 {
        if doAlpha != 0 {
            legs.renderfx |= RF_FORCE_ENT_ALPHA;
            legs.shaderRGBA[3] = ctx.world.entity(centNum).trickAlpha as u8;

            if legs.shaderRGBA[3] < 1 {
                //don't cancel it out even if it's < 1
                legs.shaderRGBA[3] = 1;
            }
        }

        if ctx.world.entity(centNum).teamPowerEffectTime > ctx.world.cg.time {
            if ctx.world.entity(centNum).teamPowerType == 3 {
                //absorb is a somewhat different effect entirely, handled below
            } else {
                let preRFX = legs.renderfx;

                legs.renderfx |= RF_RGB_TINT;
                legs.renderfx |= RF_FORCE_ENT_ALPHA;

                // preCol saves the four byte components (Raven uses a vec4_t; the
                // 0..255 values round-trip exactly either way).
                let preCol = legs.shaderRGBA;

                match ctx.world.entity(centNum).teamPowerType {
                    1 => {
                        //heal
                        legs.shaderRGBA[0] = 0;
                        legs.shaderRGBA[1] = 255;
                        legs.shaderRGBA[2] = 0;
                    }
                    0 => {
                        //regen
                        legs.shaderRGBA[0] = 0;
                        legs.shaderRGBA[1] = 0;
                        legs.shaderRGBA[2] = 255;
                    }
                    _ => {
                        //drain
                        legs.shaderRGBA[0] = 255;
                        legs.shaderRGBA[1] = 0;
                        legs.shaderRGBA[2] = 0;
                    }
                }

                let teamPowerEffectTime = ctx.world.entity(centNum).teamPowerEffectTime;
                let cgTime = ctx.world.cg.time;
                legs.shaderRGBA[3] = ((teamPowerEffectTime - cgTime) / 8) as u8;

                legs.customShader = trap::R_RegisterShader(engine, "powerups/ysalimarishell");
                trap::R_AddRefEntityToScene(engine, &legs);

                legs.customShader = 0;
                legs.renderfx = preRFX;
                legs.shaderRGBA = preCol;
            }
        }

        //If you've tricked this client.
        let number = ctx.world.entity(centNum).currentState.number;
        if CG_IsMindTricked(ctx.world, snapMt1, snapMt2, snapMt3, snapMt4, number)
            && !ctx.world.entity(centNum).ghoul2.is_null()
        {
            let mut efOrg: vec3_t = [0.0; 3];
            let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];

            let tAng = ctx.world.entity(centNum).turAngles;
            let g2 = ctx.world.entity(centNum).ghoul2;
            let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
            let modelScale = ctx.world.entity(centNum).modelScale;
            let cgTime = ctx.world.cg.time;
            let boltHead = player_ci(ctx.world, is_npc, centNum, clientNum).bolt_head;
            trap::G2API_GetBoltMatrix(
                engine,
                g2,
                0,
                boltHead,
                &mut boltMatrix,
                &tAng,
                &lerpOrigin,
                cgTime,
                Some(&mut ctx.world.cgs.gameModels[0]),
                &modelScale,
            );

            BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut efOrg);
            // Raven's `fxAng` NEGATIVE_Y read is a pure dead store; dropped.

            axis[0][0] = boltMatrix.matrix[0][0];
            axis[0][1] = boltMatrix.matrix[1][0];
            axis[0][2] = boltMatrix.matrix[2][0];

            axis[1][0] = boltMatrix.matrix[0][1];
            axis[1][1] = boltMatrix.matrix[1][1];
            axis[1][2] = boltMatrix.matrix[2][1];

            axis[2][0] = boltMatrix.matrix[0][2];
            axis[2][1] = boltMatrix.matrix[1][2];
            axis[2][2] = boltMatrix.matrix[2][2];

            let fx = ctx.world.cgs.effects.mForceConfustionOld;
            trap::FX_PlayEntityEffectID(engine, fx, &efOrg, &axis, -1, -1, -1, -1);
        }

        if ctx.world.cgs.gametype == GT_HOLOCRON
            && ctx.world.entity(centNum).currentState.time2 != 0
            && (ctx.world.cg.renderingThirdPerson != qfalse
                || snapClientNum != ctx.world.entity(centNum).currentState.number)
        {
            let mut i: c_int = 0;
            let mut renderedHolos = 0;
            let mut holoRef;

            while i < NUM_FORCE_POWERS as c_int && renderedHolos < 3 {
                if (ctx.world.entity(centNum).currentState.time2 & (1 << i)) != 0 {
                    holoRef = refEntity_t::zeroed();

                    let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
                    _VectorCopy(lerpOrigin, &mut elevated);
                    elevated[2] += 8.0;

                    _VectorCopy(elevated, &mut holoRef.lightingOrigin);
                    holoRef.shadowPlane = shadowPlane;
                    holoRef.renderfx = 0; //RF_THIRD_PERSON;

                    let cgTime = ctx.world.cg.time;
                    if renderedHolos == 0 {
                        angle = (((cgTime / 8) & 255) as f64 * (PI * 2.0) / 255.0) as f32;
                        dir[0] = ((angle as f64).cos() * 20.0) as f32;
                        dir[1] = ((angle as f64).sin() * 20.0) as f32;
                        dir[2] = ((angle as f64).cos() * 20.0) as f32;
                        _VectorAdd(elevated, dir, &mut holoRef.origin);

                        angles[0] = ((angle as f64).sin() * 30.0) as f32;
                        angles[1] = (angle as f64 * 180.0 / PI + 90.0) as f32;
                        if angles[1] > 360.0 {
                            angles[1] -= 360.0;
                        }
                        angles[2] = 0.0;
                        AnglesToAxis(angles, holoRef.axis.as_mut_ptr());
                    } else if renderedHolos == 1 {
                        angle = ((((cgTime / 8) & 255) as f64 * (PI * 2.0) / 255.0) + PI) as f32;
                        if angle as f64 > PI * 2.0 {
                            angle -= (PI as f32) * 2.0;
                        }
                        dir[0] = ((angle as f64).sin() * 20.0) as f32;
                        dir[1] = ((angle as f64).cos() * 20.0) as f32;
                        dir[2] = ((angle as f64).cos() * 20.0) as f32;
                        _VectorAdd(elevated, dir, &mut holoRef.origin);

                        angles[0] = ((angle as f64 - 0.5 * PI).cos() * 30.0) as f32;
                        angles[1] = (360.0 - (angle as f64 * 180.0 / PI)) as f32;
                        if angles[1] > 360.0 {
                            angles[1] -= 360.0;
                        }
                        angles[2] = 0.0;
                        AnglesToAxis(angles, holoRef.axis.as_mut_ptr());
                    } else {
                        angle =
                            ((((cgTime / 6) & 255) as f64 * (PI * 2.0) / 255.0) + 0.5 * PI) as f32;
                        if angle as f64 > PI * 2.0 {
                            angle -= (PI as f32) * 2.0;
                        }
                        dir[0] = ((angle as f64).sin() * 20.0) as f32;
                        dir[1] = ((angle as f64).cos() * 20.0) as f32;
                        dir[2] = 0.0;
                        _VectorAdd(elevated, dir, &mut holoRef.origin);

                        _VectorCopy(dir, &mut holoRef.axis[1]);
                        VectorNormalize(&mut holoRef.axis[1]);
                        VectorSet(&mut holoRef.axis[2], 0.0, 0.0, 1.0);
                        let a1 = holoRef.axis[1];
                        let a2 = holoRef.axis[2];
                        CrossProduct(a1, a2, &mut holoRef.axis[0]);
                    }

                    holoRef.modelScale[0] = 0.5;
                    holoRef.modelScale[1] = 0.5;
                    holoRef.modelScale[2] = 0.5;
                    ScaleModelAxis(&mut holoRef);

                    {
                        // SAFETY: `addspriteArgStruct_t` is all POD (vec3/f32/int
                        // handles); all-zero is a valid bit pattern, matching
                        // Raven's stack `fxSArgs` before it fills the fields.
                        let mut fxSArgs: addspriteArgStruct_t = unsafe { core::mem::zeroed() };
                        let mut holoCenter: vec3_t = [0.0; 3];

                        holoCenter[0] = holoRef.origin[0] + holoRef.axis[2][0] * 18.0;
                        holoCenter[1] = holoRef.origin[1] + holoRef.axis[2][1] * 18.0;
                        holoCenter[2] = holoRef.origin[2] + holoRef.axis[2][2] * 18.0;

                        let wv: f32 =
                            (((cgTime as f32 * 0.004f32) as f64).sin() * 0.08f32 as f64 + 0.1f32 as f64) as f32;

                        _VectorCopy(holoCenter, &mut fxSArgs.origin);
                        VectorClear(&mut fxSArgs.vel);
                        VectorClear(&mut fxSArgs.accel);
                        fxSArgs.scale = wv * 60.0;
                        fxSArgs.dscale = wv * 60.0;
                        fxSArgs.sAlpha = wv * 12.0;
                        fxSArgs.eAlpha = wv * 12.0;
                        fxSArgs.rotation = 0.0;
                        fxSArgs.bounce = 0.0;
                        fxSArgs.life = 1;

                        fxSArgs.flags = 0x08000000 | 0x00000001;

                        if forcePowerDarkLight[i as usize] == FORCE_DARKSIDE {
                            //dark
                            fxSArgs.sAlpha *= 3.0;
                            fxSArgs.eAlpha *= 3.0;
                            fxSArgs.shader = ctx.world.cgs.media.redSaberGlowShader;
                            trap::FX_AddSprite(engine, &mut fxSArgs);
                        } else if forcePowerDarkLight[i as usize] == FORCE_LIGHTSIDE {
                            //light
                            fxSArgs.sAlpha *= 1.5;
                            fxSArgs.eAlpha *= 1.5;
                            fxSArgs.shader = ctx.world.cgs.media.redSaberGlowShader;
                            trap::FX_AddSprite(engine, &mut fxSArgs);
                            fxSArgs.shader = ctx.world.cgs.media.greenSaberGlowShader;
                            trap::FX_AddSprite(engine, &mut fxSArgs);
                            fxSArgs.shader = ctx.world.cgs.media.blueSaberGlowShader;
                            trap::FX_AddSprite(engine, &mut fxSArgs);
                        } else {
                            //neutral
                            if i == FP_SABER_OFFENSE || i == FP_SABER_DEFENSE || i == FP_SABERTHROW
                            {
                                //saber power
                                fxSArgs.sAlpha *= 1.5;
                                fxSArgs.eAlpha *= 1.5;
                                fxSArgs.shader = ctx.world.cgs.media.greenSaberGlowShader;
                                trap::FX_AddSprite(engine, &mut fxSArgs);
                            } else {
                                fxSArgs.sAlpha *= 0.5;
                                fxSArgs.eAlpha *= 0.5;
                                fxSArgs.shader = ctx.world.cgs.media.greenSaberGlowShader;
                                trap::FX_AddSprite(engine, &mut fxSArgs);
                                fxSArgs.shader = ctx.world.cgs.media.blueSaberGlowShader;
                                trap::FX_AddSprite(engine, &mut fxSArgs);
                            }
                        }
                    }

                    holoRef.hModel = trap::R_RegisterModel(engine, forceHolocronModels[i as usize]);
                    trap::R_AddRefEntityToScene(engine, &holoRef);

                    renderedHolos += 1;
                }
                i += 1;
            }
        }

        let powerups = ctx.world.entity(centNum).currentState.powerups;
        let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
        if (powerups & (1 << PW_YSALAMIRI)) != 0
            || (ctx.world.cgs.gametype == GT_CTY
                && ((powerups & (1 << PW_REDFLAG)) != 0 || (powerups & (1 << PW_BLUEFLAG)) != 0))
        {
            if ctx.world.cgs.gametype == GT_CTY && (powerups & (1 << PW_REDFLAG)) != 0 {
                let sh = ctx.world.cgs.media.ysaliredShader;
                cg_draw_player_sphere(ctx, centNum, &lerpOrigin, 1.4, sh);
            } else if ctx.world.cgs.gametype == GT_CTY && (powerups & (1 << PW_BLUEFLAG)) != 0 {
                let sh = ctx.world.cgs.media.ysaliblueShader;
                cg_draw_player_sphere(ctx, centNum, &lerpOrigin, 1.4, sh);
            } else {
                let sh = ctx.world.cgs.media.ysalimariShader;
                cg_draw_player_sphere(ctx, centNum, &lerpOrigin, 1.4, sh);
            }
        }

        if (ctx.world.entity(centNum).currentState.powerups & (1 << PW_FORCE_BOON)) != 0 {
            let sh = ctx.world.cgs.media.boonShader;
            cg_draw_player_sphere(ctx, centNum, &lerpOrigin, 2.0, sh);
        }

        if (ctx.world.entity(centNum).currentState.powerups & (1 << PW_FORCE_ENLIGHTENED_DARK)) != 0
        {
            let sh = ctx.world.cgs.media.endarkenmentShader;
            cg_draw_player_sphere(ctx, centNum, &lerpOrigin, 2.0, sh);
        } else if (ctx.world.entity(centNum).currentState.powerups
            & (1 << PW_FORCE_ENLIGHTENED_LIGHT))
            != 0
        {
            let sh = ctx.world.cgs.media.enlightenmentShader;
            cg_draw_player_sphere(ctx, centNum, &lerpOrigin, 2.0, sh);
        }

        if (ctx.world.entity(centNum).currentState.eFlags & EF_INVULNERABLE) != 0 {
            let sh = ctx.world.cgs.media.invulnerabilityShader;
            cg_draw_player_sphere(ctx, centNum, &lerpOrigin, 1.0, sh);
        }
    }

    // stillDoSaber:
    let stateWeapon = ctx.world.entity(centNum).currentState.weapon;
    let eFlags = ctx.world.entity(centNum).currentState.eFlags;
    let saberHolstered = ctx.world.entity(centNum).currentState.saberHolstered;
    if (eFlags & EF_DEAD) != 0 && stateWeapon == WP_SABER {
        let s0: *mut saberInfo_t =
            &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[0];
        BG_SI_SetDesiredLength(s0, 0.0, -1);
        let s1: *mut saberInfo_t =
            &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[1];
        BG_SI_SetDesiredLength(s1, 0.0, -1);

        drawPlayerSaber = true;
    } else if stateWeapon == WP_SABER && saberHolstered < 2 {
        let saberInFlight = ctx.world.entity(centNum).currentState.saberInFlight;
        let s1SoundLoop = player_ci(ctx.world, is_npc, centNum, clientNum).saber[1].soundLoop;
        if (saberInFlight == qfalse || s1SoundLoop != 0) && (eFlags & EF_DEAD) == 0
        //still alive
        {
            let soundSpot: vec3_t;
            let mut didFirstSound = false;

            let number = ctx.world.entity(centNum).currentState.number;
            if snapClientNum == number {
                soundSpot = ctx.world.cg.refdef.vieworg;
            } else {
                soundSpot = ctx.world.entity(centNum).lerpOrigin;
            }

            let s0Model0 = player_ci(ctx.world, is_npc, centNum, clientNum).saber[0].model[0];
            let s0SoundLoop = player_ci(ctx.world, is_npc, centNum, clientNum).saber[0].soundLoop;
            if s0Model0 != 0 && s0SoundLoop != 0 && saberInFlight == qfalse {
                let numBlades = player_ci(ctx.world, is_npc, centNum, clientNum).saber[0].numBlades;
                let mut i = 0;
                let mut hasLen = false;
                while i < numBlades {
                    if player_ci(ctx.world, is_npc, centNum, clientNum).saber[0].blade[i as usize]
                        .length
                        != 0.0
                    {
                        hasLen = true;
                        break;
                    }
                    i += 1;
                }

                if hasLen {
                    trap::S_AddLoopingSound(engine, number, &soundSpot, &vec3_origin, s0SoundLoop);
                    didFirstSound = true;
                }
            }

            let s1Model0 = player_ci(ctx.world, is_npc, centNum, clientNum).saber[1].model[0];
            let s1SoundLoop = player_ci(ctx.world, is_npc, centNum, clientNum).saber[1].soundLoop;
            let s0SoundLoop = player_ci(ctx.world, is_npc, centNum, clientNum).saber[0].soundLoop;
            if s1Model0 != 0 && s1SoundLoop != 0 && (!didFirstSound || s0SoundLoop != s1SoundLoop) {
                let numBlades = player_ci(ctx.world, is_npc, centNum, clientNum).saber[1].numBlades;
                let mut i = 0;
                let mut hasLen = false;
                while i < numBlades {
                    if player_ci(ctx.world, is_npc, centNum, clientNum).saber[1].blade[i as usize]
                        .length
                        != 0.0
                    {
                        hasLen = true;
                        break;
                    }
                    i += 1;
                }

                if hasLen {
                    trap::S_AddLoopingSound(engine, number, &soundSpot, &vec3_origin, s1SoundLoop);
                }
            }
        }

        let saberInFlight = ctx.world.entity(centNum).currentState.saberInFlight;
        if iwantout != 0 && saberInFlight == qfalse {
            if (ctx.world.entity(centNum).currentState.eFlags & EF_DEAD) != 0
                && !ctx.world.entity(centNum).ghoul2.is_null()
                && ctx.world.entity(centNum).currentState.saberInFlight != qfalse
                && g2HasWeapon
            {
                //special case, kill the saber on a freshly dead player if another source says to.
                trap::G2API_RemoveGhoul2Model(
                    engine,
                    &raw mut ctx.world.entity_mut(centNum).ghoul2,
                    1,
                );
                // Raven clears g2HasWeapon here but returns right after - dead store dropped.
            }
            return;
        }

        if g2HasWeapon && ctx.world.entity(centNum).currentState.saberInFlight != qfalse {
            //keep this set, so we don't re-unholster the thing when we get it back, even if it's knocked away.
            ctx.world.entity_mut(centNum).saberWasInFlight = qtrue;
        }

        if ctx.world.entity(centNum).currentState.saberInFlight != qfalse
            && ctx.world.entity(centNum).currentState.saberEntityNum != 0
        {
            let saberNum = ctx.world.entity(centNum).currentState.saberEntityNum as usize;

            let bolt3 = ctx.world.entity(centNum).bolt3;
            let saberServerHit = ctx.world.entity(saberNum).serverSaberHitIndex;
            let saberModelidx = ctx.world.entity(saberNum).currentState.modelindex;
            if g2HasWeapon || bolt3 == 0 || saberServerHit != saberModelidx {
                //saber is in flight, do not have it as a standard weapon model
                let mut addBolts = false;

                if g2HasWeapon {
                    //ah well, just stick it over the right hand right now.
                    let mut boltMat = mdxaBone_t {
                        matrix: [[0.0; 4]; 3],
                    };
                    let g2 = ctx.world.entity(centNum).ghoul2;
                    let boltRhand = player_ci(ctx.world, is_npc, centNum, clientNum).bolt_rhand;
                    let turAngles = ctx.world.entity(centNum).turAngles;
                    let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
                    let modelScale = ctx.world.entity(centNum).modelScale;
                    let cgTime = ctx.world.cg.time;
                    trap::G2API_GetBoltMatrix(
                        engine,
                        g2,
                        0,
                        boltRhand,
                        &mut boltMat,
                        &turAngles,
                        &lerpOrigin,
                        cgTime,
                        Some(&mut ctx.world.cgs.gameModels[0]),
                        &modelScale,
                    );
                    let mut trBase: vec3_t = [0.0; 3];
                    BG_GiveMeVectorFromMatrix(
                        &boltMat,
                        Eorientations::ORIGIN as c_int,
                        &mut trBase,
                    );
                    ctx.world.entity_mut(saberNum).currentState.pos.trBase = trBase;

                    trap::G2API_RemoveGhoul2Model(
                        engine,
                        &raw mut ctx.world.entity_mut(centNum).ghoul2,
                        1,
                    );
                    g2HasWeapon = false;
                }

                let cgTime = ctx.world.cg.time;
                ctx.world.entity_mut(saberNum).currentState.pos.trTime = cgTime;
                ctx.world.entity_mut(saberNum).currentState.apos.trTime = cgTime;

                let posBase = ctx.world.entity(saberNum).currentState.pos.trBase;
                ctx.world.entity_mut(saberNum).lerpOrigin = posBase;
                let aposBase = ctx.world.entity(saberNum).currentState.apos.trBase;
                ctx.world.entity_mut(saberNum).lerpAngles = aposBase;

                let newBolt3 = ctx.world.entity(saberNum).currentState.apos.trBase[0] as i32;
                ctx.world.entity_mut(centNum).bolt3 = newBolt3;
                if ctx.world.entity(centNum).bolt3 == 0 {
                    ctx.world.entity_mut(centNum).bolt3 = 1;
                }
                ctx.world.entity_mut(centNum).bolt2 = 0;

                ctx.world.entity_mut(saberNum).currentState.bolt2 = 123;

                let saberGhoul2 = ctx.world.entity(saberNum).ghoul2;
                let saberServerHit = ctx.world.entity(saberNum).serverSaberHitIndex;
                let saberModelidx = ctx.world.entity(saberNum).currentState.modelindex;
                if !saberGhoul2.is_null() && saberServerHit == saberModelidx {
                    // now set up the gun bolt on it
                    addBolts = true;
                } else {
                    let saberModel = CG_ConfigString(
                        ctx,
                        CS_MODELS + ctx.world.entity(saberNum).currentState.modelindex,
                    );

                    let modelidx = ctx.world.entity(saberNum).currentState.modelindex;
                    ctx.world.entity_mut(saberNum).serverSaberHitIndex = modelidx;

                    if !ctx.world.entity(saberNum).ghoul2.is_null() {
                        //clean if we already have one (because server changed model string index)
                        let g2 = ctx.world.entity(saberNum).ghoul2;
                        let mut g2p = g2;
                        trap::G2API_CleanGhoul2Models(engine, &mut g2p);
                        ctx.world.entity_mut(saberNum).ghoul2 = g2p;
                        ctx.world.entity_mut(saberNum).ghoul2 = null_mut();
                    }

                    if !saberModel.is_empty() {
                        let mut g2p = ctx.world.entity(saberNum).ghoul2;
                        trap::G2API_InitGhoul2Model(engine, &mut g2p, &saberModel, 0, 0, 0, 0, 0);
                        ctx.world.entity_mut(saberNum).ghoul2 = g2p;
                    } else {
                        let s0Model0 =
                            player_ci(ctx.world, is_npc, centNum, clientNum).saber[0].model[0];
                        if s0Model0 != 0 {
                            let s0Model = unsafe {
                                cstr_to_str(
                                    player_ci(ctx.world, is_npc, centNum, clientNum).saber[0]
                                        .model
                                        .as_ptr(),
                                )
                            };
                            let mut g2p = ctx.world.entity(saberNum).ghoul2;
                            trap::G2API_InitGhoul2Model(engine, &mut g2p, &s0Model, 0, 0, 0, 0, 0);
                            ctx.world.entity_mut(saberNum).ghoul2 = g2p;
                        } else {
                            let mut g2p = ctx.world.entity(saberNum).ghoul2;
                            trap::G2API_InitGhoul2Model(
                                engine,
                                &mut g2p,
                                "models/weapons2/saber/saber_w.glm",
                                0,
                                0,
                                0,
                                0,
                                0,
                            );
                            ctx.world.entity_mut(saberNum).ghoul2 = g2p;
                        }
                    }

                    if !ctx.world.entity(saberNum).ghoul2.is_null() {
                        addBolts = true;

                        let posBase = ctx.world.entity(saberNum).currentState.pos.trBase;
                        ctx.world.entity_mut(saberNum).lerpOrigin = posBase;
                        let aposBase = ctx.world.entity(saberNum).currentState.apos.trBase;
                        ctx.world.entity_mut(saberNum).lerpAngles = aposBase;
                        let cgTime = ctx.world.cg.time;
                        ctx.world.entity_mut(saberNum).currentState.pos.trTime = cgTime;
                        ctx.world.entity_mut(saberNum).currentState.apos.trTime = cgTime;
                    }
                }

                if addBolts {
                    let numBlades =
                        player_ci(ctx.world, is_npc, centNum, clientNum).saber[0].numBlades;
                    let mut m = 0;
                    while m < numBlades {
                        let tagName = format!("*blade{}", m + 1);
                        let saberGhoul2 = ctx.world.entity(saberNum).ghoul2;
                        let mut tagBolt = trap::G2API_AddBolt(engine, saberGhoul2, 0, &tagName);

                        if tagBolt == -1 {
                            if m == 0 {
                                //guess this is an 0ldsk3wl saber
                                tagBolt = trap::G2API_AddBolt(engine, saberGhoul2, 0, "*flash");

                                if tagBolt == -1 {
                                    debug_assert!(false, "CG_Player: no *flash bolt on saber");
                                }
                                break;
                            }

                            // a later blade tag is missing - Raven's second arm
                            // (its assert is the redundant part, not the break)
                            // stops the scan here.
                            break;
                        }

                        m += 1;
                    }
                }
            }

            let saberGhoul2 = ctx.world.entity(saberNum).ghoul2;
            if !saberGhoul2.is_null() {
                let mut bladeAngles: vec3_t;
                let mut efOrg: vec3_t = [0.0; 3];
                let wv: f32;

                if ctx.world.entity(centNum).bolt2 == 0 {
                    let cgTime = ctx.world.cg.time;
                    ctx.world.entity_mut(centNum).bolt2 = cgTime;
                }

                if ctx.world.entity(centNum).bolt3 != 90 {
                    if ctx.world.entity(centNum).bolt3 < 90 {
                        let cgTime = ctx.world.cg.time;
                        let bolt2 = ctx.world.entity(centNum).bolt2;
                        let newBolt3 = (ctx.world.entity(centNum).bolt3 as f64
                            + (cgTime - bolt2) as f64 * 0.5)
                            as i32;
                        ctx.world.entity_mut(centNum).bolt3 = newBolt3;

                        if ctx.world.entity(centNum).bolt3 > 90 {
                            ctx.world.entity_mut(centNum).bolt3 = 90;
                        }
                    } else if ctx.world.entity(centNum).bolt3 > 90 {
                        let cgTime = ctx.world.cg.time;
                        let bolt2 = ctx.world.entity(centNum).bolt2;
                        let newBolt3 = (ctx.world.entity(centNum).bolt3 as f64
                            - (cgTime - bolt2) as f64 * 0.5)
                            as i32;
                        ctx.world.entity_mut(centNum).bolt3 = newBolt3;

                        if ctx.world.entity(centNum).bolt3 < 90 {
                            ctx.world.entity_mut(centNum).bolt3 = 90;
                        }
                    }
                }

                let cgTime = ctx.world.cg.time;
                ctx.world.entity_mut(centNum).bolt2 = cgTime;

                let bolt3 = ctx.world.entity(centNum).bolt3;
                ctx.world.entity_mut(saberNum).currentState.apos.trBase[0] = bolt3 as f32;
                ctx.world.entity_mut(saberNum).lerpAngles[0] = bolt3 as f32;

                let saberInFlightState = ctx.world.entity(saberNum).currentState.saberInFlight;
                let saberStateBolt2 = ctx.world.entity(saberNum).currentState.bolt2;
                if saberInFlightState == qfalse && saberStateBolt2 != 123 {
                    //owner is pulling is back
                    let saberFlags =
                        player_ci(ctx.world, is_npc, centNum, clientNum).saber[0].saberFlags;
                    let saberHolstered = ctx.world.entity(centNum).currentState.saberHolstered;
                    if (saberFlags & SFL_RETURN_DAMAGE) == 0 || saberHolstered != 0 {
                        let mut owndir: vec3_t = [0.0; 3];

                        let saberLerp = ctx.world.entity(saberNum).lerpOrigin;
                        let centLerp = ctx.world.entity(centNum).lerpOrigin;
                        _VectorSubtract(saberLerp, centLerp, &mut owndir);
                        VectorNormalize(&mut owndir);

                        let mut owndir2: vec3_t = [0.0; 3];
                        vectoangles(owndir, &mut owndir2);
                        owndir2[0] += 90.0;

                        ctx.world.entity_mut(saberNum).currentState.apos.trBase = owndir2;
                        ctx.world.entity_mut(saberNum).lerpAngles = owndir2;
                        VectorClear(&mut ctx.world.entity_mut(saberNum).currentState.apos.trDelta);
                    }
                }

                //We don't actually want to rely entirely on server updates to render the position of the saber...
                let saberInFlightState = ctx.world.entity(saberNum).currentState.saberInFlight;
                let saberStateBolt2 = ctx.world.entity(saberNum).currentState.bolt2;
                if saberInFlightState == qfalse && saberStateBolt2 != 123 {
                    //tell it that we're a saber and to render the glow around our handle because we're being pulled back
                    ctx.world.entity_mut(saberNum).bolt3 = 999;
                }

                ctx.world.entity_mut(saberNum).currentState.modelGhoul2 = 1;
                CG_ManualEntityRender(ctx, saberNum);
                ctx.world.entity_mut(saberNum).bolt3 = 0;
                ctx.world.entity_mut(saberNum).currentState.modelGhoul2 = 127;

                bladeAngles = ctx.world.entity(saberNum).lerpAngles;
                bladeAngles[ROLL] = 0.0;

                let numBlades = player_ci(ctx.world, is_npc, centNum, clientNum).saber[0].numBlades;
                let saberHolstered = ctx.world.entity(centNum).currentState.saberHolstered;
                if numBlades > 1 && saberHolstered == 1 {
                    //only first blade should be on
                    let s0: *mut saberInfo_t =
                        &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[0];
                    BG_SI_SetDesiredLength(s0, 0.0, -1);
                    let s0: *mut saberInfo_t =
                        &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[0];
                    BG_SI_SetDesiredLength(s0, -1.0, 0);
                } else {
                    let s0: *mut saberInfo_t =
                        &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[0];
                    BG_SI_SetDesiredLength(s0, -1.0, -1);
                }
                // Raven tests `ci->saber[1].model` - the array itself, always
                // true - so this is just the holster check.
                if saberHolstered == 1 {
                    let s1: *mut saberInfo_t =
                        &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[1];
                    BG_SI_SetDesiredLength(s1, 0.0, -1);
                } else {
                    let s1: *mut saberInfo_t =
                        &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[1];
                    BG_SI_SetDesiredLength(s1, -1.0, -1);
                }

                //Only want to do for the first saber actually, it's the one in flight.
                let mut l = 0;
                while l < 1 {
                    if player_ci(ctx.world, is_npc, centNum, clientNum).saber[l].model[0] == 0 {
                        break;
                    }

                    let numBlades =
                        player_ci(ctx.world, is_npc, centNum, clientNum).saber[l].numBlades;
                    let mut k = 0;
                    while k < numBlades {
                        let saberHolstered = ctx.world.entity(centNum).currentState.saberHolstered;
                        let saberLerp = ctx.world.entity(saberNum).lerpOrigin;
                        let mut nullSaber = refEntity_t::zeroed();
                        if l == 0 && saberHolstered == 1 && k > 0 {
                            //extra blades off — don't draw them
                            CG_AddSaberBlade(
                                ctx,
                                centNum,
                                saberNum,
                                &mut nullSaber,
                                0,
                                0,
                                l,
                                k,
                                &saberLerp,
                                &bladeAngles,
                                true,
                                true,
                            );
                        } else {
                            CG_AddSaberBlade(
                                ctx,
                                centNum,
                                saberNum,
                                &mut nullSaber,
                                0,
                                0,
                                l,
                                k,
                                &saberLerp,
                                &bladeAngles,
                                true,
                                false,
                            );
                        }

                        k += 1;
                    }
                    if player_ci(ctx.world, is_npc, centNum, clientNum).saber[l].numBlades > 2 {
                        //add a single glow for the saber based on all the blade colors combined
                        let saberCopy = player_ci(ctx.world, is_npc, centNum, clientNum).saber[l];
                        CG_DoSaberLight(ctx, Some(&saberCopy));
                    }

                    l += 1;
                }

                //Make the player's hand glow while guiding the saber
                let mut tAng: vec3_t = [0.0; 3];
                let turAngles = ctx.world.entity(centNum).turAngles;
                VectorSet(&mut tAng, turAngles[PITCH], turAngles[YAW], turAngles[ROLL]);

                let g2 = ctx.world.entity(centNum).ghoul2;
                let boltRhand = player_ci(ctx.world, is_npc, centNum, clientNum).bolt_rhand;
                let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
                let modelScale = ctx.world.entity(centNum).modelScale;
                let cgTime = ctx.world.cg.time;
                trap::G2API_GetBoltMatrix(
                    engine,
                    g2,
                    0,
                    boltRhand,
                    &mut boltMatrix,
                    &tAng,
                    &lerpOrigin,
                    cgTime,
                    Some(&mut ctx.world.cgs.gameModels[0]),
                    &modelScale,
                );

                efOrg[0] = boltMatrix.matrix[0][3];
                efOrg[1] = boltMatrix.matrix[1][3];
                efOrg[2] = boltMatrix.matrix[2][3];

                wv = (((cgTime as f32 * 0.003f32) as f64).sin() * 0.08f32 as f64 + 0.1f32 as f64) as f32;

                let mut fxSArgs: addspriteArgStruct_t = unsafe { core::mem::zeroed() };
                _VectorCopy(efOrg, &mut fxSArgs.origin);
                VectorClear(&mut fxSArgs.vel);
                VectorClear(&mut fxSArgs.accel);
                fxSArgs.scale = 8.0;
                fxSArgs.dscale = 8.0;
                fxSArgs.sAlpha = wv;
                fxSArgs.eAlpha = wv;
                fxSArgs.rotation = 0.0;
                fxSArgs.bounce = 0.0;
                fxSArgs.life = 1;
                fxSArgs.shader = ctx.world.cgs.media.yellowDroppedSaberShader;
                fxSArgs.flags = 0x08000000;
                trap::FX_AddSprite(engine, &mut fxSArgs);
            }
        } else {
            let numBlades = player_ci(ctx.world, is_npc, centNum, clientNum).saber[0].numBlades;
            let saberHolstered = ctx.world.entity(centNum).currentState.saberHolstered;
            if numBlades > 1 && saberHolstered == 1 {
                //only first blade should be on
                let s0: *mut saberInfo_t =
                    &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[0];
                BG_SI_SetDesiredLength(s0, 0.0, -1);
                let s0: *mut saberInfo_t =
                    &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[0];
                BG_SI_SetDesiredLength(s0, -1.0, 0);
            } else {
                let s0: *mut saberInfo_t =
                    &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[0];
                BG_SI_SetDesiredLength(s0, -1.0, -1);
            }
            // Raven tests `ci->saber[1].model` - the array itself, always true -
            // so this is just the holster check.
            if saberHolstered == 1 {
                let s1: *mut saberInfo_t =
                    &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[1];
                BG_SI_SetDesiredLength(s1, 0.0, -1);
            } else {
                let s1: *mut saberInfo_t =
                    &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[1];
                BG_SI_SetDesiredLength(s1, -1.0, -1);
            }
        }

        //If the arm the saber is in is broken, turn it off.
        //Leaving right arm on, at least for now.
        if (ctx.world.entity(centNum).currentState.brokenLimbs
            & (1 << brokenLimb_t::BROKENLIMB_LARM as c_int))
            != 0
        {
            let s1: *mut saberInfo_t =
                &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[1];
            BG_SI_SetDesiredLength(s1, 0.0, -1);
        }

        if ctx.world.entity(centNum).currentState.saberEntityNum == 0 {
            let s0: *mut saberInfo_t =
                &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[0];
            BG_SI_SetDesiredLength(s0, 0.0, -1);
        }
        drawPlayerSaber = true;
    } else if stateWeapon == WP_SABER {
        let s0: *mut saberInfo_t =
            &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[0];
        BG_SI_SetDesiredLength(s0, 0.0, -1);
        let s1: *mut saberInfo_t =
            &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[1];
        BG_SI_SetDesiredLength(s1, 0.0, -1);

        drawPlayerSaber = true;
    } else {
        let s0: *mut saberInfo_t =
            &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[0];
        BG_SI_SetDesiredLength(s0, 0.0, -1);
        let s1: *mut saberInfo_t =
            &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[1];
        BG_SI_SetDesiredLength(s1, 0.0, -1);

        let s0: *mut saberInfo_t =
            &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[0];
        BG_SI_SetLength(s0, 0.0);
        let s1: *mut saberInfo_t =
            &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[1];
        BG_SI_SetLength(s1, 0.0);
    }

    if ctx.world.entity(centNum).currentState.weapon == WP_SABER {
        let cgTime = ctx.world.cg.time;
        let s0: *mut saberInfo_t =
            &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[0];
        BG_SI_SetLengthGradual(s0, cgTime);
        let s1: *mut saberInfo_t =
            &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[1];
        BG_SI_SetLengthGradual(s1, cgTime);
    }

    if drawPlayerSaber {
        let mut l = 0;

        if ctx.world.entity(centNum).currentState.saberEntityNum == 0 {
            l = 1; //The "primary" saber is missing or in flight or something, so only try to draw in the second one
        } else if ctx.world.entity(centNum).currentState.saberInFlight == qfalse {
            let saberNum = ctx.world.entity(centNum).currentState.saberEntityNum as usize;

            if !g2HasWeapon {
                let g2 = ctx.world.entity(centNum).ghoul2;
                let src = {
                    let world = &*ctx.world;
                    CG_G2WeaponInstance(world, world.entity(centNum), WP_SABER)
                };
                trap::G2API_CopySpecificGhoul2Model(engine, src, 0, g2, 1);

                let saberGhoul2 = ctx.world.entity(saberNum).ghoul2;
                if !saberGhoul2.is_null() {
                    let mut g2p = saberGhoul2;
                    trap::G2API_CleanGhoul2Models(engine, &mut g2p);
                    ctx.world.entity_mut(saberNum).ghoul2 = g2p;
                }

                ctx.world.entity_mut(saberNum).currentState.modelindex = 0;
                ctx.world.entity_mut(saberNum).ghoul2 = null_mut();
                VectorClear(&mut ctx.world.entity_mut(saberNum).currentState.pos.trBase);
            }

            ctx.world.entity_mut(centNum).bolt3 = 0;
            ctx.world.entity_mut(centNum).bolt2 = 0;
        } else {
            l = 1; //The "primary" saber is missing or in flight or something, so only try to draw in the second one
        }

        while l < MAX_SABERS {
            let mut k = 0;

            if player_ci(ctx.world, is_npc, centNum, clientNum).saber[l].model[0] == 0 {
                break;
            }

            if (ctx.world.entity(centNum).currentState.eFlags2 & EF2_HELD_BY_MONSTER) != 0 {
                let axis0 = legs.axis[0];
                vectoangles(axis0, &mut rootAngles);
            }

            let numBlades = player_ci(ctx.world, is_npc, centNum, clientNum).saber[l].numBlades;
            while k < numBlades {
                let saberHolstered = ctx.world.entity(centNum).currentState.saberHolstered;
                let bladeLen = player_ci(ctx.world, is_npc, centNum, clientNum).saber[l].blade
                    [k as usize]
                    .length;
                let s1Model0 = player_ci(ctx.world, is_npc, centNum, clientNum).saber[1].model[0];
                let origin = legs.origin;
                let mut nullSaber = refEntity_t::zeroed();
                if saberHolstered == 1 && k > 0 && bladeLen <= 0.0 {
                    //extra blades off — don't draw them
                    CG_AddSaberBlade(
                        ctx,
                        centNum,
                        centNum,
                        &mut nullSaber,
                        0,
                        0,
                        l,
                        k,
                        &origin,
                        &rootAngles,
                        false,
                        true,
                    );
                } else if s1Model0 != 0 && saberHolstered == 1 && l > 0 && bladeLen <= 0.0 {
                    //second saber is turned off and this blade is done with turning off
                    CG_AddSaberBlade(
                        ctx,
                        centNum,
                        centNum,
                        &mut nullSaber,
                        0,
                        0,
                        l,
                        k,
                        &origin,
                        &rootAngles,
                        false,
                        true,
                    );
                } else {
                    CG_AddSaberBlade(
                        ctx,
                        centNum,
                        centNum,
                        &mut nullSaber,
                        0,
                        0,
                        l,
                        k,
                        &origin,
                        &rootAngles,
                        false,
                        false,
                    );
                }

                k += 1;
            }
            if player_ci(ctx.world, is_npc, centNum, clientNum).saber[l].numBlades > 2 {
                //add a single glow for the saber based on all the blade colors combined
                let saberCopy = player_ci(ctx.world, is_npc, centNum, clientNum).saber[l];
                CG_DoSaberLight(ctx, Some(&saberCopy));
            }

            l += 1;
        }
    }

    if ctx.world.entity(centNum).currentState.saberInFlight != qfalse
        && ctx.world.entity(centNum).currentState.saberEntityNum == 0
    {
        //reset the length if the saber is knocked away
        let s0: *mut saberInfo_t =
            &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[0];
        BG_SI_SetDesiredLength(s0, 0.0, -1);
        let s1: *mut saberInfo_t =
            &mut player_ci_mut(ctx.world, is_npc, centNum, clientNum).saber[1];
        BG_SI_SetDesiredLength(s1, 0.0, -1);

        if g2HasWeapon {
            //and remember to kill the bolton model in case we didn't get a thrown saber update first
            trap::G2API_RemoveGhoul2Model(engine, &raw mut ctx.world.entity_mut(centNum).ghoul2, 1);
            g2HasWeapon = false;
        }
        ctx.world.entity_mut(centNum).bolt3 = 0;
        ctx.world.entity_mut(centNum).bolt2 = 0;
    }

    if (ctx.world.entity(centNum).currentState.eFlags & EF_DEAD) != 0
        && !ctx.world.entity(centNum).ghoul2.is_null()
        && ctx.world.entity(centNum).currentState.saberInFlight != qfalse
        && g2HasWeapon
    {
        //special case, kill the saber on a freshly dead player if another source says to.
        trap::G2API_RemoveGhoul2Model(engine, &raw mut ctx.world.entity_mut(centNum).ghoul2, 1);
        g2HasWeapon = false;
    }
    let _ = g2HasWeapon;

    if iwantout != 0 {
        return;
    }

    let number = ctx.world.entity(centNum).currentState.number;
    if (snapFdForceActive & (1 << FP_SEE)) != 0 && snapClientNum != number {
        legs.shaderRGBA[0] = 255;
        legs.shaderRGBA[1] = 255;
        legs.shaderRGBA[2] = 0;
        legs.renderfx |= RF_MINLIGHT;
    }

    if snapDuelInProgress != qfalse {
        //I guess go ahead and glow your own client too in a duel
        if number != snapDuelIndex && number != snapClientNum {
            //everyone not involved in the duel is drawn very dark
            legs.shaderRGBA[0] = (legs.shaderRGBA[0] as f32 / 5.0) as i32 as u8;
            legs.shaderRGBA[1] = (legs.shaderRGBA[1] as f32 / 5.0) as i32 as u8;
            legs.shaderRGBA[2] = (legs.shaderRGBA[2] as f32 / 5.0) as i32 as u8;
            legs.renderfx |= RF_RGB_TINT;
        } else {
            //adjust the glow by how far away you are from your dueling partner
            let duelLerp = ctx.world.entity(snapDuelIndex as usize).lerpOrigin;
            let mut vecSub: vec3_t = [0.0; 3];
            _VectorSubtract(duelLerp, snapOrigin, &mut vecSub);
            let mut subLen = VectorLength(vecSub);

            if subLen < 1.0 {
                subLen = 1.0;
            }
            if subLen > 1020.0 {
                subLen = 1020.0;
            }

            let savRGBA: [u8; 3] = [legs.shaderRGBA[0], legs.shaderRGBA[1], legs.shaderRGBA[2]];
            legs.shaderRGBA[0] = (255.0 - subLen / 4.0).max(1.0) as i32 as u8;
            legs.shaderRGBA[1] = (255.0 - subLen / 4.0).max(1.0) as i32 as u8;
            legs.shaderRGBA[2] = (255.0 - subLen / 4.0).max(1.0) as i32 as u8;

            legs.renderfx &= !RF_RGB_TINT;
            legs.renderfx &= !RF_FORCE_ENT_ALPHA;
            legs.customShader = ctx.world.cgs.media.forceShell;

            trap::R_AddRefEntityToScene(engine, &legs); //draw the shell

            legs.customShader = 0; //reset to player model

            legs.shaderRGBA[0] = (savRGBA[0] as f32 - subLen / 8.0).max(1.0) as i32 as u8;
            legs.shaderRGBA[1] = (savRGBA[1] as f32 - subLen / 8.0).max(1.0) as i32 as u8;
            legs.shaderRGBA[2] = (savRGBA[2] as f32 - subLen / 8.0).max(1.0) as i32 as u8;

            if subLen <= 1024.0 {
                legs.renderfx |= RF_RGB_TINT;
            }
        }
    } else if ctx.world.entity(centNum).currentState.bolt1 != 0
        && (ctx.world.entity(centNum).currentState.eFlags & EF_DEAD) == 0
        && number != snapClientNum
        && (snapDuelInProgress == qfalse || snapDuelIndex != number)
    {
        legs.shaderRGBA[0] = 50;
        legs.shaderRGBA[1] = 50;
        legs.shaderRGBA[2] = 50;
        legs.renderfx |= RF_RGB_TINT;
    }

    if (ctx.world.entity(centNum).currentState.eFlags & EF_DISINTEGRATION) != 0 {
        if ctx.world.entity(centNum).dustTrailTime == 0 {
            let cgTime = ctx.world.cg.time;
            ctx.world.entity_mut(centNum).dustTrailTime = cgTime;
            ctx.world.entity_mut(centNum).miscTime = legs.frame;
        }

        if (ctx.world.cg.time - ctx.world.entity(centNum).dustTrailTime) > 1500 {
            //avoid rendering the entity after disintegration has finished anyway
            return;
        }

        let miscTime = ctx.world.entity(centNum).miscTime;
        let cgTime = ctx.world.cg.time;
        trap::G2API_SetBoneAnim(
            engine,
            legs.ghoul2,
            0,
            "model_root",
            miscTime,
            miscTime,
            BONE_ANIM_OVERRIDE_FREEZE,
            1.0,
            cgTime,
            miscTime as f32,
            -1,
        );

        if ctx.world.entity(centNum).noLumbar == qfalse {
            trap::G2API_SetBoneAnim(
                engine,
                legs.ghoul2,
                0,
                "lower_lumbar",
                miscTime,
                miscTime,
                BONE_ANIM_OVERRIDE_FREEZE,
                1.0,
                cgTime,
                miscTime as f32,
                -1,
            );

            if ctx.world.entity(centNum).localAnimIndex <= 1 {
                trap::G2API_SetBoneAnim(
                    engine,
                    legs.ghoul2,
                    0,
                    "Motion",
                    miscTime,
                    miscTime,
                    BONE_ANIM_OVERRIDE_FREEZE,
                    1.0,
                    cgTime,
                    miscTime as f32,
                    -1,
                );
            }
        }

        // CG_Disintegration re-reads the entity by number, so the slot must stay
        // in place (no take/put-back); `legs` is a local, no borrow conflict.
        CG_Disintegration(ctx, centNum, &mut legs);

        return;
    } else {
        ctx.world.entity_mut(centNum).dustTrailTime = 0;
        ctx.world.entity_mut(centNum).miscTime = 0;
    }

    if (ctx.world.entity(centNum).currentState.powerups & (1 << PW_CLOAKED)) != 0 {
        if ctx.world.entity(centNum).cloaked == qfalse {
            ctx.world.entity_mut(centNum).cloaked = qtrue;
            let cgTime = ctx.world.cg.time;
            ctx.world.entity_mut(centNum).uncloaking = cgTime + 2000;
        }
    } else if ctx.world.entity(centNum).cloaked != qfalse {
        ctx.world.entity_mut(centNum).cloaked = qfalse;
        let cgTime = ctx.world.cg.time;
        ctx.world.entity_mut(centNum).uncloaking = cgTime + 2000;
    }

    if ctx.world.entity(centNum).uncloaking > ctx.world.cg.time {
        //in the middle of cloaking
        let number = ctx.world.entity(centNum).currentState.number;
        if (snapFdForceActive & (1 << FP_SEE)) != 0 && snapClientNum != number {
            //just draw him
            trap::R_AddRefEntityToScene(engine, &legs);
        } else {
            let uncloaking = ctx.world.entity(centNum).uncloaking;
            let cgTime = ctx.world.cg.time;
            let mut perc = (uncloaking - cgTime) as f32 / 2000.0;
            if (ctx.world.entity(centNum).currentState.powerups & (1 << PW_CLOAKED)) != 0 {
                //actually cloaking, so reverse it
                perc = 1.0 - perc;
            }

            if (0.0..=1.0).contains(&perc) {
                legs.renderfx &= !RF_FORCE_ENT_ALPHA;
                legs.renderfx |= RF_RGB_TINT;
                let v = (255.0 * perc) as i32 as u8;
                legs.shaderRGBA[0] = v;
                legs.shaderRGBA[1] = v;
                legs.shaderRGBA[2] = v;
                legs.shaderRGBA[3] = 0;
                legs.customShader = ctx.world.cgs.media.cloakedShader;
                trap::R_AddRefEntityToScene(engine, &legs);

                legs.shaderRGBA[0] = 255;
                legs.shaderRGBA[1] = 255;
                legs.shaderRGBA[2] = 255;
                legs.shaderRGBA[3] = (255.0 * (1.0 - perc)) as i32 as u8; // let model alpha in
                legs.customShader = 0; // use regular skin
                legs.renderfx &= !RF_RGB_TINT;
                legs.renderfx |= RF_FORCE_ENT_ALPHA;
                trap::R_AddRefEntityToScene(engine, &legs);
            }
        }
    } else if (ctx.world.entity(centNum).currentState.powerups & (1 << PW_CLOAKED)) != 0 {
        //fully cloaked
        let number = ctx.world.entity(centNum).currentState.number;
        if (snapFdForceActive & (1 << FP_SEE)) != 0 && snapClientNum != number {
            //just draw him
            trap::R_AddRefEntityToScene(engine, &legs);
        } else if ctx.world.cg.renderingThirdPerson != qfalse
            || number != ctx.world.cg.predictedPlayerState.clientNum
        {
            if ctx.world.cvars.cg_shadows.integer != 2
                && ctx.world.cgs.glconfig.stencilBits >= 4
                && ctx.world.cvars.cg_renderToTextureFX.integer != 0
            {
                trap::R_SetRefractProp(engine, 1.0, 0.0, false, false); //don't need to do this every frame.. but..
                legs.customShader = 2; //crazy "refractive" shader
                trap::R_AddRefEntityToScene(engine, &legs);
                legs.customShader = 0;
            } else {
                //stencil buffer's in use, sorry
                legs.renderfx = 0; //&= ~(RF_RGB_TINT|RF_ALPHA_FADE);
                legs.shaderRGBA[0] = 255;
                legs.shaderRGBA[1] = 255;
                legs.shaderRGBA[2] = 255;
                legs.shaderRGBA[3] = 255;
                legs.customShader = ctx.world.cgs.media.cloakedShader;
                trap::R_AddRefEntityToScene(engine, &legs);
                legs.customShader = 0;
            }
        }
    }

    if (ctx.world.entity(centNum).currentState.powerups & (1 << PW_CLOAKED)) == 0 {
        //don't add the normal model if cloaked
        CG_CheckThirdPersonAlpha(ctx, centNum, &mut legs);
        trap::R_AddRefEntityToScene(engine, &legs);
    }

    let f1 = ctx.world.entity(centNum).frame_minus1;
    ctx.world.entity_mut(centNum).frame_minus2 = f1;

    if ctx.world.entity(centNum).frame_minus1_refreshed != 0 {
        ctx.world.entity_mut(centNum).frame_minus2_refreshed = 1;
    }

    ctx.world.entity_mut(centNum).frame_minus1 = legs.origin;

    ctx.world.entity_mut(centNum).frame_minus1_refreshed = 1;

    if ctx.world.entity(centNum).frame_hold_refreshed == 0
        && (ctx.world.entity(centNum).currentState.powerups & (1 << PW_SPEEDBURST)) != 0
    {
        let cgTime = ctx.world.cg.time;
        ctx.world.entity_mut(centNum).frame_hold_time = cgTime + 254;
    }

    if ctx.world.entity(centNum).frame_hold_time >= ctx.world.cg.time {
        let mut reframe_hold;

        if ctx.world.entity(centNum).frame_hold_refreshed == 0 {
            //We're taking the ghoul2 instance from the original refent and duplicating it onto our refent alias
            let frame_hold = ctx.world.entity(centNum).frame_hold;
            let centGhoul2 = ctx.world.entity(centNum).ghoul2;
            if !frame_hold.is_null()
                && trap::G2_HaveWeGhoul2Models(engine, frame_hold)
                && frame_hold != centGhoul2
            {
                let mut g2p = frame_hold;
                trap::G2API_CleanGhoul2Models(engine, &mut g2p);
                ctx.world.entity_mut(centNum).frame_hold = g2p;
            }
            reframe_hold = legs;
            ctx.world.entity_mut(centNum).frame_hold_refreshed = 1;
            reframe_hold.ghoul2 = null_mut();

            let centGhoul2 = ctx.world.entity(centNum).ghoul2;
            let mut g2p = ctx.world.entity(centNum).frame_hold;
            trap::G2API_DuplicateGhoul2Instance(engine, centGhoul2, &mut g2p);
            ctx.world.entity_mut(centNum).frame_hold = g2p;

            //Set the animation to the current frame and freeze on end
            let cgTime = ctx.world.cg.time;
            let holdGhoul2 = ctx.world.entity(centNum).frame_hold;
            trap::G2API_SetBoneAnim(
                engine,
                holdGhoul2,
                0,
                "model_root",
                legs.frame,
                legs.frame,
                0,
                1.0,
                cgTime,
                legs.frame as f32,
                -1,
            );
        } else {
            reframe_hold = legs;
            reframe_hold.ghoul2 = ctx.world.entity(centNum).frame_hold;
        }

        reframe_hold.renderfx |= RF_FORCE_ENT_ALPHA;
        let frame_hold_time = ctx.world.entity(centNum).frame_hold_time;
        let cgTime = ctx.world.cg.time;
        reframe_hold.shaderRGBA[3] = (frame_hold_time - cgTime) as u8;
        if reframe_hold.shaderRGBA[3] > 254 {
            reframe_hold.shaderRGBA[3] = 254;
        }
        if reframe_hold.shaderRGBA[3] < 1 {
            reframe_hold.shaderRGBA[3] = 1;
        }

        reframe_hold.ghoul2 = ctx.world.entity(centNum).frame_hold;
        trap::R_AddRefEntityToScene(engine, &reframe_hold);
    } else {
        ctx.world.entity_mut(centNum).frame_hold_refreshed = 0;
    }

    //
    // add the gun / barrel / flash
    //
    if ctx.world.entity(centNum).currentState.weapon != WP_EMPLACED_GUN {
        CG_AddPlayerWeapon(ctx, &legs, None, centNum, team, &rootAngles, true);
    }
    // add powerups floating behind the player
    {
        // bitwise copy-in, original left in place - a zeroed-swap is visible to
        // every ctx-reading helper inside CG_PlayerPowerups (C6b referee catch).
        // The copy is read-only, so it does not write back.
        // SAFETY: `npcClient` is an owned Box, so the ManuallyDrop copy never
        // drops. The original keeps the one live Box.
        let cent_tmp = ManuallyDrop::new(unsafe { core::ptr::read(ctx.world.entity(centNum)) });
        CG_PlayerPowerups(ctx, &cent_tmp, &legs);
    }

    let number = ctx.world.entity(centNum).currentState.number;
    if (ctx.world.entity(centNum).currentState.forcePowersActive & (1 << FP_RAGE)) != 0
        && (ctx.world.cg.renderingThirdPerson != qfalse || number != snapClientNum)
    {
        legs.renderfx &= !RF_FORCE_ENT_ALPHA;
        legs.renderfx &= !RF_MINLIGHT;

        legs.renderfx |= RF_RGB_TINT;
        legs.shaderRGBA[0] = 255;
        legs.shaderRGBA[1] = 0;
        legs.shaderRGBA[2] = 0;
        legs.shaderRGBA[3] = 255;

        if ctx.world.bg_state.rng.rand() & 1 != 0 {
            legs.customShader = ctx.world.cgs.media.electricBodyShader;
        } else {
            legs.customShader = ctx.world.cgs.media.electricBody2Shader;
        }

        trap::R_AddRefEntityToScene(engine, &legs);
    }

    if snapDuelInProgress == qfalse
        && ctx.world.entity(centNum).currentState.bolt1 != 0
        && (ctx.world.entity(centNum).currentState.eFlags & EF_DEAD) == 0
        && number != snapClientNum
        && (snapDuelInProgress == qfalse || snapDuelIndex != number)
    {
        legs.shaderRGBA[0] = 50;
        legs.shaderRGBA[1] = 50;
        legs.shaderRGBA[2] = 255;

        legs.renderfx &= !RF_RGB_TINT;
        legs.renderfx &= !RF_FORCE_ENT_ALPHA;
        legs.customShader = ctx.world.cgs.media.forceSightBubble;

        trap::R_AddRefEntityToScene(engine, &legs);
    }

    let drawVehShields = CG_VehicleShouldDrawShields(ctx.world, ctx.world.entity(centNum))
        || (checkDroidShields && {
            let vehNum = ctx.world.entity(centNum).currentState.m_iVehicleNum as usize;
            CG_VehicleShouldDrawShields(ctx.world, ctx.world.entity(vehNum))
        });
    if drawVehShields {
        //Vehicles have form-fitting shields
        legs.shaderRGBA[0] = 255;
        legs.shaderRGBA[1] = 255;
        legs.shaderRGBA[2] = 255;
        let cgTime = ctx.world.cg.time;
        legs.shaderRGBA[3] = (10.0 + ((cgTime / 4) as f32 as f64).sin() * 128.0) as i32 as u8;

        legs.renderfx &= !RF_RGB_TINT;
        legs.renderfx &= !RF_FORCE_ENT_ALPHA;

        // the ship's own Vehicle_t, or the ridden vehicle's when a droid is
        // attached.
        let pVehId = if checkDroidShields {
            let vehNum = ctx.world.entity(centNum).currentState.m_iVehicleNum as usize;
            ctx.world.entity(vehNum).m_pVehicle
        } else {
            ctx.world.entity(centNum).m_pVehicle
        };
        let shieldShader = pVehId.and_then(|id| {
            let info_ptr = ctx.world.vehicle_pool[id.ent_num() as usize].m_pVehicleInfo;
            if info_ptr.is_null() {
                None
            } else {
                let h = unsafe { (*info_ptr).shieldShaderHandle };
                (h != 0).then_some(h)
            }
        });
        legs.customShader = match shieldShader {
            //use the vehicle-specific shader
            Some(h) => h,
            None => ctx.world.cgs.media.playerShieldDamage,
        };

        trap::R_AddRefEntityToScene(engine, &legs);
    }
    //For now, these two are using the old shield shader...
    if (ctx.world.entity(centNum).currentState.forcePowersActive & (1 << FP_PROTECT)) != 0 {
        //absorb is represented by green..
        let mut prot = legs;

        prot.shaderRGBA[0] = 0;
        prot.shaderRGBA[1] = 128;
        prot.shaderRGBA[2] = 0;
        prot.shaderRGBA[3] = 254;

        prot.renderfx &= !RF_RGB_TINT;
        prot.renderfx &= !RF_FORCE_ENT_ALPHA;
        prot.customShader = ctx.world.cgs.media.protectShader;

        trap::R_AddRefEntityToScene(engine, &prot);
    }
    //Showing only when the power has been active (absorbed something) recently now, instead of always.
    let number = ctx.world.entity(centNum).currentState.number;
    if (number == ctx.world.cg.predictedPlayerState.clientNum
        && (ctx.world.cg.predictedPlayerState.fd.forcePowersActive & (1 << FP_ABSORB)) != 0)
        || (ctx.world.entity(centNum).teamPowerEffectTime > ctx.world.cg.time
            && ctx.world.entity(centNum).teamPowerType == 3)
    {
        //absorb is represented by blue..
        legs.shaderRGBA[0] = 0;
        legs.shaderRGBA[1] = 0;
        legs.shaderRGBA[2] = 255;
        legs.shaderRGBA[3] = 254;

        legs.renderfx &= !RF_RGB_TINT;
        legs.renderfx &= !RF_FORCE_ENT_ALPHA;
        legs.customShader = ctx.world.cgs.media.playerShieldDamage;

        trap::R_AddRefEntityToScene(engine, &legs);
    }

    if ctx.world.entity(centNum).currentState.isJediMaster != qfalse && snapClientNum != number {
        legs.shaderRGBA[0] = 100;
        legs.shaderRGBA[1] = 100;
        legs.shaderRGBA[2] = 255;

        legs.renderfx &= !RF_RGB_TINT;
        legs.renderfx &= !RF_FORCE_ENT_ALPHA;
        legs.renderfx |= RF_NODEPTH;
        legs.customShader = ctx.world.cgs.media.forceShell;

        trap::R_AddRefEntityToScene(engine, &legs);

        legs.renderfx &= !RF_NODEPTH;
    }

    if (snapFdForceActive & (1 << FP_SEE)) != 0
        && snapClientNum != number
        && ctx.world.cvars.cg_auraShell.integer != 0
    {
        let teamCi = player_ci(ctx.world, is_npc, centNum, clientNum).team;
        if ctx.world.cgs.gametype == GT_SIEGE {
            // A team game
            let viewerTeam = ctx.world.cgs.clientinfo[snapClientNum as usize].team;
            if teamCi == TEAM_SPECTATOR || teamCi == TEAM_FREE {
                //yellow
                legs.shaderRGBA[0] = 255;
                legs.shaderRGBA[1] = 255;
                legs.shaderRGBA[2] = 0;
            } else if teamCi != viewerTeam {
                //red
                legs.shaderRGBA[0] = 255;
                legs.shaderRGBA[1] = 50;
                legs.shaderRGBA[2] = 50;
            } else {
                //green
                legs.shaderRGBA[0] = 50;
                legs.shaderRGBA[1] = 255;
                legs.shaderRGBA[2] = 50;
            }
        } else if ctx.world.cgs.gametype >= GT_TEAM {
            // A team game
            if teamCi == TEAM_RED {
                legs.shaderRGBA[0] = 255;
                legs.shaderRGBA[1] = 50;
                legs.shaderRGBA[2] = 50;
            } else if teamCi == TEAM_BLUE {
                legs.shaderRGBA[0] = 75;
                legs.shaderRGBA[1] = 75;
                legs.shaderRGBA[2] = 255;
            } else {
                legs.shaderRGBA[0] = 255;
                legs.shaderRGBA[1] = 255;
                legs.shaderRGBA[2] = 0;
            }
        } else {
            // Not a team game
            legs.shaderRGBA[0] = 255;
            legs.shaderRGBA[1] = 255;
            legs.shaderRGBA[2] = 0;
        }

        // See through walls.
        legs.renderfx |= RF_MINLIGHT | RF_NODEPTH;

        if snapSeeLevel < FORCE_LEVEL_2 {
            //only level 2+ can see players through walls
            legs.renderfx &= !RF_NODEPTH;
        }

        legs.renderfx &= !RF_RGB_TINT;
        legs.renderfx &= !RF_FORCE_ENT_ALPHA;
        legs.customShader = ctx.world.cgs.media.sightShell;

        trap::R_AddRefEntityToScene(engine, &legs);
    }

    // Electricity
    //------------------------------------------------
    if ctx.world.entity(centNum).currentState.emplacedOwner > ctx.world.cg.time {
        let emplacedOwner = ctx.world.entity(centNum).currentState.emplacedOwner;
        let cgTime = ctx.world.cg.time;
        let dif = emplacedOwner - cgTime;

        if dif > 0 && ctx.world.bg_state.rng.random() > 0.4 {
            // fade out over the last 500 ms
            let mut brightness = 255;

            if dif < 500 {
                brightness = ((dif as f32 - 500.0) / 500.0 * 255.0).floor() as i32;
            }

            legs.renderfx &= !RF_FORCE_ENT_ALPHA;
            legs.renderfx &= !RF_MINLIGHT;

            legs.renderfx |= RF_RGB_TINT;
            legs.shaderRGBA[0] = brightness as u8;
            legs.shaderRGBA[1] = brightness as u8;
            legs.shaderRGBA[2] = brightness as u8;
            legs.shaderRGBA[3] = 255;

            if ctx.world.bg_state.rng.rand() & 1 != 0 {
                legs.customShader = ctx.world.cgs.media.electricBodyShader;
            } else {
                legs.customShader = ctx.world.cgs.media.electricBody2Shader;
            }

            trap::R_AddRefEntityToScene(engine, &legs);

            if ctx.world.bg_state.rng.random() > 0.9 {
                let number = ctx.world.entity(centNum).currentState.number;
                let crackle = ctx.world.cgs.media.crackleSound;
                trap::S_StartSound(engine, None, number, CHAN_AUTO, crackle);
            }
        }

        let mut tempAngles: vec3_t = [0.0; 3];
        let lerpYaw = ctx.world.entity(centNum).lerpAngles[YAW];
        VectorSet(&mut tempAngles, 0.0, lerpYaw, 0.0);
        let legsOrigin = legs.origin;
        let boltShader = ctx.world.cgs.media.boltShader;
        CG_ForceElectrocution(ctx, centNum, &legsOrigin, &tempAngles, boltShader, false);
    }

    if (ctx.world.entity(centNum).currentState.powerups & (1 << PW_SHIELDHIT)) != 0 {
        let v = ctx.world.bg_state.rng.Q_irand(1, 255) as u8;
        legs.shaderRGBA[0] = v;
        legs.shaderRGBA[1] = v;
        legs.shaderRGBA[2] = v;

        legs.renderfx &= !RF_FORCE_ENT_ALPHA;
        legs.renderfx &= !RF_MINLIGHT;
        legs.renderfx &= !RF_RGB_TINT;
        legs.customShader = ctx.world.cgs.media.playerShieldDamage;

        trap::R_AddRefEntityToScene(engine, &legs);
    }
}
