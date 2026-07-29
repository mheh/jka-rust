//! `UiGameCallbacks` — the ui-tier `GameCallbacks` implementation.
//!
//! `mp_bg::bg_channel::GameCallbacks` is the bg tier's upcall surface into
//! game-tier logic (`crates/mp/bg/src/bg_channel/game_callbacks.rs`); the game
//! tier's implementor is
//! [`mp_game`'s `GameCallbacksImpl`](../../../game/src/bg_channel/game_impl.rs).
//! ui's sole `&mut dyn GameCallbacks` consumer is
//! [`crate::ui_main::UI_SiegeInit`]'s siege-loader call chain
//! (`BG_SiegeLoadClasses` -> `BG_SiegeParseClassFile`) — verified by reading
//! every `callbacks.*` call in `crates/mp/bg/src/bg_saga.rs`'s siege-load
//! path: only the two DEC-36 D5 per-module arms `siege_class_ui_portrait` and
//! `siege_class_shader` (`bg_saga.c:975-1010`'s `#else` — ui's arm — branches).
//! Every other `GameCallbacks` method is a game-tier entity/gameplay upcall
//! (`G_Damage`, vehicle dispatch, `gentity_t` field reads, …) that ui — which
//! owns no entity arena and runs no simulation — can never reach; each panics
//! loudly with its Raven subject (porting-rules §14: no silent no-ops).

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

use mp_bg::bg_channel::GameCallbacks;
use mp_engine_select::Engine;
use mp_qshared::common::mp::qcommon::{saberInfo_t, usercmd_t};
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::{qboolean, vec3_t};

use crate::trap;

/// The ui-side `GameCallbacks` implementation: holds the `&Engine` the two
/// reachable per-module arms issue `crate::trap` calls through.
pub struct UiGameCallbacks<'a> {
    pub engine: &'a Engine,
}

impl<'a> UiGameCallbacks<'a> {
    pub fn new(engine: &'a Engine) -> Self {
        Self { engine }
    }
}

impl GameCallbacks for UiGameCallbacks<'_> {
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
        // Source: `oracle/codemp/game/g_local.h:1158`
        unreachable!("G_Damage is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.damage")
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
        // Source: `oracle/codemp/game/g_combat.c` (`G_DamageFromKiller`)
        unreachable!("G_DamageFromKiller is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.damage_from_killer")
    }
    fn add_event(&mut self, _entNum: c_int, _event: c_int, _eventParm: c_int) {
        // Source: `oracle/codemp/game/g_local.h:1063`
        unreachable!("G_AddEvent is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.add_event")
    }
    fn entity_legs_anim(&self, _entNum: c_int) -> c_int {
        // Source: `oracle/codemp/game/bg_panimate.c:2610-2616`
        unreachable!("g_entities[].s.legsAnim is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.entity_legs_anim")
    }
    fn entity_torso_anim(&self, _entNum: c_int) -> c_int {
        // Source: `oracle/codemp/game/bg_panimate.c:2680-2683`
        unreachable!("g_entities[].s.torsoAnim is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.entity_torso_anim")
    }
    fn alloc(&mut self, _size: c_int) -> *mut c_void {
        // Source: `oracle/codemp/game/g_local.h:1368`
        unreachable!("G_Alloc is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.alloc")
    }
    fn new_string(&mut self, _string: &str) -> *mut c_char {
        // Source: `oracle/codemp/game/g_local.h:942`
        unreachable!("G_NewString is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.new_string")
    }
    fn play_effect(&mut self, _fxID: c_int, _org: *const vec3_t, _ang: *const vec3_t) {
        // Source: `oracle/codemp/game/g_local.h` (`G_PlayEffect`)
        unreachable!("G_PlayEffect is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.play_effect")
    }
    fn play_effect_id(&mut self, _fxID: c_int, _org: *const vec3_t, _ang: *const vec3_t) -> c_int {
        // Source: `oracle/codemp/game/g_local.h:1034`
        unreachable!("G_PlayEffectID is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.play_effect_id")
    }
    fn sound_index(&mut self, _name: &str) -> c_int {
        // The siege class/team loader never precaches sounds.
        // Source: `oracle/codemp/game/g_local.h:1002`
        unreachable!("G_SoundIndex/trap_S_RegisterSound is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.sound_index")
    }
    fn model_index(&mut self, _name: &str) -> c_int {
        // Source: `oracle/codemp/game/g_local.h:1001`
        unreachable!("G_ModelIndex/trap_R_RegisterModel is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.model_index")
    }
    fn effect_index(&mut self, _name: &str) -> c_int {
        // Source: `oracle/codemp/game/g_local.h:1004`
        unreachable!("G_EffectIndex is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.effect_index")
    }
    fn cheap_weapon_fire(&mut self, _entNum: c_int, _weapon: c_int) {
        // Source: `oracle/codemp/game/g_active.c` (`G_CheapWeaponFire`)
        unreachable!("G_CheapWeaponFire is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.cheap_weapon_fire")
    }
    fn client_check_impact_bbrush(&mut self, _entNum: c_int, _impactNum: c_int) {
        // Source: `oracle/codemp/game/g_active.c` (`Client_CheckImpactBBrush`)
        unreachable!("Client_CheckImpactBBrush is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.client_check_impact_bbrush")
    }
    fn flyveh_surface_destruction(
        &mut self,
        _entNum: c_int,
        _trace: *mut trace_t,
        _magnitude: c_int,
        _force: qboolean,
    ) {
        // Source: `oracle/codemp/game/g_vehicles.c:3190`
        unreachable!("G_FlyVehicleSurfaceDestruction is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.flyveh_surface_destruction")
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
        // Source: `oracle/codemp/game/g_local.h:1022`
        unreachable!("G_SetAnim is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.set_anim")
    }
    fn npc_set_anim(&mut self, _entNum: c_int, _type_: c_int, _anim: c_int, _priority: c_int) {
        // Source: `oracle/codemp/game/g_local.h:217`
        unreachable!("NPC_SetAnim is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.npc_set_anim")
    }
    fn wp_get_vehicle_cam_pos(
        &mut self,
        _vehEntNum: c_int,
        _pilotEntNum: c_int,
        _camPos: *mut vec3_t,
    ) {
        // Source: `oracle/codemp/game/g_weapon.c:3961-4020`
        unreachable!("WP_GetVehicleCamPos is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.wp_get_vehicle_cam_pos")
    }
    fn can_be_enemy(&mut self, _entNum: c_int, _otherNum: c_int) -> qboolean {
        // Source: `oracle/codemp/game/w_saber.c` (`G_CanBeEnemy`)
        unreachable!("G_CanBeEnemy is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.can_be_enemy")
    }
    fn get_time(&self) -> c_int {
        // Source: `oracle/codemp/game/g_local.h` (`level.time`)
        unreachable!("level.time is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.get_time")
    }
    fn try_grapple(&mut self, _entNum: c_int) -> qboolean {
        // Source: `oracle/codemp/game/g_cmds.c:3148-3191` (`TryGrapple`)
        unreachable!("TryGrapple is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.try_grapple")
    }
    fn q3_set_parm(&mut self, _entID: c_int, _parmNum: c_int, _parmValue: &str) {
        // Source: `oracle/codemp/game/g_ICARUScb.c` (`Q3_SetParm`)
        unreachable!("Q3_SetParm is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.q3_set_parm")
    }
    fn board_vehicle(&mut self, _vehEntNum: c_int, _entNum: c_int) -> qboolean {
        // Source: `oracle/codemp/game/g_vehicles.c:630` (`Board`)
        unreachable!("vehicle Board is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.board_vehicle")
    }
    fn update_vehicle(&mut self, _vehEntNum: c_int, _ucmd: *const usercmd_t) {
        // Source: `oracle/codemp/game/bg_pmove.c:10919-10944`
        unreachable!("vehicle Update is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.update_vehicle")
    }
    fn pm_animate_vehicle(&mut self, _vehEntNum: c_int) {
        // Source: `oracle/codemp/game/bg_pmove.c:10921-10945`
        unreachable!("vehicle Animate is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.pm_animate_vehicle")
    }
    fn update_rider(&mut self, _vehEntNum: c_int, _riderEntNum: c_int, _ucmd: *mut usercmd_t) {
        // Source: `oracle/codemp/game/bg_pmove.c:10947-10961`
        unreachable!("vehicle UpdateRider is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.update_rider")
    }
    fn attach_riders(&mut self, _vehEntNum: c_int) {
        // Source: `oracle/codemp/game/bg_pmove.c:11146-11149`
        unreachable!("vehicle AttachRiders is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.attach_riders")
    }
    fn my_saber(&mut self, _client_num: c_int, _saber_num: c_int) -> *mut saberInfo_t {
        // Source: `oracle/codemp/game/bg_saber.c:4100-4141`
        unreachable!("BG_MySaber is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.my_saber")
    }
    fn suspended_vehicle_boardable(&self, _veh_ent_num: c_int) -> qboolean {
        // Source: `oracle/codemp/game/bg_pmove.c` (PM_GroundTrace suspended-vehicle board)
        unreachable!("PM_GroundTrace suspended-vehicle gate is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.suspended_vehicle_boardable")
    }
    fn landed_vehicle_boardable(
        &self,
        _tr_ent_num: c_int,
        _self_num: c_int,
        _gametype: c_int,
    ) -> qboolean {
        // Source: `oracle/codemp/game/bg_pmove.c` (PM_CrashLand landed-vehicle board)
        unreachable!("PM_CrashLand landed-vehicle gate is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.landed_vehicle_boardable")
    }
    fn set_solid_hack(&mut self, _ent_num: c_int) {
        // Source: `oracle/codemp/game/bg_pmove.c` (PM_AdjustBBox solidHack)
        unreachable!("PM_AdjustBBox solidHack is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.set_solid_hack")
    }
    fn humanoid_inuse_client(&self, _ent_num: c_int) -> qboolean {
        // Source: `oracle/codemp/game/bg_pmove.c` (PM_Weapon NPC no-weapon branch)
        unreachable!("PM_Weapon NPC-no-weapon test is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.humanoid_inuse_client")
    }
    fn fighter_not_suspended(&self, _ent_num: c_int) -> qboolean {
        // Source: `oracle/codemp/game/bg_slidemove.c:313-398`
        unreachable!("PM_VehicleImpact turn-away suspended gate is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.fighter_not_suspended")
    }
    fn set_other_killer(
        &mut self,
        _ent_num: c_int,
        _mod_: c_int,
        _veh_weapon: c_int,
        _weapon_type: c_int,
    ) {
        // Source: `oracle/codemp/game/bg_slidemove.c:402-542`
        unreachable!("PM_VehicleImpact knockdown killer-credit stamp is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.set_other_killer")
    }
    fn entity_inuse(&self, _ent_num: c_int) -> qboolean {
        // Source: `oracle/codemp/game/bg_slidemove.c:49-557`
        unreachable!("gentity_t.inuse is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.entity_inuse")
    }
    fn entity_spawnflags(&self, _ent_num: c_int) -> c_int {
        // Source: `oracle/codemp/game/bg_slidemove.c:49-557`
        unreachable!("gentity_t.spawnflags is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.entity_spawnflags")
    }
    fn entity_takedamage(&self, _ent_num: c_int) -> qboolean {
        // Source: `oracle/codemp/game/bg_slidemove.c:49-557`
        unreachable!("gentity_t.takedamage is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.entity_takedamage")
    }
    fn fighter_is_landed(&self, _veh_ent_num: c_int) -> qboolean {
        // Source: `oracle/codemp/game/bg_slidemove.c:313-398`; `oracle/codemp/game/FighterNPC.c:300-308`
        unreachable!("FighterIsLanded is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.fighter_is_landed")
    }
    fn entity_sound(&mut self, _ent_num: c_int, _channel: c_int, _sound_index: c_int) {
        // Source: `oracle/codemp/game/FighterNPC.c:463,512`
        unreachable!("entity_sound is unreachable from ui: ui never runs the vehicle Pmove that hosts the fighter move")
    }
    fn fighter_is_in_space(&mut self, _ent_num: c_int) -> qboolean {
        // Source: `oracle/codemp/game/FighterNPC.c:275-287`
        unreachable!("fighter_is_in_space is unreachable from ui: ui never runs the vehicle Pmove that hosts the fighter move")
    }
    fn veh_turbo_start_fx(&mut self, _veh_ent_num: c_int) {
        // Source: `oracle/codemp/game/SpeederNPC.c:350-371`
        unreachable!("veh_turbo_start_fx is unreachable from ui: ui never runs the vehicle Pmove")
    }
    fn veh_fighter_crash_suicide(&mut self, _parent_ent_num: c_int) {
        // Source: `oracle/codemp/game/FighterNPC.c:1021-1032`
        unreachable!(
            "veh_fighter_crash_suicide is unreachable from ui: ui never runs the vehicle Pmove"
        )
    }

    // ---------------------------------------------------------------------
    // DEC-36 D5 — per-module bg arms. `UI_SiegeInit`'s call chain never
    // reaches `BG_ParseVeh(Weapon)Parm`/`VEH_LoadVehicle` (ui never loads
    // vehicles), so every `veh_*` arm is unreachable; the two siege-class
    // arms are the ones `BG_SiegeParseClassFile` actually calls.
    // ---------------------------------------------------------------------

    fn veh_field_model(&mut self, _value: &str, _dest: *mut c_int) {
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:231-237,905-911`
        unreachable!("VF_MODEL arm is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.veh_field_model")
    }
    fn veh_field_model_client(&mut self, _value: &str, _dest: *mut c_int) {
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:238-246,912-920`
        unreachable!("VF_MODEL_CLIENT arm is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.veh_field_model_client")
    }
    fn veh_field_effect(&mut self, _value: &str, _dest: *mut c_int) {
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:247-253,921-927`
        unreachable!("VF_EFFECT arm is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.veh_field_effect")
    }
    fn veh_field_effect_client(&mut self, _value: &str, _dest: *mut c_int) {
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:254-262,928-936`
        unreachable!("VF_EFFECT_CLIENT arm is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.veh_field_effect_client")
    }
    fn veh_field_shader(&mut self, _value: &str, _dest: *mut c_int) {
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:263-269,937-943`
        unreachable!("VF_SHADER arm is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.veh_field_shader")
    }
    fn veh_field_shader_nomip(&mut self, _value: &str, _dest: *mut c_int) {
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:270-274,944-948`
        unreachable!("VF_SHADER_NOMIP arm is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.veh_field_shader_nomip")
    }
    fn veh_field_sound(&mut self, _value: &str, _dest: *mut c_int) {
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:275-281,949-955`
        unreachable!("VF_SOUND arm is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.veh_field_sound")
    }
    fn veh_field_sound_client(&mut self, _value: &str, _dest: *mut c_int) {
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:282-290,956-964`
        unreachable!("VF_SOUND_CLIENT arm is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.veh_field_sound_client")
    }
    fn veh_weapon_homing_precache(&mut self) {
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:390-409`
        unreachable!("homing lock-on precache is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.veh_weapon_homing_precache")
    }
    fn vehicle_skin_precache(&mut self, _model: &str, _skin: &str) {
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:1293-1299`
        unreachable!("vehicle skin precache is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.vehicle_skin_precache")
    }
    fn vehicle_load_precache(&mut self, _hideRider: qboolean) {
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:1336-1359`
        unreachable!("VEH_LoadVehicle trailing precache is unreachable from ui: UI_SiegeInit's siege-loader path never calls callbacks.vehicle_load_precache")
    }

    fn siege_class_ui_portrait(&mut self, uishader: &str) -> (c_int, String) {
        // Real delegation — the ui `#else` arm of `bg_saga.c:975-988`: register
        // the shader with `trap_R_RegisterShaderNoMip` and keep the parsed name
        // as `uiPortrait` (game/cgame zero both instead).
        let shader = trap::R_RegisterShaderNoMip(self.engine, uishader);
        (shader, uishader.to_string())
    }
    fn siege_class_shader(&mut self, class_shader: &str, class_name: &str) -> (c_int, bool) {
        // Real delegation — the cgame/ui shared `#else` arm of
        // `bg_saga.c:994-1039`: register the shader with
        // `trap_R_RegisterShaderNoMip` and, on a miss, print the same
        // `could not find class_shader` diagnostic Raven's `Com_Printf` does
        // (the game arm stores 0 and never registers or reports). The `else`
        // that gates the class-determination block fires only when the
        // shader was found, so the returned bool mirrors `handle != 0`.
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
