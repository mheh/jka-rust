//! `prelude` — the landing prelude for the migrated bg function modules.
//!
//! The bg modules (`bg_*.c` ports) open with `use crate::prelude::*;`. This is
//! the bg-tier filter of `mp_game`'s prelude: it re-exports only what those
//! modules need, sourced from the tiers at or below bg (`mp_qshared`, `mp_bg`
//! itself, and the moved bg modules). Game-tier symbols are reached through the
//! `BgTraps`/`GameCallbacks` seam, never re-exported here.
//!
//! No new behavior lives here — only re-exports.

// Raven scalar / handle / ffi primitives.
pub use core::ffi::{
    c_char, c_double, c_float, c_int, c_long, c_schar, c_short, c_uchar, c_uint, c_ulong, c_ushort,
    c_void,
};

// Raven `byte` (`q_shared.h:349`, `typedef unsigned char byte`).
// Source: `oracle/codemp/game/q_shared.h:349`
pub type byte = c_uchar;

// --- mp_bg (this crate) re-exports ---
pub use crate::local::bg_toggleable_surfaces::bgToggleableSurfaces;
pub use crate::local::force_levels::forceJumpStrength;
pub use crate::local::force_levels::*;
pub use crate::local::force_power_needed::forcePowerNeeded;
pub use crate::local::pml_t::pml_t;
pub use crate::public::anim_number::{animNumber_t, animNumber_t::*};
pub use crate::public::anim_table::animTable;
pub use crate::public::animation::animation_t;
pub use crate::public::bg_field::BG_field_t;
pub use crate::public::bg_itemlist::{bg_itemlist, bg_numItems};
pub use crate::public::broken_limb::{brokenLimb_t, brokenLimb_t::*};
pub use crate::public::configstring::*;
pub use crate::public::damage_flags::*;
pub use crate::public::dm_flags::*;
pub use crate::public::effect_types::{effectTypes_t, effectTypes_t::*};
pub use crate::public::entity_effects::*;
pub use crate::public::entity_event::{entity_event_t, entity_event_t::*};
pub use crate::public::entity_type::{entityType_t, entityType_t::*};
pub use crate::public::force_hand_anims::{forceHandAnims_t, forceHandAnims_t::*};
pub use crate::public::g2_model_parts::{g2ModelParts_t, g2ModelParts_t::*, G2_MODEL_PART};
pub use crate::public::g_item::GItem;
pub use crate::public::gametype::{
    gametype_t, GT_CTF, GT_CTY, GT_DUEL, GT_FFA, GT_HOLOCRON, GT_JEDIMASTER, GT_MAX_GAME_TYPE,
    GT_POWERDUEL, GT_SIEGE, GT_SINGLE_PLAYER, GT_TEAM,
};
pub use crate::public::holdable::{
    holdable_t, HI_AMMODISP, HI_BINOCULARS, HI_CLOAK, HI_EWEB, HI_HEALTHDISP, HI_JETPACK,
    HI_MEDPAC, HI_MEDPAC_BIG, HI_NONE, HI_NUM_HOLDABLE, HI_SEEKER, HI_SENTRY_GUN, HI_SHIELD,
};
pub use crate::public::hyperspace::{HYPERSPACE_TELEPORT_FRAC, HYPERSPACE_TIME};
pub use crate::public::item_id::ItemId;
pub use crate::public::item_kind::ItemKind;
pub use crate::public::item_type::*;
pub use crate::public::means_of_death::{meansOfDeath_t, meansOfDeath_t::*};
pub use crate::public::pd_sounds::{pdSounds_t, pdSounds_t::*};
pub use crate::public::pers_enum::{persEnum_t, persEnum_t::*};
pub use crate::public::pmove_t::pmove_t;
pub use crate::public::pmtype::{pmtype_t, pmtype_t::*};
pub use crate::public::powerup::{powerup_t, *};
pub use crate::public::saber_consts::*;
pub use crate::public::saber_move_data_table::{saberMoveData, *};
pub use crate::public::saber_move_name::{saberMoveName_t, *};
pub use crate::public::saber_quadrant::{
    saberQuadrant_t, Q_B, Q_BL, Q_BR, Q_L, Q_NUM_QUADS, Q_R, Q_T, Q_TL, Q_TR,
};
pub use crate::public::saberlock::*;
pub use crate::public::set_anim::*;
pub use crate::public::spawn::*;
pub use crate::public::stat_index::{statIndex_t, statIndex_t::*};
pub use crate::public::team::{team_t, *};
pub use crate::public::weaponstate::{weaponstate_t, weaponstate_t::*};
pub use crate::public::{bg_parryDebounce, JUMP_VELOCITY};
pub use crate::public::{CROUCH_VIEWHEIGHT, DEAD_VIEWHEIGHT, DEFAULT_MAXS_2, DEFAULT_VIEWHEIGHT};
pub use crate::saga::siege_class_desc_t::{siegeClassDesc_t, SIEGE_CLASS_DESC_LEN};
pub use crate::saga::siege_class_flags_t::siegeClassFlags_t::*;
pub use crate::saga::siege_class_t::{siegeClass_t, MAX_SIEGE_CLASSES};
pub use crate::saga::siege_player_class_flags_t::siegePlayerClassFlags_t::{self, *};
pub use crate::saga::siege_team_t::{
    siegeTeam_t, MAX_EXDATA_ENTS_TO_SEND, MAX_SIEGE_INFO_SIZE, SIEGETEAM_TEAM1, SIEGETEAM_TEAM2,
    SIEGE_POINTS_FINALOBJECTIVECOMPLETED, SIEGE_POINTS_OBJECTIVECOMPLETED,
    SIEGE_POINTS_TEAMWONROUND, SIEGE_ROUND_BEGIN_TIME,
};
pub use crate::vehicles::e_weapon_pose::{EWeaponPose, EWeaponPose::*};
pub use crate::vehicles::turret_stats_t::{turretStats_t, MAX_VEHICLE_TURRET_MUZZLES};
pub use crate::vehicles::veh_flags_t::vehFlags_t::*;
pub use crate::vehicles::veh_weapon_info_t::vehWeaponInfo_t;
pub use crate::vehicles::vehicle_info_t::vehicleInfo_t;
pub use crate::vehicles::vehicle_s::{
    Vehicle_t, MAX_VEHICLE_TURRETS, MAX_VEHICLE_WEAPONS, VEHICLE_BASE, VEHICLE_NONE, *,
};
pub use crate::vehicles::vehicle_type_t::{vehicleType_t, vehicleType_t::*};
pub use crate::vehicles::{vehFieldType_t, vehFieldType_t::*, vehField_t};
pub use crate::weapons::ammo_data::ammoData;
pub use crate::weapons::ammo_t::{ammo_t, ammo_t::*};
pub use crate::weapons::weapon_t::{weapon_t, *};
pub use crate::weapons::{weaponData, WP_MuzzlePoint};

// --- mp_qshared re-exports ---
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
pub use mp_qshared::common::mp::game::class_t::{class_t, class_t::*};
pub use mp_qshared::common::mp::gentity::{
    material_t, moverState_t, MAT_CRATE1, MAT_CRATE2, MAT_DRK_STONE, MAT_ELECTRICAL,
    MAT_ELEC_METAL, MAT_GLASS, MAT_GLASS_METAL, MAT_GRATE1, MAT_GREY_STONE, MAT_LT_STONE,
    MAT_METAL, MAT_METAL2, MAT_METAL3, MAT_NONE, MAT_ROPE, MAT_SNOWY_ROCK, MAT_WHITE_METAL,
    MOVER_1TO2, MOVER_2TO1, MOVER_POS1, MOVER_POS2, NUM_BSETS, NUM_MATERIALS, *,
};
pub use mp_qshared::common::mp::ghoul2::bone_flags::*;
pub use mp_qshared::common::mp::qcommon::b_set_t::{bSet_t, bSet_t::*};
pub use mp_qshared::common::mp::qcommon::b_state_t::{bState_t, bState_t::*};
pub use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
pub use mp_qshared::common::mp::qcommon::failed_edge::failedEdge_t;
pub use mp_qshared::common::mp::qcommon::nav_debug_draw::*;
pub use mp_qshared::common::mp::qcommon::parms::{parms_t, MAX_PARMS};
pub use mp_qshared::common::mp::qcommon::player_state::{
    forcedata_t, playerState_t, MAX_POWERUPS, MAX_PS_EVENTS, NUM_FORCE_POWERS, *,
};
pub use mp_qshared::common::mp::qcommon::pm_flags::*;
pub use mp_qshared::common::mp::qcommon::qtime::qtime_t;
pub use mp_qshared::common::mp::qcommon::saber::blade_info::{bladeInfo_t, MAX_BLADES};
pub use mp_qshared::common::mp::qcommon::saber::saber_colors::{saber_colors_t, *};
pub use mp_qshared::common::mp::qcommon::saber::saber_flags::*;
pub use mp_qshared::common::mp::qcommon::saber::saber_info::{saberInfo_t, MAX_SABERS};
pub use mp_qshared::common::mp::qcommon::saber::saber_styles::saber_styles_t;
pub use mp_qshared::common::mp::qcommon::shared_ragdoll_params::sharedRagDollParams_t;
pub use mp_qshared::common::mp::qcommon::shared_ragdoll_update_params::sharedRagDollUpdateParams_t;
pub use mp_qshared::common::mp::qcommon::shared_set_bone_ik_state_params::sharedSetBoneIKStateParams_t;
pub use mp_qshared::common::mp::qcommon::siege_pers::siegePers_t;
pub use mp_qshared::common::mp::qcommon::task_id_t::{taskID_t, taskID_t::*};
pub use mp_qshared::common::mp::qcommon::usercmd::usercmd_t;
pub use mp_qshared::common::mp::qcommon::usercmd_button::*;
pub use mp_qshared::common::mp::trace_t::trace_t;
pub use mp_qshared::shared::cbuf_exec::{cbufExec_t, cbufExec_t::*};
pub use mp_qshared::shared::collision::{cplane_t, CollisionRecord_t};
pub use mp_qshared::shared::com_parse::{
    COM_BeginParseSession, COM_GetCurrentParseLine, COM_Parse, COM_ParseExt, QSharedScratch,
    SkipBracedSection, SkipRestOfLine,
};
pub use mp_qshared::shared::cvar::vmCvar_t;
pub use mp_qshared::shared::error_parm::{errorParm_t, errorParm_t::*};
pub use mp_qshared::shared::file_mode::{FS_APPEND, FS_APPEND_SYNC, FS_READ, FS_WRITE};
pub use mp_qshared::shared::flag_status::{flagStatus_t, *};
pub use mp_qshared::shared::force_powers::{forcePowers_t, *};
pub use mp_qshared::shared::fsMode_t;
pub use mp_qshared::shared::limits::*;
pub use mp_qshared::shared::pc_token_t;
pub use mp_qshared::shared::q_color::{
    Q_COLOR_ESCAPE, S_COLOR_BLUE, S_COLOR_GREEN, S_COLOR_RED, S_COLOR_WHITE, S_COLOR_YELLOW,
};
pub use mp_qshared::shared::q_math::{
    _DotProduct, vec3_origin, vectoangles, AngleSubtract, AngleVectors, CrossProduct, Distance,
    Q_fabs, RadiusFromBounds, VectorCompare, VectorLength, VectorLengthSquared, VectorNormalize,
    VectorSet, PITCH, ROLL, YAW,
};
pub use mp_qshared::shared::q_string::{
    va, COM_Compress, COM_StripExtension, GetIDForString, Info_ValueForKey, Q_strcat, Q_strcmp,
    Q_stricmp, Q_stricmpn, Q_strncmp, Q_strncpyz,
};
pub use mp_qshared::shared::qint64;
pub use mp_qshared::shared::saber_blocked_type::{saberBlockedType_t, saberBlockedType_t::*};
pub use mp_qshared::shared::sharedIKMoveParams_t;
pub use mp_qshared::shared::sound_channel::*;
pub use mp_qshared::shared::string_id_table::stringID_table_t;
pub use mp_qshared::shared::surface_flags::*;
pub use mp_qshared::shared::trackchan::{trackchan_t, trackchan_t::*};
pub use mp_qshared::shared::trajectory::{trType_t, trType_t::*, trajectory_t};
pub use mp_qshared::shared::wl_e::{WL_e, WL_e::*};
pub use mp_qshared::shared::wpobject::{wpobject_t, MAX_NEIGHBOR_SIZE};
pub use mp_qshared::shared::Eorientations::*;
pub use mp_qshared::shared::RAND_MAX;
pub use mp_qshared::shared::{
    fileHandle_t, mdxaBone_t, qboolean, qfalse, qhandle_t, qtrue, vec3_t, vec4_t, vec_t,
    Eorientations, MAX_QPATH,
};

// The narrow bg-visible entity view (`gentity_t`/`centity_t` head). Game files
// alias `bgEntity_t = gentity_t`; here it is the real bg overlay.
pub use crate::public::bg_entity::bgEntity_t;

// --- moved bg function-module re-exports (the bare-symbol landing surface) ---
pub use crate::bg_channel::{BgState, BgTraps, GameCallbacks, PmoveContext};
pub use crate::bg_misc::*;
pub use crate::bg_panimate::{BG_InKnockDownOnly, BG_InReboundHold, BG_InReboundJump};
pub use crate::bg_pmove::{MIN_WALK_NORMAL, *};
pub use crate::bg_saber::*;
pub use crate::bg_saberLoad::*;
pub use crate::bg_saga::{WPTable, SIEGECHAR_TAB};
pub use crate::bg_slidemove::*;
pub use crate::bg_vehicleLoad::BG_VehicleGetIndex;
pub use crate::bg_vehicleLoad_tables::*;
pub use crate::cstr_util::{atoi, cstr, cstr_to_str, write_cstr_field};
pub use native_string::sscanf::sscanf_f32s;

// Pin the first-winner of a const transcribed identically in two modules (same
// value), matching the mp_game prelude, so the glob re-exports above are
// unambiguous.
pub use mp_qshared::common::mp::qcommon::task_id_t::taskID_t::NUM_TIDS;
