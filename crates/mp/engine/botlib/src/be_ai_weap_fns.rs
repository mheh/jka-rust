#![allow(non_camel_case_types, non_snake_case, clippy::missing_safety_doc)]

//! MP botlib `be_ai_weap.cpp` — bot weapon AI: weapon-config loading, the
//! per-client weapon-state handle table, and best-weapon selection.
//!
//! Source: `oracle/codemp/botlib/be_ai_weap.cpp`
//!
//! Destination `_fns` escape: the `be_ai_weap/` directory already holds the
//! types, so `be_ai_weap.cpp`'s functions land here.

use core::ffi::{c_char, c_int, c_ulong};
use std::ffi::{CStr, CString};

use mp_engine_qcommon::common_fns::{Com_Memcpy, Com_Memset};
use mp_qshared::common::mp::botlib::botlib_error::{
    BLERR_CANNOTLOADWEAPONCONFIG, BLERR_CANNOTLOADWEAPONWEIGHTS, BLERR_NOERROR,
};
use mp_qshared::common::mp::botlib::botlib_misc::BOTFILESBASEFOLDER;
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_FATAL, PRT_MESSAGE, PRT_WARNING};
use mp_qshared::common::mp::botlib::projectileinfo_s::projectileinfo_t;
use mp_qshared::common::mp::botlib::weaponinfo_s::weaponinfo_t;
use mp_qshared::shared::limits::MAX_CLIENTS;
use mp_qshared::shared::{qfalse, qtrue};

use crate::be_ai_weap::bot_weaponstate_s::bot_weaponstate_t;
use crate::be_ai_weap::weaponconfig_s::weaponconfig_t;
use crate::be_ai_weight::weightconfig_s::weightconfig_t;
use crate::be_ai_weight_fns::{FindFuzzyWeight, FreeWeightConfig, FuzzyWeight, ReadWeightConfig};
use crate::l_libvar_fns::{LibVarSet, LibVarString, LibVarValue};
use crate::l_memory_fns::{FreeMemory, GetClearedHunkMemory, GetClearedMemory};
use crate::l_precomp_fns::{FreeSource, LoadSourceFile, PC_ReadToken, PC_SetBaseFolder};
use crate::l_script::token_s::Token;
use crate::l_struct::fielddef_s::fielddef_t;
use crate::l_struct::l_struct_consts::{FT_ARRAY, FT_FLOAT, FT_INT, FT_STRING};
use crate::l_struct::structdef_s::structdef_t;
use crate::l_struct_fns::ReadStructure;
use crate::BotLib;

/// Raven `BotValidWeaponNumber`.
///
/// Source: `oracle/codemp/botlib/be_ai_weap.cpp:120-128`
pub fn BotValidWeaponNumber(bot: &mut BotLib, weaponnum: c_int) -> c_int {
    if weaponnum <= 0 || weaponnum > unsafe { (*bot.weaponconfig).numweapons } {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"weapon number out of range\n".as_ptr() as *mut c_char,
            );
        }
        return qfalse;
    }
    qtrue
}

/// Raven `BotWeaponStateFromHandle`.
///
/// Source: `oracle/codemp/botlib/be_ai_weap.cpp:135-148`
pub fn BotWeaponStateFromHandle(bot: &mut BotLib, handle: c_int) -> *mut bot_weaponstate_t {
    if handle <= 0 || handle > MAX_CLIENTS as c_int {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"move state handle %d out of range\n".as_ptr() as *mut c_char,
                handle,
            );
        }
        return core::ptr::null_mut();
    }
    if bot.botweaponstates[handle as usize].is_null() {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"invalid move state %d\n".as_ptr() as *mut c_char,
                handle,
            );
        }
        return core::ptr::null_mut();
    }
    bot.botweaponstates[handle as usize]
}

/// Raven `LoadWeaponConfig`.
///
/// Source: `oracle/codemp/botlib/be_ai_weap.cpp:181-306`
pub fn LoadWeaponConfig(bot: &mut BotLib, filename: *mut c_char) -> *mut weaponconfig_t {
    // Raven's file-scope `static structdef_t weaponinfo_struct`/
    // `projectileinfo_struct` and their `fielddef_t[]` tables are read-only
    // const tables referenced only here (the sole non-debug caller); built as
    // function locals to keep the raw-pointer field/name tables out of a
    // non-`Sync` `static`.
    // Source: `oracle/codemp/botlib/be_ai_weap.cpp:34-93`
    let fd = |name: &core::ffi::CStr, offset: usize, ty: c_int, maxarray: c_int| fielddef_t {
        name: name.as_ptr() as *mut c_char,
        offset: offset as i32,
        r#type: ty,
        maxarray,
        floatmin: 0.0,
        floatmax: 0.0,
        substruct: core::ptr::null_mut(),
    };
    let fd_end = || fielddef_t {
        name: core::ptr::null_mut(),
        offset: 0,
        r#type: 0,
        maxarray: 0,
        floatmin: 0.0,
        floatmax: 0.0,
        substruct: core::ptr::null_mut(),
    };
    let mut weaponinfo_fields: [fielddef_t; 23] = [
        fd(
            c"number",
            core::mem::offset_of!(weaponinfo_t, number),
            FT_INT,
            0,
        ),
        fd(
            c"name",
            core::mem::offset_of!(weaponinfo_t, name),
            FT_STRING,
            0,
        ),
        fd(
            c"level",
            core::mem::offset_of!(weaponinfo_t, level),
            FT_INT,
            0,
        ),
        fd(
            c"model",
            core::mem::offset_of!(weaponinfo_t, model),
            FT_STRING,
            0,
        ),
        fd(
            c"weaponindex",
            core::mem::offset_of!(weaponinfo_t, weaponindex),
            FT_INT,
            0,
        ),
        fd(
            c"flags",
            core::mem::offset_of!(weaponinfo_t, flags),
            FT_INT,
            0,
        ),
        fd(
            c"projectile",
            core::mem::offset_of!(weaponinfo_t, projectile),
            FT_STRING,
            0,
        ),
        fd(
            c"numprojectiles",
            core::mem::offset_of!(weaponinfo_t, numprojectiles),
            FT_INT,
            0,
        ),
        fd(
            c"hspread",
            core::mem::offset_of!(weaponinfo_t, hspread),
            FT_FLOAT,
            0,
        ),
        fd(
            c"vspread",
            core::mem::offset_of!(weaponinfo_t, vspread),
            FT_FLOAT,
            0,
        ),
        fd(
            c"speed",
            core::mem::offset_of!(weaponinfo_t, speed),
            FT_FLOAT,
            0,
        ),
        fd(
            c"acceleration",
            core::mem::offset_of!(weaponinfo_t, acceleration),
            FT_FLOAT,
            0,
        ),
        fd(
            c"recoil",
            core::mem::offset_of!(weaponinfo_t, recoil),
            FT_FLOAT | FT_ARRAY,
            3,
        ),
        fd(
            c"offset",
            core::mem::offset_of!(weaponinfo_t, offset),
            FT_FLOAT | FT_ARRAY,
            3,
        ),
        fd(
            c"angleoffset",
            core::mem::offset_of!(weaponinfo_t, angleoffset),
            FT_FLOAT | FT_ARRAY,
            3,
        ),
        fd(
            c"extrazvelocity",
            core::mem::offset_of!(weaponinfo_t, extrazvelocity),
            FT_FLOAT,
            0,
        ),
        fd(
            c"ammoamount",
            core::mem::offset_of!(weaponinfo_t, ammoamount),
            FT_INT,
            0,
        ),
        fd(
            c"ammoindex",
            core::mem::offset_of!(weaponinfo_t, ammoindex),
            FT_INT,
            0,
        ),
        fd(
            c"activate",
            core::mem::offset_of!(weaponinfo_t, activate),
            FT_FLOAT,
            0,
        ),
        fd(
            c"reload",
            core::mem::offset_of!(weaponinfo_t, reload),
            FT_FLOAT,
            0,
        ),
        fd(
            c"spinup",
            core::mem::offset_of!(weaponinfo_t, spinup),
            FT_FLOAT,
            0,
        ),
        fd(
            c"spindown",
            core::mem::offset_of!(weaponinfo_t, spindown),
            FT_FLOAT,
            0,
        ),
        fd_end(),
    ];
    let mut projectileinfo_fields: [fielddef_t; 15] = [
        fd(
            c"name",
            core::mem::offset_of!(projectileinfo_t, name),
            FT_STRING,
            0,
        ),
        // Raven quirk: this row uses `WEAPON_OFS(model)`, not `PROJECTILE_OFS`
        // — preserved (be_ai_weap.cpp:69).
        fd(
            c"model",
            core::mem::offset_of!(weaponinfo_t, model),
            FT_STRING,
            0,
        ),
        fd(
            c"flags",
            core::mem::offset_of!(projectileinfo_t, flags),
            FT_INT,
            0,
        ),
        fd(
            c"gravity",
            core::mem::offset_of!(projectileinfo_t, gravity),
            FT_FLOAT,
            0,
        ),
        fd(
            c"damage",
            core::mem::offset_of!(projectileinfo_t, damage),
            FT_INT,
            0,
        ),
        fd(
            c"radius",
            core::mem::offset_of!(projectileinfo_t, radius),
            FT_FLOAT,
            0,
        ),
        fd(
            c"visdamage",
            core::mem::offset_of!(projectileinfo_t, visdamage),
            FT_INT,
            0,
        ),
        fd(
            c"damagetype",
            core::mem::offset_of!(projectileinfo_t, damagetype),
            FT_INT,
            0,
        ),
        fd(
            c"healthinc",
            core::mem::offset_of!(projectileinfo_t, healthinc),
            FT_INT,
            0,
        ),
        fd(
            c"push",
            core::mem::offset_of!(projectileinfo_t, push),
            FT_FLOAT,
            0,
        ),
        fd(
            c"detonation",
            core::mem::offset_of!(projectileinfo_t, detonation),
            FT_FLOAT,
            0,
        ),
        fd(
            c"bounce",
            core::mem::offset_of!(projectileinfo_t, bounce),
            FT_FLOAT,
            0,
        ),
        fd(
            c"bouncefric",
            core::mem::offset_of!(projectileinfo_t, bouncefric),
            FT_FLOAT,
            0,
        ),
        fd(
            c"bouncestop",
            core::mem::offset_of!(projectileinfo_t, bouncestop),
            FT_FLOAT,
            0,
        ),
        fd_end(),
    ];
    let mut weaponinfo_struct = structdef_t {
        size: core::mem::size_of::<weaponinfo_t>() as i32,
        fields: weaponinfo_fields.as_mut_ptr(),
    };
    let mut projectileinfo_struct = structdef_t {
        size: core::mem::size_of::<projectileinfo_t>() as i32,
        fields: projectileinfo_fields.as_mut_ptr(),
    };

    unsafe {
        let mut max_weaponinfo = LibVarValue(bot, "max_weaponinfo", "32") as c_int;
        if max_weaponinfo < 0 {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"max_weaponinfo = %d\n".as_ptr() as *mut c_char,
                max_weaponinfo,
            );
            max_weaponinfo = 32;
            LibVarSet(bot, "max_weaponinfo", "32");
        }
        let mut max_projectileinfo = LibVarValue(bot, "max_projectileinfo", "32") as c_int;
        if max_projectileinfo < 0 {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"max_projectileinfo = %d\n".as_ptr() as *mut c_char,
                max_projectileinfo,
            );
            max_projectileinfo = 32;
            LibVarSet(bot, "max_projectileinfo", "32");
        }
        // §19: `path` is a fixed local buffer Raven writes via strncpy before
        // any read; zero-init to avoid reading uninitialized bytes.
        const MAX_PATH: usize = 260;
        let mut path = [0 as c_char; MAX_PATH];
        libc::strncpy(path.as_mut_ptr(), filename, MAX_PATH);
        PC_SetBaseFolder(bot, BOTFILESBASEFOLDER);
        let path_str = CStr::from_ptr(path.as_ptr()).to_string_lossy().into_owned();
        let mut source = match LoadSourceFile(bot, &path_str) {
            Some(s) => s,
            None => {
                bot.botimport.Print.unwrap()(
                    PRT_ERROR,
                    c"counldn't load %s\n".as_ptr() as *mut c_char,
                    path.as_ptr(),
                );
                return core::ptr::null_mut();
            }
        };
        //initialize weapon config
        let wc = GetClearedHunkMemory(
            bot,
            (core::mem::size_of::<weaponconfig_t>()
                + max_weaponinfo as usize * core::mem::size_of::<weaponinfo_t>()
                + max_projectileinfo as usize * core::mem::size_of::<projectileinfo_t>())
                as c_ulong,
        ) as *mut weaponconfig_t;
        (*wc).weaponinfo =
            (wc as *mut u8).add(core::mem::size_of::<weaponconfig_t>()) as *mut weaponinfo_t;
        (*wc).projectileinfo = ((*wc).weaponinfo as *mut u8)
            .add(max_weaponinfo as usize * core::mem::size_of::<weaponinfo_t>())
            as *mut projectileinfo_t;
        (*wc).numweapons = max_weaponinfo;
        (*wc).numprojectiles = 0;
        //parse the source file
        let mut token = Token::default();
        while PC_ReadToken(bot, &mut source, &mut token) != 0 {
            if token.string == "weaponinfo" {
                let mut weaponinfo: weaponinfo_t = core::mem::zeroed();
                Com_Memset(
                    &mut weaponinfo as *mut _ as *mut (),
                    0,
                    core::mem::size_of::<weaponinfo_t>(),
                );
                if ReadStructure(
                    bot,
                    &mut source,
                    &mut weaponinfo_struct,
                    &mut weaponinfo as *mut _ as *mut c_char,
                ) == 0
                {
                    FreeMemory(bot, wc as *mut ());
                    FreeSource(source);
                    return core::ptr::null_mut();
                }
                if weaponinfo.number < 0 || weaponinfo.number >= max_weaponinfo {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        c"weapon info number %d out of range in %s\n".as_ptr() as *mut c_char,
                        weaponinfo.number,
                        path.as_ptr(),
                    );
                    FreeMemory(bot, wc as *mut ());
                    FreeSource(source);
                    return core::ptr::null_mut();
                }
                Com_Memcpy(
                    (*wc).weaponinfo.add(weaponinfo.number as usize) as *mut (),
                    &weaponinfo as *const _ as *const (),
                    core::mem::size_of::<weaponinfo_t>(),
                );
                (*(*wc).weaponinfo.add(weaponinfo.number as usize)).valid = qtrue;
            } else if token.string == "projectileinfo" {
                if (*wc).numprojectiles >= max_projectileinfo {
                    bot.botimport.Print.unwrap()(
                        PRT_ERROR,
                        c"more than %d projectiles defined in %s\n".as_ptr() as *mut c_char,
                        max_projectileinfo,
                        path.as_ptr(),
                    );
                    FreeMemory(bot, wc as *mut ());
                    FreeSource(source);
                    return core::ptr::null_mut();
                }
                Com_Memset(
                    (*wc).projectileinfo.add((*wc).numprojectiles as usize) as *mut (),
                    0,
                    core::mem::size_of::<projectileinfo_t>(),
                );
                if ReadStructure(
                    bot,
                    &mut source,
                    &mut projectileinfo_struct,
                    (*wc).projectileinfo.add((*wc).numprojectiles as usize) as *mut c_char,
                ) == 0
                {
                    FreeMemory(bot, wc as *mut ());
                    FreeSource(source);
                    return core::ptr::null_mut();
                }
                (*wc).numprojectiles += 1;
            } else {
                let token_string_c = CString::new(token.string.as_str()).unwrap_or_default();
                bot.botimport.Print.unwrap()(
                    PRT_ERROR,
                    c"unknown definition %s in %s\n".as_ptr() as *mut c_char,
                    token_string_c.as_ptr(),
                    path.as_ptr(),
                );
                FreeMemory(bot, wc as *mut ());
                FreeSource(source);
                return core::ptr::null_mut();
            }
        }
        FreeSource(source);
        //fix up weapons
        for i in 0..(*wc).numweapons {
            let wi = (*wc).weaponinfo.add(i as usize);
            if (*wi).valid == 0 {
                continue;
            }
            if (*wi).name[0] == 0 {
                bot.botimport.Print.unwrap()(
                    PRT_ERROR,
                    c"weapon %d has no name in %s\n".as_ptr() as *mut c_char,
                    i,
                    path.as_ptr(),
                );
                FreeMemory(bot, wc as *mut ());
                return core::ptr::null_mut();
            }
            if (*wi).projectile[0] == 0 {
                bot.botimport.Print.unwrap()(
                    PRT_ERROR,
                    c"weapon %s has no projectile in %s\n".as_ptr() as *mut c_char,
                    (*wi).name.as_ptr(),
                    path.as_ptr(),
                );
                FreeMemory(bot, wc as *mut ());
                return core::ptr::null_mut();
            }
            //find the projectile info and copy it to the weapon info
            let mut j = 0;
            while j < (*wc).numprojectiles {
                let pj = (*wc).projectileinfo.add(j as usize);
                if libc::strcmp((*pj).name.as_ptr(), (*wi).projectile.as_ptr()) == 0 {
                    Com_Memcpy(
                        &mut (*wi).proj as *mut _ as *mut (),
                        pj as *const _ as *const (),
                        core::mem::size_of::<projectileinfo_t>(),
                    );
                    break;
                }
                j += 1;
            }
            if j == (*wc).numprojectiles {
                bot.botimport.Print.unwrap()(
                    PRT_ERROR,
                    c"weapon %s uses undefined projectile in %s\n".as_ptr() as *mut c_char,
                    (*wi).name.as_ptr(),
                    path.as_ptr(),
                );
                FreeMemory(bot, wc as *mut ());
                return core::ptr::null_mut();
            }
        }
        if (*wc).numweapons == 0 {
            bot.botimport.Print.unwrap()(
                PRT_WARNING,
                c"no weapon info loaded\n".as_ptr() as *mut c_char,
            );
        }
        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"loaded %s\n".as_ptr() as *mut c_char,
            path.as_ptr(),
        );
        wc
    }
}

/// Raven `WeaponWeightIndex`.
///
/// Source: `oracle/codemp/botlib/be_ai_weap.cpp:313-325`
pub fn WeaponWeightIndex(
    bot: &mut BotLib,
    wwc: *mut weightconfig_t,
    wc: *mut weaponconfig_t,
) -> *mut c_int {
    unsafe {
        //initialize item weight index
        let index = GetClearedMemory(
            bot,
            (core::mem::size_of::<c_int>() * (*wc).numweapons as usize) as c_ulong,
        ) as *mut c_int;

        for i in 0..(*wc).numweapons {
            *index.add(i as usize) =
                FindFuzzyWeight(wwc, (*(*wc).weaponinfo.add(i as usize)).name.as_mut_ptr());
        }
        index
    }
}

/// Raven `BotFreeWeaponWeights`.
///
/// Source: `oracle/codemp/botlib/be_ai_weap.cpp:332-340`
pub fn BotFreeWeaponWeights(bot: &mut BotLib, weaponstate: c_int) {
    let ws = BotWeaponStateFromHandle(bot, weaponstate);
    if ws.is_null() {
        return;
    }
    unsafe {
        if !(*ws).weaponweightconfig.is_null() {
            FreeWeightConfig(bot, (*ws).weaponweightconfig);
        }
        if !(*ws).weaponweightindex.is_null() {
            FreeMemory(bot, (*ws).weaponweightindex as *mut ());
        }
    }
}

/// Raven `BotLoadWeaponWeights`.
///
/// Source: `oracle/codemp/botlib/be_ai_weap.cpp:347-364`
pub fn BotLoadWeaponWeights(bot: &mut BotLib, weaponstate: c_int, filename: *mut c_char) -> c_int {
    let ws = BotWeaponStateFromHandle(bot, weaponstate);
    if ws.is_null() {
        return BLERR_CANNOTLOADWEAPONWEIGHTS;
    }
    BotFreeWeaponWeights(bot, weaponstate);
    unsafe {
        (*ws).weaponweightconfig = ReadWeightConfig(bot, filename);
        if (*ws).weaponweightconfig.is_null() {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"couldn't load weapon config %s\n".as_ptr() as *mut c_char,
                filename,
            );
            return BLERR_CANNOTLOADWEAPONWEIGHTS;
        }
        if bot.weaponconfig.is_null() {
            return BLERR_CANNOTLOADWEAPONCONFIG;
        }
        (*ws).weaponweightindex =
            WeaponWeightIndex(bot, (*ws).weaponweightconfig, bot.weaponconfig);
    }
    BLERR_NOERROR
}

/// Raven `BotGetWeaponInfo`.
///
/// Source: `oracle/codemp/botlib/be_ai_weap.cpp:371-380`
pub fn BotGetWeaponInfo(
    bot: &mut BotLib,
    weaponstate: c_int,
    weapon: c_int,
    weaponinfo: *mut weaponinfo_t,
) {
    if BotValidWeaponNumber(bot, weapon) == 0 {
        return;
    }
    let ws = BotWeaponStateFromHandle(bot, weaponstate);
    if ws.is_null() {
        return;
    }
    if bot.weaponconfig.is_null() {
        return;
    }
    unsafe {
        Com_Memcpy(
            weaponinfo as *mut (),
            (*bot.weaponconfig).weaponinfo.add(weapon as usize) as *const (),
            core::mem::size_of::<weaponinfo_t>(),
        );
    }
}

/// Raven `BotChooseBestFightWeapon`.
///
/// Source: `oracle/codemp/botlib/be_ai_weap.cpp:387-417`
pub fn BotChooseBestFightWeapon(
    bot: &mut BotLib,
    weaponstate: c_int,
    inventory: *mut c_int,
) -> c_int {
    let ws = BotWeaponStateFromHandle(bot, weaponstate);
    if ws.is_null() {
        return 0;
    }
    let wc = bot.weaponconfig;
    if bot.weaponconfig.is_null() {
        return 0;
    }

    //if the bot has no weapon weight configuration
    unsafe {
        if (*ws).weaponweightconfig.is_null() {
            return 0;
        }

        let mut bestweight: f32 = 0.0;
        let mut bestweapon: c_int = 0;
        for i in 0..(*wc).numweapons {
            if (*(*wc).weaponinfo.add(i as usize)).valid == 0 {
                continue;
            }
            let index = *(*ws).weaponweightindex.add(i as usize);
            if index < 0 {
                continue;
            }
            let weight = FuzzyWeight(inventory, (*ws).weaponweightconfig, index);
            if weight > bestweight {
                bestweight = weight;
                bestweapon = i;
            }
        }
        bestweapon
    }
}

/// Raven `BotResetWeaponState`.
///
/// Source: `oracle/codemp/botlib/be_ai_weap.cpp:424-438`
pub fn BotResetWeaponState(bot: &mut BotLib, weaponstate: c_int) {
    let ws = BotWeaponStateFromHandle(bot, weaponstate);
    if ws.is_null() {
        return;
    }
    unsafe {
        let weaponweightconfig = (*ws).weaponweightconfig;
        let weaponweightindex = (*ws).weaponweightindex;

        //Com_Memset(ws, 0, sizeof(bot_weaponstate_t));
        (*ws).weaponweightconfig = weaponweightconfig;
        (*ws).weaponweightindex = weaponweightindex;
    }
}

/// Raven `BotAllocWeaponState`.
///
/// Source: `oracle/codemp/botlib/be_ai_weap.cpp:445-458`
pub fn BotAllocWeaponState(bot: &mut BotLib) -> c_int {
    for i in 1..=MAX_CLIENTS {
        if bot.botweaponstates[i].is_null() {
            bot.botweaponstates[i] =
                GetClearedMemory(bot, core::mem::size_of::<bot_weaponstate_t>() as c_ulong)
                    as *mut bot_weaponstate_t;
            return i as c_int;
        }
    }
    0
}

/// Raven `BotFreeWeaponState`.
///
/// Source: `oracle/codemp/botlib/be_ai_weap.cpp:465-480`
pub fn BotFreeWeaponState(bot: &mut BotLib, handle: c_int) {
    if handle <= 0 || handle > MAX_CLIENTS as c_int {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"move state handle %d out of range\n".as_ptr() as *mut c_char,
                handle,
            );
        }
        return;
    }
    if bot.botweaponstates[handle as usize].is_null() {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"invalid move state %d\n".as_ptr() as *mut c_char,
                handle,
            );
        }
        return;
    }
    BotFreeWeaponWeights(bot, handle);
    FreeMemory(bot, bot.botweaponstates[handle as usize] as *mut ());
    bot.botweaponstates[handle as usize] = core::ptr::null_mut();
}

/// Raven `BotSetupWeaponAI`.
///
/// Source: `oracle/codemp/botlib/be_ai_weap.cpp:487-504`
pub fn BotSetupWeaponAI(bot: &mut BotLib) -> c_int {
    let file = std::ffi::CString::new(LibVarString(bot, "weaponconfig", "weapons.c")).unwrap();
    bot.weaponconfig = LoadWeaponConfig(bot, file.as_ptr() as *mut c_char);
    if bot.weaponconfig.is_null() {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"couldn't load the weapon config\n".as_ptr() as *mut c_char,
            );
        }
        return BLERR_CANNOTLOADWEAPONCONFIG;
    }

    BLERR_NOERROR
}

/// Raven `BotShutdownWeaponAI`.
///
/// Source: `oracle/codemp/botlib/be_ai_weap.cpp:511-525`
pub fn BotShutdownWeaponAI(bot: &mut BotLib) {
    if !bot.weaponconfig.is_null() {
        FreeMemory(bot, bot.weaponconfig as *mut ());
    }
    bot.weaponconfig = core::ptr::null_mut();

    for i in 1..=MAX_CLIENTS {
        if !bot.botweaponstates[i].is_null() {
            BotFreeWeaponState(bot, i as c_int);
        }
    }
}
