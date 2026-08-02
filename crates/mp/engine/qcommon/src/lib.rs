//! `mp_engine_qcommon` crate. //TODO: Port module mp_engine_qcommon

// Raven-named functions/types (`Com_Printf`, `MSG_Clear`, `CM_LoadMap`, …) keep
// their original casing across the ABI seam, matching `mp_game`'s crate policy.
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod cm;
pub mod cm_draw;
pub mod cm_height_details;
pub mod cm_load;
pub mod cm_patch;
pub mod cm_patch_fns;
pub mod cm_polylib;
pub mod cm_randomterrain;
pub mod cm_shader;
pub mod cm_terrain;
pub mod cm_terrainmap;
pub mod cm_test;
pub mod cm_trace;
pub mod cmd;
pub mod cmd_common;
pub mod cmd_pc;
pub mod collision_world;
pub mod common;
pub mod common_fns;
pub mod cvar;
pub mod cvar_fns;
pub mod files;
pub mod files_common;
pub mod files_pc;
pub mod gp2;
pub mod md4;
pub mod md4_fns;
pub mod miniheap;
pub mod msg;
pub mod net_chan;
pub mod qcommon;
pub mod qfiles;
pub mod roff;
pub mod stringed;
pub mod sys_engine;
pub mod sys_net;
pub mod terrain_handle;
pub mod timing;
pub mod unzip;
pub mod vm;
pub mod vm_fns;
pub mod vm_interpreted;
pub mod vm_x86;
pub mod z_memman;
pub mod z_memman_pc;
pub mod zlib_seam;
