//! `CgGameCallbacks` — the cgame-tier `GameCallbacks` implementation.
//!
//! `mp_bg::bg_channel::GameCallbacks` is the bg tier's upcall surface into
//! game-tier logic (`crates/mp/bg/src/bg_channel/game_callbacks.rs`); the game
//! tier's implementor is
//! [`mp_game`'s `GameCallbacksImpl`](../../../game/src/bg_channel/game_impl.rs)
//! and the ui's is
//! [`mp_ui`'s `UiGameCallbacks`](../../../ui/src/bg_channel/ui_game_callbacks.rs).
//!
//! cgame reverses the ui's ratio. Raven compiles `bg_misc.c`, `bg_panimate.c`,
//! `bg_pmove.c`, `bg_saber.c`, `bg_saberLoad.c`, `bg_saga.c`, `bg_slidemove.c`,
//! `bg_vehicleLoad.c` and `bg_weapons.c` into cgame (`JK2_cgame.vcproj`), so
//! nearly the whole trait is *compiled* here — the question per method is which
//! `#ifdef` arm the cgame build took. DEC-46.5 is the law:
//!
//! - **16 registration arms are real** (DEC-36 D5): `sound_index`,
//!   `model_index`, `effect_index`, the eight `veh_field_*` arms,
//!   `veh_weapon_homing_precache`, `vehicle_skin_precache`,
//!   `vehicle_load_precache`, `siege_class_ui_portrait`, `siege_class_shader` —
//!   each transcribed from its own `#elif CGAME` / `#else` arm and issued
//!   through [`crate::trap`].
//! - **31 methods are inert**: every call site is inside an `#ifdef QAGAME`
//!   block that Raven's cgame build compiled out, so the neutral return keeps
//!   the gated block unreachable (accessors → `qfalse`/`0`/sentinel, mutators →
//!   no-op). Each cites the oracle `#ifdef` proving it. A live `cg_entities`
//!   read here would execute logic the oracle cgame never ran, which the C6b
//!   demo referee would see as divergence.
//! - **2 methods are genuinely unreachable**: `alloc` (`G_Alloc`) and `set_anim`
//!   (`G_SetAnim`) have no live bg call site in either build, so they take the
//!   ui-style loud panic naming the Raven subject.
//! - **2 methods are real cgame arms blocked on C5 state**: `my_saber`
//!   (`bg_saber.c:4115-4137`, needs `cgs.clientinfo` / `cg_entities[].npcClient`)
//!   and `new_string` (`bg_misc.c:375`'s `CG_NewString`, needs cgame's string
//!   pool). Both are `todo!()` + `TODO: Port` markers until `CgWorld` lands
//!   (DEC-46.1); porting-rules §14 forbids the silent fake a neutral value
//!   would be here.
//!
//! State: only the engine transport. No cgame D5 arm caches — Raven's cgame
//! arms either store the handle through the bg-owned `dest` slot or discard it
//! (the precache blocks throw their handles away), so there is no media table
//! for `CgWorld` to absorb (DEC-46.1). The two C5-blocked methods above are the
//! only places `CgWorld` enters this file.

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

use mp_bg::bg_channel::GameCallbacks;
use mp_engine_select::Engine;
use mp_qshared::common::mp::qcommon::{saberInfo_t, usercmd_t};
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::{qboolean, qfalse, vec3_t, ENTITYNUM_NONE};

use crate::trap;

/// The cgame-side `GameCallbacks` implementation: holds the `&Engine` the 16
/// DEC-36 D5 registration arms issue their `crate::trap` calls through.
pub struct CgGameCallbacks<'a> {
    pub engine: &'a Engine,
}

impl<'a> CgGameCallbacks<'a> {
    pub fn new(engine: &'a Engine) -> Self {
        Self { engine }
    }
}

impl GameCallbacks for CgGameCallbacks<'_> {
    // ---------------------------------------------------------------------
    // Inert — `#ifdef QAGAME` blocks Raven's cgame build compiled out.
    // ---------------------------------------------------------------------

    fn damage(
        &mut self,
        _targNum: c_int,
        _inflictorNum: c_int,
        _attackerNum: c_int,
        _dir: *const vec3_t,
        _point: *const vec3_t,
        _damage: c_int,
        _dflags: c_int,
        _mod_: c_int,
    ) {
        // Inert: both `PM_VehicleImpact` `G_Damage` calls sit inside `#ifdef
        // QAGAME` (the extern is `#ifdef QAGAME` too), so cgame has no arm.
        // Source: `oracle/codemp/game/bg_slidemove.c:539`
    }
    fn damage_from_killer(
        &mut self,
        _targNum: c_int,
        _inflictorNum: c_int,
        _attackerNum: c_int,
        _killerNum: c_int,
        _dir: *const vec3_t,
        _point: *const vec3_t,
        _damage: c_int,
        _dflags: c_int,
        _mod_: c_int,
    ) {
        // Inert: the killer-credited `G_Damage` is inside the same `#ifdef
        // QAGAME` impact block.
        // Source: `oracle/codemp/game/bg_slidemove.c:467`
    }
    fn add_event(&mut self, _entNum: c_int, _event: c_int, _eventParm: c_int) {
        // Inert: both bg `G_AddEvent` sites are `#ifdef QAGAME` — cgame plays
        // its wake/impact effects from the snapshot, not from pmove.
        // Source: `oracle/codemp/game/bg_pmove.c:781`;
        // `oracle/codemp/game/bg_slidemove.c:413`
    }
    fn entity_legs_anim(&self, _entNum: c_int) -> c_int {
        // Inert: `BG_StartLegsAnim`'s `else if (g_entities[...].s.legsAnim ==
        // anim)` restart check is `#ifdef QAGAME`. A sentinel that no
        // `animNumber_t` can equal (the enum starts at 0) keeps `BG_FlipPart`
        // unreached, exactly as the compiled-out branch did.
        // Source: `oracle/codemp/game/bg_panimate.c:2610-2616`
        -1
    }
    fn entity_torso_anim(&self, _entNum: c_int) -> c_int {
        // Inert: the matching `#ifdef QAGAME` restart check in
        // `BG_StartTorsoAnim`; same never-matching sentinel.
        // Source: `oracle/codemp/game/bg_panimate.c:2677-2683`
        -1
    }
    fn play_effect(&mut self, _fxID: c_int, _org: *const vec3_t, _ang: *const vec3_t) {
        // Inert: every bg `G_PlayEffect` (crash-land dust, lava/acid/water
        // splashes) is `#ifdef QAGAME`.
        // Source: `oracle/codemp/game/bg_pmove.c:3719,5721-5729`
    }
    fn play_effect_id(&mut self, _fxID: c_int, _org: *const vec3_t, _ang: *const vec3_t) -> c_int {
        // Inert: the wake-FX `G_PlayEffectID` is `#ifdef QAGAME`; the bg caller
        // discards the result, so `ENTITYNUM_NONE` is the neutral "no entity".
        // Source: `oracle/codemp/game/bg_pmove.c:845`
        ENTITYNUM_NONE
    }
    fn cheap_weapon_fire(&mut self, _entNum: c_int, _weapon: c_int) {
        // Inert: Raven's own comment on the `#ifdef QAGAME` guard — "hack, only
        // do it game-side. vehicle weapons don't really need predicting".
        // Source: `oracle/codemp/game/bg_pmove.c:7396-7403`
    }
    fn client_check_impact_bbrush(&mut self, _entNum: c_int, _impactNum: c_int) {
        // Inert: `Client_CheckImpactBBrush`'s extern and call are both `#ifdef
        // QAGAME`.
        // Source: `oracle/codemp/game/bg_slidemove.c:589,613`
    }
    fn flyveh_surface_destruction(
        &mut self,
        _entNum: c_int,
        _trace: *mut trace_t,
        _magnitude: c_int,
        _force: qboolean,
    ) {
        // Inert: `#ifdef QAGAME` extern and call site.
        // Source: `oracle/codemp/game/bg_slidemove.c:25,472`
    }
    fn npc_set_anim(&mut self, _entNum: c_int, _type_: c_int, _anim: c_int, _priority: c_int) {
        // Inert: the extern is guarded by Raven's "including game headers on
        // cgame is FORBIDDEN ^_^" `#ifdef QAGAME`, and both duel-loss call
        // sites are `#ifdef QAGAME`.
        // Source: `oracle/codemp/game/bg_saber.c:990,1119,1180`
    }
    fn wp_get_vehicle_cam_pos(
        &mut self,
        _vehEntNum: c_int,
        _pilotEntNum: c_int,
        _camPos: *mut vec3_t,
    ) {
        // Inert: `BG_VehTraceFromCamPos`'s cam-pos upcall is `#ifdef QAGAME`;
        // leaving `camPos` untouched is what the compiled-out arm did.
        // Source: `oracle/codemp/game/bg_pmove.c:5828,5840`
    }
    fn can_be_enemy(&mut self, _entNum: c_int, _otherNum: c_int) -> qboolean {
        // Inert: `G_CanBeEnemy`'s extern and impact call site are `#ifdef
        // QAGAME`.
        // Source: `oracle/codemp/game/bg_slidemove.c:26,500`
        qfalse
    }
    fn get_time(&self) -> c_int {
        // Inert: bg's only `level.time` read is the `#ifdef QAGAME`
        // impact-bbrush debounce.
        // Source: `oracle/codemp/game/bg_slidemove.c:610`
        0
    }
    fn try_grapple(&mut self, _entNum: c_int) -> qboolean {
        // Inert: `TryGrapple`'s extern and call site are `#ifdef QAGAME`.
        // Source: `oracle/codemp/game/bg_pmove.c:20,7481`
        qfalse
    }
    fn q3_set_parm(&mut self, _entID: c_int, _parmNum: c_int, _parmValue: &str) {
        // Inert: `BG_ParseField`'s `F_PARM1..F_PARM16` cases exist only under
        // `#ifdef QAGAME`; cgame's spawn-var parse never reaches them.
        // Source: `oracle/codemp/game/bg_misc.c:29,396-414`
    }
    fn board_vehicle(&mut self, _vehEntNum: c_int, _entNum: c_int) -> qboolean {
        // Inert: both `m_pVehicleInfo->Board` sites (the `PM_CrashLand` landed
        // vehicle and the `PM_GroundTrace` suspended vehicle) are `#ifdef
        // QAGAME`.
        // Source: `oracle/codemp/game/bg_pmove.c:4258,10898`
        qfalse
    }
    fn update_vehicle(&mut self, _vehEntNum: c_int, _ucmd: *const usercmd_t) {
        // Inert: `PmoveSingle`'s vehicle-NPC `Update` dispatch is `#ifdef
        // QAGAME`.
        // Source: `oracle/codemp/game/bg_pmove.c:10920,10945`
    }
    fn pm_animate_vehicle(&mut self, _vehEntNum: c_int) {
        // Inert: the paired `Animate` dispatch is inside the same `#ifdef
        // QAGAME` block.
        // Source: `oracle/codemp/game/bg_pmove.c:10921,10946`
    }
    fn update_rider(&mut self, _vehEntNum: c_int, _riderEntNum: c_int, _ucmd: *mut usercmd_t) {
        // Inert: the driver and passenger `UpdateRider` dispatches are inside
        // the same `#ifdef QAGAME` block.
        // Source: `oracle/codemp/game/bg_pmove.c:10948,10957`
    }
    fn attach_riders(&mut self, _vehEntNum: c_int) {
        // Inert. The one method whose call site is NOT `#ifdef`-gated: the
        // `AttachRiders` dispatch at `bg_pmove.c:11148` compiles into cgame,
        // where `G_Set<Type>VehicleFunctions` binds the slot to bg's own
        // `AttachRidersGeneric` (`bg_vehicleLoad.c:1643`) — the game's
        // overriding `AttachRiders` comes from `G_SetSharedVehicleFunctions`,
        // which is `#ifdef QAGAME` (`bg_vehicleLoad.c:685-688`). The enclosing
        // block still requires `pm->ps->clientNum >= MAX_CLIENTS`, and cgame
        // only ever `Pmove`s the local client's playerState
        // (`cg_predict.c:1319,1408`), so the block is dead in the cgame build.
        // DEC-46.5's "prefer inert" applies; C5 revisits it if a vehicle-NPC
        // prediction path ever appears.
        // Source: `oracle/codemp/game/bg_pmove.c:11133-11150`
    }
    fn suspended_vehicle_boardable(&self, _veh_ent_num: c_int) -> qboolean {
        // Inert: the suspended-vehicle gate lives inside the `#ifdef QAGAME`
        // `PM_GroundTrace` boarding block.
        // Source: `oracle/codemp/game/bg_pmove.c:10887-10900`
        qfalse
    }
    fn landed_vehicle_boardable(
        &self,
        _tr_ent_num: c_int,
        _self_num: c_int,
        _gametype: c_int,
    ) -> qboolean {
        // Inert: the landed-vehicle gate lives inside the `#ifdef QAGAME`
        // `PM_CrashLand` boarding block.
        // Source: `oracle/codemp/game/bg_pmove.c:4240-4263`
        qfalse
    }
    fn set_solid_hack(&mut self, _ent_num: c_int) {
        // Inert: `PM_AdjustBBox`'s `client->solidHack = level.time + 200` stamp
        // is `#ifdef QAGAME`.
        // Source: `oracle/codemp/game/bg_pmove.c:4453`
    }
    fn humanoid_inuse_client(&self, _ent_num: c_int) -> qboolean {
        // Inert: `PM_Weapon`'s whole NPC-with-no-weapon branch, including the
        // `gent->inuse && gent->client && !gent->localAnimIndex` humanoid test,
        // is `#ifdef QAGAME`.
        // Source: `oracle/codemp/game/bg_pmove.c:6649-6663`
        qfalse
    }
    fn fighter_not_suspended(&self, _ent_num: c_int) -> qboolean {
        // Inert: the turn-away gate is inside `#ifdef QAGAME//server-side, turn
        // the guy we hit away from us, too`.
        // Source: `oracle/codemp/game/bg_slidemove.c:313-320`
        qfalse
    }
    fn set_other_killer(
        &mut self,
        _ent_num: c_int,
        _mod_: c_int,
        _veh_weapon: c_int,
        _weapon_type: c_int,
    ) {
        // Inert: the `gclient_t` killer-credit stamps are `#ifdef QAGAME`.
        // Source: `oracle/codemp/game/bg_slidemove.c:509-514`
    }
    fn entity_inuse(&self, _ent_num: c_int) -> qboolean {
        // Inert: every `hitEnt->inuse` read in `PM_VehicleImpact` is `#ifdef
        // QAGAME`.
        // Source: `oracle/codemp/game/bg_slidemove.c:60,429,479`
        qfalse
    }
    fn entity_spawnflags(&self, _ent_num: c_int) -> c_int {
        // Inert: the func_rotating and terrain `hitEnt->spawnflags` reads are
        // `#ifdef QAGAME`.
        // Source: `oracle/codemp/game/bg_slidemove.c:96,419`
        0
    }
    fn entity_takedamage(&self, _ent_num: c_int) -> qboolean {
        // Inert: both `hitEnt->takedamage` gates are `#ifdef QAGAME`.
        // Source: `oracle/codemp/game/bg_slidemove.c:429,480`
        qfalse
    }
    fn fighter_is_landed(&self, _veh_ent_num: c_int) -> qboolean {
        // Inert: `FighterIsLanded`'s extern and its turn-away call site are
        // `#ifdef QAGAME`.
        // Source: `oracle/codemp/game/bg_slidemove.c:42,316`
        qfalse
    }

    // ---------------------------------------------------------------------
    // Genuinely unreachable — no live bg call site in any build.
    // ---------------------------------------------------------------------

    fn alloc(&mut self, _size: c_int) -> *mut c_void {
        // `G_Alloc` has no bg call site: bg allocates from its own pool
        // (`BG_Alloc`, `bg_misc.c:3328`), which is a `BgTraps`-free bg-tier
        // body, not a game upcall.
        // Source: `oracle/codemp/game/g_local.h:1368`
        unreachable!("G_Alloc is unreachable from cgame: no bg call site reaches callbacks.alloc")
    }
    fn set_anim(
        &mut self,
        _entNum: c_int,
        _ucmd: *mut usercmd_t,
        _setAnimParts: c_int,
        _anim: c_int,
        _setAnimFlags: c_int,
        _blendTime: c_int,
    ) {
        // `G_SetAnim` has no live bg call site — bg's only reference is the
        // `#if 0` grapple block; bg sets animations through `BG_SetAnim`.
        // Source: `oracle/codemp/game/g_local.h:1022`;
        // `oracle/codemp/game/bg_pmove.c:7464` (`#if 0`)
        unreachable!(
            "G_SetAnim is unreachable from cgame: no bg call site reaches callbacks.set_anim"
        )
    }

    // ---------------------------------------------------------------------
    // Real cgame arms blocked on C5 state (`CgWorld`, DEC-46.1).
    // ---------------------------------------------------------------------

    fn new_string(&mut self, _string: &str) -> *mut c_char {
        //TODO: Port CG_NewString
        // `BG_ParseField`'s `F_LSTRING` case takes `CG_NewString` in the cgame
        // build (the `#else` of `#ifdef QAGAME`), reached from
        // `CG_ParseSpawnVars`. `CG_NewString` copies into cgame's string pool
        // (`CG_StrPool_Alloc`) — C5 state, so this cannot be answered yet, and
        // a neutral pointer would be the silent fake porting-rules §14 forbids.
        // Source: `oracle/codemp/game/bg_misc.c:342-375`;
        // `oracle/codemp/cgame/cg_main.c:3344-3370`
        todo!("Port CG_NewString — oracle/codemp/cgame/cg_main.c:3344-3370 (needs CgWorld's string pool, DEC-46.1)")
    }
    fn my_saber(&mut self, _client_num: c_int, _saber_num: c_int) -> *mut saberInfo_t {
        //TODO: Port BG_MySaber
        // `BG_MySaber`'s `#elif defined CGAME` arm is a real implementation:
        // resolve `cgs.clientinfo[clientNum]` (or `cg_entities[clientNum]
        // .npcClient` above `MAX_CLIENTS`), require `ci->infoValid` and a
        // non-empty `saber[saberNum].model`, and return `&ci->saber[saberNum]`.
        // Both tables are C5 `CgWorld` state; returning NULL instead would
        // silently disable saber prediction rather than reproduce the arm.
        // Source: `oracle/codemp/game/bg_saber.c:4115-4137`
        todo!("Port BG_MySaber cgame arm — oracle/codemp/game/bg_saber.c:4115-4137 (needs CgWorld's cgs.clientinfo / cg_entities, DEC-46.1)")
    }

    // ---------------------------------------------------------------------
    // DEC-36 D5 — per-module bg arms. cgame takes the `#elif CGAME` /
    // `#else` arm of each, registering through `crate::trap`.
    // ---------------------------------------------------------------------

    fn sound_index(&mut self, name: &str) -> c_int {
        // cgame arm: `BG_SoundIndex`'s `#elif defined CGAME` is
        // `trap_S_RegisterSound`; likewise `VEH_LoadVehicle`'s flammable
        // `fire_lp.wav` precache.
        // Source: `oracle/codemp/game/bg_saberLoad.c:32-39`;
        // `oracle/codemp/game/bg_vehicleLoad.c:1314-1322`
        trap::S_RegisterSound(self.engine, name)
    }
    fn model_index(&mut self, name: &str) -> c_int {
        // cgame arm: `VEH_LoadVehicle`'s `#else` registers the vehicle's
        // `model.glm` with `trap_R_RegisterModel`.
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:1262-1269`
        trap::R_RegisterModel(self.engine, name)
    }
    fn effect_index(&mut self, name: &str) -> c_int {
        // cgame arm: `#elif CGAME` is `trap_FX_RegisterEffect` (the
        // explosion-mark and hover-dust precaches).
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:1306-1312,1325-1334`
        trap::FX_RegisterEffect(self.engine, name)
    }

    fn veh_field_model(&mut self, value: &str, dest: *mut c_int) {
        // cgame arm: `*(int *)(b+ofs) = trap_R_RegisterModel( value );`
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:231-237,905-911`
        unsafe { *dest = trap::R_RegisterModel(self.engine, value) }
    }
    fn veh_field_model_client(&mut self, value: &str, dest: *mut c_int) {
        // cgame arm: the `#else` of the `_JK2MP`/`QAGAME` chain —
        // `trap_R_RegisterModel`.
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:238-246,912-920`
        unsafe { *dest = trap::R_RegisterModel(self.engine, value) }
    }
    fn veh_field_effect(&mut self, value: &str, dest: *mut c_int) {
        // cgame arm: `*(int *)(b+ofs) = trap_FX_RegisterEffect( value );`
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:247-253,921-927`
        unsafe { *dest = trap::FX_RegisterEffect(self.engine, value) }
    }
    fn veh_field_effect_client(&mut self, value: &str, dest: *mut c_int) {
        // cgame arm: the `#elif CGAME` — `trap_FX_RegisterEffect`.
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:254-262,928-936`
        unsafe { *dest = trap::FX_RegisterEffect(self.engine, value) }
    }
    fn veh_field_shader(&mut self, value: &str, dest: *mut c_int) {
        // cgame arm: `#elif CGAME` is `trap_R_RegisterShader` — the mipped
        // variant, unlike the ui's `trap_R_RegisterShaderNoMip`.
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:263-269,937-943`
        unsafe { *dest = trap::R_RegisterShader(self.engine, value) }
    }
    fn veh_field_shader_nomip(&mut self, value: &str, dest: *mut c_int) {
        // cgame arm: the `#ifndef QAGAME` — `trap_R_RegisterShaderNoMip`.
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:270-274,944-948`
        unsafe { *dest = trap::R_RegisterShaderNoMip(self.engine, value) }
    }
    fn veh_field_sound(&mut self, value: &str, dest: *mut c_int) {
        // cgame arm: the `#else` — `trap_S_RegisterSound`.
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:275-281,949-955`
        unsafe { *dest = trap::S_RegisterSound(self.engine, value) }
    }
    fn veh_field_sound_client(&mut self, value: &str, dest: *mut c_int) {
        // cgame arm: the `#else` of the `_JK2MP`/`QAGAME` chain —
        // `trap_S_RegisterSound`.
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:282-290,956-964`
        unsafe { *dest = trap::S_RegisterSound(self.engine, value) }
    }
    fn veh_weapon_homing_precache(&mut self) {
        // cgame arm: the five lock-on samples, in Raven's order.
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:390-409`
        trap::S_RegisterSound(self.engine, "sound/vehicles/weapons/common/tick.wav");
        trap::S_RegisterSound(self.engine, "sound/vehicles/weapons/common/lock.wav");
        trap::S_RegisterSound(self.engine, "sound/vehicles/common/lockalarm1.wav");
        trap::S_RegisterSound(self.engine, "sound/vehicles/common/lockalarm2.wav");
        trap::S_RegisterSound(self.engine, "sound/vehicles/common/lockalarm3.wav");
    }
    fn vehicle_skin_precache(&mut self, model: &str, skin: &str) {
        // cgame arm: the `#ifndef QAGAME` inside the `_JK2MP` `#else` —
        // `trap_R_RegisterSkin` under Raven's `skin && skin[0]` guard (bg hands
        // an empty string where Raven's pointer is NULL).
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:1293-1299`
        if !skin.is_empty() {
            trap::R_RegisterSkin(
                self.engine,
                &format!("models/players/{}/model_{}.skin", model, skin),
            );
        }
    }
    fn vehicle_load_precache(&mut self, hideRider: qboolean) {
        // cgame arm: the radar/HUD set, in Raven's order, widened by three
        // circle-base shaders when the vehicle hides its rider.
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:1336-1359`
        trap::R_RegisterShader(self.engine, "gfx/menus/radar/bracket");
        trap::R_RegisterShader(self.engine, "gfx/menus/radar/lead");
        trap::R_RegisterShaderNoMip(self.engine, "gfx/menus/radar/asteroid");
        trap::S_RegisterSound(self.engine, "sound/vehicles/common/impactalarm.wav");
        trap::S_RegisterSound(self.engine, "sound/vehicles/common/linkweaps.wav");
        trap::S_RegisterSound(self.engine, "sound/vehicles/common/release.wav");
        trap::FX_RegisterEffect(self.engine, "effects/ships/dest_burning.efx");
        trap::FX_RegisterEffect(self.engine, "effects/ships/dest_destroyed.efx");
        trap::FX_RegisterEffect(self.engine, "volumetric/black_smoke");
        trap::FX_RegisterEffect(self.engine, "ships/fire");
        trap::FX_RegisterEffect(self.engine, "ships/hyperspace_stars");

        if hideRider != qfalse {
            trap::R_RegisterShaderNoMip(self.engine, "gfx/menus/radar/circle_base");
            trap::R_RegisterShaderNoMip(self.engine, "gfx/menus/radar/circle_base_frame");
            trap::R_RegisterShaderNoMip(self.engine, "gfx/menus/radar/circle_base_shield");
        }
    }
    fn siege_class_ui_portrait(&mut self, _uishader: &str) -> (c_int, String) {
        // cgame arm: `#elif defined CGAME` zeroes `uiPortraitShader` and
        // NUL-fills `uiPortrait` — identical to the QAGAME arm; only the ui
        // registers the portrait.
        // Source: `oracle/codemp/game/bg_saga.c:975-988`
        (0, String::new())
    }
    fn siege_class_shader(&mut self, class_shader: &str, class_name: &str) -> (c_int, bool) {
        // cgame arm: the `#else //cgame, ui` — register with
        // `trap_R_RegisterShaderNoMip`, print Raven's `could not find
        // class_shader` diagnostic on a miss, and gate the trailing
        // class-determination block behind `else` (shader found).
        // Source: `oracle/codemp/game/bg_saga.c:994-1039`
        let shader = trap::R_RegisterShaderNoMip(self.engine, class_shader);
        if shader == 0 {
            trap::Print(
                self.engine,
                &format!(
                    "ERROR: could not find class_shader {} for class {}\n",
                    class_shader, class_name
                ),
            );
        }
        (shader, shader != 0)
    }
}
