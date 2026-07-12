//! Raven `interface_export_t` — the outbound `I_*` dispatch table.

use mp_host_interface::EngineHost;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::shared::vec3_t;

use crate::Icarus;

/// Raven `interface_export_t` → `InterfaceExport` (§F idiomatic; the `#[repr(C)]`
/// type-port skeleton is **superseded** — ICARUS internal types never cross the
/// module ABI, so layout is free, ICARUS-D3 / ruling 24).
///
/// A plain Rust struct of **bare `fn` items** (fork-5), one slot per
/// `interface.h:17-70` `I_*` entry. Each slot is a
/// `fn(&mut Icarus, &mut dyn EngineHost, …)` pointer (`&mut dyn`, not `impl`,
/// because a bare `fn` pointer is concrete). The fields have no `None` state, so
/// [`Default`] seeds each with the real `Q3_*`/`I_*` fn — identical to
/// `Interface_Init`'s 1:1 assignment, which (re-)populates the table before any
/// `I_*` call, so the seed is overwritten with the same fn before it is observed.
/// Type definition source: `oracle/codemp/icarus/interface.h:17-70`
pub struct InterfaceExport {
    // General
    pub i_load_file: fn(&mut Icarus, &mut dyn EngineHost, name: &str) -> Option<Vec<u8>>,
    pub i_center_print: fn(&mut Icarus, &mut dyn EngineHost, msg: &str),
    pub i_dprintf: fn(&mut Icarus, &mut dyn EngineHost, level: i32, msg: &str),
    pub i_get_entity_by_name:
        fn(&mut Icarus, &mut dyn EngineHost, name: &str) -> *mut sharedEntity_t,
    pub i_get_time: fn(&mut Icarus, &mut dyn EngineHost) -> u32,
    pub i_get_time_scale: fn(&mut Icarus, &mut dyn EngineHost) -> u32,
    pub i_play_sound: fn(
        &mut Icarus,
        &mut dyn EngineHost,
        task_id: i32,
        ent_id: i32,
        name: &str,
        channel: &str,
    ) -> i32,
    pub i_lerp2_pos: fn(
        &mut Icarus,
        &mut dyn EngineHost,
        task_id: i32,
        ent_id: i32,
        origin: vec3_t,
        angles: vec3_t,
        duration: f32,
    ),
    pub i_lerp2_origin: fn(
        &mut Icarus,
        &mut dyn EngineHost,
        task_id: i32,
        ent_id: i32,
        origin: vec3_t,
        duration: f32,
    ),
    pub i_lerp2_angles: fn(
        &mut Icarus,
        &mut dyn EngineHost,
        task_id: i32,
        ent_id: i32,
        angles: vec3_t,
        duration: f32,
    ),
    pub i_get_tag: fn(
        &mut Icarus,
        &mut dyn EngineHost,
        ent_id: i32,
        name: &str,
        lookup: i32,
        info: &mut vec3_t,
    ) -> i32,
    pub i_lerp2_start:
        fn(&mut Icarus, &mut dyn EngineHost, task_id: i32, ent_id: i32, duration: f32),
    pub i_lerp2_end: fn(&mut Icarus, &mut dyn EngineHost, task_id: i32, ent_id: i32, duration: f32),
    pub i_set: fn(
        &mut Icarus,
        &mut dyn EngineHost,
        task_id: i32,
        ent_id: i32,
        type_name: &str,
        data: &str,
    ),
    pub i_use: fn(&mut Icarus, &mut dyn EngineHost, ent_id: i32, name: &str),
    pub i_kill: fn(&mut Icarus, &mut dyn EngineHost, ent_id: i32, name: &str),
    pub i_remove: fn(&mut Icarus, &mut dyn EngineHost, ent_id: i32, name: &str),
    pub i_random: fn(&mut Icarus, &mut dyn EngineHost, min: f32, max: f32) -> f32,
    pub i_play:
        fn(&mut Icarus, &mut dyn EngineHost, task_id: i32, ent_id: i32, type_: &str, name: &str),

    // Camera functions
    pub i_camera_pan:
        fn(&mut Icarus, &mut dyn EngineHost, angles: vec3_t, dir: vec3_t, duration: f32),
    pub i_camera_move: fn(&mut Icarus, &mut dyn EngineHost, origin: vec3_t, duration: f32),
    pub i_camera_zoom: fn(&mut Icarus, &mut dyn EngineHost, fov: f32, duration: f32),
    pub i_camera_roll: fn(&mut Icarus, &mut dyn EngineHost, angle: f32, duration: f32),
    pub i_camera_follow:
        fn(&mut Icarus, &mut dyn EngineHost, name: &str, speed: f32, init_lerp: f32),
    pub i_camera_track:
        fn(&mut Icarus, &mut dyn EngineHost, name: &str, speed: f32, init_lerp: f32),
    pub i_camera_distance: fn(&mut Icarus, &mut dyn EngineHost, dist: f32, init_lerp: f32),
    pub i_camera_fade: fn(
        &mut Icarus,
        &mut dyn EngineHost,
        sr: f32,
        sg: f32,
        sb: f32,
        sa: f32,
        dr: f32,
        dg: f32,
        db: f32,
        da: f32,
        duration: f32,
    ),
    pub i_camera_path: fn(&mut Icarus, &mut dyn EngineHost, name: &str),
    pub i_camera_enable: fn(&mut Icarus, &mut dyn EngineHost),
    pub i_camera_disable: fn(&mut Icarus, &mut dyn EngineHost),
    pub i_camera_shake: fn(&mut Icarus, &mut dyn EngineHost, intensity: f32, duration: i32),

    // Variable information
    pub i_get_float: fn(
        &mut Icarus,
        &mut dyn EngineHost,
        ent_id: i32,
        var_type: i32,
        name: &str,
        value: &mut f32,
    ) -> i32,
    pub i_get_vector: fn(
        &mut Icarus,
        &mut dyn EngineHost,
        ent_id: i32,
        var_type: i32,
        name: &str,
        value: &mut vec3_t,
    ) -> i32,
    pub i_get_string: fn(
        &mut Icarus,
        &mut dyn EngineHost,
        ent_id: i32,
        var_type: i32,
        name: &str,
    ) -> Option<String>,
    pub i_evaluate: fn(
        &mut Icarus,
        &mut dyn EngineHost,
        p1_type: i32,
        p1: &str,
        p2_type: i32,
        p2: &str,
        operator_type: i32,
    ) -> i32,
    pub i_declare_variable: fn(&mut Icarus, &mut dyn EngineHost, var_type: i32, name: &str),
    pub i_free_variable: fn(&mut Icarus, &mut dyn EngineHost, name: &str),

    // Save / Load functions
    pub i_write_save_data: fn(&mut Icarus, &mut dyn EngineHost, chid: u32, data: &[u8]) -> i32,
    pub i_read_save_data: fn(&mut Icarus, &mut dyn EngineHost, chid: u32, length: i32) -> i32,
    pub i_link_entity: fn(&mut Icarus, &mut dyn EngineHost, ent_id: i32) -> i32,
}

impl Default for InterfaceExport {
    /// Seeds every slot with the real crate `Q3_*`/`I_*` fn — identical to
    /// `Interface_Init` (`Q3_Interface.cpp:956-1008`), which re-assigns the same
    /// fn before any `I_*` call, so no slot is ever observed pre-`Interface_Init`.
    fn default() -> Self {
        use crate::game_interface::ICARUS_LinkEntity;
        use crate::q3_interface::{
            AppendToSaveGame, CGCam_Disable, CGCam_Distance, CGCam_Enable, CGCam_Follow,
            CGCam_Move, CGCam_Pan, CGCam_Roll, CGCam_Shake, CGCam_Track, CGCam_Zoom, Q3_CameraFade,
            Q3_CameraPath, Q3_CenterPrint, Q3_DebugPrint, Q3_Evaluate, Q3_GetEntityByName,
            Q3_GetFloat, Q3_GetString, Q3_GetTag, Q3_GetTime, Q3_GetTimeScale, Q3_GetVector,
            Q3_Kill, Q3_Lerp2Angles, Q3_Lerp2End, Q3_Lerp2Origin, Q3_Lerp2Pos, Q3_Lerp2Start,
            Q3_Play, Q3_PlaySound, Q3_ReadScript, Q3_Remove, Q3_Set, Q3_Use, Q_flrand,
            ReadFromSaveGame,
        };
        use crate::q3_registers::{q3_declare_variable, q3_free_variable};

        InterfaceExport {
            i_load_file: Q3_ReadScript,
            i_center_print: Q3_CenterPrint,
            i_dprintf: Q3_DebugPrint,
            i_get_entity_by_name: Q3_GetEntityByName,
            i_get_time: Q3_GetTime,
            i_get_time_scale: Q3_GetTimeScale,
            i_play_sound: Q3_PlaySound,
            i_lerp2_pos: Q3_Lerp2Pos,
            i_lerp2_origin: Q3_Lerp2Origin,
            i_lerp2_angles: Q3_Lerp2Angles,
            i_get_tag: Q3_GetTag,
            i_lerp2_start: Q3_Lerp2Start,
            i_lerp2_end: Q3_Lerp2End,
            i_set: Q3_Set,
            i_use: Q3_Use,
            i_kill: Q3_Kill,
            i_remove: Q3_Remove,
            i_random: Q_flrand,
            i_play: Q3_Play,
            i_camera_pan: CGCam_Pan,
            i_camera_move: CGCam_Move,
            i_camera_zoom: CGCam_Zoom,
            i_camera_roll: CGCam_Roll,
            i_camera_follow: CGCam_Follow,
            i_camera_track: CGCam_Track,
            i_camera_distance: CGCam_Distance,
            i_camera_fade: Q3_CameraFade,
            i_camera_path: Q3_CameraPath,
            i_camera_enable: CGCam_Enable,
            i_camera_disable: CGCam_Disable,
            i_camera_shake: CGCam_Shake,
            i_get_float: Q3_GetFloat,
            i_get_vector: Q3_GetVector,
            i_get_string: Q3_GetString,
            i_evaluate: Q3_Evaluate,
            i_declare_variable: q3_declare_variable,
            i_free_variable: q3_free_variable,
            i_write_save_data: AppendToSaveGame,
            i_read_save_data: ReadFromSaveGame,
            i_link_entity: ICARUS_LinkEntity,
        }
    }
}
