//! MP `tagOwner_t` and its `TAG_*` constants.
//!
//! Raven: ref tag stuff ported from SP (and C-ified).
//! Type definition source: `oracle/oracle/codemp/game/g_misc.c:2867-2884`

use mp_qshared::shared::qboolean;

use crate::level::reference_tag::{reference_tag_t, MAX_REFNAME};

/// Raven `TAG_GENERIC_NAME` — if a designer chooses this name, cut a finger
/// off as an example to the others.
///
/// Source: `oracle/oracle/codemp/game/g_misc.c:2868`
pub const TAG_GENERIC_NAME: &str = "__WORLD__";

/// Raven `MAX_TAGS` — each tag owner has preallocated space for tags up to
/// this many.
///
/// Source: `oracle/oracle/codemp/game/g_misc.c:2873`
pub const MAX_TAGS: usize = 256;

/// Raven `MAX_TAG_OWNERS` — 16 for now in order to not use too much VM
/// memory.
///
/// Source: `oracle/oracle/codemp/game/g_misc.c:2874`
pub const MAX_TAG_OWNERS: usize = 16;

/// Raven `tagOwner_t` (`tagOwner_s`).
///
/// Type definition source: `oracle/oracle/codemp/game/g_misc.c:2879-2884`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct tagOwner_t {
    pub name: [core::ffi::c_char; MAX_REFNAME],
    pub tags: [reference_tag_t; MAX_TAGS],
    pub inuse: qboolean,
}
