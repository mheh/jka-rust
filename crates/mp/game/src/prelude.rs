//! `prelude` re-exports the types the jampgame function-skeleton modules need.
//!
//! `fnskel.py` generates function skeletons with parameter and return types that resolve against
//! already-ported crates, but it emits them without `use` statements.
//! This module re-exports every such type at one import path, routed through `mp_game`'s dependency
//! set (`mp_qshared`, `mp_bg`, `mp_abi`, plus the crate's own `world`, `client`, and other modules).
//! Each skeleton module opens with `use crate::prelude::*;`.
//!
//! This module holds no behavior, only re-exports.
//! The `//TODO: Port` markers for still-unported parameter types stay at their call sites in the
//! skeletons.

// This module is pure re-exports, with no code and no unsafe blocks.
#![deny(unsafe_code)]

// Raven scalar, handle, and FFI primitives.
// `native_*` is not a direct dependency of `mp_game`.
// The cross-mode primitives come through `mp_qshared`'s re-export umbrella, the same path the other
// modules use.
pub use core::ffi::{
    c_char, c_double, c_float, c_int, c_long, c_schar, c_short, c_uchar, c_uint, c_ulong, c_ushort,
    c_void,
};

// Raven `byte`: `typedef unsigned char byte`.
// Source: `oracle/codemp/game/q_shared.h:349`
pub use native_types::byte;

// The fnskel packets transcribe Raven constant spellings verbatim, without enumerating their owning
// module's `use`.
// These glob-imports resolve them against the same crates already named above.
// Explicit single-item imports below, for example `holdable_t`, are unaffected.
// Rust lets an explicit import shadow a glob without ambiguity.
pub use mp_bg::public::bg_itemlist::{bg_itemlist, bg_numItems};
pub use mp_bg::public::configstring::*;
pub use mp_bg::public::dm_flags::*;
pub use mp_bg::public::entity_effects::*;
pub use mp_bg::public::g_item::GItem;
pub use mp_bg::public::gametype::{
    gametype_t, GT_CTF, GT_CTY, GT_DUEL, GT_FFA, GT_HOLOCRON, GT_JEDIMASTER, GT_MAX_GAME_TYPE,
    GT_POWERDUEL, GT_SIEGE, GT_SINGLE_PLAYER, GT_TEAM,
};
pub use mp_bg::public::hyperspace::{HYPERSPACE_TELEPORT_FRAC, HYPERSPACE_TIME};
pub use mp_bg::public::item_id::ItemId;
pub use mp_bg::public::item_kind::ItemKind;
pub use mp_bg::public::item_type::*;
pub use mp_bg::public::powerup::*;
pub use mp_bg::public::saber_move_name::*;
pub use mp_bg::public::set_anim::*;
pub use mp_bg::public::team::*;
pub use mp_bg::public::{bg_parryDebounce, JUMP_VELOCITY};
pub use mp_bg::public::{CROUCH_VIEWHEIGHT, DEAD_VIEWHEIGHT, DEFAULT_MAXS_2, DEFAULT_VIEWHEIGHT};
pub use mp_bg::vehicles::e_weapon_pose::EWeaponPose;
pub use mp_bg::vehicles::e_weapon_pose::EWeaponPose::*;
pub use mp_bg::vehicles::veh_flags_t::vehFlags_t::*;
pub use mp_bg::vehicles::vehicle_s::{
    MAX_VEHICLE_TURRETS, MAX_VEHICLE_WEAPONS, VEHICLE_BASE, VEHICLE_NONE,
};
pub use mp_bg::vehicles::vehicle_type_t::vehicleType_t::*;
pub use mp_bg::weapons::weaponData;
pub use mp_bg::weapons::weapon_t::*;
pub use mp_bg::weapons::WP_MuzzlePoint;
pub use mp_qshared::common::mp::ghoul2::bone_flags::*;
pub use mp_qshared::common::mp::qcommon::pm_flags::*;
pub use mp_qshared::shared::force_powers::*;
pub use mp_qshared::shared::limits::*;
pub use mp_qshared::shared::sound_channel::*;
pub use mp_qshared::shared::surface_flags::*;

// Game-crate-local const families that were ported but never wired into the prelude glob.
// See `docs/porting-rules.md` §E13.
pub use mp_bg::bg_misc::{
    bgForcePowerCost, bgForcePowerCostSaberThrow, forceMasteryPoints, forcePowerDarkLight,
    forcePowerSorted,
};
pub use crate::ai_main_consts::*;
pub use crate::anim_table::animTable;
pub use crate::cstr_util::{cstr, cstr_from_chars, cstr_to_str, write_cstr_field};
pub use crate::entity::flags::*;
pub use crate::g_client::{playerMaxs, playerMins};
pub use crate::g_items::FRAMETIME;
pub use crate::g_items::{PLAYEREVENT_GAUNTLETREWARD, REWARD_SPRITE_TIME};
pub use crate::g_local_consts::*;
pub use crate::g_mover::{BMS_END, BMS_MID, BMS_START};
pub use crate::g_nav_consts::*;
pub use crate::g_public_consts::*;
pub use crate::g_target::Q3_SCRIPT_DIR;
pub use crate::level::damage_flags::*;
pub use crate::npc::ai_flags::*;
pub use crate::npc::check_flags::*;
pub use crate::npc::combat_point_flags::*;
pub use crate::npc::script_flags::*;
pub use crate::npc::squad_state::*;
pub use crate::q_math::{
    vec3_origin, vectoangles, RadiusFromBounds, PITCH, ROLL, VEC3_ORIGIN, YAW,
};
pub use crate::q_shared_cvar_flags::*;
pub use crate::w_force::mindTrickTime;
pub use mp_bg::bg_vehicleLoad_tables::*;
pub use mp_bg::vehicles::{vehFieldType_t, vehFieldType_t::*, vehField_t};
pub use mp_qshared::shared::RAND_MAX;
pub use native_string::sscanf_f32s;
// `BG_GiveMeVectorFromMatrix` lives in `bg_misc.c:736`.
// This is the export for bare-use sites.
pub use crate::saber::w_saber_consts::*;
pub use crate::teams::npcteam::*;
pub use mp_bg::bg_misc::BG_GiveMeVectorFromMatrix;

// Enum types transcribed as `#[repr(i32)] enum`, per porting-rules' enum-vs-alias fidelity rule.
// The fnskel packets carry their bare Raven variant spellings, for example `STAT_MAX_HEALTH`, not
// `statIndex_t::STAT_MAX_HEALTH`.
// So this file re-exports both the type name, for sites that do qualify it, and a variant glob, for
// the more common bare spelling.
pub use mp_bg::public::anim_number::{animNumber_t, animNumber_t::*};
pub use mp_bg::public::broken_limb::{brokenLimb_t, brokenLimb_t::*};
pub use mp_bg::public::effect_types::{effectTypes_t, effectTypes_t::*};
pub use mp_bg::public::entity_event::{entity_event_t, entity_event_t::*};
pub use mp_bg::public::entity_type::{entityType_t, entityType_t::*};
pub use mp_bg::public::force_hand_anims::{forceHandAnims_t, forceHandAnims_t::*};
pub use mp_bg::public::g2_model_parts::{g2ModelParts_t, g2ModelParts_t::*, G2_MODEL_PART};
pub use mp_bg::public::means_of_death::{meansOfDeath_t, meansOfDeath_t::*};
pub use mp_bg::public::pd_sounds::{pdSounds_t, pdSounds_t::*};
pub use mp_bg::public::pers_enum::{persEnum_t, persEnum_t::*};
pub use mp_bg::public::pmtype::{pmtype_t, pmtype_t::*};
pub use mp_bg::public::saber_quadrant::{
    saberQuadrant_t, Q_B, Q_BL, Q_BR, Q_L, Q_NUM_QUADS, Q_R, Q_T, Q_TL, Q_TR,
};
pub use mp_bg::public::stat_index::{statIndex_t, statIndex_t::*};
pub use mp_bg::public::weaponstate::{weaponstate_t, weaponstate_t::*};
pub use mp_qshared::common::mp::qcommon::b_set_t::{bSet_t, bSet_t::*};
pub use mp_qshared::common::mp::qcommon::b_state_t::bState_t::*;
pub use mp_qshared::common::mp::qcommon::nav_debug_draw::*;
pub use mp_qshared::common::mp::qcommon::task_id_t::{taskID_t, taskID_t::*};
pub use mp_qshared::common::mp::qcommon::usercmd_button::*;
pub use mp_qshared::shared::error_parm::{errorParm_t, errorParm_t::*};
pub use mp_qshared::shared::trackchan::{trackchan_t, trackchan_t::*};
pub use mp_qshared::shared::trajectory::trType_t::*;
pub use mp_qshared::shared::wl_e::{WL_e, WL_e::*};

// The module-island dispatch receiver, injected as the first param of every game-tier needs-ctx fn.
// See `world/game_context.rs`.
pub use crate::world::GameContext;

// The entity handle, the spawn fn-ID enum, and the bg-channel state/trait set, hoisted into the
// prelude so porter bodies can name them unqualified.
// - `EntityId`: the `Option<EntityId>` stored-field handle.
// - `EntSpawn`: the `spawns[]` classname-to-fn dispatch enum.
// - `BgState`, `PmoveContext`, `BgTraps`, `GameCallbacks`: the bg session and per-call state, plus
//   the two boundary traits, `BgTraps` and `GameCallbacks`.
// - `pml_t`: the bg pmove local working-set type.
pub use crate::bg_channel::{BgState, BgTraps, GameCallbacks, PmoveContext};
pub use crate::ent_fn_enums::EntSpawn;
// `EntityId` and the `ent - g_entities` helper functions.
// `Some(ent_id(base, p))` and `ent_id_opt(base, p)` fill the `Option<EntityId>` stored fields at
// pointer-assignment sites.
// `field.is_none()` and id-equality replace NULL and address compares.
pub use crate::world::{ent_id, ent_id_opt, to_num, EntityId};
// The index-to-pointer counterpart.
// `crate::ent_id::resolve` re-derives a `gentity_t*` pointer from a stored `Option<EntityId>` field.
pub use crate::ent_id;
pub use mp_bg::local::pml_t::pml_t;

// The `crate::trap` seam module, spelled bare `trap::X` throughout porter bodies.
// Re-exported so the `use crate::prelude::*` glob resolves those call sites.
pub use crate::trap;
// Raven `trap_R_RegisterSkin` (`g_syscalls.c:1179-1182`): re-exported under the Raven name for porter
// bodies that transcribe the bare spelling.
pub use crate::trap::R_RegisterSkin as trap_R_RegisterSkin;
// Raven `trap_G2API_*` re-exports under Raven names for porter bodies that transcribe the bare
// spellings, from `oracle/codemp/game/g_syscalls.c`.
pub use crate::trap::G2API_AnimateG2Models as trap_G2API_AnimateG2Models;
pub use crate::trap::G2API_CleanGhoul2Models as trap_G2API_CleanGhoul2Models;
pub use crate::trap::G2API_GetBoltMatrix as trap_G2API_GetBoltMatrix;
pub use crate::trap::G2API_GetBoltMatrix_NoRecNoRot as trap_G2API_GetBoltMatrix_NoRecNoRot;
pub use crate::trap::G2API_GetBoltMatrix_NoReconstruct as trap_G2API_GetBoltMatrix_NoReconstruct;
pub use crate::trap::G2API_GetBoneAnim as trap_G2API_GetBoneAnim;
pub use crate::trap::G2API_IKMove as trap_G2API_IKMove;
pub use crate::trap::G2API_InitGhoul2Model as trap_G2API_InitGhoul2Model;
pub use crate::trap::G2API_SetBoneAngles as trap_G2API_SetBoneAngles;
pub use crate::trap::G2API_SetBoneAnim as trap_G2API_SetBoneAnim;
pub use crate::trap::G2API_SetBoneIKState as trap_G2API_SetBoneIKState;
pub use crate::trap::G2API_SetRagDoll as trap_G2API_SetRagDoll;
pub use crate::trap::TrueFree as trap_TrueFree;
pub use crate::trap::TrueMalloc as trap_TrueMalloc;

// The entity fn-ID dispatch enums (`ent_fn_enums`), named bare at spawn, think, touch, and other
// assignment sites.
pub use crate::ent_fn_enums::{
    EntBlocked, EntDie, EntPain, EntReached, EntThink, EntTouch, EntUse, FnId,
};

pub use crate::ai::group_info::AIGroupInfo_t;
pub use crate::botai::bot_state_s::bot_state_t;
pub use crate::client::gclient::gclient_t;
pub use crate::level::alert_event::{alertEventLevel_e, alertEventLevel_e::*};
pub use crate::level::reference_tag::reference_tag_t;
pub use crate::level::waypoint_data::waypointData_t;
pub use crate::npc::g_npc_t::gNPC_t;
pub use crate::npc::jump_state_t::{jumpState_t, jumpState_t::*};
pub use crate::npc::nav_info_s::{
    navInfo_t, NIF_BLOCKED, NIF_COLLISION, NIF_FAILED, NIF_MACRO_NAV, NIF_NONE,
};
pub use crate::npc::spot_t::{spot_t, spot_t::*};
pub use crate::npc::visibility_t::{visibility_t, visibility_t::*};
pub use crate::saber::evasion_type_t::evasionType_t;
pub use crate::teams::class::class_t;
pub use crate::teams::class::class_t::*;

pub use mp_bg::public::animation::animation_t;
pub use mp_bg::public::bg_field::BG_field_t;
pub use mp_bg::public::holdable::{
    holdable_t, HI_AMMODISP, HI_BINOCULARS, HI_CLOAK, HI_EWEB, HI_HEALTHDISP, HI_JETPACK,
    HI_MEDPAC, HI_MEDPAC_BIG, HI_NONE, HI_NUM_HOLDABLE, HI_SEEKER, HI_SENTRY_GUN, HI_SHIELD,
};
pub use mp_bg::public::pmove_t::pmove_t;
pub use mp_bg::public::powerup::powerup_t;
pub use mp_bg::public::saber_move_name::saberMoveName_t;
pub use mp_bg::public::team::team_t;
pub use mp_bg::saga::siege_class_desc_t::{siegeClassDesc_t, SIEGE_CLASS_DESC_LEN};
pub use mp_bg::saga::siege_class_flags_t::siegeClassFlags_t::*;
pub use mp_bg::saga::siege_class_t::{siegeClass_t, MAX_SIEGE_CLASSES};
pub use mp_bg::saga::siege_player_class_flags_t::siegePlayerClassFlags_t::{self, *};
pub use mp_bg::saga::siege_team_t::{
    siegeTeam_t, MAX_EXDATA_ENTS_TO_SEND, MAX_SIEGE_INFO_SIZE, SIEGETEAM_TEAM1, SIEGETEAM_TEAM2,
    SIEGE_POINTS_FINALOBJECTIVECOMPLETED, SIEGE_POINTS_OBJECTIVECOMPLETED,
    SIEGE_POINTS_TEAMWONROUND, SIEGE_ROUND_BEGIN_TIME,
};
pub use mp_bg::vehicles::turret_stats_t::turretStats_t;
pub use mp_bg::vehicles::veh_weapon_info_t::vehWeaponInfo_t;
pub use mp_bg::vehicles::vehicle_info_t::vehicleInfo_t;
pub use mp_bg::vehicles::vehicle_s::Vehicle_t;
pub use mp_bg::weapons::ammo_t::ammo_t;
pub use mp_bg::weapons::ammo_t::ammo_t::*;
pub use mp_bg::weapons::weapon_t::weapon_t;

pub use mp_qshared::common::mp::botlib::aas_entityinfo_s::aas_entityinfo_t;
pub use mp_qshared::common::mp::botlib::action::{
    ACTION_ALT_ATTACK, ACTION_ATTACK, ACTION_CROUCH, ACTION_DELAYEDJUMP, ACTION_FORCEPOWER,
    ACTION_GESTURE, ACTION_JUMP, ACTION_MOVEBACK, ACTION_MOVEDOWN, ACTION_MOVEFORWARD,
    ACTION_MOVELEFT, ACTION_MOVERIGHT, ACTION_MOVEUP, ACTION_RESPAWN, ACTION_TALK, ACTION_USE,
    ACTION_WALK,
};
pub use mp_qshared::common::mp::botlib::bot_input_s::bot_input_t;
pub use mp_qshared::common::mp::botlib::print_type::{
    PRT_ERROR, PRT_EXIT, PRT_FATAL, PRT_MESSAGE, PRT_WARNING,
};
// `gentity_t` moved to `mp_game` (DEC-26).
// Its constants and typedefs stay in qshared.
pub use crate::entity::{gentity_t, PrefixSet, PrefixSlot};
pub use mp_qshared::common::mp::gentity::{
    material_t, moverState_t, MAT_CRATE1, MAT_CRATE2, MAT_DRK_STONE, MAT_ELECTRICAL,
    MAT_ELEC_METAL, MAT_GLASS, MAT_GLASS_METAL, MAT_GRATE1, MAT_GREY_STONE, MAT_LT_STONE,
    MAT_METAL, MAT_METAL2, MAT_METAL3, MAT_NONE, MAT_ROPE, MAT_SNOWY_ROCK, MAT_WHITE_METAL,
    MOVER_1TO2, MOVER_2TO1, MOVER_POS1, MOVER_POS2, NUM_MATERIALS,
};

// Raven `#define bgEntity_t gentity_t` appears in jampgame source files, for example `g_vehicles.c`
// and `FighterNPC.c`.
// In the oracle, this macro makes `bgEntity_t` and `gentity_t` interchangeable at call sites that
// need server-side fields like `spawnflags`.
// For game-code bodies that cast `gentity_t` parameters through `bgEntity_t`, we re-export
// `gentity_t` under the `bgEntity_t` name, for example `(*bgEntity).spawnflags`.
// Source: oracle/codemp/game/g_vehicles.c, FighterNPC.c, etc. (local macro)
pub use crate::entity::gentity_t as bgEntity_t;
pub use mp_qshared::common::mp::qcommon::b_state_t::bState_t;
pub use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
pub use mp_qshared::common::mp::qcommon::failed_edge::failedEdge_t;
pub use mp_qshared::common::mp::qcommon::player_state::{
    forcedata_t, playerState_t, MAX_POWERUPS, MAX_PS_EVENTS,
};
pub use mp_qshared::common::mp::qcommon::qtime::qtime_t;
pub use mp_qshared::common::mp::qcommon::saber::blade_info::bladeInfo_t;
pub use mp_qshared::common::mp::qcommon::saber::saber_colors::saber_colors_t;
pub use mp_qshared::common::mp::qcommon::saber::saber_info::{saberInfo_t, MAX_SABERS};
pub use mp_qshared::common::mp::qcommon::saber::saber_styles::saber_styles_t;
pub use mp_qshared::common::mp::qcommon::shared_ragdoll_params::sharedRagDollParams_t;
pub use mp_qshared::common::mp::qcommon::shared_ragdoll_update_params::sharedRagDollUpdateParams_t;
pub use mp_qshared::common::mp::qcommon::shared_set_bone_ik_state_params::sharedSetBoneIKStateParams_t;
pub use mp_qshared::common::mp::qcommon::siege_pers::siegePers_t;
pub use mp_qshared::common::mp::qcommon::usercmd::usercmd_t;
pub use mp_qshared::common::mp::trace_t::trace_t;
pub use mp_qshared::shared::collision::{cplane_t, CollisionRecord_t};
pub use mp_qshared::shared::cvar::vmCvar_t;
pub use mp_qshared::shared::flag_status::flagStatus_t;
pub use mp_qshared::shared::force_powers::forcePowers_t;
pub use mp_qshared::shared::fsMode_t;
pub use mp_qshared::shared::pc_token_t;
pub use mp_qshared::shared::qint64;
pub use mp_qshared::shared::sharedIKMoveParams_t;
pub use mp_qshared::shared::string_id_table::stringID_table_t;
pub use mp_qshared::shared::trajectory::{trType_t, trajectory_t};
pub use mp_qshared::shared::wpobject::wpobject_t;
pub use mp_qshared::shared::Eorientations::*;
pub use mp_qshared::shared::{
    fileHandle_t, mdxaBone_t, qboolean, qfalse, qhandle_t, qtrue, vec3_t, vec4_t, vec_t,
    Eorientations, MAX_QPATH,
};

// A batch re-export of game-crate-local fns spelled bare in porter bodies but never wired into the
// prelude.
// Each resolves to a single `pub fn` or `const` definition.
pub use crate::cstr_util::atoi;
pub use crate::g_client::SpotWouldTelefrag2;
pub use crate::g_combat::{G_CheckVehicleNPCTeamDamage, G_Damage, G_RadiusDamage};
pub use crate::g_items::{RegisterItem, Touch_Item};
pub use crate::g_log::G_LogWeaponFire;
pub use crate::g_main::{Com_Printf, G_RunThink};
pub use crate::g_nav::{FlyingCreature, NAV_HitNavGoal, NPC_SetMoveGoal};
pub use crate::g_spawn::{G_SpawnFloat, G_SpawnInt, G_SpawnString, G_SpawnVector};
pub use crate::g_timer::TIMER_Done;
pub use crate::g_utils::{
    EntFindField, G_AddEvent, G_Find, G_FreeEntity, G_ModelIndex, G_PlayEffect, G_ScaleNetHealth,
    G_SetAnim, G_SetMovedir, G_SetOrigin, G_Sound, G_SoundIndex, G_SoundSetIndex, G_Spawn,
    G_TeamCommand, G_TempEntity, G_UseTargets2, TryHeal,
};
pub use crate::g_weapon::{laserTrapStick, LogAccuracyHit};
pub use crate::q_math::{
    AddPointToBounds, AngleDifference, AngleSubtract, AngleVectors, CrossProduct, DirToByte,
    Distance, DistanceHorizontalSquared, G_FindClosestPointOnLineSegment, Q_fabs, VectorCompare,
    VectorCompare2, VectorLength, VectorLengthSquared, VectorNPos, VectorNormalize, ANGLE2SHORT,
    SHORT2ANGLE,
};
pub use crate::q_shared::{
    va, COM_StripExtension, GetIDForString, Q_strchr, Q_strcmp, Q_stricmp, Q_strncmp, Q_strncpyz,
    Q_strupr,
};
pub use crate::w_saber::WP_SaberCanBlock;
pub use crate::NPC_AI_Atst::NPC_ATST_Precache;
pub use crate::NPC_AI_Droid::{
    NPC_Gonk_Precache, NPC_Mouse_Precache, NPC_Protocol_Precache, NPC_R2D2_Precache,
    NPC_R5D2_Precache,
};
pub use crate::NPC_AI_GalakMech::NPC_GalakMech_Precache;
pub use crate::NPC_AI_Howler::NPC_Howler_Precache;
pub use crate::NPC_AI_ImperialProbe::NPC_Probe_Precache;
pub use crate::NPC_AI_Interrogator::NPC_Interrogator_Precache;
pub use crate::NPC_AI_Jedi::{Jedi_Decloak, NPC_ShadowTrooper_Precache};
pub use crate::NPC_AI_Mark1::NPC_Mark1_Precache;
pub use crate::NPC_AI_Mark2::NPC_Mark2_Precache;
pub use crate::NPC_AI_MineMonster::NPC_MineMonster_Precache;
pub use crate::NPC_AI_Remote::NPC_Remote_Precache;
pub use crate::NPC_AI_Seeker::NPC_Seeker_Precache;
pub use crate::NPC_AI_Sentry::NPC_Sentry_Precache;
pub use crate::NPC_AI_Stormtrooper::NPC_CheckPlayerTeamStealth;
pub use crate::NPC_AI_Wampa::NPC_Wampa_Precache;
pub use crate::NPC_behavior::NPC_StartFlee;
pub use crate::NPC_combat::{
    CanShoot, EntIsGlass, G_ClearEnemy, G_SetEnemy, NPC_AimAdjust, NPC_ChangeWeapon,
    NPC_FindCombatPoint, NPC_FreeCombatPoint, NPC_SetCombatPoint, NPC_ShotEntity, ShotThroughGlass,
    WeaponThink,
};
pub use crate::NPC_goal::{NPC_ClearGoal, NPC_ReachedGoal, UpdateGoal};
pub use crate::NPC_move::{NAV_GetLastMove, NPC_MoveToGoal, NPC_SlideMoveToGoal};
pub use crate::NPC_senses::{InFOV3, NPC_CheckAlertEvents, NPC_CheckForDanger};
pub use crate::NPC_utils::{
    CalcEntitySpot, NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_FaceEnemy, NPC_UpdateAngles,
};
pub use mp_bg::bg_misc::{BG_EmplacedView, BG_FindItemForWeapon};
pub use mp_bg::bg_panimate::{BG_InKnockDownOnly, BG_InReboundHold, BG_InReboundJump};
pub use mp_bg::bg_pmove::BG_SabersOff;
pub use mp_bg::vehicles::fighter_npc::FighterIsLanded;
pub use native_string::{Info_RemoveKey, Info_Validate, Info_ValueForKey};

// A file-level symbol re-export block, generated by `preflight.py`.
pub use crate::ai::consts::*;
pub use crate::ai::distance::*;
pub use crate::ai::rank::*;
pub use crate::ai_wpnav::*;
pub use crate::botai::bweaponrange::*;
pub use crate::client::client_connected::*;
pub use crate::client::client_persistant::*;
pub use crate::g_active::*;
pub use crate::g_client::*;
pub use crate::g_cmds::*;
pub use crate::g_icarus_set_type::{setTable, setType_t, setType_t::*};
pub use crate::g_items::*;
pub use crate::g_mem::G_Alloc;
pub use crate::g_mover::*;
pub use crate::g_spawn::MAX_AMBIENT_SETS;
pub use crate::g_spawn::*;
pub use crate::g_svcmds::MAX_IPFILTERS;
pub use crate::g_team::*;
pub use crate::g_timer::*;
pub use crate::g_trigger::*;
pub use crate::g_turret::*;
pub use crate::g_turret_G2::*;
pub use crate::g_vehicles::*;
pub use crate::game_globals::MAX_ITEMS;
pub use crate::level::alert_event::MAX_ALERT_EVENTS;
pub use crate::level::combat_point::MAX_COMBAT_POINTS;
pub use crate::level::interest_point::MAX_INTEREST_POINTS;
pub use crate::level::level_locals::BODY_QUEUE_SIZE;
pub use crate::level::reference_tag::*;
pub use crate::level::tag_owner::*;
pub use crate::npc_c::*;
pub use crate::q_math::*;
pub use crate::saber::saber_flags::*;
pub use crate::tri_coll_test::*;
pub use crate::NPC_AI_Atst::*;
pub use crate::NPC_AI_Droid::*;
pub use crate::NPC_AI_GalakMech::*;
pub use crate::NPC_AI_Grenadier::*;
pub use crate::NPC_AI_Howler::*;
pub use crate::NPC_AI_ImperialProbe::*;
pub use crate::NPC_AI_Interrogator::*;
pub use crate::NPC_AI_Mark1::*;
pub use crate::NPC_AI_Mark2::*;
pub use crate::NPC_AI_MineMonster::MAX_DISTANCE;
pub use crate::NPC_AI_Seeker::*;
pub use crate::NPC_AI_Sentry::*;
pub use crate::NPC_AI_Stormtrooper::*;
pub use crate::NPC_AI_Utils::MAX_RADIUS_ENTS;
pub use crate::NPC_behavior::*;
pub use crate::NPC_combat::*;
pub use crate::NPC_senses::*;
pub use crate::NPC_spawn::*;
pub use crate::NPC_stats::BSTable;
pub use crate::NPC_utils::*;
pub use mp_bg::bg_misc::*;
pub use mp_bg::bg_pmove::MIN_WALK_NORMAL;
pub use mp_bg::bg_pmove::*;
pub use mp_bg::bg_saber::*;
pub use mp_bg::bg_saberLoad::*;
pub use mp_bg::bg_saga::{WPTable, SIEGECHAR_TAB};
pub use mp_bg::bg_slidemove::*;
pub use mp_bg::bg_vehicleLoad::BG_VehicleGetIndex;
pub use mp_bg::local::bg_toggleable_surfaces::bgToggleableSurfaces;
pub use mp_bg::local::force_levels::forceJumpStrength;
pub use mp_bg::local::force_levels::*;
pub use mp_bg::local::force_power_needed::forcePowerNeeded;
pub use mp_bg::public::dm_flags::DF_NO_FALLING;
pub use mp_bg::public::saber_move_data_table::saberMoveData;
pub use mp_bg::public::saber_move_data_table::*;
pub use mp_bg::public::saberlock::*;
pub use mp_bg::public::spawn::*;
pub use mp_bg::vehicles::turret_stats_t::MAX_VEHICLE_TURRET_MUZZLES;
pub use mp_bg::vehicles::vehicle_s::*;
pub use mp_bg::vehicles::vehicle_type_t::vehicleType_t;
pub use mp_bg::weapons::ammo_data::ammoData;
pub use mp_qshared::common::mp::gentity::NUM_BSETS;
pub use mp_qshared::common::mp::gentity::*;
pub use mp_qshared::common::mp::qcommon::parms::{parms_t, MAX_PARMS};
pub use mp_qshared::common::mp::qcommon::player_state::NUM_FORCE_POWERS;
pub use mp_qshared::common::mp::qcommon::player_state::*;
pub use mp_qshared::common::mp::qcommon::saber::blade_info::MAX_BLADES;
pub use mp_qshared::common::mp::qcommon::saber::saber_colors::*;
pub use mp_qshared::shared::cbuf_exec::{cbufExec_t, cbufExec_t::*};
pub use mp_qshared::shared::file_mode::{FS_APPEND, FS_APPEND_SYNC, FS_READ, FS_WRITE};
pub use mp_qshared::shared::flag_status::*;
pub use mp_qshared::shared::q_color::Q_COLOR_ESCAPE;
pub use mp_qshared::shared::saber_blocked_type::{saberBlockedType_t, saberBlockedType_t::*};
pub use mp_qshared::shared::wpobject::MAX_NEIGHBOR_SIZE;

// These lines pin the first-winner definition of consts that two modules transcribe with the same
// values, so the glob re-exports above stay unambiguous.
// A future duplicate-const consolidation pass removes one copy of each.
pub use crate::npc::squad_state::NUM_SQUAD_STATES;
pub use mp_bg::public::configstring::{CS_CLIENT_JEDIMASTER, CS_ITEMS};
pub use mp_qshared::common::mp::qcommon::task_id_t::taskID_t::NUM_TIDS;

// Raven `G_ICARUS_TASKIDPENDING` args re-export under the misspelled `GICARUSTaskIDPendingArgs`
// spelling that `NPC_sounds.rs` transcribes bare.
// The camelCase port name is `GIcarusTaskidpendingArgs`.
// Source: oracle/codemp/game/g_syscalls.c:329-332
pub use mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::GIcarusTaskidpendingArgs as GICARUSTaskIDPendingArgs;
