// PORT-COMPLETE: bg_saberLoad.c 31/7 (pass-3 zero-park fill)
//! Port of `oracle/oracle/codemp/game/bg_saberLoad.c`.
//!
//! This crate is `mp/game` (jampgame == Raven's `QAGAME`); the file's
//! `#elif defined CGAME` branches are dead code here and are dropped per
//! porting-rules §20 ("drop dead surface").
//!
//! Several fns reach `SaberParms`/`bgSaberParseTBuffer` through `BgState`
//! (ruling 12) and the engine surface through `BgTraps` (ruling 13). Some
//! `SFL2_` saberFlags bitflag consts (e.g. `SFL2_NO_MANUAL_DEACTIVATE`) and the
//! `SaberTable`/`SaberMoveTable`/`FPTable`/`animTable` lookup statics are not
//! yet ported into the Rust tree; those sites reference the Raven names
//! directly per the pass-3 zero-park policy and are reported as missing
//! symbols rather than parked.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::q_math::Q_irand;

use mp_qshared::common::mp::qcommon::saber::saber_colors::{
    SABER_BLUE, SABER_GREEN, SABER_ORANGE, SABER_PURPLE, SABER_RED, SABER_YELLOW,
};
use mp_qshared::common::mp::qcommon::saber::saber_type::saberType_t;
use mp_qshared::shared::{QFALSE, QTRUE};

// PORT-NOTE(missing-symbol): `MAX_ANIMATIONS` is an `animNumber_t` enum
// terminator (anims.h), not yet exposed as a bare const; referenced through
// the enum per zero-park policy.
use mp_bg::public::anim_number::animNumber_t;
use mp_qshared::shared::limits::MAX_CLIENTS_I32;

// Local helper: mirrors `!Q_stricmp(name, lit)` (Q_stricmp returns 0 on
// case-insensitive equality). `Q_stricmp` itself is an out-of-file callee
// (q_shared.c:900, staged skeleton at `crate::q_shared::Q_stricmp`).
unsafe fn qstricmp_eq(name: *const c_char, lit: &std::ffi::CStr) -> bool {
    crate::q_shared::Q_stricmp(name, lit.as_ptr()) == 0
}

// Local helper: mirrors libc `strcpy` (copies through the terminating NUL,
// no bounds check — faithful to Raven's unchecked fixed-buffer usage here).
unsafe fn c_strcpy(dst: *mut c_char, src: *const c_char) {
    let mut i: isize = 0;
    loop {
        let c = *src.offset(i);
        *dst.offset(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
}

// Local helper: mirrors libc `atoi` (skip leading whitespace, optional
// sign, longest leading run of digits, `0` on no parse).
unsafe fn c_atoi(s: *const c_char) -> c_int {
    let cstr = std::ffi::CStr::from_ptr(s);
    let text = String::from_utf8_lossy(cstr.to_bytes());
    let trimmed = text.trim_start();
    let mut end = 0usize;
    let mut chars = trimmed.char_indices();
    let mut idx = 0usize;
    if let Some((_, c)) = trimmed.char_indices().next() {
        if c == '+' || c == '-' {
            idx = c.len_utf8();
        }
    }
    end = idx;
    for (i, c) in trimmed[idx..].char_indices() {
        if c.is_ascii_digit() {
            end = idx + i + c.len_utf8();
        } else {
            break;
        }
    }
    trimmed[..end].parse::<c_int>().unwrap_or(0)
}

/// Raven `BG_SoundIndex`.
///
/// Raven: builds under both `QAGAME` and `CGAME`; only the `QAGAME` branch
/// (`G_SoundIndex`) is live in this crate (jampgame).
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:32-39`
pub fn BG_SoundIndex(sound: *mut c_char) -> c_int {
    crate::g_utils::G_SoundIndex(sound as *const c_char)
}

/// Raven `BG_ParseLiteral`.
///
/// Raven: "Also used in npc code".
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:129-147`
pub fn BG_ParseLiteral(data: *mut *const c_char, string: *const c_char) -> qboolean {
    unsafe {
        let token = crate::q_shared::COM_ParseExt(data, QTRUE);
        if *token == 0 {
            let msg = std::ffi::CString::new("unexpected EOF\n").unwrap();
            crate::g_main::Com_Printf(msg.as_ptr());
            return QTRUE;
        }

        if crate::q_shared::Q_stricmp(token as *const c_char, string) != 0 {
            let s = std::ffi::CStr::from_ptr(string).to_string_lossy();
            let msg =
                std::ffi::CString::new(format!("required string '{}' missing\n", s)).unwrap();
            crate::g_main::Com_Printf(msg.as_ptr());
            return QTRUE;
        }

        QFALSE
    }
}

/// Raven `TranslateSaberColor`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:149-180`
pub fn TranslateSaberColor(name: *const c_char) -> saber_colors_t {
    unsafe {
        if qstricmp_eq(name, c"red") {
            return SABER_RED;
        }
        if qstricmp_eq(name, c"orange") {
            return SABER_ORANGE;
        }
        if qstricmp_eq(name, c"yellow") {
            return SABER_YELLOW;
        }
        if qstricmp_eq(name, c"green") {
            return SABER_GREEN;
        }
        if qstricmp_eq(name, c"blue") {
            return SABER_BLUE;
        }
        if qstricmp_eq(name, c"purple") {
            return SABER_PURPLE;
        }
        if qstricmp_eq(name, c"random") {
            return crate::q_math::Q_irand(SABER_ORANGE, SABER_PURPLE);
        }
    }
    SABER_BLUE
}

/// Raven `TranslateSaberStyle`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:182-213`
pub fn TranslateSaberStyle(name: *const c_char) -> saber_styles_t {
    unsafe {
        if qstricmp_eq(name, c"fast") {
            return saber_styles_t::SS_FAST;
        }
        if qstricmp_eq(name, c"medium") {
            return saber_styles_t::SS_MEDIUM;
        }
        if qstricmp_eq(name, c"strong") {
            return saber_styles_t::SS_STRONG;
        }
        if qstricmp_eq(name, c"desann") {
            return saber_styles_t::SS_DESANN;
        }
        if qstricmp_eq(name, c"tavion") {
            return saber_styles_t::SS_TAVION;
        }
        if qstricmp_eq(name, c"dual") {
            return saber_styles_t::SS_DUAL;
        }
        if qstricmp_eq(name, c"staff") {
            return saber_styles_t::SS_STAFF;
        }
    }
    saber_styles_t::SS_NONE
}

/// Raven `WP_SaberBladeUseSecondBladeStyle`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:215-228`
pub fn WP_SaberBladeUseSecondBladeStyle(saber: *mut saberInfo_t, bladeNum: c_int) -> qboolean {
    if !saber.is_null() {
        let saber = unsafe { &*saber };
        if saber.bladeStyle2Start > 0 && bladeNum >= saber.bladeStyle2Start {
            return QTRUE;
        }
    }
    QFALSE
}

/// Raven `WP_SaberBladeDoTransitionDamage`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:230-243`
pub fn WP_SaberBladeDoTransitionDamage(saber: *mut saberInfo_t, bladeNum: c_int) -> qboolean {
    unsafe {
        if WP_SaberBladeUseSecondBladeStyle(saber, bladeNum) == QFALSE
            && ((*saber).saberFlags2 & SFL2_TRANSITION_DAMAGE) != 0
        {
            // use first blade style for this blade
            return QTRUE;
        } else if WP_SaberBladeUseSecondBladeStyle(saber, bladeNum) != QFALSE
            && ((*saber).saberFlags2 & SFL2_TRANSITION_DAMAGE2) != 0
        {
            // use second blade style for this blade
            return QTRUE;
        }
        QFALSE
    }
}

/// Raven `WP_UseFirstValidSaberStyle`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:245-351`
pub fn WP_UseFirstValidSaberStyle(
    saber1: *mut saberInfo_t,
    saber2: *mut saberInfo_t,
    saberHolstered: c_int,
    saberAnimLevel: *mut c_int,
) -> qboolean {
    unsafe {
        let mut styleInvalid = QFALSE;
        let saber1Active: qboolean;
        let saber2Active: qboolean;
        let mut dualSabers = QFALSE;
        let mut validStyles: c_int = 0;

        let s1 = if saber1.is_null() { None } else { Some(&*saber1) };
        let s2 = if saber2.is_null() { None } else { Some(&*saber2) };

        if let Some(sb2) = s2 {
            if sb2.model[0] != 0 {
                dualSabers = QTRUE;
            }
        }

        if dualSabers != QFALSE {
            // dual
            if saberHolstered > 1 {
                saber1Active = QFALSE;
                saber2Active = QFALSE;
            } else if saberHolstered > 0 {
                saber1Active = QTRUE;
                saber2Active = QFALSE;
            } else {
                saber1Active = QTRUE;
                saber2Active = QTRUE;
            }
        } else {
            saber2Active = QFALSE;
            if s1.is_none() || s1.unwrap().model[0] == 0 {
                saber1Active = QFALSE;
            } else if s1.unwrap().numBlades > 1 {
                // staff
                saber1Active = if saberHolstered > 1 { QFALSE } else { QTRUE };
            } else {
                // single
                saber1Active = if saberHolstered != 0 { QFALSE } else { QTRUE };
            }
        }

        // initially, all styles are valid
        let mut styleNum = saber_styles_t::SS_NONE as c_int + 1;
        while styleNum < saber_styles_t::SS_NUM_SABER_STYLES as c_int {
            validStyles |= 1 << styleNum;
            styleNum += 1;
        }

        if saber1Active != QFALSE
            && s1.is_some()
            && s1.unwrap().model[0] != 0
            && s1.unwrap().stylesForbidden != 0
        {
            let sf = s1.unwrap().stylesForbidden;
            if (sf & (1 << *saberAnimLevel)) != 0 {
                // not a valid style for first saber!
                styleInvalid = QTRUE;
                validStyles &= !sf;
            }
        }
        if dualSabers != QFALSE {
            // check second saber, too
            if saber2Active != QFALSE && s2.unwrap().stylesForbidden != 0 {
                let sf2 = s2.unwrap().stylesForbidden;
                if (sf2 & (1 << *saberAnimLevel)) != 0 {
                    // not a valid style for second saber!
                    styleInvalid = QTRUE;
                    // only the ones both sabers allow is valid
                    validStyles &= !sf2;
                }
            }
        }
        if styleInvalid != QFALSE && validStyles != 0 {
            // using an invalid style and have at least one valid style to
            // use, so switch to it
            let mut styleNum = saber_styles_t::SS_FAST as c_int;
            while styleNum < saber_styles_t::SS_NUM_SABER_STYLES as c_int {
                if (validStyles & (1 << styleNum)) != 0 {
                    *saberAnimLevel = styleNum;
                    return QTRUE;
                }
                styleNum += 1;
            }
        }
        QFALSE
    }
}

/// Raven `WP_SaberStyleValidForSaber`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:353-464`
pub fn WP_SaberStyleValidForSaber(
    saber1: *mut saberInfo_t,
    saber2: *mut saberInfo_t,
    saberHolstered: c_int,
    saberAnimLevel: c_int,
) -> qboolean {
    unsafe {
        let saber1Active: qboolean;
        let saber2Active: qboolean;
        let mut dualSabers = QFALSE;

        let s1 = if saber1.is_null() { None } else { Some(&*saber1) };
        let s2 = if saber2.is_null() { None } else { Some(&*saber2) };

        if let Some(sb2) = s2 {
            if sb2.model[0] != 0 {
                dualSabers = QTRUE;
            }
        }

        if dualSabers != QFALSE {
            // dual
            if saberHolstered > 1 {
                saber1Active = QFALSE;
                saber2Active = QFALSE;
            } else if saberHolstered > 0 {
                saber1Active = QTRUE;
                saber2Active = QFALSE;
            } else {
                saber1Active = QTRUE;
                saber2Active = QTRUE;
            }
        } else {
            saber2Active = QFALSE;
            if s1.is_none() || s1.unwrap().model[0] == 0 {
                saber1Active = QFALSE;
            } else if s1.unwrap().numBlades > 1 {
                // staff
                saber1Active = if saberHolstered > 1 { QFALSE } else { QTRUE };
            } else {
                // single
                saber1Active = if saberHolstered != 0 { QFALSE } else { QTRUE };
            }
        }

        if saber1Active != QFALSE
            && s1.is_some()
            && s1.unwrap().model[0] != 0
            && s1.unwrap().stylesForbidden != 0
        {
            if (s1.unwrap().stylesForbidden & (1 << saberAnimLevel)) != 0 {
                // not a valid style for first saber!
                return QFALSE;
            }
        }
        if dualSabers != QFALSE
            && saber2Active != QFALSE
            && s2.is_some()
            && s2.unwrap().model[0] != 0
        {
            let sb2 = s2.unwrap();
            if sb2.stylesForbidden != 0 {
                // check second saber, too
                if (sb2.stylesForbidden & (1 << saberAnimLevel)) != 0 {
                    // not a valid style for second saber!
                    return QFALSE;
                }
            }
            // now: if using dual sabers, only dual and tavion (if given
            // with this saber) are allowed
            if saberAnimLevel != saber_styles_t::SS_DUAL as c_int {
                // dual is okay
                if saberAnimLevel != saber_styles_t::SS_TAVION as c_int {
                    // tavion might be okay, all others are not
                    return QFALSE;
                } else {
                    // see if "tavion" style is okay
                    let first_gave_it = saber1Active != QFALSE
                        && s1.is_some()
                        && s1.unwrap().model[0] != 0
                        && (s1.unwrap().stylesLearned & (1 << (saber_styles_t::SS_TAVION as c_int)))
                            != 0;
                    let second_gave_it = (sb2.stylesLearned
                        & (1 << (saber_styles_t::SS_TAVION as c_int)))
                        != 0;
                    if first_gave_it {
                        // okay to use tavion style, first saber gave it to us
                    } else if second_gave_it {
                        // okay to use tavion style, second saber gave it to us
                    } else {
                        // tavion style is not allowed because neither of the
                        // sabers we're using gave it to us
                        return QFALSE;
                    }
                }
            }
        }
        QTRUE
    }
}

// PORT-NOTE(missing-const): `SFL2_NO_MANUAL_DEACTIVATE`/
// `SFL2_NO_MANUAL_DEACTIVATE2` are not yet ported (bare bitflag consts);
// referenced by name per zero-park policy, reported as missing symbols.
/// Raven `WP_SaberCanTurnOffSomeBlades`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:466-486`
pub fn WP_SaberCanTurnOffSomeBlades(saber: *mut saberInfo_t) -> qboolean {
    unsafe {
        if (*saber).bladeStyle2Start > 0 && (*saber).numBlades > (*saber).bladeStyle2Start {
            if ((*saber).saberFlags2 & SFL2_NO_MANUAL_DEACTIVATE) != 0
                && ((*saber).saberFlags2 & SFL2_NO_MANUAL_DEACTIVATE2) != 0
            {
                // all blades are always on
                return QFALSE;
            }
        } else if ((*saber).saberFlags2 & SFL2_NO_MANUAL_DEACTIVATE) != 0 {
            // all blades are always on
            return QFALSE;
        }
        // you can turn some off
        QTRUE
    }
}

/// Raven `WP_SaberSetDefaults`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:488-613`
pub fn WP_SaberSetDefaults(saber: *mut saberInfo_t) {
    unsafe {
        let s = &mut *saber;

        // Set defaults so that, if it fails, there's at least something there
        for i in 0..MAX_BLADES {
            s.blade[i].color = SABER_RED;
            s.blade[i].radius = SABER_RADIUS_STANDARD;
            s.blade[i].lengthMax = 32.0;
        }

        c_strcpy(s.name.as_mut_ptr(), c"default".as_ptr());
        c_strcpy(s.fullName.as_mut_ptr(), c"lightsaber".as_ptr());
        c_strcpy(
            s.model.as_mut_ptr(),
            c"models/weapons2/saber_reborn/saber_w.glm".as_ptr(),
        );
        s.skin = 0;
        s.soundOn = BG_SoundIndex(c"sound/weapons/saber/enemy_saber_on.wav".as_ptr() as *mut c_char);
        s.soundLoop = BG_SoundIndex(c"sound/weapons/saber/saberhum3.wav".as_ptr() as *mut c_char);
        s.soundOff =
            BG_SoundIndex(c"sound/weapons/saber/enemy_saber_off.wav".as_ptr() as *mut c_char);
        s.numBlades = 1;
        s.r#type = saberType_t::SABER_SINGLE;
        s.stylesLearned = 0;
        s.stylesForbidden = 0; // allow all styles
        s.maxChain = 0; // 0 = use default behavior
        s.forceRestrictions = 0;
        s.lockBonus = 0;
        s.parryBonus = 0;
        s.breakParryBonus = 0;
        s.breakParryBonus2 = 0;
        s.disarmBonus = 0;
        s.disarmBonus2 = 0;
        s.singleBladeStyle = saber_styles_t::SS_NONE; // makes it so that you use a different style if you only have the first blade active
        // saber->brokenSaber1 = NULL; saber->brokenSaber2 = NULL; -- not present in this port (no brokenSaberN fields)

        // ===NEW========================================================================================
        // done in cgame (client-side code)
        s.saberFlags = 0; // see all the SFL_ flags
        s.saberFlags2 = 0; // see all the SFL2_ flags

        s.spinSound = 0; // none - if set, plays this sound as it spins when thrown
        s.swingSound[0] = 0;
        s.swingSound[1] = 0;
        s.swingSound[2] = 0;

        // done in game (server-side code)
        s.moveSpeedScale = 1.0; // 1.0 - you move faster/slower when using this saber
        s.animSpeedScale = 1.0; // 1.0 - plays normal attack animations faster/slower

        s.kataMove = LS_INVALID;
        s.lungeAtkMove = LS_INVALID;
        s.jumpAtkUpMove = LS_INVALID;
        s.jumpAtkFwdMove = LS_INVALID;
        s.jumpAtkBackMove = LS_INVALID;
        s.jumpAtkRightMove = LS_INVALID;
        s.jumpAtkLeftMove = LS_INVALID;
        s.readyAnim = -1;
        s.drawAnim = -1;
        s.putawayAnim = -1;
        s.tauntAnim = -1;
        s.bowAnim = -1;
        s.meditateAnim = -1;
        s.flourishAnim = -1;
        s.gloatAnim = -1;

        // ***NOTE: only 2 "styles" of blades — bladeStyle2Start is the first
        // blade to use the secondary values below.***
        s.bladeStyle2Start = 0;

        // ===PRIMARY BLADES=====================
        // done in cgame (client-side code)
        s.trailStyle = 0;
        s.g2MarksShader = 0;
        s.g2WeaponMarkShader = 0;
        s.hitSound[0] = 0;
        s.hitSound[1] = 0;
        s.hitSound[2] = 0;
        s.blockSound[0] = 0;
        s.blockSound[1] = 0;
        s.blockSound[2] = 0;
        s.bounceSound[0] = 0;
        s.bounceSound[1] = 0;
        s.bounceSound[2] = 0;
        s.blockEffect = 0;
        s.hitPersonEffect = 0;
        s.hitOtherEffect = 0;
        s.bladeEffect = 0;

        // done in game (server-side code)
        s.knockbackScale = 0.0;
        s.damageScale = 1.0;
        s.splashRadius = 0.0;
        s.splashDamage = 0;
        s.splashKnockback = 0.0;

        // ===SECONDARY BLADES===================
        // done in cgame (client-side code)
        s.trailStyle2 = 0;
        s.g2MarksShader2 = 0;
        s.g2WeaponMarkShader2 = 0;
        s.hit2Sound[0] = 0;
        s.hit2Sound[1] = 0;
        s.hit2Sound[2] = 0;
        s.block2Sound[0] = 0;
        s.block2Sound[1] = 0;
        s.block2Sound[2] = 0;
        s.bounce2Sound[0] = 0;
        s.bounce2Sound[1] = 0;
        s.bounce2Sound[2] = 0;
        s.blockEffect2 = 0;
        s.hitPersonEffect2 = 0;
        s.hitOtherEffect2 = 0;
        s.bladeEffect2 = 0;

        // done in game (server-side code)
        s.knockbackScale2 = 0.0;
        s.damageScale2 = 1.0;
        s.splashRadius2 = 0.0;
        s.splashDamage2 = 0;
        s.splashKnockback2 = 0.0;
        // =========================================================================================================================
    }
}

// Raven's file-local `#define MAX_SABER_DATA_SIZE 0x80000` / `DEFAULT_SABER
// "Kyle"` (bg_saberLoad.c:43, 615). Not cross-file symbols — ported as local
// consts in this file, matching Raven's own file-static scope.
const MAX_SABER_DATA_SIZE: usize = 0x80000;
const DEFAULT_SABER: &std::ffi::CStr = c"Kyle";

/// Raven `WP_SaberParseParms`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:617-2572`
// PORT-NOTE(missing-symbols): `SFL_*`/`SFL2_*` bitflag consts beyond
// `SFL_TWO_HANDED` and the `SaberTable`/`SaberMoveTable`/`FPTable`/
// `animTable` lookup statics are not yet ported; referenced by name per
// zero-park policy (reported as missing symbols). `trap_R_RegisterSkin` is
// not in `BgTraps`; called here as `traps.r_register_skin(...)` matching the
// trait's naming convention, also reported missing.
pub fn WP_SaberParseParms(
    SaberName: *const c_char,
    saber: *mut saberInfo_t,
    bg: &mut BgState,
    traps: &dyn BgTraps,
) -> qboolean {
    unsafe {
        let mut useSaber: [c_char; 1024] = [0; 1024];
        let mut triedDefault = QFALSE;
        let mut saberMove: c_int = LS_INVALID;
        let mut anim: c_int = -1;

        if saber.is_null() {
            return QFALSE;
        }

        // Set defaults so that, if it fails, there's at least something there
        WP_SaberSetDefaults(saber);

        if SaberName.is_null() || *SaberName == 0 {
            c_strcpy(useSaber.as_mut_ptr(), DEFAULT_SABER.as_ptr());
            triedDefault = QTRUE;
        } else {
            c_strcpy(useSaber.as_mut_ptr(), SaberName);
        }

        if bg.SaberParms.is_empty() {
            bg.SaberParms.push(0);
        }

        // try to parse it out
        let mut p: *const c_char = bg.SaberParms.as_ptr() as *const c_char;
        crate::q_shared::COM_BeginParseSession(c"saberinfo".as_ptr());

        // look for the right saber
        loop {
            if p.is_null() {
                break;
            }
            let token = crate::q_shared::COM_ParseExt(&mut p, QTRUE);
            if *token == 0 {
                if triedDefault == QFALSE {
                    // fall back to default and restart, should always be there
                    p = bg.SaberParms.as_ptr() as *const c_char;
                    crate::q_shared::COM_BeginParseSession(c"saberinfo".as_ptr());
                    c_strcpy(useSaber.as_mut_ptr(), DEFAULT_SABER.as_ptr());
                    triedDefault = QTRUE;
                    continue;
                } else {
                    return QFALSE;
                }
            }

            if crate::q_shared::Q_stricmp(token as *const c_char, useSaber.as_ptr()) == 0 {
                break;
            }

            crate::q_shared::SkipBracedSection(&mut p);
        }
        if p.is_null() {
            // even the default saber isn't found?
            return QFALSE;
        }

        // got the name we're using for sure
        c_strcpy((*saber).name.as_mut_ptr(), useSaber.as_ptr());

        if BG_ParseLiteral(&mut p, c"{".as_ptr()) != QFALSE {
            return QFALSE;
        }

        // parse the saber info block
        loop {
            let token = crate::q_shared::COM_ParseExt(&mut p, QTRUE);
            if *token == 0 {
                let s = format!(
                    "ERROR: unexpected EOF while parsing '{}'\n",
                    cstr_to_str(useSaber.as_ptr())
                );
                crate::g_main::Com_Printf(cstr(&s).as_ptr());
                return QFALSE;
            }

            let tok = token as *const c_char;

            if qstricmp_eq(tok, c"}") {
                break;
            }

            macro_rules! parse_string_field {
                () => {{
                    let mut value: *const c_char = std::ptr::null();
                    if crate::q_shared::COM_ParseString(&mut p, &mut value) != QFALSE {
                        continue;
                    }
                    value
                }};
            }
            macro_rules! parse_int_field {
                ($n:expr) => {{
                    if crate::q_shared::COM_ParseInt(&mut p, &mut $n) != QFALSE {
                        crate::q_shared::SkipRestOfLine(&mut p);
                        continue;
                    }
                }};
            }
            macro_rules! parse_float_field {
                ($f:expr) => {{
                    if crate::q_shared::COM_ParseFloat(&mut p, &mut $f) != QFALSE {
                        crate::q_shared::SkipRestOfLine(&mut p);
                        continue;
                    }
                }};
            }

            let s = &mut *saber;

            // saber fullName
            if qstricmp_eq(tok, c"name") {
                let value = parse_string_field!();
                c_strcpy(s.fullName.as_mut_ptr(), value);
                continue;
            }

            // saber type
            if qstricmp_eq(tok, c"saberType") {
                let value = parse_string_field!();
                let saberType = crate::q_shared::GetIDForString(SaberTable, value);
                if saberType >= saberType_t::SABER_SINGLE as c_int
                    && saberType <= saberType_t::NUM_SABERS as c_int
                {
                    s.r#type = std::mem::transmute::<c_int, saberType_t>(saberType);
                }
                continue;
            }

            // saber hilt
            if qstricmp_eq(tok, c"saberModel") {
                let value = parse_string_field!();
                c_strcpy(s.model.as_mut_ptr(), value);
                continue;
            }

            if qstricmp_eq(tok, c"customSkin") {
                let value = parse_string_field!();
                // PORT-NOTE(missing-trait-method): `trap_R_RegisterSkin` is
                // not in `BgTraps`; called as `r_register_skin` matching the
                // trait's naming convention (reported as missing symbol).
                s.skin = traps.r_register_skin(value);
                continue;
            }

            // on sound
            if qstricmp_eq(tok, c"soundOn") {
                let value = parse_string_field!();
                s.soundOn = BG_SoundIndex(value as *mut c_char);
                continue;
            }

            // loop sound
            if qstricmp_eq(tok, c"soundLoop") {
                let value = parse_string_field!();
                s.soundLoop = BG_SoundIndex(value as *mut c_char);
                continue;
            }

            // off sound
            if qstricmp_eq(tok, c"soundOff") {
                let value = parse_string_field!();
                s.soundOff = BG_SoundIndex(value as *mut c_char);
                continue;
            }

            if qstricmp_eq(tok, c"numBlades") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n < 1 || n > MAX_BLADES as c_int {
                    let s_useSaber = cstr_to_str(useSaber.as_ptr());
                    let msg = format!(
                        "WP_SaberParseParms: saber {} has illegal number of blades ({}) max: {}",
                        s_useSaber, n, MAX_BLADES
                    );
                    crate::g_main::Com_Error(ERR_DROP, cstr(&msg).as_ptr());
                    continue;
                }
                s.numBlades = n;
                continue;
            }

            // saberColor
            if crate::q_shared::Q_stricmpn(tok, c"saberColor".as_ptr(), 10) == 0 {
                let toklen = std::ffi::CStr::from_ptr(tok).to_bytes().len();
                let mut n: c_int;
                if toklen == 10 {
                    n = -1;
                } else if toklen == 11 {
                    n = c_atoi(tok.offset(10)) - 1;
                    if n > 7 || n < 1 {
                        let msg = format!(
                            "WARNING: bad saberColor '{}' in {}\n",
                            cstr_to_str(tok),
                            cstr_to_str(useSaber.as_ptr())
                        );
                        crate::g_main::Com_Printf(cstr(&msg).as_ptr());
                        continue;
                    }
                } else {
                    let msg = format!(
                        "WARNING: bad saberColor '{}' in {}\n",
                        cstr_to_str(tok),
                        cstr_to_str(useSaber.as_ptr())
                    );
                    crate::g_main::Com_Printf(cstr(&msg).as_ptr());
                    continue;
                }

                let value = parse_string_field!(); // read the color

                if n == -1 {
                    // this fills in the rest of the blades with the same color by default
                    let color = TranslateSaberColor(value);
                    for i in 0..MAX_BLADES as c_int {
                        s.blade[i as usize].color = color;
                    }
                } else {
                    s.blade[n as usize].color = TranslateSaberColor(value);
                }
                continue;
            }

            // saber length
            if crate::q_shared::Q_stricmpn(tok, c"saberLength".as_ptr(), 11) == 0 {
                let toklen = std::ffi::CStr::from_ptr(tok).to_bytes().len();
                let n: c_int;
                if toklen == 11 {
                    n = -1;
                } else if toklen == 12 {
                    let idx = c_atoi(tok.offset(11)) - 1;
                    if idx > 7 || idx < 1 {
                        let msg = format!(
                            "WARNING: bad saberLength '{}' in {}\n",
                            cstr_to_str(tok),
                            cstr_to_str(useSaber.as_ptr())
                        );
                        crate::g_main::Com_Printf(cstr(&msg).as_ptr());
                        continue;
                    }
                    n = idx;
                } else {
                    let msg = format!(
                        "WARNING: bad saberLength '{}' in {}\n",
                        cstr_to_str(tok),
                        cstr_to_str(useSaber.as_ptr())
                    );
                    crate::g_main::Com_Printf(cstr(&msg).as_ptr());
                    continue;
                }

                let mut f: f32 = 0.0;
                parse_float_field!(f);
                if f < 4.0 {
                    f = 4.0;
                }

                if n == -1 {
                    for i in 0..MAX_BLADES as c_int {
                        s.blade[i as usize].lengthMax = f;
                    }
                } else {
                    s.blade[n as usize].lengthMax = f;
                }
                continue;
            }

            // blade radius
            if crate::q_shared::Q_stricmpn(tok, c"saberRadius".as_ptr(), 11) == 0 {
                let toklen = std::ffi::CStr::from_ptr(tok).to_bytes().len();
                let n: c_int;
                if toklen == 11 {
                    n = -1;
                } else if toklen == 12 {
                    let idx = c_atoi(tok.offset(11)) - 1;
                    if idx > 7 || idx < 1 {
                        let msg = format!(
                            "WARNING: bad saberRadius '{}' in {}\n",
                            cstr_to_str(tok),
                            cstr_to_str(useSaber.as_ptr())
                        );
                        crate::g_main::Com_Printf(cstr(&msg).as_ptr());
                        continue;
                    }
                    n = idx;
                } else {
                    let msg = format!(
                        "WARNING: bad saberRadius '{}' in {}\n",
                        cstr_to_str(tok),
                        cstr_to_str(useSaber.as_ptr())
                    );
                    crate::g_main::Com_Printf(cstr(&msg).as_ptr());
                    continue;
                }

                let mut f: f32 = 0.0;
                parse_float_field!(f);
                if f < 0.25 {
                    f = 0.25;
                }
                if n == -1 {
                    for i in 0..MAX_BLADES as c_int {
                        s.blade[i as usize].radius = f;
                    }
                } else {
                    s.blade[n as usize].radius = f;
                }
                continue;
            }

            // locked saber style
            if qstricmp_eq(tok, c"saberStyle") {
                let value = parse_string_field!();
                // OLD WAY: only allowed ONE style
                let style = TranslateSaberStyle(value) as c_int;
                // learn only this style
                s.stylesLearned = 1 << style;
                // forbid all other styles
                s.stylesForbidden = 0;
                let mut styleNum = saber_styles_t::SS_NONE as c_int + 1;
                while styleNum < saber_styles_t::SS_NUM_SABER_STYLES as c_int {
                    if styleNum != style {
                        s.stylesForbidden |= 1 << styleNum;
                    }
                    styleNum += 1;
                }
                continue;
            }

            // learned saber style
            if qstricmp_eq(tok, c"saberStyleLearned") {
                let value = parse_string_field!();
                s.stylesLearned |= 1 << (TranslateSaberStyle(value) as c_int);
                continue;
            }

            // forbidden saber style
            if qstricmp_eq(tok, c"saberStyleForbidden") {
                let value = parse_string_field!();
                s.stylesForbidden |= 1 << (TranslateSaberStyle(value) as c_int);
                continue;
            }

            // maxChain
            if qstricmp_eq(tok, c"maxChain") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                s.maxChain = n;
                continue;
            }

            // lockable
            if qstricmp_eq(tok, c"lockable") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n == 0 {
                    s.saberFlags |= SFL_NOT_LOCKABLE;
                }
                continue;
            }

            // throwable
            if qstricmp_eq(tok, c"throwable") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n == 0 {
                    s.saberFlags |= SFL_NOT_THROWABLE;
                }
                continue;
            }

            // disarmable
            if qstricmp_eq(tok, c"disarmable") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n == 0 {
                    s.saberFlags |= SFL_NOT_DISARMABLE;
                }
                continue;
            }

            // active blocking
            if qstricmp_eq(tok, c"blocking") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n == 0 {
                    s.saberFlags |= SFL_NOT_ACTIVE_BLOCKING;
                }
                continue;
            }

            // twoHanded
            if qstricmp_eq(tok, c"twoHanded") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags |= SFL_TWO_HANDED;
                }
                continue;
            }

            // force power restrictions
            if qstricmp_eq(tok, c"forceRestrict") {
                let value = parse_string_field!();
                let fp = crate::q_shared::GetIDForString(FPTable, value);
                if fp >= FP_FIRST && fp < NUM_FORCE_POWERS {
                    s.forceRestrictions |= 1 << fp;
                }
                continue;
            }

            // lockBonus
            if qstricmp_eq(tok, c"lockBonus") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                s.lockBonus = n;
                continue;
            }

            // parryBonus
            if qstricmp_eq(tok, c"parryBonus") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                s.parryBonus = n;
                continue;
            }

            // breakParryBonus
            if qstricmp_eq(tok, c"breakParryBonus") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                s.breakParryBonus = n;
                continue;
            }

            // breakParryBonus2
            if qstricmp_eq(tok, c"breakParryBonus2") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                s.breakParryBonus2 = n;
                continue;
            }

            // disarmBonus
            if qstricmp_eq(tok, c"disarmBonus") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                s.disarmBonus = n;
                continue;
            }

            // disarmBonus2
            if qstricmp_eq(tok, c"disarmBonus2") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                s.disarmBonus2 = n;
                continue;
            }

            // single blade saber style
            if qstricmp_eq(tok, c"singleBladeStyle") {
                let value = parse_string_field!();
                s.singleBladeStyle = TranslateSaberStyle(value);
                continue;
            }

            // single blade throwable
            if qstricmp_eq(tok, c"singleBladeThrowable") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags |= SFL_SINGLE_BLADE_THROWABLE;
                }
                continue;
            }

            // broken replacement saber1 (right hand)
            if qstricmp_eq(tok, c"brokenSaber1") {
                let _value = parse_string_field!();
                // saber->brokenSaber1 = G_NewString( value ); -- field not present in this port
                continue;
            }

            // broken replacement saber2 (left hand)
            if qstricmp_eq(tok, c"brokenSaber2") {
                let _value = parse_string_field!();
                // saber->brokenSaber2 = G_NewString( value ); -- field not present in this port
                continue;
            }

            // spins and does damage on return from saberthrow
            if qstricmp_eq(tok, c"returnDamage") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags |= SFL_RETURN_DAMAGE;
                }
                continue;
            }

            // spin sound (when thrown)
            if qstricmp_eq(tok, c"spinSound") {
                let value = parse_string_field!();
                s.spinSound = BG_SoundIndex(value as *mut c_char);
                continue;
            }

            // swing sound - NOTE: must provide all 3!!!
            if qstricmp_eq(tok, c"swingSound1") {
                let value = parse_string_field!();
                s.swingSound[0] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"swingSound2") {
                let value = parse_string_field!();
                s.swingSound[1] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"swingSound3") {
                let value = parse_string_field!();
                s.swingSound[2] = BG_SoundIndex(value as *mut c_char);
                continue;
            }

            // you move faster/slower when using this saber
            if qstricmp_eq(tok, c"moveSpeedScale") {
                let mut f: f32 = 0.0;
                parse_float_field!(f);
                s.moveSpeedScale = f;
                continue;
            }

            // plays normal attack animations faster/slower
            if qstricmp_eq(tok, c"animSpeedScale") {
                let mut f: f32 = 0.0;
                parse_float_field!(f);
                s.animSpeedScale = f;
                continue;
            }

            // if non-zero, the saber will bounce back when it hits solid architecture
            if qstricmp_eq(tok, c"bounceOnWalls") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags |= SFL_BOUNCE_ON_WALLS;
                }
                continue;
            }

            // if set, saber model is bolted to wrist, not in hand
            if qstricmp_eq(tok, c"boltToWrist") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags |= SFL_BOLT_TO_WRIST;
                }
                continue;
            }

            // kata move
            if qstricmp_eq(tok, c"kataMove") {
                let value = parse_string_field!();
                saberMove = crate::q_shared::GetIDForString(SaberMoveTable, value);
                if saberMove >= LS_INVALID && saberMove < LS_MOVE_MAX {
                    s.kataMove = saberMove;
                }
                continue;
            }
            // lungeAtkMove move
            if qstricmp_eq(tok, c"lungeAtkMove") {
                let value = parse_string_field!();
                saberMove = crate::q_shared::GetIDForString(SaberMoveTable, value);
                if saberMove >= LS_INVALID && saberMove < LS_MOVE_MAX {
                    s.lungeAtkMove = saberMove;
                }
                continue;
            }
            // jumpAtkUpMove move
            if qstricmp_eq(tok, c"jumpAtkUpMove") {
                let value = parse_string_field!();
                saberMove = crate::q_shared::GetIDForString(SaberMoveTable, value);
                if saberMove >= LS_INVALID && saberMove < LS_MOVE_MAX {
                    s.jumpAtkUpMove = saberMove;
                }
                continue;
            }
            // jumpAtkFwdMove move
            if qstricmp_eq(tok, c"jumpAtkFwdMove") {
                let value = parse_string_field!();
                saberMove = crate::q_shared::GetIDForString(SaberMoveTable, value);
                if saberMove >= LS_INVALID && saberMove < LS_MOVE_MAX {
                    s.jumpAtkFwdMove = saberMove;
                }
                continue;
            }
            // jumpAtkBackMove move
            if qstricmp_eq(tok, c"jumpAtkBackMove") {
                let value = parse_string_field!();
                saberMove = crate::q_shared::GetIDForString(SaberMoveTable, value);
                if saberMove >= LS_INVALID && saberMove < LS_MOVE_MAX {
                    s.jumpAtkBackMove = saberMove;
                }
                continue;
            }
            // jumpAtkRightMove move
            if qstricmp_eq(tok, c"jumpAtkRightMove") {
                let value = parse_string_field!();
                saberMove = crate::q_shared::GetIDForString(SaberMoveTable, value);
                if saberMove >= LS_INVALID && saberMove < LS_MOVE_MAX {
                    s.jumpAtkRightMove = saberMove;
                }
                continue;
            }
            // jumpAtkLeftMove move
            if qstricmp_eq(tok, c"jumpAtkLeftMove") {
                let value = parse_string_field!();
                saberMove = crate::q_shared::GetIDForString(SaberMoveTable, value);
                if saberMove >= LS_INVALID && saberMove < LS_MOVE_MAX {
                    s.jumpAtkLeftMove = saberMove;
                }
                continue;
            }
            // readyAnim
            if qstricmp_eq(tok, c"readyAnim") {
                let value = parse_string_field!();
                anim = crate::q_shared::GetIDForString(animTable, value);
                if anim >= 0 && anim < (animNumber_t::MAX_ANIMATIONS as c_int) {
                    s.readyAnim = anim;
                }
                continue;
            }
            // drawAnim
            if qstricmp_eq(tok, c"drawAnim") {
                let value = parse_string_field!();
                anim = crate::q_shared::GetIDForString(animTable, value);
                if anim >= 0 && anim < (animNumber_t::MAX_ANIMATIONS as c_int) {
                    s.drawAnim = anim;
                }
                continue;
            }
            // putawayAnim
            if qstricmp_eq(tok, c"putawayAnim") {
                let value = parse_string_field!();
                anim = crate::q_shared::GetIDForString(animTable, value);
                if anim >= 0 && anim < (animNumber_t::MAX_ANIMATIONS as c_int) {
                    s.putawayAnim = anim;
                }
                continue;
            }
            // tauntAnim
            if qstricmp_eq(tok, c"tauntAnim") {
                let value = parse_string_field!();
                anim = crate::q_shared::GetIDForString(animTable, value);
                if anim >= 0 && anim < (animNumber_t::MAX_ANIMATIONS as c_int) {
                    s.tauntAnim = anim;
                }
                continue;
            }
            // bowAnim
            if qstricmp_eq(tok, c"bowAnim") {
                let value = parse_string_field!();
                anim = crate::q_shared::GetIDForString(animTable, value);
                if anim >= 0 && anim < (animNumber_t::MAX_ANIMATIONS as c_int) {
                    s.bowAnim = anim;
                }
                continue;
            }
            // meditateAnim
            if qstricmp_eq(tok, c"meditateAnim") {
                let value = parse_string_field!();
                anim = crate::q_shared::GetIDForString(animTable, value);
                if anim >= 0 && anim < (animNumber_t::MAX_ANIMATIONS as c_int) {
                    s.meditateAnim = anim;
                }
                continue;
            }
            // flourishAnim
            if qstricmp_eq(tok, c"flourishAnim") {
                let value = parse_string_field!();
                anim = crate::q_shared::GetIDForString(animTable, value);
                if anim >= 0 && anim < (animNumber_t::MAX_ANIMATIONS as c_int) {
                    s.flourishAnim = anim;
                }
                continue;
            }
            // gloatAnim
            if qstricmp_eq(tok, c"gloatAnim") {
                let value = parse_string_field!();
                anim = crate::q_shared::GetIDForString(animTable, value);
                if anim >= 0 && anim < (animNumber_t::MAX_ANIMATIONS as c_int) {
                    s.gloatAnim = anim;
                }
                continue;
            }

            // if set, cannot do roll-stab move at end of roll
            if qstricmp_eq(tok, c"noRollStab") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags |= SFL_NO_ROLL_STAB;
                }
                continue;
            }
            // if set, cannot do pull+attack move
            if qstricmp_eq(tok, c"noPullAttack") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags |= SFL_NO_PULL_ATTACK;
                }
                continue;
            }
            // if set, cannot do back-stab moves
            if qstricmp_eq(tok, c"noBackAttack") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags |= SFL_NO_BACK_ATTACK;
                }
                continue;
            }
            // if set, cannot do stabdown move
            if qstricmp_eq(tok, c"noStabDown") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags |= SFL_NO_STABDOWN;
                }
                continue;
            }
            // if set, cannot side-run or forward-run on walls
            if qstricmp_eq(tok, c"noWallRuns") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags |= SFL_NO_WALL_RUNS;
                }
                continue;
            }
            // if set, cannot do backflip off wall or side-flips off walls
            if qstricmp_eq(tok, c"noWallFlips") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags |= SFL_NO_WALL_FLIPS;
                }
                continue;
            }
            // if set, cannot grab wall & jump off
            if qstricmp_eq(tok, c"noWallGrab") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags |= SFL_NO_WALL_GRAB;
                }
                continue;
            }
            // if set, cannot roll
            if qstricmp_eq(tok, c"noRolls") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags |= SFL_NO_ROLLS;
                }
                continue;
            }
            // if set, cannot do flips
            if qstricmp_eq(tok, c"noFlips") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags |= SFL_NO_FLIPS;
                }
                continue;
            }
            // if set, cannot do cartwheels
            if qstricmp_eq(tok, c"noCartwheels") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags |= SFL_NO_CARTWHEELS;
                }
                continue;
            }
            // if set, cannot do kicks
            if qstricmp_eq(tok, c"noKicks") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags |= SFL_NO_KICKS;
                }
                continue;
            }
            // if set, cannot do the simultaneous attack left/right moves
            if qstricmp_eq(tok, c"noMirrorAttacks") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags |= SFL_NO_MIRROR_ATTACKS;
                }
                continue;
            }

            // stays on in water
            if qstricmp_eq(tok, c"onInWater") {
                // ignore in MP
                crate::q_shared::SkipRestOfLine(&mut p);
                continue;
            }

            if qstricmp_eq(tok, c"notInMP") {
                // ignore this
                crate::q_shared::SkipRestOfLine(&mut p);
                continue;
            }

            // ===ABOVE THIS, ALL VALUES ARE GLOBAL TO THE SABER========================================================
            // bladeStyle2Start - where to start using the second set of blade data
            if qstricmp_eq(tok, c"bladeStyle2Start") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                s.bladeStyle2Start = n;
                continue;
            }
            // ===BLADE-SPECIFIC FIELDS=================================================================================

            // ===PRIMARY BLADE====================================
            if qstricmp_eq(tok, c"noWallMarks") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_NO_WALL_MARKS;
                }
                continue;
            }
            if qstricmp_eq(tok, c"noDlight") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_NO_DLIGHT;
                }
                continue;
            }
            if qstricmp_eq(tok, c"noBlade") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_NO_BLADE;
                }
                continue;
            }
            if qstricmp_eq(tok, c"trailStyle") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                s.trailStyle = n;
                continue;
            }
            // `g2MarksShader`/`g2WeaponMarkShader`: Raven's `#ifdef QAGAME` branch
            // is `SkipRestOfLine` (this crate is QAGAME/jampgame); the
            // `trap_R_RegisterShader` call is CGAME-only dead code here (§20).
            if qstricmp_eq(tok, c"g2MarksShader") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(&mut p, &mut value) != QFALSE {
                    crate::q_shared::SkipRestOfLine(&mut p);
                    continue;
                }
                crate::q_shared::SkipRestOfLine(&mut p);
                continue;
            }
            if qstricmp_eq(tok, c"g2WeaponMarkShader") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(&mut p, &mut value) != QFALSE {
                    crate::q_shared::SkipRestOfLine(&mut p);
                    continue;
                }
                crate::q_shared::SkipRestOfLine(&mut p);
                continue;
            }
            if qstricmp_eq(tok, c"knockbackScale") {
                let mut f: f32 = 0.0;
                parse_float_field!(f);
                s.knockbackScale = f;
                continue;
            }
            if qstricmp_eq(tok, c"damageScale") {
                let mut f: f32 = 0.0;
                parse_float_field!(f);
                s.damageScale = f;
                continue;
            }
            if qstricmp_eq(tok, c"noDismemberment") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_NO_DISMEMBERMENT;
                }
                continue;
            }
            if qstricmp_eq(tok, c"noIdleEffect") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_NO_IDLE_EFFECT;
                }
                continue;
            }
            if qstricmp_eq(tok, c"alwaysBlock") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_ALWAYS_BLOCK;
                }
                continue;
            }
            if qstricmp_eq(tok, c"noManualDeactivate") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_NO_MANUAL_DEACTIVATE;
                }
                continue;
            }
            if qstricmp_eq(tok, c"transitionDamage") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_TRANSITION_DAMAGE;
                }
                continue;
            }
            if qstricmp_eq(tok, c"splashRadius") {
                let mut f: f32 = 0.0;
                parse_float_field!(f);
                s.splashRadius = f;
                continue;
            }
            if qstricmp_eq(tok, c"splashDamage") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                s.splashDamage = n;
                continue;
            }
            if qstricmp_eq(tok, c"splashKnockback") {
                let mut f: f32 = 0.0;
                parse_float_field!(f);
                s.splashKnockback = f;
                continue;
            }
            if qstricmp_eq(tok, c"hitSound1") {
                let value = parse_string_field!();
                s.hitSound[0] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"hitSound2") {
                let value = parse_string_field!();
                s.hitSound[1] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"hitSound3") {
                let value = parse_string_field!();
                s.hitSound[2] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"blockSound1") {
                let value = parse_string_field!();
                s.blockSound[0] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"blockSound2") {
                let value = parse_string_field!();
                s.blockSound[1] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"blockSound3") {
                let value = parse_string_field!();
                s.blockSound[2] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"bounceSound1") {
                let value = parse_string_field!();
                s.bounceSound[0] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"bounceSound2") {
                let value = parse_string_field!();
                s.bounceSound[1] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"bounceSound3") {
                let value = parse_string_field!();
                s.bounceSound[2] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            // block/hitPerson/hitOther/blade effects: QAGAME branch is
            // SkipRestOfLine (CGAME-only trap_FX_RegisterEffect dead here, §20).
            if qstricmp_eq(tok, c"blockEffect") {
                let _value = parse_string_field!();
                continue;
            }
            if qstricmp_eq(tok, c"hitPersonEffect") {
                let _value = parse_string_field!();
                continue;
            }
            if qstricmp_eq(tok, c"hitOtherEffect") {
                let _value = parse_string_field!();
                continue;
            }
            if qstricmp_eq(tok, c"bladeEffect") {
                let _value = parse_string_field!();
                continue;
            }
            if qstricmp_eq(tok, c"noClashFlare") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_NO_CLASH_FLARE;
                }
                continue;
            }

            // ===SECONDARY BLADE====================================
            if qstricmp_eq(tok, c"noWallMarks2") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_NO_WALL_MARKS2;
                }
                continue;
            }
            if qstricmp_eq(tok, c"noDlight2") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_NO_DLIGHT2;
                }
                continue;
            }
            if qstricmp_eq(tok, c"noBlade2") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_NO_BLADE2;
                }
                continue;
            }
            if qstricmp_eq(tok, c"trailStyle2") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                s.trailStyle2 = n;
                continue;
            }
            if qstricmp_eq(tok, c"g2MarksShader2") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(&mut p, &mut value) != QFALSE {
                    crate::q_shared::SkipRestOfLine(&mut p);
                    continue;
                }
                crate::q_shared::SkipRestOfLine(&mut p);
                continue;
            }
            if qstricmp_eq(tok, c"g2WeaponMarkShader2") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(&mut p, &mut value) != QFALSE {
                    crate::q_shared::SkipRestOfLine(&mut p);
                    continue;
                }
                crate::q_shared::SkipRestOfLine(&mut p);
                continue;
            }
            if qstricmp_eq(tok, c"knockbackScale2") {
                let mut f: f32 = 0.0;
                parse_float_field!(f);
                s.knockbackScale2 = f;
                continue;
            }
            if qstricmp_eq(tok, c"damageScale2") {
                let mut f: f32 = 0.0;
                parse_float_field!(f);
                s.damageScale2 = f;
                continue;
            }
            if qstricmp_eq(tok, c"noDismemberment2") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_NO_DISMEMBERMENT2;
                }
                continue;
            }
            if qstricmp_eq(tok, c"noIdleEffect2") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_NO_IDLE_EFFECT2;
                }
                continue;
            }
            if qstricmp_eq(tok, c"alwaysBlock2") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_ALWAYS_BLOCK2;
                }
                continue;
            }
            if qstricmp_eq(tok, c"noManualDeactivate2") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_NO_MANUAL_DEACTIVATE2;
                }
                continue;
            }
            if qstricmp_eq(tok, c"transitionDamage2") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_TRANSITION_DAMAGE2;
                }
                continue;
            }
            if qstricmp_eq(tok, c"splashRadius2") {
                let mut f: f32 = 0.0;
                parse_float_field!(f);
                s.splashRadius2 = f;
                continue;
            }
            if qstricmp_eq(tok, c"splashDamage2") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                s.splashDamage2 = n;
                continue;
            }
            if qstricmp_eq(tok, c"splashKnockback2") {
                let mut f: f32 = 0.0;
                parse_float_field!(f);
                s.splashKnockback2 = f;
                continue;
            }
            if qstricmp_eq(tok, c"hit2Sound1") {
                let value = parse_string_field!();
                s.hit2Sound[0] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"hit2Sound2") {
                let value = parse_string_field!();
                s.hit2Sound[1] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"hit2Sound3") {
                let value = parse_string_field!();
                s.hit2Sound[2] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"block2Sound1") {
                let value = parse_string_field!();
                s.block2Sound[0] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"block2Sound2") {
                let value = parse_string_field!();
                s.block2Sound[1] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"block2Sound3") {
                let value = parse_string_field!();
                s.block2Sound[2] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"bounce2Sound1") {
                let value = parse_string_field!();
                s.bounce2Sound[0] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"bounce2Sound2") {
                let value = parse_string_field!();
                s.bounce2Sound[1] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"bounce2Sound3") {
                let value = parse_string_field!();
                s.bounce2Sound[2] = BG_SoundIndex(value as *mut c_char);
                continue;
            }
            if qstricmp_eq(tok, c"blockEffect2") {
                let _value = parse_string_field!();
                continue;
            }
            if qstricmp_eq(tok, c"hitPersonEffect2") {
                let _value = parse_string_field!();
                continue;
            }
            if qstricmp_eq(tok, c"hitOtherEffect2") {
                let _value = parse_string_field!();
                continue;
            }
            if qstricmp_eq(tok, c"bladeEffect2") {
                let _value = parse_string_field!();
                continue;
            }
            if qstricmp_eq(tok, c"noClashFlare2") {
                let mut n: c_int = 0;
                parse_int_field!(n);
                if n != 0 {
                    s.saberFlags2 |= SFL2_NO_CLASH_FLARE2;
                }
                continue;
            }
            // ===END BLADE-SPECIFIC FIELDS=============================================================================

            // FIXME: saber sounds (on, off, loop)

            #[cfg(debug_assertions)]
            {
                let msg = format!(
                    "WARNING: unknown keyword '{}' while parsing '{}'\n",
                    cstr_to_str(tok),
                    cstr_to_str(useSaber.as_ptr())
                );
                crate::g_main::Com_Printf(cstr(&msg).as_ptr());
            }
            crate::q_shared::SkipRestOfLine(&mut p);
        }

        // FIXME: precache the saberModel(s)?

        QTRUE
    }
}

/// Raven `WP_SaberParseParm`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2574-2645`
// SHAPE-MISMATCH: this fn gained `bg: &mut BgState` (rulings 12/15) but its
// existing caller `WP_SaberValidForPlayerInMP` (out of this packet's scope,
// already ported above) still calls it with the old 3-arg shape — reported
// in shape_mismatches for the integration pass.
pub fn WP_SaberParseParm(
    saberName: *const c_char,
    parmname: *const c_char,
    saberData: *mut c_char,
    bg: &mut BgState,
) -> qboolean {
    unsafe {
        if saberName.is_null() || *saberName == 0 {
            return QFALSE;
        }

        if bg.SaberParms.is_empty() {
            bg.SaberParms.push(0);
        }

        // try to parse it out
        let mut p: *const c_char = bg.SaberParms.as_ptr() as *const c_char;
        crate::q_shared::COM_BeginParseSession(c"saberinfo".as_ptr());

        // look for the right saber
        loop {
            if p.is_null() {
                return QFALSE;
            }
            let token = crate::q_shared::COM_ParseExt(&mut p, QTRUE);
            if *token == 0 {
                return QFALSE;
            }

            if crate::q_shared::Q_stricmp(token as *const c_char, saberName) == 0 {
                break;
            }

            crate::q_shared::SkipBracedSection(&mut p);
        }
        if p.is_null() {
            return QFALSE;
        }

        if BG_ParseLiteral(&mut p, c"{".as_ptr()) != QFALSE {
            return QFALSE;
        }

        // parse the saber info block
        loop {
            let token = crate::q_shared::COM_ParseExt(&mut p, QTRUE);
            if *token == 0 {
                let s = format!(
                    "ERROR: unexpected EOF while parsing '{}'\n",
                    cstr_to_str(saberName)
                );
                crate::g_main::Com_Printf(cstr(&s).as_ptr());
                return QFALSE;
            }

            if qstricmp_eq(token as *const c_char, c"}") {
                break;
            }

            if crate::q_shared::Q_stricmp(token as *const c_char, parmname) == 0 {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                c_strcpy(saberData, value);
                return QTRUE;
            }

            crate::q_shared::SkipRestOfLine(&mut p);
        }

        QFALSE
    }
}

/// Raven `WP_SaberValidForPlayerInMP`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2647-2662`
pub fn WP_SaberValidForPlayerInMP(saberName: *const c_char) -> qboolean {
    unsafe {
        let mut allowed: [c_char; 8] = [0; 8];
        if WP_SaberParseParm(saberName, c"notInMP".as_ptr(), allowed.as_mut_ptr()) == QFALSE {
            // not defined, default is yes
            return QTRUE;
        }
        if allowed[0] == 0 {
            // not defined, default is yes
            return QTRUE;
        }
        // return value
        if c_atoi(allowed.as_ptr()) == 0 {
            QTRUE
        } else {
            QFALSE
        }
    }
}

/// Raven `WP_RemoveSaber`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2664-2688`
pub fn WP_RemoveSaber(sabers: *mut saberInfo_t, saberNum: c_int) {
    if sabers.is_null() {
        return;
    }
    unsafe {
        let entry = sabers.offset(saberNum as isize);
        // reset everything for this saber just in case
        WP_SaberSetDefaults(entry);

        c_strcpy((*entry).name.as_mut_ptr(), c"none".as_ptr());
        (*entry).model[0] = 0;

        BG_SI_Deactivate(entry);
        BG_SI_SetLength(entry, 0.0);
    }
}

/// Raven `WP_SetSaber`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2690-2725`
pub fn WP_SetSaber(
    entNum: c_int,
    sabers: *mut saberInfo_t,
    saberNum: c_int,
    saberName: *const c_char,
    bg: &mut BgState,
    traps: &dyn BgTraps,
) {
    unsafe {
        if sabers.is_null() {
            return;
        }
        if qstricmp_eq(saberName, c"none") || qstricmp_eq(saberName, c"remove") {
            if saberNum != 0 {
                // can't remove saber 0 ever
                WP_RemoveSaber(sabers, saberNum);
            }
            return;
        }

        if entNum < MAX_CLIENTS_I32 && WP_SaberValidForPlayerInMP(saberName) == QFALSE {
            WP_SaberParseParms(c"Kyle".as_ptr(), sabers.offset(saberNum as isize), bg, traps); // get saber info
        } else {
            WP_SaberParseParms(saberName, sabers.offset(saberNum as isize), bg, traps); // get saber info
        }
        if ((*sabers.offset(1)).saberFlags & SFL_TWO_HANDED) != 0 {
            // not allowed to use a 2-handed saber as second saber
            WP_RemoveSaber(sabers, 1);
            return;
        } else if ((*sabers.offset(0)).saberFlags & SFL_TWO_HANDED) != 0
            && (*sabers.offset(1)).model[0] != 0
        {
            // you can't use a two-handed saber with a second saber, so remove saber 2
            WP_RemoveSaber(sabers, 1);
        }
    }
}

/// Raven `WP_SaberSetColor`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2727-2734`
pub fn WP_SaberSetColor(
    sabers: *mut saberInfo_t,
    saberNum: c_int,
    bladeNum: c_int,
    colorName: *mut c_char,
) {
    if sabers.is_null() {
        return;
    }
    unsafe {
        let entry = sabers.offset(saberNum as isize);
        (*entry).blade[bladeNum as usize].color = TranslateSaberColor(colorName as *const c_char);
    }
}

/// Raven `WP_SaberLoadParms`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2738-2790`
// PORT-NOTE(vec-as-fixed-buffer): Raven's `SaberParms`/`bgSaberParseTBuffer`
// are fixed `char[MAX_SABER_DATA_SIZE]` statics; `BgState` owns them as
// growable `Vec<u8>` (ruling 12/§9). Pre-sized here to `MAX_SABER_DATA_SIZE`
// so the pointer-arithmetic parse below (raw `*mut c_char` into the backing
// storage) stays faithful to Raven's fixed-buffer indexing.
pub fn WP_SaberLoadParms(bg: &mut BgState, traps: &dyn BgTraps) {
    unsafe {
        if bg.SaberParms.len() < MAX_SABER_DATA_SIZE {
            bg.SaberParms.resize(MAX_SABER_DATA_SIZE, 0);
        }
        if bg.bgSaberParseTBuffer.len() < MAX_SABER_DATA_SIZE {
            bg.bgSaberParseTBuffer.resize(MAX_SABER_DATA_SIZE, 0);
        }

        let mut totallen: c_int;
        let mainBlockLen: c_int;
        let mut len: c_int = 0;

        // remember where to store the next one
        totallen = len;
        mainBlockLen = len;
        let mut marker: *mut c_char = bg.SaberParms.as_mut_ptr().offset(totallen as isize) as *mut c_char;
        *marker = 0;

        // now load in the extra .sab extensions
        let mut saberExtensionListBuf: [c_char; 2048] = [0; 2048];
        let path = cstr("ext_data/sabers");
        let ext = cstr(".sab");
        let fileCnt = traps.fs_getfilelist(
            path.as_ptr(),
            ext.as_ptr(),
            saberExtensionListBuf.as_mut_ptr(),
            saberExtensionListBuf.len() as c_int,
        );

        let mut holdChar: *const c_char = saberExtensionListBuf.as_ptr();
        let mut i = 0;
        while i < fileCnt {
            let saberExtFNLen = std::ffi::CStr::from_ptr(holdChar).to_bytes().len() as c_int;

            let name = cstr_to_str(holdChar);
            let path_s = format!("ext_data/sabers/{}", name);
            let path_c = cstr(&path_s);
            let mut f: fileHandle_t = 0;
            len = traps.fs_fopen(path_c.as_ptr(), &mut f, FS_READ);

            if len == -1 {
                crate::g_main::Com_Printf(c"error reading file\n".as_ptr());
            } else {
                if (totallen + len + 1 /* for the endline */) >= MAX_SABER_DATA_SIZE as c_int {
                    crate::g_main::Com_Error(
                        ERR_DROP,
                        c"Saber extensions (*.sab) are too large".as_ptr(),
                    );
                }

                traps.fs_read(
                    bg.bgSaberParseTBuffer.as_mut_ptr() as *mut c_void,
                    len,
                    f,
                );
                bg.bgSaberParseTBuffer[len as usize] = 0;

                len = crate::q_shared::COM_Compress(bg.bgSaberParseTBuffer.as_mut_ptr() as *mut c_char);

                marker = bg.SaberParms.as_mut_ptr().offset(totallen as isize) as *mut c_char;
                crate::q_shared::Q_strcat(
                    marker,
                    MAX_SABER_DATA_SIZE as c_int - totallen,
                    bg.bgSaberParseTBuffer.as_ptr() as *const c_char,
                );
                traps.fs_fclose(f);

                // get around the stupid problem of not having an endline at the bottom
                // of a sab file -rww
                crate::q_shared::Q_strcat(
                    marker,
                    MAX_SABER_DATA_SIZE as c_int - totallen,
                    c"\n".as_ptr(),
                );
                len += 1;

                totallen += len;
                marker = bg.SaberParms.as_mut_ptr().offset(totallen as isize) as *mut c_char;
            }

            holdChar = holdChar.offset((saberExtFNLen + 1) as isize);
            i += 1;
        }
        let _ = mainBlockLen;
    }
}

// rww - The following were struct functions in SP. Of course we can't have
// that in this codebase so I'm having to externalize them. Which is why
// this probably seems structured a bit oddly. But it's to make porting
// stuff easier on myself. SI indicates it was under saberinfo, and BLADE
// indicates it was under bladeinfo.

/// Raven `BG_BLADE_ActivateTrail`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2803-2807`
pub fn BG_BLADE_ActivateTrail(blade: *mut bladeInfo_t, duration: f32) {
    unsafe {
        (*blade).trail.inAction = QTRUE;
        // Raven's `saberTrail_t::duration` is `int`; the float `duration`
        // parameter truncates on assignment, matching C's implicit
        // narrowing conversion.
        (*blade).trail.duration = duration as c_int;
    }
}

/// Raven `BG_BLADE_DeactivateTrail`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2809-2813`
pub fn BG_BLADE_DeactivateTrail(blade: *mut bladeInfo_t, duration: f32) {
    unsafe {
        (*blade).trail.inAction = QFALSE;
        (*blade).trail.duration = duration as c_int;
    }
}

/// Raven `BG_SI_Activate`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2815-2823`
pub fn BG_SI_Activate(saber: *mut saberInfo_t) {
    unsafe {
        for i in 0..(*saber).numBlades {
            (*saber).blade[i as usize].active = QTRUE;
        }
    }
}

/// Raven `BG_SI_Deactivate`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2825-2833`
pub fn BG_SI_Deactivate(saber: *mut saberInfo_t) {
    unsafe {
        for i in 0..(*saber).numBlades {
            (*saber).blade[i as usize].active = QFALSE;
        }
    }
}

/// Raven `BG_SI_BladeActivate`.
///
/// Raven: Activate a specific Blade of this Saber.
/// Created: 10/03/02 by Aurelio Reis, Modified: 10/03/02 by Aurelio Reis.
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2840-2847`
pub fn BG_SI_BladeActivate(saber: *mut saberInfo_t, iBlade: c_int, bActive: qboolean) {
    unsafe {
        // Validate blade ID/Index.
        if iBlade < 0 || iBlade >= (*saber).numBlades {
            return;
        }
        (*saber).blade[iBlade as usize].active = bActive;
    }
}

/// Raven `BG_SI_Active`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2849-2861`
pub fn BG_SI_Active(saber: *mut saberInfo_t) -> qboolean {
    unsafe {
        for i in 0..(*saber).numBlades {
            if (*saber).blade[i as usize].active != QFALSE {
                return QTRUE;
            }
        }
    }
    QFALSE
}

/// Raven `BG_SI_SetLength`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2863-2871`
pub fn BG_SI_SetLength(saber: *mut saberInfo_t, length: f32) {
    unsafe {
        for i in 0..(*saber).numBlades {
            (*saber).blade[i as usize].length = length;
        }
    }
}

/// Raven `BG_SI_SetDesiredLength`.
///
/// Raven: "not in sp, added it for my own convenience".
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2874-2887`
pub fn BG_SI_SetDesiredLength(saber: *mut saberInfo_t, len: f32, bladeNum: c_int) {
    unsafe {
        let mut startBlade = 0;
        let mut maxBlades = (*saber).numBlades;

        if bladeNum >= 0 && bladeNum < (*saber).numBlades {
            // doing this on a specific blade
            startBlade = bladeNum;
            maxBlades = bladeNum + 1;
        }
        for i in startBlade..maxBlades {
            (*saber).blade[i as usize].desiredLength = len;
        }
    }
}

/// Raven `BG_SI_SetLengthGradual`.
///
/// Raven: "also not in sp, added it for my own convenience".
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2890-2957`
pub fn BG_SI_SetLengthGradual(saber: *mut saberInfo_t, time: c_int) {
    unsafe {
        for i in 0..(*saber).numBlades {
            let blade = &mut (*saber).blade[i as usize];
            let mut dLen = blade.desiredLength;

            if dLen == -1.0 {
                // assume we want max blade len
                dLen = blade.lengthMax;
            }

            if blade.length == dLen {
                continue;
            }

            if blade.length == blade.lengthMax || blade.length == 0.0 {
                blade.extendDebounce = time;
                if blade.length == 0.0 {
                    blade.length += 1.0;
                } else {
                    blade.length -= 1.0;
                }
            }

            let mut amt = (time - blade.extendDebounce) as f32 * 0.01;

            if amt < 0.2 {
                amt = 0.2;
            }

            if blade.length < dLen {
                blade.length += amt;

                if blade.length > dLen {
                    blade.length = dLen;
                }
                if blade.length > blade.lengthMax {
                    blade.length = blade.lengthMax;
                }
            } else if blade.length > dLen {
                blade.length -= amt;

                if blade.length < dLen {
                    blade.length = dLen;
                }
                if blade.length < 0.0 {
                    blade.length = 0.0;
                }
            }
        }
    }
}

/// Raven `BG_SI_Length`.
///
/// Raven: return largest length.
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2959-2972`
pub fn BG_SI_Length(saber: *mut saberInfo_t) -> f32 {
    unsafe {
        // Raven's `len1` is `int`; the float blade length truncates on
        // assignment, matching C's implicit narrowing conversion.
        let mut len1: c_int = 0;
        for i in 0..(*saber).numBlades {
            let length = (*saber).blade[i as usize].length;
            if length > len1 as f32 {
                len1 = length as c_int;
            }
        }
        len1 as f32
    }
}

/// Raven `BG_SI_LengthMax`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2974-2987`
pub fn BG_SI_LengthMax(saber: *mut saberInfo_t) -> f32 {
    unsafe {
        let mut len1: c_int = 0;
        for i in 0..(*saber).numBlades {
            let lengthMax = (*saber).blade[i as usize].lengthMax;
            if lengthMax > len1 as f32 {
                len1 = lengthMax as c_int;
            }
        }
        len1 as f32
    }
}

/// Raven `BG_SI_ActivateTrail`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:2989-2998`
pub fn BG_SI_ActivateTrail(saber: *mut saberInfo_t, duration: f32) {
    unsafe {
        for i in 0..(*saber).numBlades {
            BG_BLADE_ActivateTrail(&mut (*saber).blade[i as usize] as *mut bladeInfo_t, duration);
        }
    }
}

/// Raven `BG_SI_DeactivateTrail`.
///
/// Source: `oracle/oracle/codemp/game/bg_saberLoad.c:3000-3009`
pub fn BG_SI_DeactivateTrail(saber: *mut saberInfo_t, duration: f32) {
    unsafe {
        for i in 0..(*saber).numBlades {
            BG_BLADE_DeactivateTrail(
                &mut (*saber).blade[i as usize] as *mut bladeInfo_t,
                duration,
            );
        }
    }
}
