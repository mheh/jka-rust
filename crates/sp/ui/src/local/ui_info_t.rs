#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use sp_qshared::shared::{qboolean, qhandle_t, NUM_FORCE_POWERS};
use sp_uishared::shared::display_context_def_t::displayContextDef_t;
use sp_uishared::shared::item_def_s::itemDef_t;

use super::mod_info_t::modInfo_t;
use super::player_species_info_t::playerSpeciesInfo_t;

/// `MAX_MODS`.
///
/// Source: `oracle/code/ui/ui_local.h:96`
const MAX_MODS: usize = 64;

/// `MAX_PLAYERMODELS`.
///
/// Source: `oracle/code/ui/ui_local.h:13`
const MAX_PLAYERMODELS: usize = 32;

/// `MAX_DEFERRED_SCRIPT`.
///
/// Source: `oracle/code/ui/ui_local.h:14`
const MAX_DEFERRED_SCRIPT: usize = 1024;

/// Raven `uiInfo_t` — the SP UI module's top-level runtime state (display
/// context, mod list, player-species list, and the Force-power / weapon
/// selection screens' scratch state).
///
/// SP diverges from MP: SP's `uiInfo_t` is a small, single-purpose struct
/// (no team/gametype/map/server-browser caches) built around the SP-only
/// Force-power allocation and weapon-selection screens; it lacks all of
/// MP's multiplayer browser/lobby state.
/// Type definition source: `oracle/code/ui/ui_local.h:119-170`
#[repr(C)]
pub struct uiInfo_t {
    pub uiDC: displayContextDef_t,

    pub effectsColor: i32,
    pub currentCrosshair: i32,

    pub modList: [modInfo_t; MAX_MODS],
    pub modIndex: i32,
    pub modCount: i32,

    pub playerSpeciesCount: i32,
    pub playerSpecies: [playerSpeciesInfo_t; MAX_PLAYERMODELS],
    pub playerSpeciesIndex: i32,

    pub deferredScript: [c_char; MAX_DEFERRED_SCRIPT],
    pub deferredScriptItem: *mut itemDef_t,

    pub runScriptItem: *mut itemDef_t,

    pub inGameLoad: qboolean,
    // Used by Force Power allocation screen
    /// Enum of which power had the point allocated
    pub forcePowerUpdated: i16,
    // Used by Weapon allocation screen
    /// 1st weapon chosen
    pub selectedWeapon1: i16,
    /// Item name of weapon chosen
    pub selectedWeapon1ItemName: [c_char; 64],
    /// Holds index to ammo
    pub selectedWeapon1AmmoIndex: i32,
    /// 2nd weapon chosen
    pub selectedWeapon2: i16,
    /// Item name of weapon chosen
    pub selectedWeapon2ItemName: [c_char; 64],
    /// Holds index to ammo
    pub selectedWeapon2AmmoIndex: i32,
    /// throwable weapon chosen
    pub selectedThrowWeapon: i16,
    /// Item name of weapon chosen
    pub selectedThrowWeaponItemName: [c_char; 64],
    /// Holds index to ammo
    pub selectedThrowWeaponAmmoIndex: i32,

    pub weapon1ItemButton: *mut itemDef_t,
    pub litWeapon1Icon: qhandle_t,
    pub unlitWeapon1Icon: qhandle_t,
    pub weapon2ItemButton: *mut itemDef_t,
    pub litWeapon2Icon: qhandle_t,
    pub unlitWeapon2Icon: qhandle_t,

    pub weaponThrowButton: *mut itemDef_t,
    pub litThrowableIcon: qhandle_t,
    pub unlitThrowableIcon: qhandle_t,
    pub movesTitleIndex: i16,
    pub movesBaseAnim: *mut c_char,
    pub moveAnimTime: i32,
    pub languageCount: i32,
    pub languageCountIndex: i32,

    pub forcePowerLevel: [i32; NUM_FORCE_POWERS as usize],
}

const _: () = assert!(core::mem::size_of::<uiInfo_t>() == 251568);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, uiDC) == 0);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, effectsColor) == 792);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, currentCrosshair) == 796);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, modList) == 800);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, modIndex) == 1824);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, modCount) == 1828);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, playerSpeciesCount) == 1832);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, playerSpecies) == 1836);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, playerSpeciesIndex) == 250156);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, deferredScript) == 250160);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, deferredScriptItem) == 251184);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, runScriptItem) == 251192);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, inGameLoad) == 251200);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, forcePowerUpdated) == 251204);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, selectedWeapon1) == 251206);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, selectedWeapon1ItemName) == 251208);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, selectedWeapon1AmmoIndex) == 251272);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, selectedWeapon2) == 251276);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, selectedWeapon2ItemName) == 251278);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, selectedWeapon2AmmoIndex) == 251344);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, selectedThrowWeapon) == 251348);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, selectedThrowWeaponItemName) == 251350);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, selectedThrowWeaponAmmoIndex) == 251416);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, weapon1ItemButton) == 251424);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, litWeapon1Icon) == 251432);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, unlitWeapon1Icon) == 251436);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, weapon2ItemButton) == 251440);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, litWeapon2Icon) == 251448);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, unlitWeapon2Icon) == 251452);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, weaponThrowButton) == 251456);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, litThrowableIcon) == 251464);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, unlitThrowableIcon) == 251468);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, movesTitleIndex) == 251472);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, movesBaseAnim) == 251480);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, moveAnimTime) == 251488);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, languageCount) == 251492);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, languageCountIndex) == 251496);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, forcePowerLevel) == 251500);
