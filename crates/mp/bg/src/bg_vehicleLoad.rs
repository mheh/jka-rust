// PORT-COMPLETE: bg_vehicleLoad.c 12/12 (pass 3)
//! Port of `oracle/codemp/game/bg_vehicleLoad.c` — the `.veh`/`.vwp`
//! vehicle/vehicle-weapon text-format loader.
//!
//! bg-tier fns that touch mutable `BgState` route through `bg: &mut BgState`
//! (+ `traps: &dyn BgTraps` for engine calls) per rulings 12/15; this is
//! jampgame (QAGAME, `_JK2MP`) only, so every `#elif CGAME`/`#else` SP/cgame/ui
//! arm is dropped (porting-rules §20).
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::vehicles::vehicle_s::VEH_MAX_PASSENGERS;
use mp_qshared::shared::com_parse::{
    COM_BeginParseSession, COM_ParseExt, SkipBracedSection, SkipRestOfLine,
};
use native_string::atof::atof;
use native_string::atoi::atoi;

/// Raven `BG_ClearVehicleParseParms`.
///
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:79-84`
pub fn BG_ClearVehicleParseParms(bg: &mut BgState) {
    bg.VehWeaponParms[0] = 0;
    bg.VehicleParms[0] = 0;
}

/// Raven `BG_ParseVehWeaponParm`.
///
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:167-307`
// This is jampgame (QAGAME) only: the `#elif CGAME`/`#else` arms of every
// `#ifdef QAGAME` field type are dead here and dropped (porting-rules §20).
pub fn BG_ParseVehWeaponParm(
    vehWeapon: *mut vehWeaponInfo_t,
    parmName: *mut c_char,
    pValue: *mut c_char,
    // Threaded so the MP (`_JK2MP`) `VF_LSTRING` branch can bump the bg pool
    // (`BG_Alloc`); the game pool is unreachable from bg-tier code.
    bg: &mut BgState,
    // Engine `Com_Printf` print seam, routed through the trap channel.
    traps: &dyn BgTraps,
    // Routes the `VF_MODEL`/`VF_EFFECT`/`VF_SOUND` field types to the game-tier
    // `G_ModelIndex`/`G_EffectIndex`/`G_SoundIndex` via the bg→game upcall seam.
    callbacks: &mut dyn GameCallbacks,
) -> qboolean {
    unsafe {
        let value = cstr_to_str(pValue);
        let b = vehWeapon as *mut u8;
        let mut i = 0usize;
        while i < NUM_VWEAP_PARMS {
            let field = &vehWeaponFields[i];
            if !field.name.is_null() && Q_stricmp(field.name, parmName) == 0 {
                match field.r#type {
                    VF_INT => {
                        *(b.add(field.ofs as usize) as *mut c_int) = atoi(&value);
                    }
                    VF_FLOAT => {
                        *(b.add(field.ofs as usize) as *mut f32) = atof(&value) as f32;
                    }
                    VF_LSTRING => {
                        let slot = b.add(field.ofs as usize) as *mut *mut c_char;
                        if (*slot).is_null() {
                            // Raven `_JK2MP` (MP) branch: a fixed 1024-byte bg-pool block
                            // (overwritable later), NOT SP's `G_NewString` — bg-tier code has
                            // no game pool. Source: oracle/codemp/game/bg_vehicleLoad.c:195-197
                            let dest = crate::bg_misc::BG_Alloc(1024, bg) as *mut c_char;
                            Q_strncpyz(dest, cstr(&value).as_ptr(), 1024);
                            *slot = dest;
                        }
                    }
                    VF_VECTOR => {
                        let mut vec: vec3_t = [0.0, 0.0, 0.0];
                        // Raven: `_iFieldsRead = sscanf(value, "%f %f %f", &vec[0],
                        // &vec[1], &vec[2]); assert(_iFieldsRead==3); if
                        // (_iFieldsRead!=3) Com_Printf(...)` — reproduced via the
                        // shared libc-`%f` scanner's returned count.
                        let n = sscanf_f32s(&value, &mut vec);
                        if n != 3 {
                            traps.com_printf(
                                cstr(&format!(
                                    "{}BG_ParseVehWeaponParm: VEC3 sscanf() failed to read 3 floats ('angle' key bug?)\n",
                                    S_COLOR_YELLOW.to_str().unwrap()
                                ))
                                .as_ptr(),
                            );
                        }
                        let dst = b.add(field.ofs as usize) as *mut f32;
                        *dst.add(0) = vec[0];
                        *dst.add(1) = vec[1];
                        *dst.add(2) = vec[2];
                    }
                    VF_BOOL => {
                        *(b.add(field.ofs as usize) as *mut qboolean) =
                            if atof(&value) != 0.0 { qtrue } else { qfalse };
                    }
                    VF_VEHTYPE => {
                        let vt = GetIDForString(
                            VehicleTable.as_ptr() as *mut stringID_table_t,
                            cstr(&value).as_ptr(),
                        );
                        *(b.add(field.ofs as usize) as *mut vehicleType_t) =
                            core::mem::transmute(vt);
                    }
                    VF_ANIM => {
                        let anim = GetIDForString(
                            animTable.as_ptr() as *mut stringID_table_t,
                            cstr(&value).as_ptr(),
                        );
                        *(b.add(field.ofs as usize) as *mut c_int) = anim;
                    }
                    VF_WEAPON => {
                        // Raven: commented-out (`//*(int *)... = VEH_VehWeaponIndexForName(...)`).
                    }
                    VF_MODEL | VF_MODEL_CLIENT => {
                        *(b.add(field.ofs as usize) as *mut c_int) =
                            callbacks.model_index(cstr(&value).as_ptr());
                    }
                    VF_EFFECT | VF_EFFECT_CLIENT => {
                        *(b.add(field.ofs as usize) as *mut c_int) =
                            callbacks.effect_index(cstr(&value).as_ptr());
                    }
                    VF_SHADER | VF_SHADER_NOMIP => {
                        // QAGAME: neither `WE_ARE_IN_THE_UI` nor `CGAME`; dead here.
                    }
                    VF_SOUND => {
                        *(b.add(field.ofs as usize) as *mut c_int) =
                            callbacks.sound_index(cstr(&value).as_ptr());
                    }
                    // `VF_SOUND_CLIENT` (MP cgame only): the `#elif QAGAME` branch
                    // comments out the `G_SoundIndex` — the game module leaves the
                    // field untouched (cgame registers it via `trap_S_RegisterSound`).
                    // Source: `oracle/codemp/game/bg_vehicleLoad.c:282-290`
                    VF_SOUND_CLIENT => {}
                    _ => return qfalse,
                }
                break;
            }
            i += 1;
        }
        if i == NUM_VWEAP_PARMS {
            qfalse
        } else {
            qtrue
        }
    }
}

/// Raven `VEH_LoadVehWeapon`.
///
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:309-411`
pub fn VEH_LoadVehWeapon(
    vehWeaponName: *const c_char,
    bg: &mut BgState,
    // Engine `Com_Printf` print seam; also threaded to `BG_ParseVehWeaponParm`.
    traps: &dyn BgTraps,
    // Threaded through to `BG_ParseVehWeaponParm` for the model/effect/sound
    // field-index upcalls.
    callbacks: &mut dyn GameCallbacks,
) -> c_int {
    unsafe {
        // `p` walks the `VehWeaponParms` text buffer via `COM_ParseExt`'s
        // `*const *const c_char` cursor idiom.
        let mut p: *const c_char = bg.VehWeaponParms.as_ptr() as *const c_char;
        COM_BeginParseSession(&mut bg.qs, cstr("vehWeapons").as_ptr());

        let veh_index = bg.numVehicleWeapons as usize;
        let vehWeapon: *mut vehWeaponInfo_t = &mut bg.g_vehWeaponInfo[veh_index];

        loop {
            if p.is_null() {
                return VEH_WEAPON_NONE;
            }
            let token = COM_ParseExt(&mut bg.qs, &mut p as *mut *const c_char, qtrue);
            if *token == 0 {
                return qfalse as c_int;
            }
            if Q_stricmp(token, vehWeaponName) == 0 {
                break;
            }
            SkipBracedSection(&mut bg.qs, &mut p as *mut *const c_char);
        }
        if p.is_null() {
            return VEH_WEAPON_NONE;
        }

        let token = COM_ParseExt(&mut bg.qs, &mut p as *mut *const c_char, qtrue);
        if *token == 0 {
            return VEH_WEAPON_NONE;
        }
        if Q_stricmp(token, cstr("{").as_ptr()) != 0 {
            return VEH_WEAPON_NONE;
        }

        loop {
            SkipRestOfLine(&mut bg.qs, &mut p as *mut *const c_char);
            let token = COM_ParseExt(&mut bg.qs, &mut p as *mut *const c_char, qtrue);
            if *token == 0 {
                let name = cstr_to_str(vehWeaponName);
                traps.com_printf(
                    cstr(&format!(
                        "{}ERROR: unexpected EOF while parsing Vehicle Weapon '{}'\n",
                        S_COLOR_RED.to_str().unwrap(),
                        name
                    ))
                    .as_ptr(),
                );
                return VEH_WEAPON_NONE;
            }
            if Q_stricmp(token, cstr("}").as_ptr()) == 0 {
                break;
            }
            let mut parmName: [c_char; 128] = [0; 128];
            Q_strncpyz(parmName.as_mut_ptr(), token, 128);
            let value = COM_ParseExt(&mut bg.qs, &mut p as *mut *const c_char, qtrue);
            if value.is_null() || *value == 0 {
                let pn = cstr_to_str(parmName.as_ptr());
                traps.com_printf(
                    cstr(&format!(
                        "{}ERROR: Vehicle Weapon token '{}' has no value!\n",
                        S_COLOR_RED.to_str().unwrap(),
                        pn
                    ))
                    .as_ptr(),
                );
            } else if BG_ParseVehWeaponParm(
                vehWeapon,
                parmName.as_mut_ptr(),
                value,
                bg,
                traps,
                callbacks,
            ) == qfalse
            {
                let pn = cstr_to_str(parmName.as_ptr());
                let v = cstr_to_str(value);
                traps.com_printf(
                    cstr(&format!(
                        "{}ERROR: Unknown Vehicle Weapon key/value pair '{}','{}'!\n",
                        S_COLOR_RED.to_str().unwrap(),
                        pn,
                        v
                    ))
                    .as_ptr(),
                );
            }
        }

        // QAGAME: the lock-on sound registrations are commented out here
        // (`#ifdef QAGAME` arm is a no-op); the `#elif CGAME`/`#else` sound
        // registrations are dead in jampgame and dropped (porting-rules §20).

        let idx = bg.numVehicleWeapons;
        bg.numVehicleWeapons += 1;
        idx
    }
}

/// Raven `VEH_VehWeaponIndexForName`.
///
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:413-443`
pub fn VEH_VehWeaponIndexForName(
    vehWeaponName: *const c_char,
    bg: &mut BgState,
    // Engine `Com_Printf` print seam; also threaded to `VEH_LoadVehWeapon`.
    traps: &dyn BgTraps,
    // Threaded through to `VEH_LoadVehWeapon` for the field-index upcalls.
    callbacks: &mut dyn GameCallbacks,
) -> c_int {
    unsafe {
        if vehWeaponName.is_null() || *vehWeaponName == 0 {
            traps.com_printf(
                cstr(&format!(
                    "{}ERROR: Trying to read Vehicle Weapon with no name!\n",
                    S_COLOR_RED.to_str().unwrap()
                ))
                .as_ptr(),
            );
            return VEH_WEAPON_NONE;
        }
        let mut vw = VEH_WEAPON_BASE;
        while vw < bg.numVehicleWeapons {
            let name = bg.g_vehWeaponInfo[vw as usize].name;
            if !name.is_null() && Q_stricmp(name, vehWeaponName) == 0 {
                return vw;
            }
            vw += 1;
        }
        if vw >= MAX_VEH_WEAPONS as c_int {
            let name = cstr_to_str(vehWeaponName);
            traps.com_printf(
                cstr(&format!(
                    "{}ERROR: Too many Vehicle Weapons (max 16), aborting load on {}!\n",
                    S_COLOR_RED.to_str().unwrap(),
                    name
                ))
                .as_ptr(),
            );
            return VEH_WEAPON_NONE;
        }
        vw = VEH_LoadVehWeapon(vehWeaponName, bg, traps, callbacks);
        if vw == VEH_WEAPON_NONE {
            let name = cstr_to_str(vehWeaponName);
            traps.com_printf(
                cstr(&format!(
                    "{}ERROR: Could not find Vehicle Weapon {}!\n",
                    S_COLOR_RED.to_str().unwrap(),
                    name
                ))
                .as_ptr(),
            );
        }
        vw
    }
}

/// Raven `BG_SetSharedVehicleFunctions`.
///
/// //TODO: Port BG_SetSharedVehicleFunctions
/// Deliberately a no-op: Raven's body filled
/// the `vehicleInfo_t` fn-ptr slots (via `G_SetSharedVehicleFunctions` +
/// `G_Set<Type>VehicleFunctions`), but those slots are retired for stateless
/// `vehicleType_t`-keyed dispatch (`crate::veh_dispatch`), so `.veh`-load has no
/// per-vehicle function setup left to do. The call is kept in the load sequence
/// (`BG_VehicleLoadParms`/`BG_VehicleClampData`) to mirror Raven's shape.
///
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:683-707`
pub fn BG_SetSharedVehicleFunctions(_pVehInfo: *mut vehicleInfo_t) {}

/// Raven `BG_VehicleSetDefaults`.
///
/// Raven: the field-by-field default assignments below the `memset` are
/// commented out in the oracle source (`/* ... */`, bg_vehicleLoad.c:712-809)
/// — only the `memset` is live code. Faithful transcription is the memset
/// alone.
///
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:709-810`
pub fn BG_VehicleSetDefaults(vehicle: *mut vehicleInfo_t) {
    unsafe {
        std::ptr::write_bytes(vehicle, 0, 1);
    }
}

/// Raven `BG_VehicleClampData`.
///
/// Raven: sanity check and clamp the vehicle's data.
///
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:812-837`
pub fn BG_VehicleClampData(vehicle: *mut vehicleInfo_t) {
    unsafe {
        for i in 0..3usize {
            if (*vehicle).centerOfGravity[i] > 1.0 {
                (*vehicle).centerOfGravity[i] = 1.0;
            } else if (*vehicle).centerOfGravity[i] < -1.0 {
                (*vehicle).centerOfGravity[i] = -1.0;
            }
        }

        // Validate passenger max.
        if (*vehicle).maxPassengers > VEH_MAX_PASSENGERS as c_int {
            (*vehicle).maxPassengers = VEH_MAX_PASSENGERS as c_int;
        } else if (*vehicle).maxPassengers < 0 {
            (*vehicle).maxPassengers = 0;
        }
    }
}

/// Raven `BG_ParseVehicleParm`.
///
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:839-981`
// QAGAME-only (porting-rules §20): `#elif CGAME`/`#else` arms dropped.
pub fn BG_ParseVehicleParm(
    vehicle: *mut vehicleInfo_t,
    parmName: *mut c_char,
    pValue: *mut c_char,
    // Threaded so the MP (`_JK2MP`) `VF_LSTRING` branch can bump the bg pool
    // (`BG_Alloc`); the game pool is unreachable from bg-tier code.
    bg: &mut BgState,
    // Engine `Com_Printf` print seam, routed through the trap channel.
    traps: &dyn BgTraps,
    // Routes the `VF_MODEL`/`VF_EFFECT`/`VF_SOUND` field types to the game-tier
    // `G_ModelIndex`/`G_EffectIndex`/`G_SoundIndex` via the bg→game upcall seam.
    callbacks: &mut dyn GameCallbacks,
) -> qboolean {
    unsafe {
        let value = cstr_to_str(pValue);
        let b = vehicle as *mut u8;
        let mut i = 0usize;
        while vehicleFields[i].ofs != -1 {
            if Q_stricmp(vehicleFields[i].name, parmName) == 0 {
                let field = &vehicleFields[i];
                match field.r#type {
                    VF_IGNORE => {}
                    VF_INT => {
                        *(b.add(field.ofs as usize) as *mut c_int) = atoi(&value);
                    }
                    VF_FLOAT => {
                        *(b.add(field.ofs as usize) as *mut f32) = atof(&value) as f32;
                    }
                    VF_LSTRING => {
                        let slot = b.add(field.ofs as usize) as *mut *mut c_char;
                        if (*slot).is_null() {
                            // Raven `_JK2MP` (MP) branch: a fixed 128-byte bg-pool block
                            // (overwritable later), NOT SP's `G_NewString` — bg-tier code has
                            // no game pool. Source: oracle/codemp/game/bg_vehicleLoad.c:195-197
                            let dest = crate::bg_misc::BG_Alloc(128, bg) as *mut c_char;
                            Q_strncpyz(dest, cstr(&value).as_ptr(), 128);
                            *slot = dest;
                        }
                    }
                    VF_VECTOR => {
                        let mut vec: vec3_t = [0.0, 0.0, 0.0];
                        // Raven: `_iFieldsRead = sscanf(value, "%f %f %f", &vec[0],
                        // &vec[1], &vec[2]); assert(_iFieldsRead==3); if
                        // (_iFieldsRead!=3) Com_Printf(...)` — reproduced via the
                        // shared libc-`%f` scanner's returned count.
                        let n = sscanf_f32s(&value, &mut vec);
                        if n != 3 {
                            traps.com_printf(
                                cstr(&format!(
                                    "{}BG_ParseVehicleParm: VEC3 sscanf() failed to read 3 floats ('angle' key bug?)\n",
                                    S_COLOR_YELLOW.to_str().unwrap()
                                ))
                                .as_ptr(),
                            );
                        }
                        // Raven bug (bg_vehicleLoad.c:885-887): the store offset is
                        // taken from `vehWeaponFields[i]`, not `vehicleFields[i]` —
                        // preserved faithfully (porting-rules §A2/§10).
                        let dst = b.add(vehWeaponFields[i].ofs as usize) as *mut f32;
                        *dst.add(0) = vec[0];
                        *dst.add(1) = vec[1];
                        *dst.add(2) = vec[2];
                    }
                    VF_BOOL => {
                        *(b.add(field.ofs as usize) as *mut qboolean) =
                            if atof(&value) != 0.0 { qtrue } else { qfalse };
                    }
                    VF_VEHTYPE => {
                        let vt = GetIDForString(
                            VehicleTable.as_ptr() as *mut stringID_table_t,
                            cstr(&value).as_ptr(),
                        );
                        *(b.add(field.ofs as usize) as *mut vehicleType_t) =
                            core::mem::transmute(vt);
                    }
                    VF_ANIM => {
                        let anim = GetIDForString(
                            animTable.as_ptr() as *mut stringID_table_t,
                            cstr(&value).as_ptr(),
                        );
                        *(b.add(field.ofs as usize) as *mut c_int) = anim;
                    }
                    VF_WEAPON => {
                        // Raven: assignment is commented out in the oracle — the
                        // VF_WEAPON case is a no-op `break;`.
                        // Source: `oracle/codemp/game/bg_vehicleLoad.c:228-230`
                        //*(b.add(field.ofs as usize) as *mut c_int) =
                        //    VEH_VehWeaponIndexForName(cstr(&value).as_ptr(), bg);
                    }
                    VF_MODEL | VF_MODEL_CLIENT => {
                        *(b.add(field.ofs as usize) as *mut c_int) =
                            callbacks.model_index(cstr(&value).as_ptr());
                    }
                    VF_EFFECT | VF_EFFECT_CLIENT => {
                        *(b.add(field.ofs as usize) as *mut c_int) =
                            callbacks.effect_index(cstr(&value).as_ptr());
                    }
                    VF_SHADER | VF_SHADER_NOMIP => {
                        // QAGAME: dead (neither WE_ARE_IN_THE_UI nor CGAME).
                    }
                    VF_SOUND => {
                        *(b.add(field.ofs as usize) as *mut c_int) =
                            callbacks.sound_index(cstr(&value).as_ptr());
                    }
                    // `VF_SOUND_CLIENT` (MP cgame only): the `#elif QAGAME` branch
                    // comments out the `G_SoundIndex` — the game module leaves the
                    // field untouched (cgame registers it via `trap_S_RegisterSound`).
                    // Source: `oracle/codemp/game/bg_vehicleLoad.c:956-964`
                    VF_SOUND_CLIENT => {}
                    _ => return qfalse,
                }
                break;
            }
            i += 1;
        }
        if vehicleFields[i].ofs == -1 {
            qfalse
        } else {
            qtrue
        }
    }
}

/// Raven `VEH_LoadVehicle`.
///
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:983-1362`
pub fn VEH_LoadVehicle(
    vehicleName: *const c_char,
    bg: &mut BgState,
    traps: &dyn BgTraps,
    // Routes the model/effect/sound registrations (field parse + the trailing
    // precache block) to the game tier via the bg→game upcall seam.
    callbacks: &mut dyn GameCallbacks,
) -> c_int {
    unsafe {
        if bg.numVehicles == 0 {
            // `BG_VehicleLoadParms` reaches the engine, so `traps: &dyn BgTraps`
            // is threaded through here per rulings 12/15 (the loader call chain
            // is self-contained within this module).
            BG_VehicleLoadParms(bg, traps);
        }

        let mut p: *const c_char = bg.VehicleParms.as_ptr() as *const c_char;
        COM_BeginParseSession(&mut bg.qs, cstr("vehicles").as_ptr());

        let veh_index = bg.numVehicles as usize;
        let vehicle: *mut vehicleInfo_t = &mut bg.g_vehicleInfo[veh_index];

        let mut weap1: [c_char; 128] = [0; 128];
        let mut weap2: [c_char; 128] = [0; 128];
        let mut weap_muzzle: [[c_char; 128]; 10] = [[0; 128]; 10];

        loop {
            if p.is_null() {
                return VEHICLE_NONE;
            }
            let token = COM_ParseExt(&mut bg.qs, &mut p as *mut *const c_char, qtrue);
            if *token == 0 {
                return VEHICLE_NONE;
            }
            if Q_stricmp(token, vehicleName) == 0 {
                break;
            }
            SkipBracedSection(&mut bg.qs, &mut p as *mut *const c_char);
        }
        if p.is_null() {
            return VEHICLE_NONE;
        }

        let token = COM_ParseExt(&mut bg.qs, &mut p as *mut *const c_char, qtrue);
        if *token == 0 {
            return VEHICLE_NONE;
        }
        if Q_stricmp(token, cstr("{").as_ptr()) != 0 {
            return VEHICLE_NONE;
        }

        BG_VehicleSetDefaults(vehicle);
        loop {
            SkipRestOfLine(&mut bg.qs, &mut p as *mut *const c_char);
            let token = COM_ParseExt(&mut bg.qs, &mut p as *mut *const c_char, qtrue);
            if *token == 0 {
                let name = cstr_to_str(vehicleName);
                traps.com_printf(
                    cstr(&format!(
                        "{}ERROR: unexpected EOF while parsing Vehicle '{}'\n",
                        S_COLOR_RED.to_str().unwrap(),
                        name
                    ))
                    .as_ptr(),
                );
                return VEHICLE_NONE;
            }
            if Q_stricmp(token, cstr("}").as_ptr()) == 0 {
                break;
            }
            let mut parmName: [c_char; 128] = [0; 128];
            Q_strncpyz(parmName.as_mut_ptr(), token, 128);
            let value = COM_ParseExt(&mut bg.qs, &mut p as *mut *const c_char, qtrue);
            if value.is_null() || *value == 0 {
                let pn = cstr_to_str(parmName.as_ptr());
                traps.com_printf(
                    cstr(&format!(
                        "{}ERROR: Vehicle token '{}' has no value!\n",
                        S_COLOR_RED.to_str().unwrap(),
                        pn
                    ))
                    .as_ptr(),
                );
            } else if Q_stricmp(cstr("weap1").as_ptr(), parmName.as_ptr()) == 0 {
                Q_strncpyz(weap1.as_mut_ptr(), value, 128);
            } else if Q_stricmp(cstr("weap2").as_ptr(), parmName.as_ptr()) == 0 {
                Q_strncpyz(weap2.as_mut_ptr(), value, 128);
            } else if let Some(n) = (1..=10).find(|n| {
                Q_stricmp(
                    cstr(&format!("weapMuzzle{}", n)).as_ptr(),
                    parmName.as_ptr(),
                ) == 0
            }) {
                Q_strncpyz(weap_muzzle[n - 1].as_mut_ptr(), value, 128);
            } else if BG_ParseVehicleParm(
                vehicle,
                parmName.as_mut_ptr(),
                value,
                bg,
                traps,
                callbacks,
            ) == qfalse
            {
                let pn = cstr_to_str(parmName.as_ptr());
                let v = cstr_to_str(value);
                traps.com_printf(
                    cstr(&format!(
                        "{}ERROR: Unknown Vehicle key/value pair '{}', '{}'!\n",
                        S_COLOR_RED.to_str().unwrap(),
                        pn,
                        v
                    ))
                    .as_ptr(),
                );
            }
        }

        // NOW: if we have any weapons, go ahead and load them.
        if weap1[0] != 0 {
            if BG_ParseVehicleParm(
                vehicle,
                cstr("weap1").as_ptr() as *mut c_char,
                weap1.as_mut_ptr(),
                bg,
                traps,
                callbacks,
            ) == qfalse
            {
                let w = cstr_to_str(weap1.as_ptr());
                traps.com_printf(
                    cstr(&format!(
                        "{}ERROR: Unknown Vehicle key/value pair 'weap1', '{}'!\n",
                        S_COLOR_RED.to_str().unwrap(),
                        w
                    ))
                    .as_ptr(),
                );
            }
        }
        if weap2[0] != 0 {
            if BG_ParseVehicleParm(
                vehicle,
                cstr("weap2").as_ptr() as *mut c_char,
                weap2.as_mut_ptr(),
                bg,
                traps,
                callbacks,
            ) == qfalse
            {
                let w = cstr_to_str(weap2.as_ptr());
                traps.com_printf(
                    cstr(&format!(
                        "{}ERROR: Unknown Vehicle key/value pair 'weap2', '{}'!\n",
                        S_COLOR_RED.to_str().unwrap(),
                        w
                    ))
                    .as_ptr(),
                );
            }
        }
        for n in 1..=10usize {
            if weap_muzzle[n - 1][0] != 0 {
                let key = format!("weapMuzzle{}", n);
                if BG_ParseVehicleParm(
                    vehicle,
                    cstr(&key).as_ptr() as *mut c_char,
                    weap_muzzle[n - 1].as_mut_ptr(),
                    bg,
                    traps,
                    callbacks,
                ) == qfalse
                {
                    let w = cstr_to_str(weap_muzzle[n - 1].as_ptr());
                    traps.com_printf(
                        cstr(&format!(
                            "{}ERROR: Unknown Vehicle key/value pair '{}', '{}'!\n",
                            S_COLOR_RED.to_str().unwrap(),
                            key,
                            w
                        ))
                        .as_ptr(),
                    );
                }
            }
        }

        // let's give these guys some defaults
        if (*vehicle).health_front == 0 {
            (*vehicle).health_front = (*vehicle).armor / 4;
        }
        if (*vehicle).health_back == 0 {
            (*vehicle).health_back = (*vehicle).armor / 4;
        }
        if (*vehicle).health_right == 0 {
            (*vehicle).health_right = (*vehicle).armor / 4;
        }
        if (*vehicle).health_left == 0 {
            (*vehicle).health_left = (*vehicle).armor / 4;
        }

        if !(*vehicle).model.is_null() {
            let model = cstr_to_str((*vehicle).model);
            (*vehicle).modelIndex = callbacks
                .model_index(cstr(&format!("models/players/{}/model.glm", model)).as_ptr());
        }

        // SP-only skin-registration block (`#ifndef _JK2MP`) and the MP
        // cgame-only skin block (`#ifndef QAGAME`) are both dead in this
        // jampgame (QAGAME, `_JK2MP`) build; dropped per porting-rules §20.

        BG_VehicleClampData(vehicle);
        BG_SetSharedVehicleFunctions(vehicle);

        if (*vehicle).explosionDamage != 0 {
            callbacks.effect_index(cstr("ships/ship_explosion_mark").as_ptr());
        }
        if (*vehicle).flammable != 0 {
            callbacks.sound_index(cstr("sound/vehicles/common/fire_lp.wav").as_ptr());
        }
        if (*vehicle).hoverHeight > 0.0 {
            callbacks.effect_index(cstr("ships/swoop_dust").as_ptr());
        }

        callbacks.effect_index(cstr("volumetric/black_smoke").as_ptr());
        callbacks.effect_index(cstr("ships/fire").as_ptr());
        callbacks.sound_index(cstr("sound/vehicles/common/release.wav").as_ptr());
        // QAGAME: the CGAME-only shader/fx/hideRider registrations (`#elif
        // CGAME`) are dead here; dropped per porting-rules §20.

        let idx = bg.numVehicles;
        bg.numVehicles += 1;
        idx
    }
}

/// Raven `VEH_VehicleIndexForName`.
///
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:1364-1394`
pub fn VEH_VehicleIndexForName(
    vehicleName: *const c_char,
    bg: &mut BgState,
    traps: &dyn BgTraps,
    // Threaded through to `VEH_LoadVehicle` for the model/effect/sound upcalls.
    callbacks: &mut dyn GameCallbacks,
) -> c_int {
    unsafe {
        if vehicleName.is_null() || *vehicleName == 0 {
            traps.com_printf(
                cstr(&format!(
                    "{}ERROR: Trying to read Vehicle with no name!\n",
                    S_COLOR_RED.to_str().unwrap()
                ))
                .as_ptr(),
            );
            return VEHICLE_NONE;
        }
        let mut v = VEHICLE_BASE;
        while v < bg.numVehicles {
            let name = bg.g_vehicleInfo[v as usize].name;
            if !name.is_null() && Q_stricmp(name, vehicleName) == 0 {
                return v;
            }
            v += 1;
        }
        if v >= MAX_VEHICLES as c_int {
            let name = cstr_to_str(vehicleName);
            traps.com_printf(
                cstr(&format!(
                    "{}ERROR: Too many Vehicles (max 64), aborting load on {}!\n",
                    S_COLOR_RED.to_str().unwrap(),
                    name
                ))
                .as_ptr(),
            );
            return VEHICLE_NONE;
        }
        v = VEH_LoadVehicle(vehicleName, bg, traps, callbacks);
        if v == VEHICLE_NONE {
            let name = cstr_to_str(vehicleName);
            traps.com_printf(
                cstr(&format!(
                    "{}ERROR: Could not find Vehicle {}!\n",
                    S_COLOR_RED.to_str().unwrap(),
                    name
                ))
                .as_ptr(),
            );
        }
        v
    }
}

/// Raven `BG_VehWeaponLoadParms`.
///
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:1396-1485`
// `_JK2MP` is always true (MP crate), so the `trap_FS_*`/`BG_TempAlloc`/
// `BG_TempFree` arm is live; the SP `gi.*` arm is dropped (porting-rules §20).
pub fn BG_VehWeaponLoadParms(bg: &mut BgState, traps: &dyn BgTraps) {
    unsafe {
        let mut total_len: usize = 0;
        // `marker` tracks the write cursor into `bg.VehWeaponParms` by offset,
        // mirroring Raven's `marker = VehWeaponParms + totallen` pointer.
        bg.VehWeaponParms[0] = 0;

        let mut list_buf: [c_char; 2048] = [0; 2048];
        let file_cnt = traps.fs_getfilelist(
            cstr("ext_data/vehicles/weapons").as_ptr(),
            cstr(".vwp").as_ptr(),
            list_buf.as_mut_ptr(),
            2048,
        );

        let mut temp_read_buffer: Vec<u8> = vec![0u8; MAX_VEH_WEAPON_DATA_SIZE as usize];
        let mut hold_char = list_buf.as_ptr();

        for _ in 0..file_cnt {
            let veh_ext_fn_len = cstr_to_str(hold_char).len();

            let mut f: fileHandle_t = 0;
            let path = format!("ext_data/vehicles/weapons/{}", cstr_to_str(hold_char));
            let len = traps.fs_fopen(cstr(&path).as_ptr(), &mut f as *mut fileHandle_t, FS_READ);

            if len == -1 {
                traps.com_printf(cstr("error reading file\n").as_ptr());
            } else {
                traps.fs_read(temp_read_buffer.as_mut_ptr() as *mut c_void, len, f);
                temp_read_buffer[len as usize] = 0;

                // Don't let the accumulated text end on a bare '}'.
                if total_len > 0 && bg.VehWeaponParms[total_len - 1] == b'}' as c_char {
                    bg.VehWeaponParms[total_len] = b' ' as c_char;
                    total_len += 1;
                }

                if total_len + len as usize >= MAX_VEH_WEAPON_DATA_SIZE as usize {
                    traps.com_error(
                        ERR_DROP as c_int,
                        cstr("Vehicle Weapon extensions (*.vwp) are too large").as_ptr(),
                    );
                }
                let appended = cstr_to_str(temp_read_buffer.as_ptr() as *const c_char);
                for (i, byte) in appended.bytes().enumerate() {
                    bg.VehWeaponParms[total_len + i] = byte as c_char;
                }
                total_len += appended.len();
                bg.VehWeaponParms[total_len] = 0;

                traps.fs_fclose(f);
            }

            hold_char = hold_char.add(veh_ext_fn_len + 1);
        }
    }
}

/// Raven `BG_VehicleLoadParms`.
///
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:1487-1588`
// `_JK2MP` always true (MP crate) so the `trap_FS_*` arm is live; the SP
// `gi.*` arm is dropped (porting-rules §20).
pub fn BG_VehicleLoadParms(bg: &mut BgState, traps: &dyn BgTraps) {
    unsafe {
        let mut total_len: usize = 0;
        bg.VehicleParms[0] = 0;

        let mut list_buf: [c_char; 2048] = [0; 2048];
        let file_cnt = traps.fs_getfilelist(
            cstr("ext_data/vehicles").as_ptr(),
            cstr(".veh").as_ptr(),
            list_buf.as_mut_ptr(),
            2048,
        );

        let mut temp_read_buffer: Vec<u8> = vec![0u8; MAX_VEHICLE_DATA_SIZE as usize];
        let mut hold_char = list_buf.as_ptr();

        for _ in 0..file_cnt {
            let veh_ext_fn_len = cstr_to_str(hold_char).len();

            let mut f: fileHandle_t = 0;
            let path = format!("ext_data/vehicles/{}", cstr_to_str(hold_char));
            let len = traps.fs_fopen(cstr(&path).as_ptr(), &mut f as *mut fileHandle_t, FS_READ);

            if len == -1 {
                traps.com_printf(cstr("error reading file\n").as_ptr());
            } else {
                traps.fs_read(temp_read_buffer.as_mut_ptr() as *mut c_void, len, f);
                temp_read_buffer[len as usize] = 0;

                if total_len > 0 && bg.VehicleParms[total_len - 1] == b'}' as c_char {
                    bg.VehicleParms[total_len] = b' ' as c_char;
                    total_len += 1;
                }

                if total_len + len as usize >= MAX_VEHICLE_DATA_SIZE as usize {
                    traps.com_error(
                        ERR_DROP as c_int,
                        cstr("Vehicle extensions (*.veh) are too large").as_ptr(),
                    );
                }
                let appended = cstr_to_str(temp_read_buffer.as_ptr() as *const c_char);
                for (i, byte) in appended.bytes().enumerate() {
                    bg.VehicleParms[total_len + i] = byte as c_char;
                }
                total_len += appended.len();
                bg.VehicleParms[total_len] = 0;

                traps.fs_fclose(f);
            }

            hold_char = hold_char.add(veh_ext_fn_len + 1);
        }

        bg.numVehicles = 1;
        let base = &mut bg.g_vehicleInfo[VEHICLE_BASE as usize] as *mut vehicleInfo_t;
        BG_VehicleSetDefaults(base);
        BG_VehicleClampData(base);
        BG_SetSharedVehicleFunctions(base);

        BG_VehWeaponLoadParms(bg, traps);
    }
}

/// Raven `BG_VehicleGetIndex`.
///
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:1590-1593`
pub fn BG_VehicleGetIndex(
    vehicleName: *const c_char,
    bg: &mut BgState,
    traps: &dyn BgTraps,
    // Threaded through to `VEH_VehicleIndexForName` for the model/effect/sound
    // upcalls when loading an as-yet-unregistered vehicle.
    callbacks: &mut dyn GameCallbacks,
) -> c_int {
    VEH_VehicleIndexForName(vehicleName, bg, traps, callbacks)
}

/// Raven `BG_GetVehicleModelName`.
///
/// Raven: we get the vehicle name passed in as `modelname` with a `$` in
/// front of it; we are expected to then get the model for the vehicle and
/// stomp over `modelname` with it.
///
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:1599-1611`
pub fn BG_GetVehicleModelName(
    modelname: *mut c_char,
    bg: &mut BgState,
    traps: &dyn BgTraps,
    callbacks: &mut dyn GameCallbacks,
) {
    unsafe {
        let veh_name = modelname.add(1);
        let v_index = BG_VehicleGetIndex(veh_name, bg, traps, callbacks);
        debug_assert!(*modelname == b'$' as c_char);

        if v_index == VEHICLE_NONE {
            let name = cstr_to_str(veh_name);
            traps.com_error(
                ERR_DROP as c_int,
                cstr(&format!(
                    "BG_GetVehicleModelName:  couldn't find vehicle {}",
                    name
                ))
                .as_ptr(),
            );
        }

        let model = bg.g_vehicleInfo[v_index as usize].model;
        let model_str = cstr_to_str(model);
        Q_strncpyz(
            modelname,
            cstr(&model_str).as_ptr(),
            (model_str.len() + 1) as c_int,
        );
    }
}

/// Raven `BG_GetVehicleSkinName`.
///
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:1613-1633`
pub fn BG_GetVehicleSkinName(
    skinname: *mut c_char,
    bg: &mut BgState,
    traps: &dyn BgTraps,
    callbacks: &mut dyn GameCallbacks,
) {
    unsafe {
        let veh_name = skinname.add(1);
        let v_index = BG_VehicleGetIndex(veh_name, bg, traps, callbacks);
        debug_assert!(*skinname == b'$' as c_char);

        if v_index == VEHICLE_NONE {
            let name = cstr_to_str(veh_name);
            traps.com_error(
                ERR_DROP as c_int,
                cstr(&format!(
                    "BG_GetVehicleSkinName:  couldn't find vehicle {}",
                    name
                ))
                .as_ptr(),
            );
        }

        let skin = bg.g_vehicleInfo[v_index as usize].skin;
        if skin.is_null() || *skin == 0 {
            *skinname = 0;
        } else {
            let skin_str = cstr_to_str(skin);
            Q_strncpyz(
                skinname,
                cstr(&skin_str).as_ptr(),
                (skin_str.len() + 1) as c_int,
            );
        }
    }
}

/// Raven `AttachRidersGeneric`.
///
/// Source: `oracle/codemp/game/bg_vehicleLoad.c:1643-1664`
pub fn AttachRidersGeneric(
    pVeh: *mut Vehicle_t,
    bg: &BgState,
    traps: &dyn BgTraps,
    levelTime: c_int,
) {
    unsafe {
        if !(*pVeh).m_pPilot.is_null() {
            let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
            let mut yawOnlyAngles: vec3_t = [0.0, 0.0, 0.0];
            let parent = (*pVeh).m_pParentEntity;
            let pilot = (*pVeh).m_pPilot;
            let crotchBolt = traps.g2api_add_bolt((*parent).ghoul2, 0, cstr("*driver").as_ptr());

            debug_assert!(!(*parent).playerState.is_null());

            VectorSet(
                &mut yawOnlyAngles,
                0.0,
                (*(*parent).playerState).viewangles[YAW],
                0.0,
            );

            traps.g2api_get_bolt_matrix(
                (*parent).ghoul2,
                0,
                crotchBolt,
                &mut boltMatrix as *mut mdxaBone_t,
                &yawOnlyAngles as *const vec3_t,
                &(*(*parent).playerState).origin as *const vec3_t,
                levelTime,
                core::ptr::null_mut(),
                &(*parent).modelScale as *const vec3_t,
            );
            let mut out: vec3_t = [0.0, 0.0, 0.0];
            BG_GiveMeVectorFromMatrix(
                &boltMatrix as *const mdxaBone_t,
                Eorientations::ORIGIN as c_int,
                &mut out,
            );
            (*(*pilot).playerState).origin = out;
        }
    }
}
