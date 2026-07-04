//! `prelude` — the landing prelude for the jampgame function-skeleton modules.
//!
//! The staged skeletons (`fnskel.py`) carry faithful signatures whose parameter
//! and return types resolve against already-ported crates, but the generator
//! emits them WITHOUT `use` statements. This module re-exports every such type
//! at one legal import path (routed through `mp_game`'s frozen dependency set:
//! `mp_qshared`, `mp_bg`, `mp_abi`, plus the crate's own `world`/`client`/…
//! modules); each landed skeleton opens with `use crate::prelude::*;`.
//!
//! No new behavior lives here — only re-exports. The `//TODO: Port` markers for
//! the still-unported parameter types stay at their call sites in the skeletons.

// Raven scalar / handle / ffi primitives. `native_*` is not a direct dependency
// of `mp_game`; the cross-mode primitives are reached through `mp_qshared`'s
// re-export umbrella, exactly as the live modules already spell them.
pub use core::ffi::{
    c_char, c_double, c_float, c_int, c_long, c_schar, c_short, c_uchar, c_uint, c_ulong,
    c_ushort, c_void,
};

// Raven `byte` (`q_shared.h:349`, `typedef unsigned char byte`). `native_types`
// defines it but is not re-exported for this name through `mp_qshared`; the
// local alias matches the same `c_uchar` width.
// Source: `oracle/oracle/codemp/game/q_shared.h:349`
pub type byte = c_uchar;

// Integration round-1 addendum: the fnskel packets transcribe Raven constant
// spellings verbatim (per each file's own "integration-deferred" note) without
// enumerating their owning module's `use`; these glob-imports resolve them
// against the same crates already named above. Explicit single-item imports
// below (e.g. `holdable_t`) are unaffected — Rust lets an explicit import
// shadow a glob without ambiguity.
pub use mp_bg::public::gametype::{gametype_t, GT_CTF, GT_CTY, GT_HOLOCRON, GT_SIEGE, GT_TEAM};
pub use mp_bg::public::holdable::*;
pub use mp_bg::public::powerup::*;
pub use mp_bg::public::saber_move_name::*;
pub use mp_bg::public::team::*;
pub use mp_bg::public::entity_effects::*;
pub use mp_bg::public::item_type::*;
pub use mp_bg::public::set_anim::*;
pub use mp_bg::vehicles::vehicle_s::{MAX_VEHICLE_TURRETS, VEHICLE_BASE, VEHICLE_NONE};
pub use mp_bg::weapons::weapon_t::*;
pub use mp_qshared::shared::force_powers::*;
pub use mp_qshared::shared::limits::*;
pub use mp_qshared::shared::sound_channel::*;
pub use mp_qshared::shared::surface_flags::*;

// Enum types transcribed as `#[repr(i32)] enum` per porting-rules'
// enum-vs-alias fidelity rule; the fnskel packets carry their bare Raven
// variant spellings (e.g. `STAT_MAX_HEALTH`, not `statIndex_t::STAT_MAX_HEALTH`),
// so both the type name (for sites that do qualify) and a variant glob (for
// the far more common bare spelling) are re-exported here.
pub use mp_bg::public::broken_limb::{brokenLimb_t, brokenLimb_t::*};
pub use mp_bg::public::entity_type::{entityType_t, entityType_t::*};
pub use mp_bg::public::force_hand_anims::{forceHandAnims_t, forceHandAnims_t::*};
pub use mp_bg::public::means_of_death::{meansOfDeath_t, meansOfDeath_t::*};
pub use mp_bg::public::pd_sounds::{pdSounds_t, pdSounds_t::*};
pub use mp_bg::public::pers_enum::{persEnum_t, persEnum_t::*};
pub use mp_bg::public::pmtype::{pmtype_t, pmtype_t::*};
pub use mp_bg::public::stat_index::{statIndex_t, statIndex_t::*};
pub use mp_qshared::common::mp::qcommon::b_set_t::{bSet_t, bSet_t::*};
pub use mp_qshared::common::mp::qcommon::task_id_t::{taskID_t, taskID_t::*};
pub use mp_qshared::shared::trackchan::{trackchan_t, trackchan_t::*};
pub use mp_qshared::shared::wl_e::{WL_e, WL_e::*};

pub use crate::ai::group_info::AIGroupInfo_t;
pub use crate::botai::bot_state_s::bot_state_t;
pub use crate::client::gclient::gclient_t;
pub use crate::level::alert_event::alertEventLevel_e;
pub use crate::level::reference_tag::reference_tag_t;
pub use crate::npc::g_npc_t::gNPC_t;
pub use crate::npc::nav_info_s::navInfo_t;
pub use crate::npc::spot_t::spot_t;
pub use crate::npc::visibility_t::visibility_t;
pub use crate::saber::evasion_type_t::evasionType_t;
pub use crate::teams::class::class_t;
pub use crate::teams::class::class_t::*;

pub use mp_bg::public::animation::animation_t;
pub use mp_bg::public::bg_entity::bgEntity_t;
pub use mp_bg::public::bg_field::BG_field_t;
pub use mp_bg::public::holdable::holdable_t;
pub use mp_bg::public::pmove_t::pmove_t;
pub use mp_bg::public::powerup::powerup_t;
pub use mp_bg::public::saber_move_name::saberMoveName_t;
pub use mp_bg::public::team::team_t;
pub use mp_bg::saga::siege_class_desc_t::siegeClassDesc_t;
pub use mp_bg::saga::siege_class_t::siegeClass_t;
pub use mp_bg::saga::siege_team_t::siegeTeam_t;
pub use mp_bg::vehicles::turret_stats_t::turretStats_t;
pub use mp_bg::vehicles::veh_weapon_info_t::vehWeaponInfo_t;
pub use mp_bg::vehicles::vehicle_info_t::vehicleInfo_t;
pub use mp_bg::vehicles::vehicle_s::Vehicle_t;
pub use mp_bg::weapons::ammo_t::ammo_t;
pub use mp_bg::weapons::ammo_t::ammo_t::*;
pub use mp_bg::weapons::weapon_t::weapon_t;

pub use mp_qshared::common::mp::botlib::aas_entityinfo_s::aas_entityinfo_t;
pub use mp_qshared::common::mp::botlib::bot_input_s::bot_input_t;
pub use mp_qshared::common::mp::gentity::{
    gentity_t, material_t, moverState_t, MOVER_1TO2, MOVER_2TO1, MOVER_POS1, MOVER_POS2,
};
pub use mp_qshared::common::mp::qcommon::b_state_t::bState_t;
pub use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
pub use mp_qshared::common::mp::qcommon::failed_edge::failedEdge_t;
pub use mp_qshared::common::mp::qcommon::game_item::gitem_t;
pub use mp_qshared::common::mp::qcommon::player_state::{forcedata_t, playerState_t};
pub use mp_qshared::common::mp::qcommon::qtime::qtime_t;
pub use mp_qshared::common::mp::qcommon::saber::blade_info::bladeInfo_t;
pub use mp_qshared::common::mp::qcommon::saber::saber_colors::saber_colors_t;
pub use mp_qshared::common::mp::qcommon::saber::saber_info::saberInfo_t;
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
pub use mp_qshared::shared::trajectory::{trajectory_t, trType_t};
pub use mp_qshared::shared::wpobject::wpobject_t;
pub use mp_qshared::shared::{
    fileHandle_t, mdxaBone_t, qboolean, qhandle_t, vec3_t, vec4_t, vec_t, Eorientations, MAX_QPATH,
    QFALSE, QTRUE,
};
pub use mp_qshared::shared::Eorientations::*;
