//! `native_types` — Raven-free scalar/handle primitives that are byte-identical
//! across SP and MP `q_shared.h`. Cross-mode; re-exported by each mode's
//! `qshared` umbrella.

#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_uchar, c_ulong, c_ushort};

pub mod anim;
pub mod cgame;
pub mod client;
pub mod cvar;
pub mod miniheap;
pub mod mp3;
pub mod rmg;
pub mod say;
pub mod snd;
pub mod stdio;
pub mod ui;
pub mod uishared;

pub use anim::anim_event_type::animEventType_t;
pub use anim::anim_number::{animNumber_t, SABER_ANIM_GROUP_SIZE};
pub use anim::footstep_type::footstepType_t;
pub use cgame::footstep_t::footstep_t;
pub use cgame::le_flag_t::leFlag_t;
pub use cgame::powerup_info_t::powerupInfo_t;
pub use client::field_t::field_t;
pub use client::kbutton_t::kbutton_t;
pub use cvar::vm_cvar_t::vmCvar_t;
pub use miniheap::cmini_heap::CMiniHeap;
pub use mp3::in_out::IN_OUT;
pub use mp3::sample::SAMPLE;
pub use mp3::sbt_function::SBT_FUNCTION;
pub use mp3::xform_function::XFORM_FUNCTION;
pub use rmg::ermdir::{ERMDir, DIR_FIRST};
pub use rmg::symmetry_t::symmetry_t;
pub use say::saying_t::saying_t;
pub use snd::ambient_set_s::{ambientSet_s, ambientSet_t, MAX_SET_NAME_LENGTH, MAX_WAVES_PER_GROUP};
pub use snd::dma_t::dma_t;
pub use snd::id3v1_1::id3v1_1;
pub use snd::portable_samplepair_t::portable_samplepair_t;
pub use snd::set_e::set_e;
pub use snd::set_keyword_e::setKeyword_e;
pub use snd::sound_compression_method_t::SoundCompressionMethod_t;
pub use snd::streamingbuffer::STREAMINGBUFFER;
pub use snd::wavinfo_t::wavinfo_t;
pub use stdio::file::FILE;
pub use ui::mod_info_t::modInfo_t;
pub use ui::player_species_info_t::playerSpeciesInfo_t;
pub use uishared::column_info_s::{columnInfo_s, columnInfo_t};
pub use uishared::edit_field_def_s::{editFieldDef_s, editFieldDef_t};
pub use uishared::list_box_def_s::{listBoxDef_s, listBoxDef_t};
pub use uishared::rect_def_t::rectDef_t;
pub use uishared::text_scroll_def_s::{textScrollDef_s, textScrollDef_t};

/// Raven `byte`.
///
/// Type definition source: `oracle/code/game/q_shared.h:176`
/// Type definition source: `oracle/codemp/game/q_shared.h:349`
pub type byte = c_uchar;

/// Raven `word`.
///
/// Type definition source: `oracle/code/game/q_shared.h:174`
/// Type definition source: `oracle/codemp/game/q_shared.h:350`
pub type word = c_ushort;

/// Raven `ulong`.
///
/// Type definition source: `oracle/code/game/q_shared.h:173`
/// Type definition source: `oracle/codemp/game/q_shared.h:351`
pub type ulong = c_ulong;

/// Raven `qboolean`.
///
/// Type definition source: `oracle/code/game/q_shared.h`
/// Type definition source: `oracle/codemp/game/q_shared.h`
pub type qboolean = c_int;

/// Raven `qfalse`/`qtrue` — the `qboolean` enum values, in Raven's lowercase
/// spelling.
///
/// Definition source: `oracle/code/game/q_shared.h`
/// Definition source: `oracle/codemp/game/q_shared.h`
#[allow(non_upper_case_globals)]
pub const qfalse: qboolean = 0;
#[allow(non_upper_case_globals)]
pub const qtrue: qboolean = 1;

/// Raven `fileHandle_t`.
///
/// Type definition source: `oracle/code/game/q_shared.h:187`
/// Type definition source: `oracle/codemp/game/q_shared.h:362`
pub type fileHandle_t = c_int;

/// Raven `clipHandle_t` collision model handle.
///
/// Type definition source: `oracle/code/game/q_shared.h:188`
/// Type definition source: `oracle/codemp/game/q_shared.h:363`
pub type clipHandle_t = c_int;

/// Raven `qhandle_t`.
///
/// Type definition source: `oracle/code/game/q_shared.h:183`
/// Type definition source: `oracle/codemp/game/q_shared.h:358`
pub type qhandle_t = c_int;

/// Raven `thandle_t`.
///
/// Type definition source: `oracle/code/game/q_shared.h:184`
/// Type definition source: `oracle/codemp/game/q_shared.h:359`
pub type thandle_t = c_int;

/// Raven `fxHandle_t`.
///
/// Type definition source: `oracle/code/game/q_shared.h:185`
/// Type definition source: `oracle/codemp/game/q_shared.h:360`
pub type fxHandle_t = c_int;

/// Raven `sfxHandle_t`.
///
/// Type definition source: `oracle/code/game/q_shared.h:186`
/// Type definition source: `oracle/codemp/game/q_shared.h:361`
pub type sfxHandle_t = c_int;

/// Raven `mdxaBone_t`.
///
/// Type definition source: `oracle/code/renderer/mdx_format.h:137`
/// Type definition source: `oracle/codemp/game/q_shared.h:3078`
/// Type definition source: `oracle/codemp/renderer/mdx_format.h:137`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct mdxaBone_t {
    pub matrix: [[f32; 4]; 3],
}

const _: () = assert!(core::mem::size_of::<mdxaBone_t>() == 48);
const _: () = assert!(core::mem::offset_of!(mdxaBone_t, matrix) == 0);

/// Raven `MAX_QPATH`.
///
/// Definition source: `oracle/code/game/q_shared.h:215`
/// Definition source: `oracle/codemp/game/q_shared.h:393`
pub const MAX_QPATH: usize = 64;
