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

// Raven `qtrue`/`qfalse` bare spellings (`q_shared.h` `qboolean` enum values,
// pass-3 symbol backfill). The canonical port lives as `QTRUE`/`QFALSE` in
// `native_types`; skeleton bodies transcribe Raven's exact lowercase macro
// names, so both spellings are provided.
pub const qtrue: qboolean = QTRUE;
pub const qfalse: qboolean = QFALSE;

// Integration round-1 addendum: the fnskel packets transcribe Raven constant
// spellings verbatim (per each file's own "integration-deferred" note) without
// enumerating their owning module's `use`; these glob-imports resolve them
// against the same crates already named above. Explicit single-item imports
// below (e.g. `holdable_t`) are unaffected — Rust lets an explicit import
// shadow a glob without ambiguity.
pub use mp_bg::public::gametype::{
    gametype_t, GT_CTF, GT_CTY, GT_DUEL, GT_FFA, GT_HOLOCRON, GT_JEDIMASTER, GT_MAX_GAME_TYPE,
    GT_POWERDUEL, GT_SIEGE, GT_SINGLE_PLAYER, GT_TEAM,
};
pub use mp_bg::public::holdable::*;
pub use mp_bg::public::powerup::*;
pub use mp_bg::public::saber_move_name::*;
pub use mp_bg::public::team::*;
pub use mp_bg::public::dm_flags::*;
pub use mp_bg::public::entity_effects::*;
pub use mp_bg::public::item_type::*;
pub use mp_bg::public::set_anim::*;
pub use mp_bg::vehicles::vehicle_s::{MAX_VEHICLE_TURRETS, VEHICLE_BASE, VEHICLE_NONE};
pub use mp_bg::vehicles::vehicle_type_t::vehicleType_t::*;
pub use mp_bg::weapons::weapon_t::*;
pub use mp_bg::weapons::weaponData;
pub use mp_qshared::shared::force_powers::*;
pub use mp_qshared::shared::limits::*;
pub use mp_qshared::shared::sound_channel::*;
pub use mp_qshared::shared::surface_flags::*;
pub use mp_qshared::common::mp::qcommon::pm_flags::*;

// Pass-3 symbol backfill: game-crate-local const families that were ported
// but never wired into the prelude glob (see `docs/porting-rules.md` §E13).
pub use crate::bg_misc::{
    bgForcePowerCost, bgForcePowerCostSaberThrow, forceMasteryPoints,
    forcePowerDarkLight, forcePowerSorted,
};
// Canonical seam string helpers (porting-rules pass-3 packet primer contract).
pub use crate::cstr_util::{cstr, cstr_to_str, cstr_to_string, write_cstr_field};
pub use crate::entity::flags::*;
pub use crate::g_client::{playerMaxs, playerMins};
pub use crate::g_items::FRAMETIME;
pub use crate::g_mover::{BMS_END, BMS_MID, BMS_START};
pub use crate::g_nav_consts::*;
pub use crate::g_public_consts::*;
pub use crate::g_target::Q3_SCRIPT_DIR;
pub use crate::level::damage_flags::*;
pub use crate::npc::ai_flags::*;
pub use crate::npc::script_flags::*;
pub use crate::npc::squad_state::*;
pub use crate::q_math::{
    vec3_origin, vectoangles, RadiusFromBounds, PITCH, ROLL, VEC3_ORIGIN, YAW,
};
pub use crate::w_force::mindTrickTime;
pub use mp_qshared::shared::q_math_rand::RAND_MAX;
// `BG_GiveMeVectorFromMatrix` has independent per-NPC-file transcriptions
// (Raven copy-paste convention, porting-rules §F20); `NPC_AI_Mark2`'s copy is
// the only one already `pub` and is the canonical export for bare-use sites.
pub use crate::NPC_AI_Mark2::BG_GiveMeVectorFromMatrix;
pub use crate::saber::w_saber_consts::*;
pub use crate::teams::npcteam::*;

// Enum types transcribed as `#[repr(i32)] enum` per porting-rules'
// enum-vs-alias fidelity rule; the fnskel packets carry their bare Raven
// variant spellings (e.g. `STAT_MAX_HEALTH`, not `statIndex_t::STAT_MAX_HEALTH`),
// so both the type name (for sites that do qualify) and a variant glob (for
// the far more common bare spelling) are re-exported here.
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
pub use mp_bg::public::stat_index::{statIndex_t, statIndex_t::*};
pub use mp_bg::public::saber_quadrant::{
    saberQuadrant_t, Q_BR, Q_R, Q_TR, Q_T, Q_TL, Q_L, Q_BL, Q_B, Q_NUM_QUADS,
};
pub use mp_qshared::common::mp::qcommon::b_set_t::{bSet_t, bSet_t::*};
pub use mp_qshared::common::mp::qcommon::b_state_t::bState_t::*;
pub use mp_qshared::common::mp::qcommon::task_id_t::{taskID_t, taskID_t::*};
pub use mp_qshared::common::mp::qcommon::usercmd_button::*;
pub use mp_qshared::shared::trackchan::{trackchan_t, trackchan_t::*};
pub use mp_qshared::shared::trajectory::trType_t::*;
pub use mp_qshared::shared::wl_e::{WL_e, WL_e::*};

// Pass-2 ctx threading (fork 8): the module-island dispatch receiver, injected
// as the first param of every game-tier needs-ctx fn.
// Source: `docs/handoffs/jampgame-fork-discovery.md` fork 8; `world/game_context.rs`
pub use crate::world::GameContext;

// Pass-3 prep C1 riders (ruling 21 / rulings 12-16, 22): the entity handle, the
// spawn fn-ID enum, and the bg-channel state/trait set — hoisted into the
// prelude so the pass-3 porter bodies name them unqualified.
// - `EntityId` (ruling 22): the `Option<EntityId>` stored-field handle.
// - `EntSpawn` (agenda C13): the `spawns[]` classname->fn dispatch enum.
// - `BgState`/`PmoveContext`/`BgTraps`/`GameCallbacks` (rulings 12-16): the bg
//   session/per-call state + the two seam traits.
// - `pml_t` (ruling 21): bg pmove local working-set type.
pub use crate::bg_channel::{BgState, BgTraps, GameCallbacks, PmoveContext};
pub use crate::ent_fn_enums::EntSpawn;
// `EntityId` + the `ent - g_entities` seam helpers (ruling 22): `Some(ent_id(base,
// p))` / `ent_id_opt(base, p)` fill the `Option<EntityId>` stored fields at
// pointer-assignment sites; `field.is_none()` / id-equality replace NULL/address
// compares.
pub use crate::world::{ent_id, ent_id_opt, EntityId};
pub use mp_bg::local::pml_t::pml_t;

// Pass-3 prep C1 (agenda B10 porter-instruction rider): the `crate::trap` seam
// module, spelled bare `trap::X` throughout the pass-2 bodies. Re-exported so the
// `use crate::prelude::*` glob resolves those call sites.
pub use crate::trap;
// Raven `trap_R_RegisterSkin` re-export with Raven name for pass-3 bodies
// that transcribe the bare Raven spelling (oracle/oracle/codemp/game/g_syscalls.c:1179-1182)
pub use crate::trap::R_RegisterSkin as trap_R_RegisterSkin;
// Raven `trap_G2API_*` re-exports with Raven names for pass-3 bodies that
// transcribe the bare Raven spellings (oracle/oracle/codemp/game/g_syscalls.c)
pub use crate::trap::G2API_InitGhoul2Model as trap_G2API_InitGhoul2Model;
pub use crate::trap::G2API_CleanGhoul2Models as trap_G2API_CleanGhoul2Models;

// The entity fn-ID dispatch enums (ruling 2 / `ent_fn_enums`), named bare in the
// spawn/think/touch/… assignment sites.
pub use crate::ent_fn_enums::{
    EntBlocked, EntDie, EntPain, EntReached, EntThink, EntTouch, EntUse,
};

pub use crate::ai::group_info::AIGroupInfo_t;
pub use crate::botai::bot_state_s::bot_state_t;
pub use crate::client::gclient::gclient_t;
pub use crate::level::alert_event::{alertEventLevel_e, alertEventLevel_e::*};
pub use crate::level::reference_tag::reference_tag_t;
pub use crate::npc::g_npc_t::gNPC_t;
pub use crate::npc::nav_info_s::navInfo_t;
pub use crate::npc::spot_t::{spot_t, spot_t::*};
pub use crate::npc::visibility_t::visibility_t;
pub use crate::saber::evasion_type_t::evasionType_t;
pub use crate::teams::class::class_t;
pub use crate::teams::class::class_t::*;

pub use mp_bg::public::animation::animation_t;
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
    gentity_t, material_t, moverState_t, MAT_CRATE1, MAT_CRATE2, MAT_DRK_STONE, MAT_ELECTRICAL,
    MAT_ELEC_METAL, MAT_GLASS, MAT_GLASS_METAL, MAT_GRATE1, MAT_GREY_STONE, MAT_LT_STONE,
    MAT_METAL, MAT_METAL2, MAT_METAL3, MAT_NONE, MAT_ROPE, MAT_SNOWY_ROCK, MAT_WHITE_METAL,
    MOVER_1TO2, MOVER_2TO1, MOVER_POS1, MOVER_POS2, NUM_MATERIALS,
};

// Raven `#define bgEntity_t gentity_t` in jampgame source files (g_vehicles.c, FighterNPC.c, etc).
// In the oracle, this macro makes bgEntity_t and gentity_t interchangeable at call sites that
// need to access server-side fields like spawnflags. For game-code bodies that have gentity_t
// parameters cast through bgEntity_t, we re-export gentity_t under the bgEntity_t name to allow
// those accesses (e.g. `(*bgEntity).spawnflags`).
// Source: oracle/oracle/codemp/game/g_vehicles.c, FighterNPC.c, etc. (local macro)
pub use mp_qshared::common::mp::gentity::gentity_t as bgEntity_t;
pub use mp_qshared::common::mp::qcommon::b_state_t::bState_t;
pub use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
pub use mp_qshared::common::mp::qcommon::failed_edge::failedEdge_t;
pub use mp_qshared::common::mp::qcommon::game_item::gitem_t;
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
pub use mp_qshared::shared::trajectory::{trajectory_t, trType_t};
pub use mp_qshared::shared::wpobject::wpobject_t;
pub use mp_qshared::shared::{
    fileHandle_t, mdxaBone_t, qboolean, qhandle_t, vec3_t, vec4_t, vec_t, Eorientations, MAX_QPATH,
    QFALSE, QTRUE,
};
pub use mp_qshared::shared::Eorientations::*;

// Pass-3 prep C1 (agenda B6/B10): batch re-export of game-crate-local fns
// spelled bare in pass-2 porter bodies but never wired into the prelude.
// Each resolves to a single `pub fn`/`const` definition (scripted).
pub use crate::FighterNPC::FighterIsLanded;
pub use crate::NPC_AI_Atst::NPC_ATST_Precache;
pub use crate::NPC_AI_Droid::{NPC_Gonk_Precache, NPC_Mouse_Precache, NPC_Protocol_Precache, NPC_R2D2_Precache, NPC_R5D2_Precache};
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
pub use crate::NPC_combat::{CanShoot, EntIsGlass, G_ClearEnemy, G_SetEnemy, NPC_AimAdjust, NPC_ChangeWeapon, NPC_FindCombatPoint, NPC_FreeCombatPoint, NPC_SetCombatPoint, NPC_ShotEntity, ShotThroughGlass, WeaponThink};
pub use crate::NPC_goal::{NPC_ReachedGoal, UpdateGoal};
pub use crate::NPC_move::{NAV_GetLastMove, NPC_MoveToGoal};
pub use crate::NPC_senses::{InFOV3, NPC_CheckAlertEvents, NPC_CheckForDanger};
pub use crate::NPC_utils::{CalcEntitySpot, NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_FaceEnemy, NPC_UpdateAngles};
pub use crate::bg_lib::atof;
pub use crate::bg_lib::atoi;
pub use crate::bg_misc::{BG_EmplacedView, BG_FindItemForWeapon};
pub use crate::bg_panimate::BG_InKnockDownOnly;
pub use crate::bg_pmove::BG_SabersOff;
pub use crate::g_client::SpotWouldTelefrag2;
pub use crate::g_combat::{G_CheckVehicleNPCTeamDamage, G_Damage, G_RadiusDamage};
pub use crate::g_items::{RegisterItem, Touch_Item};
pub use crate::g_log::G_LogWeaponFire;
pub use crate::g_main::{Com_Printf, G_RunThink};
pub use crate::g_nav::{FlyingCreature, NAV_HitNavGoal, NPC_SetMoveGoal};
pub use crate::g_spawn::{G_NewString, G_SpawnFloat, G_SpawnInt, G_SpawnString, G_SpawnVector};
pub use crate::g_timer::TIMER_Done;
pub use crate::g_utils::{G_AddEvent, G_Find, G_FreeEntity, G_ModelIndex, G_PlayEffect, G_ScaleNetHealth, G_SetAnim, G_SetMovedir, G_SetOrigin, G_Sound, G_SoundIndex, G_SoundSetIndex, G_Spawn, G_TeamCommand, G_TempEntity, G_UseTargets2, TryHeal};
pub use crate::g_weapon::{LogAccuracyHit, laserTrapStick};
pub use crate::q_math::{AddPointToBounds, AngleSubtract, AngleVectors, CrossProduct, DirToByte, Distance, DistanceHorizontalSquared, G_FindClosestPointOnLineSegment, Q_fabs, VectorCompare, VectorLength, VectorLengthSquared, VectorNormalize};
pub use crate::q_shared::{COM_StripExtension, GetIDForString, Q_stricmp, Q_strncmp, Q_strncpyz, Q_strupr, va};
pub use crate::w_saber::WP_SaberCanBlock;

// preflight.py: file-level symbol re-exports (aggregate re-export plan)
pub use crate::NPC_AI_Stormtrooper::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_AI_Stormtrooper.rs
pub use crate::g_items::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/g_items.rs
pub use crate::ai_wpnav::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/ai_wpnav.rs
pub use crate::teams::class::{ class_t::*};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/teams/class.rs
pub use mp_qshared::common::mp::gentity::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/qshared/src/common/mp/gentity.rs
pub use crate::bg_saber::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/bg_saber.rs
pub use crate::NPC_AI_Mark1::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_AI_Mark1.rs
pub use crate::g_vehicles::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/g_vehicles.rs
pub use crate::g_mover::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/g_mover.rs
pub use crate::bg_pmove::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/bg_pmove.rs
pub use crate::NPC_AI_Grenadier::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_AI_Grenadier.rs
pub use crate::g_trigger::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/g_trigger.rs
pub use mp_bg::public::gametype::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/bg/src/public/gametype.rs
pub use crate::NPC_AI_Sentry::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_AI_Sentry.rs
pub use crate::g_active::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/g_active.rs
pub use crate::g_client::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/g_client.rs
pub use crate::NPC_AI_Mark2::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_AI_Mark2.rs
pub use mp_bg::public::saber_quadrant::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/bg/src/public/saber_quadrant.rs
pub use crate::NPC_senses::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_senses.rs
pub use mp_bg::weapons::ammo_t::{ ammo_t::*};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/bg/src/weapons/ammo_t.rs
pub use crate::NPC_AI_ImperialProbe::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_AI_ImperialProbe.rs
pub use crate::NPC_AI_Atst::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_AI_Atst.rs
pub use crate::bg_slidemove::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/bg_slidemove.rs
pub use mp_qshared::common::mp::qcommon::saber::saber_colors::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/qshared/src/common/mp/qcommon/saber/saber_colors.rs
pub use crate::NPC_AI_Seeker::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_AI_Seeker.rs
pub use crate::NPC_combat::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_combat.rs
pub use mp_qshared::shared::saber_blocked_type::{saberBlockedType_t, saberBlockedType_t::*};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/qshared/src/shared/saber_blocked_type.rs
pub use crate::bg_misc::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/bg_misc.rs
pub use crate::NPC_AI_Droid::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_AI_Droid.rs
pub use mp_bg::public::saberlock::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/bg/src/public/saberlock.rs
pub use mp_bg::public::saber_move_data_table::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/bg/src/public/saber_move_data_table.rs
pub use crate::botai::bweaponrange::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/botai/bweaponrange.rs
pub use mp_qshared::shared::file_mode::{FS_APPEND, FS_APPEND_SYNC, FS_READ, FS_WRITE};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/qshared/src/shared/fsMode_t.rs
pub use mp_bg::public::g2_model_parts::{ g2ModelParts_t::*};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/bg/src/public/g2_model_parts.rs
pub use crate::NPC_AI_GalakMech::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_AI_GalakMech.rs
pub use crate::g_timer::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/g_timer.rs
pub use crate::client::client_persistant::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/client/client_persistant.rs
pub use crate::q_math::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/q_math.rs
pub use crate::g_turret_G2::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/g_turret_G2.rs
pub use crate::g_team::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/g_team.rs
pub use crate::NPC_utils::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_utils.rs
pub use crate::client::client_connected::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/client/client_connected.rs
pub use mp_qshared::shared::flag_status::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/qshared/src/shared/flag_status.rs
pub use crate::ai::consts::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/ai/consts.rs
pub use mp_qshared::common::mp::qcommon::player_state::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/qshared/src/common/mp/qcommon/player_state.rs
pub use mp_bg::vehicles::vehicle_s::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/bg/src/vehicles/vehicle_s.rs
pub use mp_qshared::shared::trajectory::{ trType_t::*};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/qshared/src/shared/trajectory.rs
pub use crate::level::alert_event::{MAX_ALERT_EVENTS};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/level/alert_event.rs
pub use crate::NPC_behavior::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_behavior.rs
pub use crate::tri_coll_test::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/tri_coll_test.rs
pub use crate::NPC_AI_Interrogator::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_AI_Interrogator.rs
pub use crate::NPC_AI_Howler::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_AI_Howler.rs
pub use crate::g_spawn::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/g_spawn.rs
pub use crate::g_cmds::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/g_cmds.rs
pub use crate::level::reference_tag::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/level/reference_tag.rs
pub use mp_bg::public::spawn::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/bg/src/public/spawn.rs
pub use crate::npc_c::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/npc_c.rs
pub use crate::NPC_spawn::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_spawn.rs
pub use crate::g_turret::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/g_turret.rs
pub use mp_bg::vehicles::vehicle_type_t::{vehicleType_t, vehicleType_t::*};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/bg/src/vehicles/vehicle_type_t.rs
pub use mp_bg::local::force_levels::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/bg/src/local/force_levels.rs
pub use crate::level::level_locals::{BODY_QUEUE_SIZE};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/level/level_locals.rs
pub use mp_bg::public::dm_flags::{DF_NO_FALLING};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/bg/src/public/dm_flags.rs
pub use mp_qshared::shared::cbuf_exec::{cbufExec_t, cbufExec_t::*};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/qshared/src/shared/cbuf_exec.rs
pub use mp_qshared::common::mp::qcommon::saber::blade_info::{MAX_BLADES};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/qshared/src/common/mp/qcommon/saber/blade_info.rs
pub use crate::level::combat_point::{MAX_COMBAT_POINTS};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/level/combat_point.rs
pub use crate::NPC_AI_MineMonster::{MAX_DISTANCE};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_AI_MineMonster.rs
pub use crate::level::interest_point::{MAX_INTEREST_POINTS};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/level/interest_point.rs
pub use crate::g_svcmds::{MAX_IPFILTERS};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/g_svcmds.rs
pub use crate::game_globals::{MAX_ITEMS};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/game_globals.rs
pub use mp_qshared::shared::wpobject::{MAX_NEIGHBOR_SIZE};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/qshared/src/shared/wpobject.rs
pub use mp_qshared::common::mp::qcommon::parms::{MAX_PARMS};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/qshared/src/common/mp/qcommon/parms.rs
pub use crate::NPC_AI_Utils::{MAX_RADIUS_ENTS};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/NPC_AI_Utils.rs
pub use mp_bg::vehicles::turret_stats_t::{MAX_VEHICLE_TURRET_MUZZLES};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/bg/src/vehicles/turret_stats_t.rs
pub use mp_qshared::shared::q_color::{Q_COLOR_ESCAPE};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/qshared/src/shared/q_color.rs
pub use crate::saber::saber_flags::*;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/saber/saber_flags.rs
pub use crate::bg_saga::{SIEGECHAR_TAB};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/bg_saga.rs
pub use mp_bg::weapons::ammo_data::{ammoData};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/bg/src/weapons/ammo_data.rs
pub use mp_bg::local::bg_toggleable_surfaces::{bgToggleableSurfaces};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/bg/src/local/bg_toggleable_surfaces.rs
pub use mp_bg::local::force_power_needed::{forcePowerNeeded};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/bg/src/local/force_power_needed.rs
pub use crate::g_spawn::MAX_AMBIENT_SETS;  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/g_spawn.rs
pub use crate::g_cmds::{MAX_TOKEN_CHARS};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/g_cmds.rs
pub use crate::bg_pmove::{MIN_WALK_NORMAL};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/game/src/bg_pmove.rs
pub use mp_qshared::common::mp::gentity::{NUM_BSETS};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/qshared/src/common/mp/gentity.rs
pub use mp_qshared::common::mp::qcommon::player_state::{NUM_FORCE_POWERS};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/qshared/src/common/mp/qcommon/player_state.rs
pub use mp_bg::public::saber_move_data_table::{saberMoveData};  // .claude/worktrees/agent-a43cc53200d2fdf54/crates/mp/bg/src/public/saber_move_data_table.rs
