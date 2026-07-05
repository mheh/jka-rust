//! `GameCallbacks` — the bg tier's upcalls into game logic.
//!
//! The boundary dossier's "16 both" class: functions reachable from bg code
//! (Pmove every frame via `PM_SlideMove`→`PM_VehicleImpact`→`G_Damage`, etc.)
//! whose bodies live in the game tier and cannot be stubbed. Raven reached them
//! by hard-linking; here bg holds a `&mut dyn GameCallbacks` on `PmoveContext`
//! and the game tier implements it, delegating to the ported `G_*` bodies.
//!
//! Signatures are bg-visible only: entity references are entity numbers
//! (`c_int`, the index bg already uses for `bgEntity_t`), never `gentity_t*`,
//! `EntityId`, or `GameContext`. The game impl resolves nums to its arena.
#![allow(non_snake_case, clippy::too_many_arguments)]

use core::ffi::{c_char, c_int};

use crate::prelude::*;

/// The bg→game upcall surface. Each method cites the game-tier Raven function
/// it delegates to; the `entNum`-style params replace that function's
/// `gentity_t*` arguments.
pub trait GameCallbacks {
    /// Raven `G_Damage(targ, inflictor, attacker, dir, point, damage, dflags, mod)`.
    /// Source: `oracle/oracle/codemp/game/g_local.h:1158`
    fn damage(
        &mut self,
        targNum: c_int,
        inflictorNum: c_int,
        attackerNum: c_int,
        dir: *const vec3_t,
        point: *const vec3_t,
        damage: c_int,
        dflags: c_int,
        mod_: c_int,
    );

    /// `G_Damage` variant used where the killer credit differs from the
    /// inflictor (e.g. vehicle impacts crediting the pilot).
    /// Source: `oracle/oracle/codemp/game/g_combat.c` (`G_Damage` call sites)
    fn damage_from_killer(
        &mut self,
        targNum: c_int,
        inflictorNum: c_int,
        attackerNum: c_int,
        killerNum: c_int,
        dir: *const vec3_t,
        point: *const vec3_t,
        damage: c_int,
        dflags: c_int,
        mod_: c_int,
    );

    /// Raven `G_AddEvent(ent, event, eventParm)`.
    /// Source: `oracle/oracle/codemp/game/g_local.h:1063`
    fn add_event(&mut self, entNum: c_int, event: c_int, eventParm: c_int);

    /// Reads `g_entities[entNum].s.legsAnim` — the `#ifdef QAGAME` branch of
    /// Raven `BG_StartLegsAnim` that only compiles on the game side.
    /// Source: `oracle/oracle/codemp/game/bg_panimate.c:2610-2616`
    fn entity_legs_anim(&self, entNum: c_int) -> c_int;

    /// Reads `g_entities[entNum].s.torsoAnim` — the `#ifdef QAGAME` branch of
    /// Raven `BG_StartTorsoAnim` that only compiles on the game side.
    /// Source: `oracle/oracle/codemp/game/bg_panimate.c:2680-2683`
    fn entity_torso_anim(&self, entNum: c_int) -> c_int;

    /// Raven `G_Alloc(size)` — the game-tier bump allocator (distinct from the
    /// bg `BG_Alloc` pool). Returns the address of the reserved block.
    /// Source: `oracle/oracle/codemp/game/g_local.h:1368`
    fn alloc(&mut self, size: c_int) -> *mut core::ffi::c_void;

    /// Raven `G_NewString(string)` — copies a string into game memory.
    /// Source: `oracle/oracle/codemp/game/g_local.h:942`
    fn new_string(&mut self, string: *const c_char) -> *mut c_char;

    /// Raven `G_PlayEffect(fxID, org, ang)` — play a cached effect by index.
    /// Source: `oracle/oracle/codemp/game/g_local.h` (`G_PlayEffect`)
    fn play_effect(&mut self, fxID: c_int, org: *const vec3_t, ang: *const vec3_t);

    /// Raven `G_PlayEffectID(fxID, org, ang)` — spawns a temp-entity effect;
    /// returns the entity number of the spawned effect (`ENTITYNUM_NONE` if none).
    /// Source: `oracle/oracle/codemp/game/g_local.h:1034`
    fn play_effect_id(&mut self, fxID: c_int, org: *const vec3_t, ang: *const vec3_t) -> c_int;

    /// Raven `G_SoundIndex(name)` — register/lookup a sound, return its index.
    /// Source: `oracle/oracle/codemp/game/g_local.h:1002`
    fn sound_index(&mut self, name: *const c_char) -> c_int;

    /// Raven `G_ModelIndex(name)`.
    /// Source: `oracle/oracle/codemp/game/g_local.h:1001`
    fn model_index(&mut self, name: *const c_char) -> c_int;

    /// Raven `G_EffectIndex(name)`.
    /// Source: `oracle/oracle/codemp/game/g_local.h:1004`
    fn effect_index(&mut self, name: *const c_char) -> c_int;

    /// Raven cheap weapon-fire path fired from movement code (bg melee/impacts).
    /// Source: `oracle/oracle/codemp/game/g_weapon.c` (cheap-fire helper)
    fn cheap_weapon_fire(&mut self, entNum: c_int, weapon: c_int);

    /// Raven client-vs-breakable-brush impact check invoked from the slide path.
    /// Source: `oracle/oracle/codemp/game/g_active.c` (impact-bbrush helper)
    fn client_check_impact_bbrush(&mut self, entNum: c_int, impactNum: c_int);

    /// Raven flyer-vehicle surface-destruction upcall from vehicle impacts.
    /// Source: `oracle/oracle/codemp/game/g_vehicles.c` (surface-destruction)
    fn flyveh_surface_destruction(&mut self, entNum: c_int, trNum: c_int, magnitude: f32);

    /// Raven `G_SetAnim(ent, ucmd, setAnimParts, anim, setAnimFlags, blendTime)`.
    /// Source: `oracle/oracle/codemp/game/g_local.h:1022`
    fn set_anim(
        &mut self,
        entNum: c_int,
        ucmd: *mut usercmd_t,
        setAnimParts: c_int,
        anim: c_int,
        setAnimFlags: c_int,
        blendTime: c_int,
    );

    /// Raven `NPC_SetAnim(ent, type, anim, priority)`.
    /// Source: `oracle/oracle/codemp/game/g_local.h:217`
    fn npc_set_anim(&mut self, entNum: c_int, type_: c_int, anim: c_int, priority: c_int);

    /// Raven `WP_GetVehicleCamPos(ent, pilot, camPos)` — the third-person
    /// vehicle camera origin, queried from the bg crosshair-trace path
    /// (`BG_VehTraceFromCamPos`). The game body needs a `GameContext`
    /// (`G_EstimateCamPos`), so bg reaches it here by entity number.
    /// Source: `oracle/oracle/codemp/game/g_weapon.c:3961-4020`
    fn wp_get_vehicle_cam_pos(&mut self, vehEntNum: c_int, pilotEntNum: c_int, camPos: *mut vec3_t);

    /// Raven `BG`/game enemy-eligibility test consulted during movement.
    /// Source: `oracle/oracle/codemp/game/g_combat.c` (`CanBeEnemy`)
    fn can_be_enemy(&mut self, entNum: c_int, otherNum: c_int) -> qboolean;

    /// Raven `level.time` accessor — the current game time in msec.
    /// Source: `oracle/oracle/codemp/game/g_local.h` (`level.time`)
    fn get_time(&self) -> c_int;

    /// Raven grapple-attempt upcall from the movement code.
    /// Source: `oracle/oracle/codemp/game/g_active.c` (grapple helper)
    fn try_grapple(&mut self, entNum: c_int) -> qboolean;

    /// Raven `Q3_SetParm`-style ICARUS parameter set reachable from game logic.
    /// Source: `oracle/oracle/codemp/game/g_ICARUScb.c` (`Q3_SetParm`)
    fn q3_set_parm(&mut self, entID: c_int, parmNum: c_int, parmValue: *const c_char);

    /// Vehicle boarding upcall. `bg_pmove`'s ground-check boards a
    /// vehicle NPC by calling `pVeh->m_pVehicleInfo->Board(pVeh, pEnt)` — a
    /// game-tier body. That dispatches through
    /// [`crate::veh_dispatch::board`], which is game-tier (takes `GameContext`),
    /// so bg reaches it here by entity number rather than by calling the
    /// game-tier dispatch directly.
    /// Source: `oracle/oracle/codemp/game/bg_pmove.c` (`PM_GroundTrace` boarding);
    /// dispatch target `oracle/oracle/codemp/game/g_vehicles.c:630` (`Board`).
    fn board_vehicle(&mut self, vehEntNum: c_int, entNum: c_int) -> qboolean;
}
