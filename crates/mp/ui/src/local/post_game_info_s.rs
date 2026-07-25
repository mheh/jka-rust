#![allow(non_camel_case_types, non_snake_case)]

/// Raven `postGameInfo_t` — the single-player post-game score record.
///
/// Stays layout-frozen through the ui idiom port: `UI_LoadBestScores`/
/// `UI_SetBestScores` read and write the struct's raw bytes to disk with
/// `trap_FS_Read(&newInfo, sizeof(postGameInfo_t), f)` and gate on
/// `size == sizeof(postGameInfo_t)`, so the layout is an on-disk file format,
/// not module-private state.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:1125-1142`
/// Source: `oracle/codemp/ui/ui_atoms.c:83-130`
#[repr(C)]
pub struct postGameInfo_t {
    pub score: i32,
    pub redScore: i32,
    pub blueScore: i32,
    pub perfects: i32,
    pub accuracy: i32,
    pub impressives: i32,
    pub excellents: i32,
    pub defends: i32,
    pub assists: i32,
    pub gauntlets: i32,
    pub captures: i32,
    pub time: i32,
    pub timeBonus: i32,
    pub shutoutBonus: i32,
    pub skillBonus: i32,
    pub baseScore: i32,
}

const _: () = assert!(core::mem::size_of::<postGameInfo_t>() == 64);
const _: () = assert!(core::mem::offset_of!(postGameInfo_t, score) == 0);
const _: () = assert!(core::mem::offset_of!(postGameInfo_t, redScore) == 4);
const _: () = assert!(core::mem::offset_of!(postGameInfo_t, blueScore) == 8);
const _: () = assert!(core::mem::offset_of!(postGameInfo_t, perfects) == 12);
const _: () = assert!(core::mem::offset_of!(postGameInfo_t, accuracy) == 16);
const _: () = assert!(core::mem::offset_of!(postGameInfo_t, impressives) == 20);
const _: () = assert!(core::mem::offset_of!(postGameInfo_t, excellents) == 24);
const _: () = assert!(core::mem::offset_of!(postGameInfo_t, defends) == 28);
const _: () = assert!(core::mem::offset_of!(postGameInfo_t, assists) == 32);
const _: () = assert!(core::mem::offset_of!(postGameInfo_t, gauntlets) == 36);
const _: () = assert!(core::mem::offset_of!(postGameInfo_t, captures) == 40);
const _: () = assert!(core::mem::offset_of!(postGameInfo_t, time) == 44);
const _: () = assert!(core::mem::offset_of!(postGameInfo_t, timeBonus) == 48);
const _: () = assert!(core::mem::offset_of!(postGameInfo_t, shutoutBonus) == 52);
const _: () = assert!(core::mem::offset_of!(postGameInfo_t, skillBonus) == 56);
const _: () = assert!(core::mem::offset_of!(postGameInfo_t, baseScore) == 60);
