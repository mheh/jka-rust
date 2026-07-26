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
    /// Source: `oracle/codemp/game/g_local.h:1158`
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
    /// Source: `oracle/codemp/game/g_combat.c` (`G_Damage` call sites)
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
    /// Source: `oracle/codemp/game/g_local.h:1063`
    fn add_event(&mut self, entNum: c_int, event: c_int, eventParm: c_int);

    /// Reads `g_entities[entNum].s.legsAnim` — the `#ifdef QAGAME` branch of
    /// Raven `BG_StartLegsAnim` that only compiles on the game side.
    /// Source: `oracle/codemp/game/bg_panimate.c:2610-2616`
    fn entity_legs_anim(&self, entNum: c_int) -> c_int;

    /// Reads `g_entities[entNum].s.torsoAnim` — the `#ifdef QAGAME` branch of
    /// Raven `BG_StartTorsoAnim` that only compiles on the game side.
    /// Source: `oracle/codemp/game/bg_panimate.c:2680-2683`
    fn entity_torso_anim(&self, entNum: c_int) -> c_int;

    /// Raven `G_Alloc(size)` — the game-tier bump allocator (distinct from the
    /// bg `BG_Alloc` pool). Returns the address of the reserved block.
    /// Source: `oracle/codemp/game/g_local.h:1368`
    fn alloc(&mut self, size: c_int) -> *mut core::ffi::c_void;

    /// Raven `G_NewString(string)` — copies a string into game memory.
    /// Source: `oracle/codemp/game/g_local.h:942`
    fn new_string(&mut self, string: &str) -> *mut c_char;

    /// Raven `G_PlayEffect(fxID, org, ang)` — play a cached effect by index.
    /// Source: `oracle/codemp/game/g_local.h` (`G_PlayEffect`)
    fn play_effect(&mut self, fxID: c_int, org: *const vec3_t, ang: *const vec3_t);

    /// Raven `G_PlayEffectID(fxID, org, ang)` — spawns a temp-entity effect;
    /// returns the entity number of the spawned effect (`ENTITYNUM_NONE` if none).
    /// Source: `oracle/codemp/game/g_local.h:1034`
    fn play_effect_id(&mut self, fxID: c_int, org: *const vec3_t, ang: *const vec3_t) -> c_int;

    /// Raven `G_SoundIndex(name)` — register/lookup a sound, return its index.
    ///
    /// PORT-NOTE (DEC-36 D5): also the per-module arm for the bg precache sites
    /// Raven writes as `#ifdef QAGAME G_SoundIndex(...) #else
    /// trap_S_RegisterSound(...)` — cgame/ui register the sample instead.
    /// Source: `oracle/codemp/game/g_local.h:1002`;
    /// `oracle/codemp/game/bg_vehicleLoad.c:1319-1326`
    fn sound_index(&mut self, name: &str) -> c_int;

    /// Raven `G_ModelIndex(name)`.
    ///
    /// PORT-NOTE (DEC-36 D5): also the per-module arm for the bg precache sites
    /// Raven writes as `#ifdef QAGAME G_ModelIndex(...) #else
    /// trap_R_RegisterModel(...)` — cgame/ui register the model instead.
    /// Source: `oracle/codemp/game/g_local.h:1001`;
    /// `oracle/codemp/game/bg_vehicleLoad.c:1262-1269`
    fn model_index(&mut self, name: &str) -> c_int;

    /// Raven `G_EffectIndex(name)`.
    ///
    /// PORT-NOTE (DEC-36 D5): also the per-module arm for the bg precache sites
    /// Raven writes as `#ifdef QAGAME G_EffectIndex(...) #elif CGAME
    /// trap_FX_RegisterEffect(...)` — the ui has no arm there and registers
    /// nothing.
    /// Source: `oracle/codemp/game/g_local.h:1004`;
    /// `oracle/codemp/game/bg_vehicleLoad.c:1309-1334`
    fn effect_index(&mut self, name: &str) -> c_int;

    /// Raven cheap weapon-fire path fired from movement code (bg melee/impacts).
    /// Source: `oracle/codemp/game/g_weapon.c` (cheap-fire helper)
    fn cheap_weapon_fire(&mut self, entNum: c_int, weapon: c_int);

    /// Raven client-vs-breakable-brush impact check invoked from the slide path.
    /// Source: `oracle/codemp/game/g_active.c` (impact-bbrush helper)
    fn client_check_impact_bbrush(&mut self, entNum: c_int, impactNum: c_int);

    /// Raven `G_FlyVehicleSurfaceDestruction(veh, trace, magnitude, force)`
    /// upcall from vehicle impacts. bg holds the impact `trace_t*` and the
    /// `forceSurfDestruction` flag, so they cross the seam directly (the `trace`
    /// pointer is bg-owned scratch, valid for the call).
    /// Source: `oracle/codemp/game/g_vehicles.c:3190`; `bg_slidemove.c:472`.
    fn flyveh_surface_destruction(
        &mut self,
        entNum: c_int,
        trace: *mut trace_t,
        magnitude: c_int,
        force: qboolean,
    );

    /// Raven `G_SetAnim(ent, ucmd, setAnimParts, anim, setAnimFlags, blendTime)`.
    /// Source: `oracle/codemp/game/g_local.h:1022`
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
    /// Source: `oracle/codemp/game/g_local.h:217`
    fn npc_set_anim(&mut self, entNum: c_int, type_: c_int, anim: c_int, priority: c_int);

    /// Raven `WP_GetVehicleCamPos(ent, pilot, camPos)` — the third-person
    /// vehicle camera origin, queried from the bg crosshair-trace path
    /// (`BG_VehTraceFromCamPos`). The game body needs a `GameContext`
    /// (`G_EstimateCamPos`), so bg reaches it here by entity number.
    /// Source: `oracle/codemp/game/g_weapon.c:3961-4020`
    fn wp_get_vehicle_cam_pos(&mut self, vehEntNum: c_int, pilotEntNum: c_int, camPos: *mut vec3_t);

    /// Raven `BG`/game enemy-eligibility test consulted during movement.
    /// Source: `oracle/codemp/game/g_combat.c` (`CanBeEnemy`)
    fn can_be_enemy(&mut self, entNum: c_int, otherNum: c_int) -> qboolean;

    /// Raven `level.time` accessor — the current game time in msec.
    /// Source: `oracle/codemp/game/g_local.h` (`level.time`)
    fn get_time(&self) -> c_int;

    /// Raven grapple-attempt upcall from the movement code.
    /// Source: `oracle/codemp/game/g_active.c` (grapple helper)
    fn try_grapple(&mut self, entNum: c_int) -> qboolean;

    /// Raven `Q3_SetParm`-style ICARUS parameter set reachable from game logic.
    /// Source: `oracle/codemp/game/g_ICARUScb.c` (`Q3_SetParm`)
    fn q3_set_parm(&mut self, entID: c_int, parmNum: c_int, parmValue: &str);

    /// Vehicle boarding upcall. `bg_pmove`'s ground-check boards a
    /// vehicle NPC by calling `pVeh->m_pVehicleInfo->Board(pVeh, pEnt)` — a
    /// game-tier body. That dispatches through
    /// [`crate::veh_dispatch::board`], which is game-tier (takes `GameContext`),
    /// so bg reaches it here by entity number rather than by calling the
    /// game-tier dispatch directly.
    /// Source: `oracle/codemp/game/bg_pmove.c` (`PM_GroundTrace` boarding);
    /// dispatch target `oracle/codemp/game/g_vehicles.c:630` (`Board`).
    fn board_vehicle(&mut self, vehEntNum: c_int, entNum: c_int) -> qboolean;

    /// Vehicle `Update` upcall from `PmoveSingle`'s vehicle-NPC path
    /// (`m_pVehicleInfo->Update`). `ucmd` is the move command bg passes directly
    /// (`&pm->cmd` when idle, `&pVeh->m_ucmd` when driven — both bg-reachable),
    /// routed game-side through [`crate::veh_dispatch::update`].
    /// Source: `oracle/codemp/game/bg_pmove.c:10919-10944`.
    fn update_vehicle(&mut self, vehEntNum: c_int, ucmd: *const usercmd_t);

    /// Vehicle `Animate` upcall (`m_pVehicleInfo->Animate`, the whole-vehicle
    /// slot — NOT the per-class `AnimateVehicle`), routed through
    /// [`crate::veh_dispatch::animate`]. `pm_`-prefixed to disambiguate from the
    /// vehicle-load `AnimateVehicle` dispatch.
    /// Source: `oracle/codemp/game/bg_pmove.c:10921-10945`.
    fn pm_animate_vehicle(&mut self, vehEntNum: c_int);

    /// Vehicle `UpdateRider` upcall. A non-null `ucmd` is the driver path
    /// (bg passes `&pVeh->m_ucmd`); a null `ucmd` is the passenger path — the
    /// impl guards `inuse && client` and uses the rider's own `client->pers.cmd`
    /// (game-side, not bg-reachable). Routed through
    /// [`crate::veh_dispatch::update_rider`].
    /// Source: `oracle/codemp/game/bg_pmove.c:10947-10961`.
    fn update_rider(&mut self, vehEntNum: c_int, riderEntNum: c_int, ucmd: *mut usercmd_t);

    /// Vehicle `AttachRiders` upcall (`m_pVehicleInfo->AttachRiders`), routed
    /// through [`crate::veh_dispatch::attach_riders`].
    /// Source: `oracle/codemp/game/bg_pmove.c:11146-11149`.
    fn attach_riders(&mut self, vehEntNum: c_int);

    /// Raven `BG_MySaber(clientNum, saberNum)` — the `#ifdef QAGAME` branch that
    /// returns `&g_entities[clientNum].client->saber[saberNum]`, or NULL unless
    /// the client is in use, has a `client`, and that saber has a model. The
    /// game arena is not bg-nameable, so bg reaches it here by entity number.
    /// Source: `oracle/codemp/game/bg_saber.c:4100-4141`
    fn my_saber(&mut self, client_num: c_int, saber_num: c_int) -> *mut saberInfo_t;

    /// `PM_GroundTrace` suspended-vehicle boarding gate: the landed vehicle
    /// entity's `client` is non-null, its `client->ps.m_iVehicleNum == 0`, and
    /// its `spawnflags & 2` (SUSPENDED) is set.
    /// Source: `oracle/codemp/game/bg_pmove.c` (PM_GroundTrace suspended-vehicle board).
    fn suspended_vehicle_boardable(&self, veh_ent_num: c_int) -> qboolean;

    /// `PM_CrashLand` landed-vehicle boarding gate: the traced entity is an
    /// in-use vehicle NPC (`inuse`, non-null `client`, `eType == ET_NPC`,
    /// `NPC_class == CLASS_VEHICLE`), not already ridden
    /// (`client->ps.m_iVehicleNum == 0`), with a non-null `m_pVehicle` that is
    /// not a WALKER/FIGHTER, and the boarder's team is allowed
    /// (`gametype < GT_TEAM || tr.alliedTeam == 0 || tr.alliedTeam ==
    /// self.client->sess.sessionTeam`). `gametype` is read bg-side from
    /// `pm->gametype` and passed in, preserving the current bg/game read split.
    /// Source: `oracle/codemp/game/bg_pmove.c` (PM_CrashLand landed-vehicle board).
    fn landed_vehicle_boardable(
        &self,
        tr_ent_num: c_int,
        self_num: c_int,
        gametype: c_int,
    ) -> qboolean;

    /// `PM_AdjustBBox` solidHack: when the client is in use and has a `client`,
    /// stamp `client->solidHack = level.time + 200`.
    /// Source: `oracle/codemp/game/bg_pmove.c` (PM_AdjustBBox solidHack).
    fn set_solid_hack(&mut self, ent_num: c_int);

    /// `PM_Weapon` NPC-with-no-weapon humanoid test: entity is in use, has a
    /// `client`, and `localAnimIndex == 0`.
    /// Source: `oracle/codemp/game/bg_pmove.c` (PM_Weapon NPC no-weapon branch).
    fn humanoid_inuse_client(&self, ent_num: c_int) -> qboolean;

    /// `PM_VehicleImpact` fighter turn-away gate: the hit entity is NOT suspended
    /// (`(spawnflags & 2) == 0`).
    /// Source: `oracle/codemp/game/bg_slidemove.c:313-398`
    fn fighter_not_suspended(&self, ent_num: c_int) -> qboolean;

    /// `PM_VehicleImpact` knockdown: stamp the hit client's non-playerState
    /// killer-credit fields (`otherKillerMOD`/`otherKillerVehWeapon`/
    /// `otherKillerWeaponType`), which live on `gclient_t`, not `ps`.
    /// Source: `oracle/codemp/game/bg_slidemove.c:402-542`
    fn set_other_killer(
        &mut self,
        ent_num: c_int,
        mod_: c_int,
        veh_weapon: c_int,
        weapon_type: c_int,
    );

    /// `PM_VehicleImpact` hit-entity `inuse` read — `gentity_t.inuse`, absent
    /// from the bg-visible `bgEntity_t` overlay. Field accessor (read at three
    /// impact-damage gates). Source: `oracle/codemp/game/bg_slidemove.c:49-557`
    fn entity_inuse(&self, ent_num: c_int) -> qboolean;

    /// `PM_VehicleImpact` hit-entity `spawnflags` read — `gentity_t.spawnflags`,
    /// absent from the bg-visible `bgEntity_t` overlay. Field accessor (read at
    /// the func_rotating and terrain gates).
    /// Source: `oracle/codemp/game/bg_slidemove.c:49-557`
    fn entity_spawnflags(&self, ent_num: c_int) -> c_int;

    /// `PM_VehicleImpact` hit-entity `takedamage` read — `gentity_t.takedamage`,
    /// absent from the bg-visible `bgEntity_t` overlay. Field accessor (read at
    /// two impact-damage gates). Source: `oracle/codemp/game/bg_slidemove.c:49-557`
    fn entity_takedamage(&self, ent_num: c_int) -> qboolean;

    /// `PM_VehicleImpact` fighter-vs-fighter turn-away gate: the game-tier
    /// `FighterIsLanded(hitEnt->m_pVehicle, hitEnt->playerState)`. The body lives
    /// in the game tier (`FighterNPC.c`) and reaches `m_pVehicle`/`playerState`
    /// off the hit entity, so bg reaches it here by entity number.
    /// Source: `oracle/codemp/game/bg_slidemove.c:313-398`;
    /// `oracle/codemp/game/FighterNPC.c:300-308`.
    fn fighter_is_landed(&self, veh_ent_num: c_int) -> qboolean;

    // ---------------------------------------------------------------------
    // DEC-36 D5 — per-module bg arms.
    //
    // Raven compiles `bg_*.c` into game/cgame/ui and branches per module with
    // `#ifdef QAGAME` / `#elif CGAME` / `#ifdef WE_ARE_IN_THE_UI`. Those arms
    // dispatch here instead: bg stays branch-free and each module's impl
    // reproduces its own arm exactly, including the arms Raven leaves empty
    // (a commented-out call, or an `#ifdef` chain with no arm for that module).
    //
    // The `veh_field_*` methods take the destination slot rather than returning
    // a value because Raven's per-module arm *is* the whole assignment
    // statement `*(int *)(b+ofs) = ...;` — a module with no arm leaves the
    // field untouched, which a returned value could not express.
    // ---------------------------------------------------------------------

    /// `BG_ParseVeh(Weapon)Parm` `VF_MODEL` arm: QAGAME stores `G_ModelIndex`,
    /// cgame/ui store `trap_R_RegisterModel`.
    /// Source: `oracle/codemp/game/bg_vehicleLoad.c:231-237,905-911`
    fn veh_field_model(&mut self, value: &str, dest: *mut c_int);

    /// `VF_MODEL_CLIENT` arm ("MP cgame only"): under `_JK2MP` the QAGAME arm is
    /// the commented-out `//G_ModelIndex` — the game module writes nothing —
    /// while cgame/ui store `trap_R_RegisterModel`.
    /// Source: `oracle/codemp/game/bg_vehicleLoad.c:238-246,912-920`
    fn veh_field_model_client(&mut self, value: &str, dest: *mut c_int);

    /// `VF_EFFECT` arm: QAGAME stores `G_EffectIndex`, cgame stores
    /// `trap_FX_RegisterEffect`; the `#ifdef` chain has no ui arm, so the ui
    /// writes nothing.
    /// Source: `oracle/codemp/game/bg_vehicleLoad.c:247-253,921-927`
    fn veh_field_effect(&mut self, value: &str, dest: *mut c_int);

    /// `VF_EFFECT_CLIENT` arm ("MP cgame only"): under `_JK2MP` the QAGAME arm
    /// is the commented-out `//G_EffectIndex` and there is no ui arm — both
    /// write nothing — while cgame stores `trap_FX_RegisterEffect`.
    /// Source: `oracle/codemp/game/bg_vehicleLoad.c:254-262,928-936`
    fn veh_field_effect_client(&mut self, value: &str, dest: *mut c_int);

    /// `VF_SHADER` arm: the ui stores `trap_R_RegisterShaderNoMip`, cgame stores
    /// `trap_R_RegisterShader`; the `#ifdef` chain has no QAGAME arm, so the
    /// game writes nothing.
    /// Source: `oracle/codemp/game/bg_vehicleLoad.c:263-269,937-943`
    fn veh_field_shader(&mut self, value: &str, dest: *mut c_int);

    /// `VF_SHADER_NOMIP` arm: cgame/ui store `trap_R_RegisterShaderNoMip` under
    /// `#ifndef QAGAME`; the game writes nothing.
    /// Source: `oracle/codemp/game/bg_vehicleLoad.c:270-274,944-948`
    fn veh_field_shader_nomip(&mut self, value: &str, dest: *mut c_int);

    /// `VF_SOUND` arm: QAGAME stores `G_SoundIndex`, cgame/ui store
    /// `trap_S_RegisterSound`.
    /// Source: `oracle/codemp/game/bg_vehicleLoad.c:275-281,949-955`
    fn veh_field_sound(&mut self, value: &str, dest: *mut c_int);

    /// `VF_SOUND_CLIENT` arm ("MP cgame only"): under `_JK2MP` the QAGAME arm is
    /// the commented-out `//G_SoundIndex` — the game module writes nothing —
    /// while cgame/ui store `trap_S_RegisterSound`.
    /// Source: `oracle/codemp/game/bg_vehicleLoad.c:282-290,956-964`
    fn veh_field_sound_client(&mut self, value: &str, dest: *mut c_int);

    /// `VEH_LoadVehWeapon`'s lock-on precache for a homing vehicle weapon: the
    /// QAGAME arm is two commented-out `G_SoundIndex` calls (nothing is
    /// registered server-side); cgame and the ui each register the same five
    /// lock-on samples.
    /// Source: `oracle/codemp/game/bg_vehicleLoad.c:390-409`
    fn veh_weapon_homing_precache(&mut self);

    /// `VEH_LoadVehicle`'s `#ifndef QAGAME` skin precache: cgame/ui register
    /// `models/players/<model>/model_<skin>.skin` when the vehicle names a skin;
    /// the game registers nothing. `model`/`skin` are the `vehicleInfo_t`
    /// strings as parsed (empty when Raven's pointer is NULL); the
    /// `skin && skin[0]` guard is part of the arm and lives in the impl.
    /// Source: `oracle/codemp/game/bg_vehicleLoad.c:1293-1299`
    fn vehicle_skin_precache(&mut self, model: &str, skin: &str);

    /// `VEH_LoadVehicle`'s trailing per-module precache block: QAGAME indexes
    /// two effects and one sound, cgame registers a different radar/HUD set
    /// (plus three more shaders when the vehicle hides its rider), and the ui
    /// has no arm.
    /// Source: `oracle/codemp/game/bg_vehicleLoad.c:1336-1359`
    fn vehicle_load_precache(&mut self, hideRider: qboolean);

    /// `BG_SiegeParseClassFile`'s `uishader` arm: game and cgame zero
    /// `uiPortraitShader` and NUL-fill `uiPortrait`; the ui registers the shader
    /// with `trap_R_RegisterShaderNoMip` and copies the name. Returns the
    /// `(uiPortraitShader, uiPortrait)` pair this module's arm produces.
    /// Source: `oracle/codemp/game/bg_saga.c:975-988`
    fn siege_class_ui_portrait(&mut self, uishader: &str) -> (c_int, String);

    /// `BG_SiegeParseClassFile`'s `class_shader` arm: the game stores 0; cgame
    /// and ui store `trap_R_RegisterShaderNoMip` and print a `could not find
    /// class_shader %s for class %s` error when it comes back 0.
    ///
    /// The trailing "very hacky way to determine class" block
    /// (`bg_saga.rs`'s class-determination span) is gated differently per
    /// module: the QAGAME `#ifdef` arm has no `else` and runs the block
    /// unconditionally (`bg_saga.c:994-1039`); the cgame/ui `#else` arm gates
    /// it under an `else` — only when the shader was found (nonzero handle).
    /// The bool return threads that gate to the caller.
    /// Source: `oracle/codemp/game/bg_saga.c:994-1039`
    fn siege_class_shader(
        &mut self,
        class_shader: &str,
        class_name: &str,
    ) -> (c_int /* handle */, bool /* run class determination */);
}
