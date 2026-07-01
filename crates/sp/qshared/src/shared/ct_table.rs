#![allow(non_camel_case_types)]

/// Raven `ct_table_t` — color-table indices.
///
/// SP-vs-MP: SP adds `CT_TITLE` immediately before `CT_MAX`; MP has no `CT_TITLE`.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:355-440`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ct_table_t {
    CT_NONE,
    CT_BLACK,
    CT_RED,
    CT_GREEN,
    CT_BLUE,
    CT_YELLOW,
    CT_MAGENTA,
    CT_CYAN,
    CT_WHITE,
    CT_LTGREY,
    CT_MDGREY,
    CT_DKGREY,
    CT_DKGREY2,

    CT_VLTORANGE,
    CT_LTORANGE,
    CT_DKORANGE,
    CT_VDKORANGE,

    CT_VLTBLUE1,
    CT_LTBLUE1,
    CT_DKBLUE1,
    CT_VDKBLUE1,

    CT_VLTBLUE2,
    CT_LTBLUE2,
    CT_DKBLUE2,
    CT_VDKBLUE2,

    CT_VLTBROWN1,
    CT_LTBROWN1,
    CT_DKBROWN1,
    CT_VDKBROWN1,

    CT_VLTGOLD1,
    CT_LTGOLD1,
    CT_DKGOLD1,
    CT_VDKGOLD1,

    CT_VLTPURPLE1,
    CT_LTPURPLE1,
    CT_DKPURPLE1,
    CT_VDKPURPLE1,

    CT_VLTPURPLE2,
    CT_LTPURPLE2,
    CT_DKPURPLE2,
    CT_VDKPURPLE2,

    CT_VLTPURPLE3,
    CT_LTPURPLE3,
    CT_DKPURPLE3,
    CT_VDKPURPLE3,

    CT_VLTRED1,
    CT_LTRED1,
    CT_DKRED1,
    CT_VDKRED1,
    CT_VDKRED,
    CT_DKRED,

    CT_VLTAQUA,
    CT_LTAQUA,
    CT_DKAQUA,
    CT_VDKAQUA,

    CT_LTPINK,
    CT_DKPINK,
    CT_LTCYAN,
    CT_DKCYAN,
    CT_LTBLUE3,
    CT_BLUE3,
    CT_DKBLUE3,

    CT_HUD_GREEN,
    CT_HUD_RED,
    CT_ICON_BLUE,
    CT_NO_AMMO_RED,
    CT_HUD_ORANGE,

    CT_TITLE,

    CT_MAX,
}
