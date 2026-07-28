//! Port of `oracle/codemp/cgame/cg_players.c` — player models, animation, ghoul2 setup and player effects. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_char, c_int, c_short, c_void};
use core::ptr::null_mut;

use mp_bg::bg_misc::BG_GiveMeVectorFromMatrix;
use mp_bg::bg_panimate::{
    BG_FlippingAnim, BG_InDeathAnim, BG_ParseAnimationFile, BG_SaberStartTransAnim,
};
use mp_bg::bg_saber::SFL2_NO_DLIGHT;
use mp_bg::bg_vehicleLoad::{BG_GetVehicleModelName, BG_GetVehicleSkinName};
use mp_bg::cstr_util::cstr_to_str;
use mp_bg::local::{bgToggleableSurfaceDebris, bgToggleableSurfaces, bg_customSiegeSoundNames};
use mp_bg::public::anim_number::animNumber_t;
use mp_bg::public::anim_table::animTable;
use mp_bg::public::configstring::CS_G2BONES;
use mp_bg::public::entity_effects::EF2_HYPERSPACE;
use mp_bg::public::entity_flags::{EF_CONNECTION, EF_DEAD, EF_RAG, EF_TALK};
use mp_bg::public::entity_type::entityType_t;
use mp_bg::public::gametype::{GT_DUEL, GT_POWERDUEL, GT_TEAM};
use mp_bg::public::gender::gender_t;
use mp_bg::public::hyperspace::HYPERSPACE_TIME;
use mp_bg::public::powerup::{PW_BLUEFLAG, PW_NEUTRALFLAG, PW_QUAD, PW_REDFLAG};
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::game::class_t::class_t;
use mp_qshared::common::mp::ghoul2::bone_flags::{
    BONE_ANGLES_POSTMULT, BONE_ANIM_BLEND, BONE_ANIM_OVERRIDE, BONE_ANIM_OVERRIDE_FREEZE,
    BONE_ANIM_OVERRIDE_LOOP,
};
use mp_qshared::common::mp::qcommon::collision_record::{G2Trace_t, MAX_G2_COLLISIONS};
use mp_qshared::common::mp::qcommon::saber::blade_info::MAX_BLADES;
use mp_qshared::common::mp::qcommon::saber::saber_colors::{
    saber_colors_t, SABER_BLUE, SABER_GREEN, SABER_ORANGE, SABER_PURPLE, SABER_RED, SABER_YELLOW,
};
use mp_qshared::common::mp::qcommon::saber::saber_flags::SFL_BOLT_TO_WRIST;
use mp_qshared::common::mp::qcommon::{
    entityState_t, saberInfo_t, sharedRagDollParams_t, sharedRagDollUpdateParams_t,
};
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::com_parse::{COM_ParseExt, COM_ParseString, QSharedScratch};
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, vec3_origin,
    vectoangles, AngleVectors, AnglesToAxis, Distance, MatrixMultiply, VectorClear, VectorInverse,
    VectorLength, VectorNormalize, VectorSet, PITCH, ROLL, YAW,
};
use mp_qshared::shared::q_string::COM_StripExtension;
use mp_qshared::shared::{
    addbezierArgStruct_t, fileHandle_t, mdxaBone_t, orientation_t, qfalse, qhandle_t, qtrue,
    sfxHandle_t, sharedEIKMoveState, sharedERagPhase, vec3_t, CollisionRecord_t, Eorientations,
    CONTENTS_LAVA, CONTENTS_SLIME, CONTENTS_SOLID, CONTENTS_WATER, ENTITYNUM_NONE, ENTITYNUM_WORLD,
    FP_SEE, FS_READ, MASK_SOLID, MAX_CLIENTS, MAX_CLIENTS_I32, MAX_GENTITIES, MAX_QPATH,
};
use native_string::{
    atoi, buf_to_string, cstr, strcat_string, strncpyz_string, Q_stricmp, Q_strncpyz,
};

use crate::bg_channel::{CgBgTraps, CgGameCallbacks};
use crate::cg_ents::ScaleModelAxis;
use crate::cg_main::{CG_ConfigString, CG_Error, CG_Printf, Com_Printf};
use crate::local::centity_s::centity_t;
use crate::local::client_info_t::{
    clientInfo_t, MAX_CUSTOM_DUEL_SOUNDS, MAX_CUSTOM_SIEGE_SOUNDS, MAX_CUSTOM_SOUNDS,
};
use crate::local::lerp_frame_t::lerpFrame_t;
use crate::local::vehicle_id::VehicleId;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

// ---------------------------------------------------------------------------
// File-scope constants and tables
// ---------------------------------------------------------------------------

/// Raven `RF_THIRD_PERSON` — don't draw through eyes, only mirrors (player
/// bodies, chat sprites). `tr_types.h`'s renderfx bits have no ported
/// cross-crate home yet, so this one lands beside its reader (the
/// `RF_DISINTEGRATE1` treatment in `cg_ents.rs`).
/// Source: `oracle/codemp/cgame/tr_types.h:19`
const RF_THIRD_PERSON: c_int = 0x00002;

/// Raven `DEFAULT_FEMALE_SOUNDPATH`.
///
/// Raven: the commented-out original was `"chars/tavion/misc"`.
/// Source: `oracle/codemp/cgame/cg_players.c:797`
const DEFAULT_FEMALE_SOUNDPATH: &str = "chars/mp_generic_female/misc";

/// Raven `DEFAULT_MALE_SOUNDPATH`.
///
/// Raven: the commented-out original was `"chars/kyle/misc"`.
/// Source: `oracle/codemp/cgame/cg_players.c:798`
const DEFAULT_MALE_SOUNDPATH: &str = "chars/mp_generic_male/misc";

/// Raven `MAX_SURF_LIST_SIZE` — the `surfOff`/`surfOn` accumulator bound
/// `CG_ParseSurfsFile` folds its comma-joined surface list into.
/// Source: `oracle/codemp/cgame/cg_players.c:313`
const MAX_SURF_LIST_SIZE: usize = 1024;

/// Raven `MAX_SHIELD_TIME` — a double literal, so the `cg.time + MAX_SHIELD_TIME`
/// it feeds is a double expression that truncates back to `int`.
/// Source: `oracle/codemp/cgame/cg_players.c:5073`
const MAX_SHIELD_TIME: f64 = 2000.0;

/// Raven `MIN_SHIELD_TIME` — same double literal, the shield-alpha divisor.
/// Source: `oracle/codemp/cgame/cg_players.c:5074`
const MIN_SHIELD_TIME: f64 = 2000.0;

/// Raven `cg_customSoundNames[MAX_CUSTOM_SOUNDS]` — the base per-client custom
/// sound set, scanned until the `NULL` sentinel (`None`). The leading `*` is
/// stripped by every reader.
/// Source: `oracle/codemp/cgame/cg_players.c:25-41`
pub static cg_customSoundNames: [Option<&str>; MAX_CUSTOM_SOUNDS] = [
    Some("*death1"),
    Some("*death2"),
    Some("*death3"),
    Some("*jump1"),
    Some("*pain25"),
    Some("*pain50"),
    Some("*pain75"),
    Some("*pain100"),
    Some("*falling1"),
    Some("*choke1"),
    Some("*choke2"),
    Some("*choke3"),
    Some("*gasp"),
    Some("*land1"),
    Some("*taunt"),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

/// Raven `cg_customDuelSoundNames[MAX_CUSTOM_DUEL_SOUNDS]`.
///
/// Raven: Used for DUEL taunts.
/// Source: `oracle/codemp/cgame/cg_players.c:142-158`
pub static cg_customDuelSoundNames: [Option<&str>; MAX_CUSTOM_DUEL_SOUNDS] = [
    // Raven: Say when acquire an enemy when didn't have one before
    Some("*anger1"),
    Some("*anger2"),
    Some("*anger3"),
    // Raven: Say when killed an enemy
    Some("*victory1"),
    Some("*victory2"),
    Some("*victory3"),
    Some("*taunt1"),
    Some("*taunt2"),
    Some("*taunt3"),
    Some("*deflect1"),
    Some("*deflect2"),
    Some("*deflect3"),
    Some("*gloat1"),
    Some("*gloat2"),
    Some("*gloat3"),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

/// Raven `MAX_CUSTOM_COMBAT_SOUNDS`.
/// Source: `oracle/codemp/cgame/cg_local.h:187`
pub const MAX_CUSTOM_COMBAT_SOUNDS: usize = 40;

/// Raven `MAX_CUSTOM_EXTRA_SOUNDS`.
/// Source: `oracle/codemp/cgame/cg_local.h:188`
pub const MAX_CUSTOM_EXTRA_SOUNDS: usize = 40;

/// Raven `MAX_CUSTOM_JEDI_SOUNDS`.
/// Source: `oracle/codemp/cgame/cg_local.h:189`
pub const MAX_CUSTOM_JEDI_SOUNDS: usize = 40;

/// Raven `cg_customCombatSoundNames[MAX_CUSTOM_COMBAT_SOUNDS]` — the enemy/hazard-team supplement set, scanned until the `NULL`
/// sentinel (`None`).
///
/// Raven: (keep numbers in ascending order in order for variant-capping to
/// work).
/// Source: `oracle/codemp/cgame/cg_players.c:47-67`
pub static cg_customCombatSoundNames: [Option<&str>; MAX_CUSTOM_COMBAT_SOUNDS] = [
    Some("*anger1"),
    Some("*anger2"),
    Some("*anger3"),
    Some("*victory1"),
    Some("*victory2"),
    Some("*victory3"),
    Some("*confuse1"),
    Some("*confuse2"),
    Some("*confuse3"),
    Some("*pushed1"),
    Some("*pushed2"),
    Some("*pushed3"),
    Some("*choke1"),
    Some("*choke2"),
    Some("*choke3"),
    Some("*ffwarn"),
    Some("*ffturn"),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

/// Raven `cg_customExtraSoundNames[MAX_CUSTOM_EXTRA_SOUNDS]` — the stormtrooper supplement set, scanned until the `NULL`
/// sentinel (`None`).
///
/// Raven: (keep numbers in ascending order in order for variant-capping to
/// work).
/// Source: `oracle/codemp/cgame/cg_players.c:71-109`
pub static cg_customExtraSoundNames: [Option<&str>; MAX_CUSTOM_EXTRA_SOUNDS] = [
    Some("*chase1"),
    Some("*chase2"),
    Some("*chase3"),
    Some("*cover1"),
    Some("*cover2"),
    Some("*cover3"),
    Some("*cover4"),
    Some("*cover5"),
    Some("*detected1"),
    Some("*detected2"),
    Some("*detected3"),
    Some("*detected4"),
    Some("*detected5"),
    Some("*lost1"),
    Some("*outflank1"),
    Some("*outflank2"),
    Some("*escaping1"),
    Some("*escaping2"),
    Some("*escaping3"),
    Some("*giveup1"),
    Some("*giveup2"),
    Some("*giveup3"),
    Some("*giveup4"),
    Some("*look1"),
    Some("*look2"),
    Some("*sight1"),
    Some("*sight2"),
    Some("*sight3"),
    Some("*sound1"),
    Some("*sound2"),
    Some("*sound3"),
    Some("*suspicious1"),
    Some("*suspicious2"),
    Some("*suspicious3"),
    Some("*suspicious4"),
    Some("*suspicious5"),
    None,
    None,
    None,
    None,
];

/// Raven `cg_customJediSoundNames[MAX_CUSTOM_JEDI_SOUNDS]` — the jedi supplement set, scanned until the `NULL`
/// sentinel (`None`).
///
/// Raven: (keep numbers in ascending order in order for variant-capping to
/// work).
/// Source: `oracle/codemp/cgame/cg_players.c:113-140`
pub static cg_customJediSoundNames: [Option<&str>; MAX_CUSTOM_JEDI_SOUNDS] = [
    Some("*combat1"),
    Some("*combat2"),
    Some("*combat3"),
    Some("*jdetected1"),
    Some("*jdetected2"),
    Some("*jdetected3"),
    Some("*taunt1"),
    Some("*taunt2"),
    Some("*taunt3"),
    Some("*jchase1"),
    Some("*jchase2"),
    Some("*jchase3"),
    Some("*jlost1"),
    Some("*jlost2"),
    Some("*jlost3"),
    Some("*deflect1"),
    Some("*deflect2"),
    Some("*deflect3"),
    Some("*gloat1"),
    Some("*gloat2"),
    Some("*gloat3"),
    Some("*pushfail"),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

/// Raven `RF_FORCE_ENT_ALPHA` — override shader alpha settings. Same
/// no-ported-home story as [`RF_THIRD_PERSON`].
/// Source: `oracle/codemp/cgame/tr_types.h:36`
const RF_FORCE_ENT_ALPHA: c_int = 0x00400;

/// Raven `RF_DISTORTION` — area distortion effect.
///
/// Raven: -rww
/// Source: `oracle/codemp/cgame/tr_types.h:41`
const RF_DISTORTION: c_int = 0x02000;

/// Raven `SHIPSURF_FRONT` — the impact-damage surface indices. `bg_vehicles.h`'s
/// `#define`s have no ported cross-crate home (mp_game keeps its own copy in
/// `g_vehicles.rs`, which cgame must not depend on), so they land beside their
/// reader.
/// Source: `oracle/codemp/game/bg_vehicles.h:426-429`
const SHIPSURF_FRONT: c_int = 0;
const SHIPSURF_BACK: c_int = 1;
const SHIPSURF_RIGHT: c_int = 2;
const SHIPSURF_LEFT: c_int = 3;

/// Raven `JETPACK_MODEL`.
/// Source: `oracle/codemp/cgame/cg_players.c:7889`
const JETPACK_MODEL: &str = "models/weapons2/jetpack/model.glm";

/// Raven `RF_FORCEPOST` — force it to post-render.
///
/// Raven: -rww. Same no-ported-home story as [`RF_THIRD_PERSON`].
/// Source: `oracle/codemp/cgame/tr_types.h:54`
const RF_FORCEPOST: c_int = 0x200000;

/// Raven `cg_effectorStringTable[]` — the ragdoll effectors a drag is allowed
/// to kick around.
///
/// Raven: list of valid ragdoll effectors. The entries Raven commented out
/// ("thoracic", "rhand", "rradiusX", "ceyebrow") stay commented out; the `NULL`
/// terminator its `while` loop walks to becomes the slice length.
/// Source: `oracle/codemp/cgame/cg_players.c:3440-3455`
const cg_effectorStringTable: [&str; 8] = [
    //	"thoracic",
    //	"rhand",
    "lhand", "rtibia", "ltibia", "rtalus", "ltalus", //	"rradiusX",
    "lradiusX", "rfemurX", "lfemurX",
    //	"ceyebrow",
];

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

/// Raven `CG_ColorFromString` — decodes a 1..7 colour index string into an RGB
/// triple; anything outside that range comes back white.
/// Source: `oracle/codemp/cgame/cg_players.c:701-722`
pub fn CG_ColorFromString(v: &str, color: &mut vec3_t) {
    VectorClear(color);

    let val = atoi(v);

    if !(1..=7).contains(&val) {
        VectorSet(color, 1.0, 1.0, 1.0);
        return;
    }

    if val & 1 != 0 {
        color[2] = 1.0;
    }
    if val & 2 != 0 {
        color[1] = 1.0;
    }
    if val & 4 != 0 {
        color[0] = 1.0;
    }
}

/// Raven `CG_ColorFromInt` — the already-parsed twin of
/// [`CG_ColorFromString`].
/// Source: `oracle/codemp/cgame/cg_players.c:729-746`
pub fn CG_ColorFromInt(val: c_int, color: &mut vec3_t) {
    VectorClear(color);

    if !(1..=7).contains(&val) {
        VectorSet(color, 1.0, 1.0, 1.0);
        return;
    }

    if val & 1 != 0 {
        color[2] = 1.0;
    }
    if val & 2 != 0 {
        color[1] = 1.0;
    }
    if val & 4 != 0 {
        color[0] = 1.0;
    }
}

/// Raven `CG_G2SkelForModel` — the animation-set index for a ghoul2 instance,
/// found by swapping the `.gla` filename for its sibling `animation.cfg`.
/// A GLA name with no `/` leaves the index at -1.
/// Source: `oracle/codemp/cgame/cg_players.c:749-767`
pub fn CG_G2SkelForModel(ctx: &mut CgContext, g2: *mut c_void) -> c_int {
    let mut animIndex: c_int = -1;

    let mut GLAName = trap::G2API_GetGLAName(ctx.engine, g2, 0, MAX_QPATH);

    if let Some(slash) = GLAName.rfind('/') {
        GLAName.truncate(slash);
        GLAName.push_str("/animation.cfg");

        let traps = CgBgTraps::new(ctx.engine);
        let mut callbacks = CgGameCallbacks::new(ctx.engine);
        let filename = cstr(&GLAName);

        animIndex = BG_ParseAnimationFile(
            &mut ctx.world.bg_state,
            &traps,
            &mut callbacks,
            filename.as_ptr(),
            null_mut(),
            qfalse,
        );
    }

    animIndex
}

/// Raven `CG_G2EvIndexForModel` — the animation-event-set index for a ghoul2
/// instance, parsed from the `animevents.cfg` beside its `.gla`.
/// Source: `oracle/codemp/cgame/cg_players.c:770-795`
pub fn CG_G2EvIndexForModel(ctx: &mut CgContext, g2: *mut c_void, animIndex: c_int) -> c_int {
    let evtIndex: c_int = -1;

    if animIndex == -1 {
        debug_assert!(false, "shouldn't happen, bad animIndex");
        return -1;
    }

    let mut GLAName = trap::G2API_GetGLAName(ctx.engine, g2, 0, MAX_QPATH);

    if let Some(slash) = GLAName.rfind('/') {
        // Raven cuts just past the slash, leaving the directory with its
        // trailing separator.
        GLAName.truncate(slash + 1);

        // DEFERRED: BG_ParseAnimationEvtFile — `oracle/codemp/game/bg_panimate.c:1756-2328`.
        // The whole anim-event block sits inside `#ifndef QAGAME`, so `mp_bg`
        // deliberately did NOT port it (see that file's module doc); there is
        // no cgame-side home for it yet. Until it lands the index stays -1.
        //   evtIndex = BG_ParseAnimationEvtFile(&GLAName, animIndex, bgNumAnimEvents);
    }

    evtIndex
}

/// Raven `CG_LoadCISounds` — registers a client's custom sound set, following
/// the `sounds*.cfg` redirect (and its `f` gender marker) when the model ships
/// one, otherwise falling back to the generic male/female path.
/// Source: `oracle/codemp/cgame/cg_players.c:799-996`
pub fn CG_LoadCISounds(ctx: &mut CgContext, ci: &mut clientInfo_t, modelloaded: bool) {
    let mut f = 0;
    let mut isFemale = false;
    #[allow(unused_assignments)]
    let mut fLen;

    let dir = buf_to_string(&ci.modelName.iter().map(|&c| c as u8).collect::<Vec<u8>>());
    let skinName = buf_to_string(&ci.skinName.iter().map(|&c| c as u8).collect::<Vec<u8>>());

    // PORT-NOTE: Raven's `!ci->skinName` tests an array, so it is always false;
    // only the `Q_stricmp` half of the disjunction ever decides.
    if Q_stricmp("default", &skinName) == 0 {
        // try default sounds.cfg first
        fLen = trap::FS_FOpenFile(
            ctx.engine,
            &format!("models/players/{}/sounds.cfg", dir),
            &mut f,
            FS_READ,
        );
        if f == 0 {
            // no?  Look for _default sounds.cfg
            fLen = trap::FS_FOpenFile(
                ctx.engine,
                &format!("models/players/{}/sounds_default.cfg", dir),
                &mut f,
                FS_READ,
            );
        }
    } else {
        // use the .skin associated with this skin
        fLen = trap::FS_FOpenFile(
            ctx.engine,
            &format!("models/players/{}/sounds_{}.cfg", dir, skinName),
            &mut f,
            FS_READ,
        );
        if f == 0 {
            // fall back to default sounds
            fLen = trap::FS_FOpenFile(
                ctx.engine,
                &format!("models/players/{}/sounds.cfg", dir),
                &mut f,
                FS_READ,
            );
        }
    }

    let mut soundpath = [0u8; MAX_QPATH];

    if f != 0 {
        // PORT-NOTE: Raven reads `fLen` bytes into a MAX_QPATH buffer and then
        // writes `soundpath[fLen] = 0` — a file longer than 63 bytes overruns
        // the stack (§19 UB). The port clamps to the buffer instead.
        let fClamped = (fLen.max(0) as usize).min(MAX_QPATH - 1);
        trap::FS_Read(ctx.engine, &mut soundpath[..fClamped], f);
        soundpath[fClamped] = 0;

        let mut i = fClamped as isize;

        while i >= 0 && soundpath[i as usize] != b'\n' {
            if soundpath[i as usize] == b'f' {
                isFemale = true;
                soundpath[i as usize] = 0;
            }

            i -= 1;
        }

        i = 0;

        while soundpath[i as usize] != 0
            && soundpath[i as usize] != b'\r'
            && soundpath[i as usize] != b'\n'
        {
            i += 1;
        }
        soundpath[i as usize] = 0;

        trap::FS_FCloseFile(ctx.engine, f);
    }

    // the redirect line, once the gender marker and the line ending are cut off
    let soundpath = buf_to_string(&soundpath);

    ci.gender = if isFemale {
        gender_t::GENDER_FEMALE
    } else {
        gender_t::GENDER_MALE
    };

    trap::S_ShutUp(ctx.engine, true);

    for i in 0..MAX_CUSTOM_SOUNDS {
        let s = match cg_customSoundNames[i] {
            Some(s) => s,
            None => break,
        };

        // strip the extension because we might want .mp3's
        let soundName = COM_StripExtension(&s[1..]);

        ci.sounds[i] = 0;
        // if the model didn't load use the sounds of the default model
        if !soundpath.is_empty() {
            ci.sounds[i] = trap::S_RegisterSound(
                ctx.engine,
                &format!("sound/chars/{}/misc/{}", soundpath, soundName),
            );
        } else if modelloaded {
            ci.sounds[i] = trap::S_RegisterSound(
                ctx.engine,
                &format!("sound/chars/{}/misc/{}", dir, soundName),
            );
        }

        if ci.sounds[i] == 0 {
            // failed the load, try one out of the generic path
            let generic = if isFemale {
                DEFAULT_FEMALE_SOUNDPATH
            } else {
                DEFAULT_MALE_SOUNDPATH
            };
            ci.sounds[i] =
                trap::S_RegisterSound(ctx.engine, &format!("sound/{}/{}", generic, soundName));
        }
    }

    if ctx.world.cgs.gametype >= GT_TEAM || ctx.world.cvars.cg_buildScript.integer != 0 {
        // load the siege sounds then
        for i in 0..MAX_CUSTOM_SIEGE_SOUNDS {
            let s = match bg_customSiegeSoundNames[i] {
                Some(s) => s.to_str().unwrap_or(""),
                None => break,
            };

            // strip the extension because we might want .mp3's
            let soundName = COM_StripExtension(&s[1..]);

            ci.siegeSounds[i] = 0;
            // if the model didn't load use the sounds of the default model
            //
            // PORT-NOTE: this arm's path has no `chars/` prefix, unlike the
            // other two sound loops — Raven's own asymmetry, kept.
            if !soundpath.is_empty() {
                ci.siegeSounds[i] = trap::S_RegisterSound(
                    ctx.engine,
                    &format!("sound/{}/{}", soundpath, soundName),
                );
            } else if modelloaded {
                ci.siegeSounds[i] = trap::S_RegisterSound(
                    ctx.engine,
                    &format!("sound/chars/{}/misc/{}", dir, soundName),
                );
            }

            if ci.siegeSounds[i] == 0 {
                // failed the load, try one out of the generic path
                let generic = if isFemale {
                    DEFAULT_FEMALE_SOUNDPATH
                } else {
                    DEFAULT_MALE_SOUNDPATH
                };
                ci.siegeSounds[i] =
                    trap::S_RegisterSound(ctx.engine, &format!("sound/{}/{}", generic, soundName));
            }
        }
    }

    if ctx.world.cgs.gametype == GT_DUEL
        || ctx.world.cgs.gametype == GT_POWERDUEL
        || ctx.world.cvars.cg_buildScript.integer != 0
    {
        // load the Duel sounds then
        for i in 0..MAX_CUSTOM_DUEL_SOUNDS {
            let s = match cg_customDuelSoundNames[i] {
                Some(s) => s,
                None => break,
            };

            // strip the extension because we might want .mp3's
            let soundName = COM_StripExtension(&s[1..]);

            ci.duelSounds[i] = 0;
            // if the model didn't load use the sounds of the default model
            if !soundpath.is_empty() {
                ci.duelSounds[i] = trap::S_RegisterSound(
                    ctx.engine,
                    &format!("sound/chars/{}/misc/{}", soundpath, soundName),
                );
            } else if modelloaded {
                ci.duelSounds[i] = trap::S_RegisterSound(
                    ctx.engine,
                    &format!("sound/chars/{}/misc/{}", dir, soundName),
                );
            }

            if ci.duelSounds[i] == 0 {
                // failed the load, try one out of the generic path
                let generic = if isFemale {
                    DEFAULT_FEMALE_SOUNDPATH
                } else {
                    DEFAULT_MALE_SOUNDPATH
                };
                ci.duelSounds[i] =
                    trap::S_RegisterSound(ctx.engine, &format!("sound/{}/{}", generic, soundName));
            }
        }
    }

    trap::S_ShutUp(ctx.engine, false);
}

/// Raven `CG_InitG2SaberData` — builds the ghoul2 instance for one of a
/// client's sabers and bolts every blade tag on it.
/// Source: `oracle/codemp/cgame/cg_players.c:1151-1203`
pub fn CG_InitG2SaberData(ctx: &mut CgContext, saberNum: usize, ci: &mut clientInfo_t) {
    let model = buf_to_string(
        &ci.saber[saberNum]
            .model
            .iter()
            .map(|&c| c as u8)
            .collect::<Vec<u8>>(),
    );
    let skin = ci.saber[saberNum].skin;
    let saberFlags = ci.saber[saberNum].saberFlags;
    let numBlades = ci.saber[saberNum].numBlades;

    trap::G2API_InitGhoul2Model(
        ctx.engine,
        &mut ci.ghoul2Weapons[saberNum] as *mut *mut c_void,
        &model,
        0,
        skin,
        0,
        0,
        0,
    );

    if !ci.ghoul2Weapons[saberNum].is_null() {
        let g2 = ci.ghoul2Weapons[saberNum];
        let mut k = 0;

        if skin != 0 {
            trap::G2API_SetSkin(ctx.engine, g2, 0, skin, skin);
        }

        if saberFlags & SFL_BOLT_TO_WRIST != 0 {
            trap::G2API_SetBoltInfo(ctx.engine, g2, 0, 3 + saberNum as c_int);
        } else {
            trap::G2API_SetBoltInfo(ctx.engine, g2, 0, saberNum as c_int);
        }

        while k < numBlades {
            let tagName = format!("*blade{}", k + 1);
            let tagBolt = trap::G2API_AddBolt(ctx.engine, g2, 0, &tagName);

            if tagBolt == -1 {
                if k == 0 {
                    // guess this is an 0ldsk3wl saber
                    let tagBolt = trap::G2API_AddBolt(ctx.engine, g2, 0, "*flash");

                    if tagBolt == -1 {
                        debug_assert!(false, "no *blade1 and no *flash on this saber");
                    }
                    break;
                }

                // Raven re-tests the same value it just branched on, so this
                // arm always takes the break.
                debug_assert!(false, "missing blade tag on this saber");
                break;
            }

            k += 1;
        }
    }
}

/// Raven `CG_CopyClientInfoModel` — moves one client's registered model, bolts
/// and sound handles onto another, duplicating (never sharing) the ghoul2
/// instance.
/// Source: `oracle/codemp/cgame/cg_players.c:1211-1281`
pub fn CG_CopyClientInfoModel(ctx: &mut CgContext, from: &clientInfo_t, to: &mut clientInfo_t) {
    _VectorCopy(from.headOffset, &mut to.headOffset);
    // to->footsteps = from->footsteps;
    to.gender = from.gender;

    to.legsModel = from.legsModel;
    to.legsSkin = from.legsSkin;
    to.torsoModel = from.torsoModel;
    to.torsoSkin = from.torsoSkin;
    //to->headModel = from->headModel;
    //to->headSkin = from->headSkin;
    to.modelIcon = from.modelIcon;

    to.newAnims = from.newAnims;

    //to->ghoul2Model = from->ghoul2Model;
    //rww - Trying to use the same ghoul2 pointer for two seperate clients == DISASTER
    //
    // PORT-NOTE: Raven's `assert(to->ghoul2Model != from->ghoul2Model)` fires
    // whenever both are NULL (the ordinary unregistered case), so it is not
    // ported as a runtime assert — retail compiles it out.

    if !to.ghoul2Model.is_null() && trap::G2_HaveWeGhoul2Models(ctx.engine, to.ghoul2Model) {
        trap::G2API_CleanGhoul2Models(ctx.engine, &mut to.ghoul2Model as *mut *mut c_void);
    }
    if !from.ghoul2Model.is_null() && trap::G2_HaveWeGhoul2Models(ctx.engine, from.ghoul2Model) {
        trap::G2API_DuplicateGhoul2Instance(
            ctx.engine,
            from.ghoul2Model,
            &mut to.ghoul2Model as *mut *mut c_void,
        );
    }

    // Raven: Don't do this, I guess. Just leave the saber info in the original,
    // so it will be properly initialized. (The saber-copy block that used to
    // live here is commented out in the oracle, `cg_players.c:1242-1268`.)

    to.bolt_head = from.bolt_head;
    to.bolt_lhand = from.bolt_lhand;
    to.bolt_rhand = from.bolt_rhand;
    to.bolt_motion = from.bolt_motion;
    to.bolt_llumbar = from.bolt_llumbar;

    to.siegeIndex = from.siegeIndex;

    to.sounds = from.sounds;
    to.siegeSounds = from.siegeSounds;
    to.duelSounds = from.duelSounds;
}

/// Raven `CG_LoadDeferredPlayers` — arms the queue flag; the actual load
/// happens at the top of the next `CG_Player`.
/// Source: `oracle/codemp/cgame/cg_players.c:1976-1978`
pub fn CG_LoadDeferredPlayers(world: &mut CgWorld) {
    world.players.cgQueueLoad = true;
}

/// Raven `CG_InRoll` — is this entity mid-roll (either a getup-roll or a plain
/// one) with time left on the animation?
/// Source: `oracle/codemp/cgame/cg_players.c:2721-2744`
pub fn CG_InRoll(world: &CgWorld, cent: &centity_t) -> bool {
    let legsAnim = cent.currentState.legsAnim;

    if legsAnim == animNumber_t::BOTH_GETUP_BROLL_B as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_BROLL_F as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_BROLL_L as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_BROLL_R as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_FROLL_B as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_FROLL_F as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_FROLL_L as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_FROLL_R as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_F as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_B as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_R as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_L as c_int
    {
        if cent.pe.legs.animationTime > world.cg.time {
            return true;
        }
    }

    false
}

/// Raven `CG_InRollAnim` — the plain-roll half of [`CG_InRoll`], with no
/// animation-time test.
/// Source: `oracle/codemp/cgame/cg_players.c:2746-2757`
pub fn CG_InRollAnim(cent: &centity_t) -> bool {
    let legsAnim = cent.currentState.legsAnim;

    legsAnim == animNumber_t::BOTH_ROLL_F as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_B as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_R as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_L as c_int
}

/// Raven `CG_FirstAnimFrame` — false only when the speed scale is unchanged;
/// the frame-position tests it used to make are commented out in the oracle
/// ("I don't care where it is in the anim now, I am going to pick up from the
/// same bone frame").
/// Source: `oracle/codemp/cgame/cg_players.c:3144-3175`
pub fn CG_FirstAnimFrame(lf: &lerpFrame_t, torsoOnly: bool, speedScale: f32) -> bool {
    if torsoOnly {
        if lf.animationTorsoSpeed == speedScale {
            return false;
        }
    } else if lf.animationSpeed == speedScale {
        return false;
    }

    true
}

/// Raven `CG_G2SetBoneAngles` — straight pass-through to the trap.
///
/// Raven: we want to hold off on setting the bone angles until the end of the
/// frame, because every time we set them the entire skeleton has to be
/// reconstructed... We don't want to go with the delayed approach, we want our
/// bolt points and everything to be updated in realtime. We'll just take the
/// reconstructs and live with them. (The deferred `cgBoneAnglePostSet` path is
/// `#if 0`'d out in the oracle, `cg_players.c:3370-3391`.)
/// Source: `oracle/codemp/cgame/cg_players.c:3365-3396`
#[allow(clippy::too_many_arguments)]
pub fn CG_G2SetBoneAngles(
    ctx: &mut CgContext,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boneName: &str,
    angles: &vec3_t,
    flags: c_int,
    up: c_int,
    right: c_int,
    forward: c_int,
    modelList: Option<&mut qhandle_t>,
    blendTime: c_int,
    currentTime: c_int,
) {
    trap::G2API_SetBoneAngles(
        ctx.engine,
        ghoul2,
        modelIndex,
        boneName,
        angles,
        flags,
        up,
        right,
        forward,
        modelList,
        blendTime,
        currentTime,
    );
}

/// Raven `CG_Rag_Trace` — the ragdoll trace callback: world brushes only, so
/// the hit is reported as the world or as nothing at all.
/// Source: `oracle/codemp/cgame/cg_players.c:3407-3411`
#[allow(clippy::too_many_arguments, unused_variables)]
pub fn CG_Rag_Trace(
    ctx: &mut CgContext,
    result: &mut trace_t,
    start: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    end: &vec3_t,
    skipNumber: c_int,
    mask: c_int,
) {
    trap::CM_BoxTrace(ctx.engine, result, start, end, mins, maxs, 0, mask);
    result.entityNum = if result.fraction != 1.0 {
        ENTITYNUM_WORLD as c_short
    } else {
        ENTITYNUM_NONE as c_short
    };
}

/// Raven `CG_RagAnimForPositioning` — picks the deadflop pose that matches
/// which way the ragdoll's pelvis is facing.
/// Source: `oracle/codemp/cgame/cg_players.c:3460-3482`
pub fn CG_RagAnimForPositioning(ctx: &mut CgContext, cent: &centity_t) -> c_int {
    let mut dir: vec3_t = [0.0; 3];
    let mut matrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };

    let engine = ctx.engine;
    let ghoul2 = cent.ghoul2;
    let turAngles = cent.turAngles;
    let lerpOrigin = cent.lerpOrigin;
    let modelScale = cent.modelScale;
    let time = ctx.world.cg.time;

    debug_assert!(!ghoul2.is_null());
    let bolt = trap::G2API_AddBolt(engine, ghoul2, 0, "pelvis");
    debug_assert!(bolt > -1);

    trap::G2API_GetBoltMatrix(
        engine,
        ghoul2,
        0,
        bolt,
        &mut matrix,
        &turAngles,
        &lerpOrigin,
        time,
        Some(&mut ctx.world.cgs.gameModels[0]),
        &modelScale,
    );
    BG_GiveMeVectorFromMatrix(&matrix, Eorientations::NEGATIVE_Z as c_int, &mut dir);

    if dir[2] > 0.0 {
        // facing up
        animNumber_t::BOTH_DEADFLOP2 as c_int
    } else {
        // facing down
        animNumber_t::BOTH_DEADFLOP1 as c_int
    }
}

/// Raven `CG_G2SetHeadBlink` — closes (or reopens) the eyes; one blink in
/// twenty is a wink, which shuts only the left eye and runs faster.
/// Source: `oracle/codemp/cgame/cg_players.c:3969-4006`
pub fn CG_G2SetHeadBlink(ctx: &mut CgContext, cent: &centity_t, bStart: bool) {
    let engine = ctx.engine;
    let ghoul2 = cent.ghoul2;

    let mut desiredAngles: vec3_t = [0.0; 3];
    let mut blendTime: c_int = 80;
    let mut bWink = false;
    let hReye = trap::G2API_AddBolt(engine, ghoul2, 0, "reye");
    let hLeye = trap::G2API_AddBolt(engine, ghoul2, 0, "leye");

    if hLeye == -1 {
        return;
    }

    VectorClear(&mut desiredAngles);

    if bStart {
        desiredAngles[YAW] = -50.0;
        if ctx.world.bg_state.rng.random() > 0.95 {
            bWink = true;
            blendTime /= 3;
        }
    }

    let time = ctx.world.cg.time;
    trap::G2API_SetBoneAngles(
        engine,
        ghoul2,
        0,
        "leye",
        &desiredAngles,
        BONE_ANGLES_POSTMULT,
        Eorientations::POSITIVE_Y as c_int,
        Eorientations::POSITIVE_Z as c_int,
        Eorientations::POSITIVE_X as c_int,
        None,
        blendTime,
        time,
    );

    if hReye == -1 {
        return;
    }

    if !bWink {
        trap::G2API_SetBoneAngles(
            engine,
            ghoul2,
            0,
            "reye",
            &desiredAngles,
            BONE_ANGLES_POSTMULT,
            Eorientations::POSITIVE_Y as c_int,
            Eorientations::POSITIVE_Z as c_int,
            Eorientations::POSITIVE_X as c_int,
            None,
            blendTime,
            time,
        );
    }
}

/// Raven `CG_G2SetHeadAnim` — plays a face animation on the head bone. A
/// negative `frameLerp` in the table means the animation runs backwards.
///
/// §F19: a model whose `animation.cfg` never parsed leaves `localAnimIndex` at
/// -1, which Raven indexes `bgAllAnims[]` with (a defined-nothing read one
/// entry before the table); `-1 as usize` would panic here, so we early-out.
/// Source: `oracle/codemp/cgame/cg_players.c:4013-4057`
pub fn CG_G2SetHeadAnim(ctx: &mut CgContext, cent: &centity_t, anim: c_int) {
    let engine = ctx.engine;
    let ghoul2 = cent.ghoul2;

    let blendTime: c_int = 50;
    // no skeleton parsed yet, nothing to look the animation up in
    if cent.localAnimIndex < 0 {
        return;
    }
    // `bgAllAnims[..].anims` is still `mp_bg`'s raw `animation_t*` table; the
    // whole workspace reads it through a pointer (see `g_vehicles.rs`,
    // `NPC_reactions.rs`, ui's `UI_ParseAnimationFile`).
    let animations = ctx.world.bg_state.bgAllAnims[cent.localAnimIndex as usize].anims;
    let mut animFlags: c_int = BONE_ANIM_OVERRIDE; //| BONE_ANIM_BLEND;

    // animSpeed is 1.0 if the frameLerp (ms/frame) is 50 (20 fps).
    let cg_timescale = ctx.world.cvars.cg_timescale.value;
    let timeScaleMod: f32 = if cg_timescale != 0.0 {
        (1.0f64 / cg_timescale as f64) as f32
    } else {
        1.0
    };

    // SAFETY: `anims` is the animation table `BG_ParseAnimationFile` allocated
    // for this skeleton; Raven indexes it unchecked and so do we.
    let (numFrames, firstFrame, frameLerp) = unsafe {
        let a = &*animations.offset(anim as isize);
        (a.numFrames as c_int, a.firstFrame as c_int, a.frameLerp)
    };

    let animSpeed = 50.0f32 / frameLerp as f32 * timeScaleMod;
    let firstFrameOut;
    let lastFrame;

    if numFrames <= 0 {
        return;
    }
    if anim == animNumber_t::FACE_DEAD as c_int {
        animFlags |= BONE_ANIM_OVERRIDE_FREEZE;
    }
    // animSpeed is 1.0 if the frameLerp (ms/frame) is 50 (20 fps).
    if animSpeed < 0.0 {
        // play anim backwards
        lastFrame = firstFrame - 1;
        firstFrameOut = (numFrames - 1) + firstFrame;
    } else {
        firstFrameOut = firstFrame;
        lastFrame = numFrames + firstFrame;
    }

    // Raven's "are we already animating the head" early-out is commented out
    // (`cg_players.c:4046-4051`), so the anim is always set.
    trap::G2API_SetBoneAnim(
        engine,
        ghoul2,
        0,
        "face",
        firstFrameOut,
        lastFrame,
        animFlags,
        animSpeed,
        ctx.world.cg.time,
        -1.0,
        blendTime,
    );
}

/// Raven `CG_PlayerFloatSprite` — the sprite that floats above a player's head
/// (chat bubble, flag icon). On your own body it renders in mirrors only.
/// Source: `oracle/codemp/cgame/cg_players.c:4505-4527`
pub fn CG_PlayerFloatSprite(ctx: &mut CgContext, cent: &centity_t, shader: qhandle_t) {
    let rf;

    // PORT-NOTE: Raven derefs `cg.snap` unguarded; `cg_t.snap` is still the C1
    // a missing snapshot reads as "not us"
    let ourClientNum = ctx.world.cg.snap_ref().map_or(-1, |snap| snap.ps.clientNum);

    if cent.currentState.number == ourClientNum && ctx.world.cg.renderingThirdPerson == 0 {
        rf = RF_THIRD_PERSON; // only show in mirrors
    } else {
        rf = 0;
    }

    let mut ent = refEntity_t::zeroed();
    _VectorCopy(cent.lerpOrigin, &mut ent.origin);
    ent.origin[2] += 48.0;
    ent.reType = refEntityType_t::RT_SPRITE;
    ent.customShader = shader;
    ent.radius = 10.0;
    ent.renderfx = rf;
    ent.shaderRGBA[0] = 255;
    ent.shaderRGBA[1] = 255;
    ent.shaderRGBA[2] = 255;
    ent.shaderRGBA[3] = 255;
    trap::R_AddRefEntityToScene(ctx.engine, &ent);
}

/// Raven `CG_PlayerSplash` — the water-wake quad under a player whose feet are
/// in liquid and whose head is not.
/// Source: `oracle/codemp/cgame/cg_players.c:4717-4795`
pub fn CG_PlayerSplash(ctx: &mut CgContext, cent: &centity_t) {
    let engine = ctx.engine;
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut trace = trace_t::zeroed();
    let mut verts = [polyVert_t {
        xyz: [0.0; 3],
        st: [0.0; 2],
        modulate: [0; 4],
    }; 4];

    if ctx.world.cvars.cg_shadows.integer == 0 {
        return;
    }

    _VectorCopy(cent.lerpOrigin, &mut end);
    end[2] -= 24.0;

    // if the feet aren't in liquid, don't make a mark
    // this won't handle moving water brushes, but they wouldn't draw right anyway...
    let contents = trap::CM_PointContents(engine, &end, 0);
    if contents & (CONTENTS_WATER | CONTENTS_SLIME | CONTENTS_LAVA) == 0 {
        return;
    }

    _VectorCopy(cent.lerpOrigin, &mut start);
    start[2] += 32.0;

    // if the head isn't out of liquid, don't make a mark
    let contents = trap::CM_PointContents(engine, &start, 0);
    if contents & (CONTENTS_SOLID | CONTENTS_WATER | CONTENTS_SLIME | CONTENTS_LAVA) != 0 {
        return;
    }

    // trace down to find the surface
    //
    // PORT-NOTE: Raven passes NULL mins/maxs; `CM_Trace` substitutes
    // `vec3_origin` for a NULL bound (`cm_trace.cpp:1603-1610`), so the zero
    // vectors below are the same point trace.
    trap::CM_BoxTrace(
        engine,
        &mut trace,
        &start,
        &end,
        &vec3_origin,
        &vec3_origin,
        0,
        CONTENTS_WATER | CONTENTS_SLIME | CONTENTS_LAVA,
    );

    if trace.fraction == 1.0 {
        return;
    }

    // create a mark polygon
    _VectorCopy(trace.endpos, &mut verts[0].xyz);
    verts[0].xyz[0] -= 32.0;
    verts[0].xyz[1] -= 32.0;
    verts[0].st[0] = 0.0;
    verts[0].st[1] = 0.0;
    verts[0].modulate[0] = 255;
    verts[0].modulate[1] = 255;
    verts[0].modulate[2] = 255;
    verts[0].modulate[3] = 255;

    _VectorCopy(trace.endpos, &mut verts[1].xyz);
    verts[1].xyz[0] -= 32.0;
    verts[1].xyz[1] += 32.0;
    verts[1].st[0] = 0.0;
    verts[1].st[1] = 1.0;
    verts[1].modulate[0] = 255;
    verts[1].modulate[1] = 255;
    verts[1].modulate[2] = 255;
    verts[1].modulate[3] = 255;

    _VectorCopy(trace.endpos, &mut verts[2].xyz);
    verts[2].xyz[0] += 32.0;
    verts[2].xyz[1] += 32.0;
    verts[2].st[0] = 1.0;
    verts[2].st[1] = 1.0;
    verts[2].modulate[0] = 255;
    verts[2].modulate[1] = 255;
    verts[2].modulate[2] = 255;
    verts[2].modulate[3] = 255;

    _VectorCopy(trace.endpos, &mut verts[3].xyz);
    verts[3].xyz[0] += 32.0;
    verts[3].xyz[1] -= 32.0;
    verts[3].st[0] = 1.0;
    verts[3].st[1] = 0.0;
    verts[3].modulate[0] = 255;
    verts[3].modulate[1] = 255;
    verts[3].modulate[2] = 255;
    verts[3].modulate[3] = 255;

    trap::R_AddPolyToScene(engine, ctx.world.cgs.media.wakeMarkShader, &verts);
}

/// Raven `CG_PlayerShieldHit` — arms the personal-shield flash on an entity and
/// points it back down the incoming damage direction. Bigger hits hold longer,
/// capped at two seconds.
/// Source: `oracle/codemp/cgame/cg_players.c:5077-5104`
pub fn CG_PlayerShieldHit(world: &mut CgWorld, entitynum: c_int, dir: &mut vec3_t, amount: c_int) {
    // PORT-NOTE: Raven guards with `tr_types.h`'s MAX_ENTITIES (2048), twice
    // the 1024-entry `cg_entities` array, so 1024..2047 is an out-of-bounds
    // read (§19). The port guards at the array bound instead.
    if entitynum < 0 || entitynum >= MAX_GENTITIES as c_int {
        return;
    }

    let time = if amount > 100 {
        (world.cg.time as f64 + MAX_SHIELD_TIME) as c_int // 2 sec.
    } else {
        world.cg.time + 500 + amount * 15
    };

    let cent = world.entity_mut(entitynum as usize);

    if time > cent.damageTime {
        cent.damageTime = time;
        let incoming = *dir;
        _VectorScale(incoming, -1.0, dir);
        vectoangles(*dir, &mut cent.damageAngles);
    }
}

/// Raven `CG_DrawPlayerShield` — the half-shield shell over the last hit
/// direction, fading out and swelling as it goes. Never drawn on a corpse.
/// Source: `oracle/codemp/cgame/cg_players.c:5107-5142`
pub fn CG_DrawPlayerShield(ctx: &mut CgContext, cent: &centity_t, origin: &vec3_t) {
    // Don't draw the shield when the player is dead.
    if cent.currentState.eFlags & EF_DEAD != 0 {
        return;
    }

    let mut ent = refEntity_t::zeroed();

    _VectorCopy(*origin, &mut ent.origin);
    ent.origin[2] += 10.0;
    AnglesToAxis(cent.damageAngles, ent.axis.as_mut_ptr());

    let jitter = ctx.world.bg_state.rng.random() * 16.0;
    let mut alpha = (255.0f64 * ((cent.damageTime - ctx.world.cg.time) as f64 / MIN_SHIELD_TIME)
        + jitter as f64) as c_int;
    if alpha > 255 {
        alpha = 255;
    }

    // Make it bigger, but tighter if more solid
    let scale = (1.4f64 - (alpha as f32 as f64 * (0.4f64 / 255.0))) as f32; // Range from 1.0 to 1.4
    let axis0 = ent.axis[0];
    _VectorScale(axis0, scale, &mut ent.axis[0]);
    let axis1 = ent.axis[1];
    _VectorScale(axis1, scale, &mut ent.axis[1]);
    let axis2 = ent.axis[2];
    _VectorScale(axis2, scale, &mut ent.axis[2]);

    ent.hModel = ctx.world.cgs.media.halfShieldModel;
    ent.customShader = ctx.world.cgs.media.halfShieldShader;
    ent.shaderRGBA[0] = alpha as u8;
    ent.shaderRGBA[1] = alpha as u8;
    ent.shaderRGBA[2] = alpha as u8;
    ent.shaderRGBA[3] = 255;
    trap::R_AddRefEntityToScene(ctx.engine, &ent);
}

/// Raven `CG_LightVerts` — lights a poly's vertices from the world lightgrid;
/// a face pointing away from the light gets the ambient term only.
///
/// Raven's `numVerts` is dropped in favour of the slice length — callers hand
/// over exactly the vertices they mean to light.
/// Source: `oracle/codemp/cgame/cg_players.c:5167-5207`
pub fn CG_LightVerts(ctx: &mut CgContext, normal: &vec3_t, verts: &mut [polyVert_t]) -> bool {
    let mut ambientLight: vec3_t = [0.0; 3];
    let mut lightDir: vec3_t = [0.0; 3];
    let mut directedLight: vec3_t = [0.0; 3];

    let point = verts[0].xyz;
    trap::R_LightForPoint(
        ctx.engine,
        &point,
        &mut ambientLight,
        &mut directedLight,
        &mut lightDir,
    );

    for vert in verts.iter_mut() {
        let incoming = _DotProduct(*normal, lightDir);
        if incoming <= 0.0 {
            vert.modulate[0] = ambientLight[0] as u8;
            vert.modulate[1] = ambientLight[1] as u8;
            vert.modulate[2] = ambientLight[2] as u8;
            vert.modulate[3] = 255;
            continue;
        }

        let mut j = (ambientLight[0] + incoming * directedLight[0]) as c_int;
        if j > 255 {
            j = 255;
        }
        vert.modulate[0] = j as u8;

        let mut j = (ambientLight[1] + incoming * directedLight[1]) as c_int;
        if j > 255 {
            j = 255;
        }
        vert.modulate[1] = j as u8;

        let mut j = (ambientLight[2] + incoming * directedLight[2]) as c_int;
        if j > 255 {
            j = 255;
        }
        vert.modulate[2] = j as u8;

        vert.modulate[3] = 255;
    }

    true
}

/// Raven `CG_RGBForSaberColor` — the blade tint for one of the six saber
/// colours. Anything outside the six leaves `rgb` exactly as the caller had it.
/// Source: `oracle/codemp/cgame/cg_players.c:5209-5232`
pub fn CG_RGBForSaberColor(color: saber_colors_t, rgb: &mut vec3_t) {
    match color {
        SABER_RED => VectorSet(rgb, 1.0, 0.2, 0.2),
        SABER_ORANGE => VectorSet(rgb, 1.0, 0.5, 0.1),
        SABER_YELLOW => VectorSet(rgb, 1.0, 1.0, 0.2),
        SABER_GREEN => VectorSet(rgb, 0.2, 1.0, 0.2),
        SABER_BLUE => VectorSet(rgb, 0.2, 0.4, 1.0),
        SABER_PURPLE => VectorSet(rgb, 0.9, 0.2, 1.0),
        _ => {}
    }
}

/// Raven `CG_GetTagWorldPosition` — where a model's tag ends up once the
/// model's own origin and axis are applied.
///
/// Raven: Can pass in NULL for the axis.
/// Source: `oracle/codemp/cgame/cg_players.c:5437-5456`
pub fn CG_GetTagWorldPosition(
    ctx: &mut CgContext,
    model: &refEntity_t,
    tag: &str,
    pos: &mut vec3_t,
    axis: Option<&mut [vec3_t; 3]>,
) {
    let mut orientation = orientation_t {
        origin: [0.0; 3],
        axis: [[0.0; 3]; 3],
    };

    // Get the requested tag
    trap::R_LerpTag(
        ctx.engine,
        &mut orientation,
        model.hModel,
        model.oldframe,
        model.frame,
        1.0 - model.backlerp,
        tag,
    );

    _VectorCopy(model.origin, pos);
    for i in 0..3 {
        let running = *pos;
        _VectorMA(running, orientation.origin[i], model.axis[i], pos);
    }

    if let Some(axis) = axis {
        MatrixMultiply(&orientation.axis, &model.axis, axis);
    }
}

/// Raven `CG_G2TraceCollide` — re-runs a landed trace against the hit entity's
/// ghoul2 skeleton. A miss rewrites the trace as "hit nothing"; a hit moves the
/// endpoint and normal onto the collided polygon.
/// Source: `oracle/codemp/cgame/cg_players.c:5632-5689`
pub fn CG_G2TraceCollide(
    ctx: &mut CgContext,
    tr: &mut trace_t,
    mins: Option<&vec3_t>,
    maxs: Option<&vec3_t>,
    lastValidStart: &vec3_t,
    lastValidEnd: &vec3_t,
) -> bool {
    let mut angles: vec3_t = [0.0; 3];
    let mut fRadius: f32 = 0.0;

    if let (Some(mins), Some(maxs)) = (mins, maxs) {
        if mins[0] != 0.0 || maxs[0] != 0.0 {
            fRadius = (maxs[0] - mins[0]) / 2.0;
        }
    }

    // Raven's `memset(&G2Trace, 0, sizeof(G2Trace))` then the -1 entity-number
    // sweep, in one go.
    let mut G2Trace: G2Trace_t = [CollisionRecord_t {
        mDistance: 0.0,
        mEntityNum: -1,
        mModelIndex: 0,
        mPolyIndex: 0,
        mSurfaceIndex: 0,
        mCollisionPosition: [0.0; 3],
        mCollisionNormal: [0.0; 3],
        mFlags: 0,
        mMaterial: 0,
        mLocation: 0,
        mBarycentricI: 0.0,
        mBarycentricJ: 0.0,
    }; MAX_G2_COLLISIONS];

    let engine = ctx.engine;
    let time = ctx.world.cg.time;
    let g2TraceLod = ctx.world.cvars.cg_g2TraceLod.integer;
    let optvehtrace = ctx.world.cvars.cg_optvehtrace.integer;

    let g2Hit = ctx.world.entity(tr.entityNum as usize);
    let ghoul2 = g2Hit.ghoul2;
    let number = g2Hit.currentState.number;
    let eType = g2Hit.currentState.eType;
    let NPC_class = g2Hit.currentState.NPC_class;
    let hasVehicle = g2Hit.m_pVehicle.is_some();
    let lerpOrigin = g2Hit.lerpOrigin;
    let modelScale = g2Hit.modelScale;
    let yaw = g2Hit.lerpAngles[YAW];

    if ghoul2.is_null() {
        return false;
    }

    angles[ROLL] = 0.0;
    angles[PITCH] = 0.0;
    angles[YAW] = yaw;

    if optvehtrace != 0
        && eType == entityType_t::ET_NPC as c_int
        && NPC_class == class_t::CLASS_VEHICLE as c_int
        && hasVehicle
    {
        trap::G2API_CollisionDetectCache(
            engine,
            &mut G2Trace,
            ghoul2,
            &angles,
            &lerpOrigin,
            time,
            number,
            lastValidStart,
            lastValidEnd,
            &modelScale,
            0,
            g2TraceLod,
            fRadius,
        );
    } else {
        trap::G2API_CollisionDetect(
            engine,
            &mut G2Trace,
            ghoul2,
            &angles,
            &lerpOrigin,
            time,
            number,
            lastValidStart,
            lastValidEnd,
            &modelScale,
            0,
            g2TraceLod,
            fRadius,
        );
    }

    if G2Trace[0].mEntityNum != number {
        tr.fraction = 1.0;
        tr.entityNum = ENTITYNUM_NONE as c_short;
        tr.startsolid = 0;
        tr.allsolid = 0;
        false
    } else {
        // Yay!
        _VectorCopy(G2Trace[0].mCollisionPosition, &mut tr.endpos);
        _VectorCopy(G2Trace[0].mCollisionNormal, &mut tr.plane.normal);
        true
    }
}

/// Raven `CG_AddGhoul2Mark` — puts a gore decal on a ghoul2 skin along the
/// start→end ray. Bails when the instance is already carrying `cg_ghoul2Marks`
/// decals, or when the ray is too short to have a direction.
///
/// DEFERRED: the `SSkinGoreData` fill and the `trap_G2API_AddSkinGore` call —
/// `SSkinGoreData` (`oracle/codemp/game/q_shared.h:3111-3144`) has no
/// mp_cgame-reachable Rust home: the only MP definition lives in the engine
/// ghoul2 crate, which this module must not depend on, and `trap.rs`'s wrapper
/// therefore takes the gore block as an opaque `*mut c_void`. Everything with a
/// side effect (the mark-count early-out, the `flrand` draw, Raven's argument-
/// swapped scale copy, the ray-length early-out) is transcribed; only the
/// engine hand-off is missing, so no mark is added yet.
/// Source: `oracle/codemp/cgame/cg_players.c:5738-5796`
#[allow(clippy::too_many_arguments)]
pub fn CG_AddGhoul2Mark(
    ctx: &mut CgContext,
    shader: c_int,
    size: f32,
    start: &vec3_t,
    end: &vec3_t,
    entnum: c_int,
    entposition: &vec3_t,
    entangle: f32,
    ghoul2: *mut c_void,
    scale: &mut vec3_t,
    lifeTime: c_int,
) {
    debug_assert!(!ghoul2.is_null());

    if trap::G2API_GetNumGoreMarks(ctx.engine, ghoul2, 0) >= ctx.world.cvars.cg_ghoul2Marks.integer
    {
        // you've got too many marks already
        return;
    }

    // Raven's field fill, in its order — the `theta` draw moves the shared RNG
    // and so has to happen even while the block itself can't be handed over.
    let theta = ctx.world.bg_state.rng.flrand(0.0, 6.28);

    // PORT-NOTE: Raven's else arm is `VectorCopy(goreSkin.scale, scale)` — the
    // arguments are the wrong way round, so it copies the memset-zeroed
    // `goreSkin.scale` OVER the caller's vector and leaves the gore scale at
    // zero. Kept: the caller really does get stomped.
    let goreScale: vec3_t = if scale[0] == 0.0 && scale[1] == 0.0 && scale[2] == 0.0 {
        [1.0, 1.0, 1.0]
    } else {
        _VectorCopy([0.0; 3], scale);
        [0.0; 3]
    };

    let mut rayDirection: vec3_t = [0.0; 3];
    _VectorSubtract(*end, *start, &mut rayDirection);
    if VectorNormalize(&mut rayDirection) < 0.1 {
        return;
    }

    // DEFERRED: SSkinGoreData — `oracle/codemp/game/q_shared.h:3111-3144`.
    // The remaining fields Raven sets are growDuration -1, goreScaleStartFraction
    // 1.0, frontFaces/backFaces true, baseModelOnly false, lifeTime, currentTime
    // cg.time, entNum, SSize/TSize size, theta, shader, scale, hitLocation start,
    // rayDirection, position entposition and angles[YAW] entangle; then
    // `trap_G2API_AddSkinGore(ghoul2, &goreSkin)`.
    let _ = (
        shader,
        size,
        start,
        entnum,
        entposition,
        entangle,
        lifeTime,
        theta,
        goreScale,
        rayDirection,
    );
}

/// Raven `CG_IsMindTricked` — is this client hidden from us by mind trick? The
/// four bitmask args are the four 16-client blocks of `trickedentindex`; force
/// sight beats the trick outright.
/// Source: `oracle/codemp/cgame/cg_players.c:6482-6518`
pub fn CG_IsMindTricked(
    world: &CgWorld,
    trickIndex1: c_int,
    trickIndex2: c_int,
    trickIndex3: c_int,
    trickIndex4: c_int,
    client: c_int,
) -> bool {
    let checkIn;
    let mut sub = 0;

    if world.entity(client as usize).currentState.forcePowersActive & (1 << FP_SEE) != 0 {
        return false;
    }

    if client > 47 {
        checkIn = trickIndex4;
        sub = 48;
    } else if client > 31 {
        checkIn = trickIndex3;
        sub = 32;
    } else if client > 15 {
        checkIn = trickIndex2;
        sub = 16;
    } else {
        checkIn = trickIndex1;
    }

    checkIn & (1 << (client - sub)) != 0
}

/// Raven `CG_DrawPlayerSphere` — the force-effect shell (invulnerability,
/// ysalamiri, enlightenment…) drawn around a player. Never on a corpse, and
/// never on your own body in first person past the first pass.
/// Source: `oracle/codemp/cgame/cg_players.c:6522-6626`
pub fn CG_DrawPlayerSphere(
    ctx: &mut CgContext,
    cent: &centity_t,
    origin: &vec3_t,
    scale: f32,
    shader: c_int,
) {
    // Don't draw the shield when the player is dead.
    if cent.currentState.eFlags & EF_DEAD != 0 {
        return;
    }

    let mut ent = refEntity_t::zeroed();
    let mut ang: vec3_t = [0.0; 3];
    let mut viewDir: vec3_t = [0.0; 3];

    _VectorCopy(*origin, &mut ent.origin);
    ent.origin[2] += 9.0;

    let vieworg = ctx.world.cg.refdef.vieworg;
    _VectorSubtract(ent.origin, vieworg, &mut ent.axis[0]);
    let vLen = VectorLength(ent.axis[0]);
    if vLen <= 0.1 {
        // Entity is right on vieworg.  quit.
        return;
    }

    _VectorCopy(ent.axis[0], &mut viewDir);
    VectorInverse(&mut viewDir);
    VectorNormalize(&mut viewDir);

    vectoangles(ent.axis[0], &mut ang);
    ang[ROLL] += 180.0;
    ang[PITCH] += 180.0;
    AnglesToAxis(ang, ent.axis.as_mut_ptr());

    let axis0 = ent.axis[0];
    _VectorScale(axis0, scale, &mut ent.axis[0]);
    let axis1 = ent.axis[1];
    _VectorScale(axis1, scale, &mut ent.axis[1]);
    let axis2 = ent.axis[2];
    _VectorScale(axis2, scale, &mut ent.axis[2]);

    ent.nonNormalizedAxes = qtrue;

    ent.hModel = ctx.world.cgs.media.halfShieldModel;
    ent.customShader = shader;

    trap::R_AddRefEntityToScene(ctx.engine, &ent);

    if ctx.world.cg.renderingThirdPerson == 0
        && cent.currentState.number == ctx.world.cg.predictedPlayerState.clientNum
    {
        // don't do the rest then
        return;
    }
    if ctx.world.cvars.cg_renderToTextureFX.integer == 0 {
        return;
    }

    ang[PITCH] -= 180.0;
    AnglesToAxis(ang, ent.axis.as_mut_ptr());

    let axis0 = ent.axis[0];
    _VectorScale(axis0, scale * 0.5, &mut ent.axis[0]);
    let axis1 = ent.axis[1];
    _VectorScale(axis1, scale * 0.5, &mut ent.axis[1]);
    let axis2 = ent.axis[2];
    _VectorScale(axis2, scale * 0.5, &mut ent.axis[2]);

    ent.renderfx = RF_DISTORTION | RF_FORCE_ENT_ALPHA;
    if shader == ctx.world.cgs.media.invulnerabilityShader {
        // ok, ok, this is a little hacky. sorry!
        ent.shaderRGBA[0] = 0;
        ent.shaderRGBA[1] = 255;
        ent.shaderRGBA[2] = 0;
        ent.shaderRGBA[3] = 100;
    } else if shader == ctx.world.cgs.media.ysalimariShader {
        ent.shaderRGBA[0] = 255;
        ent.shaderRGBA[1] = 255;
        ent.shaderRGBA[2] = 0;
        ent.shaderRGBA[3] = 100;
    } else if shader == ctx.world.cgs.media.endarkenmentShader {
        ent.shaderRGBA[0] = 100;
        ent.shaderRGBA[1] = 0;
        ent.shaderRGBA[2] = 0;
        ent.shaderRGBA[3] = 20;
    } else if shader == ctx.world.cgs.media.enlightenmentShader {
        ent.shaderRGBA[0] = 255;
        ent.shaderRGBA[1] = 255;
        ent.shaderRGBA[2] = 255;
        ent.shaderRGBA[3] = 20;
    } else {
        // ysal red/blue, boon
        ent.shaderRGBA[0] = 255;
        ent.shaderRGBA[1] = 255;
        ent.shaderRGBA[2] = 255;
        ent.shaderRGBA[3] = 20;
    }

    ent.radius = 256.0;

    let entOrigin = ent.origin;
    _VectorMA(entOrigin, 40.0, viewDir, &mut ent.origin);

    ent.customShader = trap::R_RegisterShader(ctx.engine, "effects/refract_2");
    trap::R_AddRefEntityToScene(ctx.engine, &ent);
}

/// Raven `CG_AddLightningBeam` — the wobbling force-lightning bezier between
/// two points; the "chaos" that bends the control points is three sine waves off
/// `cg.time`, so it is barely chaotic at all.
/// Source: `oracle/codemp/cgame/cg_players.c:6628-6697`
pub fn CG_AddLightningBeam(ctx: &mut CgContext, start: &vec3_t, end: &vec3_t) {
    let mut dir: vec3_t = [0.0; 3];
    let mut chaos: vec3_t = [0.0; 3];
    let mut c1: vec3_t = [0.0; 3];
    let mut c2: vec3_t = [0.0; 3];
    let mut v1: vec3_t = [0.0; 3];
    let mut v2: vec3_t = [0.0; 3];

    _VectorSubtract(*end, *start, &mut dir);
    let len = VectorNormalize(&mut dir);

    // Get the base control points, we'll work from there
    _VectorMA(*start, 0.3333 * len, dir, &mut c1);
    _VectorMA(*start, 0.6666 * len, dir, &mut c2);

    // get some chaos values that really aren't very chaotic :)
    let time = ctx.world.cg.time;
    let s1 = (((time as f32 * 0.005) as f64).sin() * 2.0
        + ctx.world.bg_state.rng.crandom() * 0.2f32 as f64) as f32;
    let s2 = ((time as f32 * 0.001) as f64).sin() as f32;
    let s3 = ((time as f32 * 0.011) as f64).sin() as f32;

    VectorSet(
        &mut chaos,
        len * 0.01 * s1,
        len * 0.02 * s2,
        len * 0.04 * (s1 + s2 + s3),
    );

    let c1in = c1;
    _VectorAdd(c1in, chaos, &mut c1);
    _VectorScale(chaos, 4.0, &mut v1);

    VectorSet(
        &mut chaos,
        -len * 0.02 * s3,
        len * 0.01 * (s1 * s2),
        -len * 0.02 * (s1 + s2 * s3),
    );

    let c2in = c2;
    _VectorAdd(c2in, chaos, &mut c2);
    _VectorScale(chaos, 2.0, &mut v2);

    VectorSet(&mut chaos, 1.0, 1.0, 1.0);

    // PORT-NOTE: `v1`, `v2` and that last `chaos` set are never read again in
    // Raven — dead stores, kept so the arithmetic reads like the oracle.
    let _ = (v1, v2, chaos);

    let sRGB: vec3_t = [255.0, 255.0, 255.0];
    let shader = trap::R_RegisterShader(ctx.engine, "gfx/misc/electric2");

    let mut b = addbezierArgStruct_t {
        start: *start,
        end: *end,
        control1: c1,
        control1Vel: vec3_origin,
        control2: c2,
        control2Vel: vec3_origin,
        size1: 6.0,
        size2: 6.0,
        sizeParm: 0.0,
        alpha1: 0.0,
        alpha2: 0.2,
        alphaParm: 0.5,
        sRGB,
        eRGB: sRGB,
        rgbParm: 0.0,
        killTime: 50,
        shader,
        flags: 0x00000001, // FX_ALPHA_LINEAR
    };

    trap::FX_AddBezier(ctx.engine, &mut b);
}

/// Raven `CG_ThereIsAMaster` — is anyone carrying the Jedi Master mantle right
/// now?
/// Source: `oracle/codemp/cgame/cg_players.c:6744-6762`
pub fn CG_ThereIsAMaster(world: &CgWorld) -> bool {
    for i in 0..MAX_CLIENTS {
        if world.entity(i).currentState.isJediMaster != 0 {
            return true;
        }
    }

    false
}

/// Raven `CG_HandleAppendedSkin` — splits a `model*skin` string in place and
/// registers the skin out of the model's own folder. `modelName` comes back cut
/// at the `*`; a `|` in the skin name means a three-part skin.
/// Source: `oracle/codemp/cgame/cg_players.c:6817-6868`
pub fn CG_HandleAppendedSkin(ctx: &mut CgContext, modelName: &mut String) -> qhandle_t {
    let mut skinID: qhandle_t = 0;

    // see if it has a skin name
    let p = modelName.rfind('*');

    if let Some(p) = p {
        // found a *, we should have a model name before it and a skin name after it.
        let skinName = modelName[p + 1..].to_string();
        modelName.truncate(p);

        if !skinName.is_empty() {
            // got it, register the skin under the model path.
            let mut baseFolder = modelName.clone();

            // go back to the first /, should be the path point
            if let Some(slash) = baseFolder.rfind('/') {
                // got it.. terminate at the slash and register.
                baseFolder.truncate(slash);

                let useSkinName = if skinName.contains('|') {
                    // three part skin
                    format!("{}/|{}", baseFolder, skinName)
                } else {
                    format!("{}/model_{}.skin", baseFolder, skinName)
                };

                skinID = trap::R_RegisterSkin(ctx.engine, &useSkinName);
            }
        }
    }

    skinID
}

/// Raven `CG_CacheG2AnimInfo` — precaches a model's skeleton and animation set
/// by building a throwaway ghoul2 instance, parsing the `animation.cfg` beside
/// its `.gla`, and cleaning the instance up again.
/// Source: `oracle/codemp/cgame/cg_players.c:6876-6939`
pub fn CG_CacheG2AnimInfo(ctx: &mut CgContext, modelName: &str) {
    let mut useModel = modelName.to_string();

    if modelName.starts_with('$') {
        // it's a vehicle name actually, let's precache the whole vehicle
        let traps = CgBgTraps::new(ctx.engine);
        let mut callbacks = CgGameCallbacks::new(ctx.engine);

        // both bg fns stomp the `$name` they were handed with their answer, so
        // each gets its own copy of it the way Raven's two stack buffers do
        let mut modelBuf: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        let mut skinBuf: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        Q_strncpyz(&mut modelBuf, modelName, MAX_QPATH);
        Q_strncpyz(&mut skinBuf, modelName, MAX_QPATH);

        BG_GetVehicleModelName(
            modelBuf.as_mut_ptr(),
            &mut ctx.world.bg_state,
            &traps,
            &mut callbacks,
        );
        BG_GetVehicleSkinName(
            skinBuf.as_mut_ptr(),
            &mut ctx.world.bg_state,
            &traps,
            &mut callbacks,
        );

        let vehModel = buf_to_string(&modelBuf.map(|c| c as u8));
        let useSkin = buf_to_string(&skinBuf.map(|c| c as u8));

        if !useSkin.is_empty() {
            // use a custom skin
            trap::R_RegisterSkin(
                ctx.engine,
                &format!("models/players/{vehModel}/model_{useSkin}.skin"),
            );
        } else {
            trap::R_RegisterSkin(
                ctx.engine,
                &format!("models/players/{vehModel}/model_default.skin"),
            );
        }
        useModel = format!("models/players/{vehModel}/model.glm");
    }

    let mut g2: *mut c_void = null_mut();
    trap::G2API_InitGhoul2Model(
        ctx.engine,
        &mut g2 as *mut *mut c_void,
        &useModel,
        0,
        0,
        0,
        0,
        0,
    );

    if !g2.is_null() {
        let mut animIndex: c_int = -1;
        let mut GLAName = trap::G2API_GetGLAName(ctx.engine, g2, 0, MAX_QPATH);

        if let Some(slash) = GLAName.rfind('/') {
            GLAName.truncate(slash);
            GLAName.push_str("/animation.cfg");

            let traps = CgBgTraps::new(ctx.engine);
            let mut callbacks = CgGameCallbacks::new(ctx.engine);
            let filename = cstr(&GLAName);

            animIndex = BG_ParseAnimationFile(
                &mut ctx.world.bg_state,
                &traps,
                &mut callbacks,
                filename.as_ptr(),
                null_mut(),
                qfalse,
            );
        }

        if animIndex != -1 {
            // DEFERRED: BG_ParseAnimationEvtFile —
            // `oracle/codemp/game/bg_panimate.c:2169-2328`. Raven cuts
            // `originalModelName` (its pre-GLA copy of `useModel`) back to its
            // directory and registers the model's animation events against
            // `bgNumAnimEvents`. The whole event-file block sits inside
            // `#ifndef QAGAME`, so `mp_bg` deliberately did NOT port it and
            // there is no cgame home for it yet.
            //   BG_ParseAnimationEvtFile(originalModelName, animIndex, bgNumAnimEvents);
        }

        // Now free the temp instance
        trap::G2API_CleanGhoul2Models(ctx.engine, &mut g2 as *mut *mut c_void);
    }
}

/// Raven `CG_RegisterVehicleAssets` — precaches a vehicle's effect set.
///
/// The whole body is commented out in the oracle (every `trap_FX_RegisterEffect`
/// line), so this really is a no-op; it is kept because `CG_Player`'s vehicle
/// path still calls it (`cg_players.c:7051`).
/// Source: `oracle/codemp/cgame/cg_players.c:6941-6986`
pub fn CG_RegisterVehicleAssets(_pVeh: VehicleId) {}

/// Raven `CG_CreateSurfaceDebris` — blows the named ghoul2 surface off a ship,
/// playing the debris effect at the surface's bolt (or at the ship's origin when
/// the surface has no bolt).
///
/// DEFERRED past the vehicle presence test: the five wing/nose arms also read
/// `m_pVehicle->m_pVehicleInfo->i{R,L}WingFX`/`iNoseFX` for the thrown-part
/// effect, and DEC-46.2's `Option<VehicleId>` carries only the vehicle cent's
/// entity number until the `Vehicle_t` referent pool lands. `lostPartFX` stays
/// 0, so the thrown ship part at the end never fires; the debris effect itself
/// plays normally.
/// Source: `oracle/codemp/cgame/cg_players.c:7266-7361`
pub fn CG_CreateSurfaceDebris(
    ctx: &mut CgContext,
    cent: &centity_t,
    surfNum: c_int,
    fxID: c_int,
    throwPart: bool,
) {
    let lostPartFX: c_int = 0;
    let mut b: c_int = -1;
    let mut v: vec3_t = [0.0; 3];
    let mut d: vec3_t = [0.0; 3];
    let mut boltMatrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };
    let mut surfName: Option<&str> = None;

    if surfNum > 0 {
        surfName = bgToggleableSurfaces
            .get(surfNum as usize)
            .and_then(|s| *s)
            .map(|s| s.to_str().unwrap_or(""));
    }

    if cent.ghoul2.is_null() {
        // oh no
        return;
    }

    let engine = ctx.engine;

    // PORT-NOTE: Raven indexes `bgToggleableSurfaceDebris[surfNum]` before it
    // tests `surfNum == -1` further down, so a -1 surface reads one slot before
    // the table (§19 UB). The port reads through a bounds check: an
    // out-of-range surface matches no arm and falls into the origin path, which
    // is exactly where Raven's own `surfNum == -1` test sends it.
    let debris = bgToggleableSurfaceDebris
        .get(surfNum as usize)
        .copied()
        .unwrap_or(-1);

    // let's add the surface as a bolt so we can get the base point of it
    if debris == 3 {
        // right wing flame
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, "*r_wingdamage");
    } else if debris == 4 {
        // left wing flame
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, "*l_wingdamage");
    } else if debris == 5 {
        // right wing flame 2
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, "*r_wingdamage");
    } else if debris == 6 {
        // left wing flame 2
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, "*l_wingdamage");
    } else if debris == 7 {
        // nose flame
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, "*nosedamage");
    } else if let Some(surfName) = surfName {
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, surfName);
    }

    if b == -1 || surfNum == -1 {
        // couldn't find this surface apparently, so play on origin?
        _VectorCopy(cent.lerpOrigin, &mut v);
        AngleVectors(cent.lerpAngles, Some(&mut d), None, None);
        VectorNormalize(&mut d);
    } else {
        // now let's get the position and direction of this surface and make a big explosion
        let time = ctx.world.cg.time;
        trap::G2API_GetBoltMatrix(
            engine,
            cent.ghoul2,
            0,
            b,
            &mut boltMatrix,
            &cent.lerpAngles,
            &cent.lerpOrigin,
            time,
            Some(&mut ctx.world.cgs.gameModels[0]),
            &cent.modelScale,
        );
        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut v);
        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::POSITIVE_Z as c_int, &mut d);
    }

    trap::FX_PlayEffectID(engine, fxID, &v, &d, -1, -1);
    if throwPart && lostPartFX != 0 {
        // throw off a ship part, too
        let mut fxFwd: vec3_t = [0.0; 3];
        AngleVectors(cent.lerpAngles, Some(&mut fxFwd), None, None);
        trap::FX_PlayEffectID(engine, lostPartFX, &v, &fxFwd, -1, -1);
    }
}

/// Raven `CG_CreateSurfaceSmoke` — the flame/smoke trail off one of a ship's
/// four damage surfaces. An unknown surface, or one with no bolt on the model,
/// draws nothing.
/// Source: `oracle/codemp/cgame/cg_players.c:7365-7411`
pub fn CG_CreateSurfaceSmoke(ctx: &mut CgContext, cent: &centity_t, shipSurf: c_int, fxID: c_int) {
    let mut v: vec3_t = [0.0; 3];
    let mut d: vec3_t = [0.0; 3];
    let mut boltMatrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };

    if cent.ghoul2.is_null() {
        // oh no
        return;
    }

    // let's add the surface as a bolt so we can get the base point of it
    let surfName = if shipSurf == SHIPSURF_FRONT {
        // front flame/smoke
        "*nosedamage"
    } else if shipSurf == SHIPSURF_BACK {
        // back flame/smoke
        //
        // Raven: FIXME: random?  Some point in-between?
        "*exhaust1"
    } else if shipSurf == SHIPSURF_RIGHT {
        // right wing flame/smoke
        "*r_wingdamage"
    } else if shipSurf == SHIPSURF_LEFT {
        // left wing flame/smoke
        "*l_wingdamage"
    } else {
        // unknown surf!
        return;
    };

    let engine = ctx.engine;
    let b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, surfName);
    if b == -1 {
        // couldn't find this surface apparently
        return;
    }

    // now let's get the position and direction of this surface and make a big explosion
    let time = ctx.world.cg.time;
    trap::G2API_GetBoltMatrix(
        engine,
        cent.ghoul2,
        0,
        b,
        &mut boltMatrix,
        &cent.lerpAngles,
        &cent.lerpOrigin,
        time,
        Some(&mut ctx.world.cgs.gameModels[0]),
        &cent.modelScale,
    );
    BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut v);
    BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::POSITIVE_Z as c_int, &mut d);

    trap::FX_PlayEffectID(engine, fxID, &v, &d, -1, -1);
}

/// Raven `CG_VehicleShouldDrawShields` — are this ship's shields currently
/// taking damage, so the shell should be drawn?
///
/// DEFERRED: Raven's last clause is `m_pVehicle->m_pVehicleInfo` non-null;
/// DEC-46.2's `Option<VehicleId>` answers only the presence half until the
/// `Vehicle_t` referent pool lands, so a vehicle with no vehicle info would
/// wrongly draw shields here.
/// Source: `oracle/codemp/cgame/cg_players.c:7415-7425`
pub fn CG_VehicleShouldDrawShields(world: &CgWorld, vehCent: &centity_t) -> bool {
    // ship shields currently taking damage
    vehCent.damageTime > world.cg.time
        && vehCent.currentState.NPC_class == class_t::CLASS_VEHICLE as c_int
        && vehCent.m_pVehicle.is_some()
}

/// Raven `CG_VehicleAttachDroidUnit` — snaps an astromech onto its ship's droid
/// tag, taking the droid's whole origin and orientation off the bolt.
///
/// ESCALATION: blocked past the presence guard — the bolt index is
/// `m_pVehicle->m_iDroidUnitTag`, and DEC-46.2's `Option<VehicleId>` carries
/// only the vehicle cent's entity number until the `Vehicle_t` referent pool
/// lands ("ported code only tests presence"). There is no honest bolt index to
/// pass, so this answers Raven's own "not attached" (`qfalse`) and the droid
/// keeps its snapshot position — a visual miss, not a crash.
/// Source: `oracle/codemp/cgame/cg_players.c:7433-7461`
#[allow(unused_variables)]
pub fn CG_VehicleAttachDroidUnit(
    ctx: &mut CgContext,
    droidCent: &mut centity_t,
    legs: &refEntity_t,
) -> bool {
    if droidCent.currentState.owner != 0 && droidCent.currentState.clientNum >= MAX_CLIENTS_I32 {
        // the only NPCs that can ride a vehicle are droids...???
        let vehCent = ctx.world.entity(droidCent.currentState.owner as usize);

        if vehCent.m_pVehicle.is_some() && !vehCent.ghoul2.is_null() {
            // DEFERRED: Vehicle_t::m_iDroidUnitTag — see the fn doc. Raven bolts
            // `droidCent->lerpOrigin` to that tag's matrix and rebuilds
            // `lerpAngles` from its POSITIVE_X/NEGATIVE_Y vectors ("WTF???" is
            // Raven's own comment on the axis choice).
        }
    }

    false
}

/// Raven `CG_CreateNPCClient` — the per-NPC `clientInfo_t` an entity carries
/// beside the real client table. Raven pool-allocates it and every caller
/// memsets it straight after, so the block is handed back zeroed (DEC-46.2 makes
/// it owned, which is what retires Raven's "always free it, never stomp over
/// it" discipline).
/// Source: `oracle/codemp/cgame/cg_players.c:7695-7699`
pub fn CG_CreateNPCClient() -> Box<clientInfo_t> {
    Box::new(zeroed_client_info())
}

/// Raven `CG_DestroyNPCClient` — zeroes the NPC's client info and keeps the
/// block. The `trap_TrueFree` beside it is commented out in the oracle, so the
/// allocation deliberately stays alive and non-NULL: the caller's next
/// `if (!cent->npcClient)` must still be false.
/// Source: `oracle/codemp/cgame/cg_players.c:7701-7705`
pub fn CG_DestroyNPCClient(ci: &mut Option<Box<clientInfo_t>>) {
    if let Some(ci) = ci {
        **ci = zeroed_client_info();
    }
}

/// The zero fill Raven gets from its `memset(*ci, 0, sizeof(clientInfo_t))`.
/// `clientInfo_t` is too wide for a derived `Default` (its `char[64]` name
/// buffers), so the fill is spelled once here for both NPC-client fns.
fn zeroed_client_info() -> clientInfo_t {
    // SAFETY: `clientInfo_t` is `#[repr(C)]` scalars, arrays, `qhandle_t`s and
    // opaque ghoul2 `*mut c_void` tokens; its two enum members (`team_t`,
    // `gender_t`) both have a 0 discriminant, so all-zero is a legal value.
    unsafe { core::mem::zeroed() }
}

/// Raven `CG_InitJetpackGhoul2` — builds the one shared jetpack instance and
/// bolts it to the player's `*chestg` tag, plus the two jet-effect bolts.
/// Source: `oracle/codemp/cgame/cg_players.c:7891-7911`
pub fn CG_InitJetpackGhoul2(ctx: &mut CgContext) {
    if !ctx.world.players.cg_g2JetpackInstance.is_null() {
        debug_assert!(false, "Tried to init jetpack inst, already init'd");
        return;
    }

    let engine = ctx.engine;
    trap::G2API_InitGhoul2Model(
        engine,
        &mut ctx.world.players.cg_g2JetpackInstance as *mut *mut c_void,
        JETPACK_MODEL,
        0,
        0,
        0,
        0,
        0,
    );

    debug_assert!(!ctx.world.players.cg_g2JetpackInstance.is_null());

    // Indicate which bolt on the player we will be attached to
    // In this case bolt 0 is rhand, 1 is lhand, and 2 is the bolt
    // for the jetpack (*chestg)
    let g2 = ctx.world.players.cg_g2JetpackInstance;
    trap::G2API_SetBoltInfo(engine, g2, 0, 2);

    // Add the bolts jet effects will be played from
    trap::G2API_AddBolt(engine, g2, 0, "torso_ljet");
    trap::G2API_AddBolt(engine, g2, 0, "torso_rjet");
}

/// Raven `CG_CleanJetpackGhoul2` — drops the shared jetpack instance at
/// shutdown.
/// Source: `oracle/codemp/cgame/cg_players.c:7913-7920`
pub fn CG_CleanJetpackGhoul2(ctx: &mut CgContext) {
    if !ctx.world.players.cg_g2JetpackInstance.is_null() {
        trap::G2API_CleanGhoul2Models(
            ctx.engine,
            &mut ctx.world.players.cg_g2JetpackInstance as *mut *mut c_void,
        );
        ctx.world.players.cg_g2JetpackInstance = null_mut();
    }
}

/// Raven `CG_RadiusForCent` — the ghoul2 cull radius for an entity: its own
/// `g2radius` when it has one, otherwise 64.
///
/// DEFERRED: an NPC vehicle first checks `m_pVehicle->m_pVehicleInfo->g2radius`
/// for an override, which DEC-46.2's `Option<VehicleId>` can't reach until the
/// `Vehicle_t` referent pool lands; a vehicle with an override falls through to
/// its entity-state radius here.
/// Source: `oracle/codemp/cgame/cg_players.c:8316-8336`
pub fn CG_RadiusForCent(cent: &centity_t) -> f32 {
    if cent.currentState.eType == entityType_t::ET_NPC as c_int {
        if cent.currentState.g2radius != 0 {
            return cent.currentState.g2radius as f32;
        }
    } else if cent.currentState.g2radius != 0 {
        return cent.currentState.g2radius as f32;
    }

    64.0
}

/// Raven `CG_ScanForExistingClientInfo` — hunts the client table for an already
/// loaded model/skin/saber set identical to `ci`'s and adopts its handles.
/// `false` means nothing matched, which is the caller's cue to defer the load.
///
/// Raven: rww - Filthy hack. If this is actually the info already belonging to
/// us, just reassign the pointer. Switching instances when not necessary
/// produces small animation glitches.
/// Source: `oracle/codemp/cgame/cg_players.c:1288-1387`
pub fn CG_ScanForExistingClientInfo(
    ctx: &mut CgContext,
    ci: &mut clientInfo_t,
    clientNum: c_int,
) -> bool {
    let ciModelName = buf_to_string(&ci.modelName.map(|c| c as u8));
    let ciSkinName = buf_to_string(&ci.skinName.map(|c| c as u8));
    let ciSaberName = buf_to_string(&ci.saberName.map(|c| c as u8));
    let ciSaber2Name = buf_to_string(&ci.saber2Name.map(|c| c as u8));

    for i in 0..ctx.world.cgs.maxclients as usize {
        // the whole match test only reads the table; the borrow dies with this
        // block so the copy below can take the entry back out mutably
        let isMatch = {
            let m = &ctx.world.cgs.clientinfo[i];
            m.infoValid != qfalse
                && m.deferred == qfalse
                && Q_stricmp(&ciModelName, &buf_to_string(&m.modelName.map(|c| c as u8))) == 0
                && Q_stricmp(&ciSkinName, &buf_to_string(&m.skinName.map(|c| c as u8))) == 0
                && Q_stricmp(&ciSaberName, &buf_to_string(&m.saberName.map(|c| c as u8))) == 0
                && Q_stricmp(&ciSaber2Name, &buf_to_string(&m.saber2Name.map(|c| c as u8))) == 0
                //			&& !Q_stricmp( ci->headModelName, match->headModelName )
                //			&& !Q_stricmp( ci->headSkinName, match->headSkinName )
                //			&& !Q_stricmp( ci->blueTeam, match->blueTeam )
                //			&& !Q_stricmp( ci->redTeam, match->redTeam )
                && (ctx.world.cgs.gametype < GT_TEAM || ci.team == m.team)
                && ci.siegeIndex == m.siegeIndex
                && !m.ghoul2Model.is_null()
                // if the bolts haven't been initialized, this "match" is useless to us
                && m.bolt_head != 0
        };

        if !isMatch {
            continue;
        }

        // this clientinfo is identical, so use it's handles
        ci.deferred = qfalse;

        if clientNum == i as c_int {
            let matchGhoul2 = ctx.world.cgs.clientinfo[i].ghoul2Model;

            //The match has a valid instance (if it didn't, we'd probably already be fudged (^_^) at this state)
            if !matchGhoul2.is_null() && trap::G2_HaveWeGhoul2Models(ctx.engine, matchGhoul2) {
                //First kill the copy we have if we have one. (but it should be null)
                if !ci.ghoul2Model.is_null()
                    && trap::G2_HaveWeGhoul2Models(ctx.engine, ci.ghoul2Model)
                {
                    trap::G2API_CleanGhoul2Models(
                        ctx.engine,
                        &mut ci.ghoul2Model as *mut *mut c_void,
                    );
                }

                let m = &ctx.world.cgs.clientinfo[i];
                _VectorCopy(m.headOffset, &mut ci.headOffset);
                // ci->footsteps = match->footsteps;
                ci.gender = m.gender;

                ci.legsModel = m.legsModel;
                ci.legsSkin = m.legsSkin;
                ci.torsoModel = m.torsoModel;
                ci.torsoSkin = m.torsoSkin;
                ci.modelIcon = m.modelIcon;

                ci.newAnims = m.newAnims;

                ci.bolt_head = m.bolt_head;
                ci.bolt_lhand = m.bolt_lhand;
                ci.bolt_rhand = m.bolt_rhand;
                ci.bolt_motion = m.bolt_motion;
                ci.bolt_llumbar = m.bolt_llumbar;
                ci.siegeIndex = m.siegeIndex;

                ci.sounds = m.sounds;
                ci.siegeSounds = m.siegeSounds;
                ci.duelSounds = m.duelSounds;

                //We can share this pointer, because it already belongs to this client.
                //The pointer itself and the ghoul2 instance is never actually changed, just passed between
                //clientinfo structures.
                ci.ghoul2Model = m.ghoul2Model;

                // Raven's saber-handle loop below this is commented out
                // (`cg_players.c:1360-1373`) — "whenever this function is called
                // the saber stuff should already be taken care of in the new info".
            }
        } else {
            // borrow split, not behavior: `CG_CopyClientInfoModel` wants the
            // match beside a live `ctx`, so the entry is lifted out of the table
            // for the copy and put straight back. Nothing in the copy reads
            // `cgs.clientinfo`.
            let matched =
                core::mem::replace(&mut ctx.world.cgs.clientinfo[i], zeroed_client_info());
            CG_CopyClientInfoModel(ctx, &matched, ci);
            ctx.world.cgs.clientinfo[i] = matched;
        }

        return true;
    }

    // nothing matches, so defer the load
    false
}

/// Raven `CG_SetLerpFrameAnimation` — starts `newAnimation` on the ghoul2
/// skeleton, picking up mid-animation where it can so a speed change doesn't
/// snap the pose.
///
/// PORT-NOTE: Raven's `lf` is `&cent->pe.legs` at every `torsoOnly == qfalse`
/// call site and `&cent->pe.torso` at every `qtrue` one
/// (`cg_players.c:3308,3326,11190,11191`), so the selector stands in for the
/// aliasing pointer parameter — nothing else was ever passed. Raven's `if (ci)`
/// guards are vestigial too: it derefs `ci` unguarded at lines 2778/2914/2943,
/// so the writes they wrap are unconditional here.
///
/// The broken-arm block after the motion bone is `#if 0`'d out in the oracle
/// (`cg_players.c:2970-3127`) and is not ported.
/// Source: `oracle/codemp/cgame/cg_players.c:2768-3129`
#[allow(clippy::too_many_arguments)]
pub fn CG_SetLerpFrameAnimation(
    ctx: &mut CgContext,
    cent: &mut centity_t,
    ci: &mut clientInfo_t,
    newAnimation: c_int,
    animSpeedMult: f32,
    torsoOnly: bool,
    flipState: bool,
) {
    let mut flags: c_int = BONE_ANIM_OVERRIDE_FREEZE;
    let mut blendTime: c_int = 100;

    if cent.localAnimIndex > 0 {
        //rockettroopers can't have broken arms, nor can anything else but humanoids
        ci.brokenLimbs = cent.currentState.brokenLimbs;
    }

    // the cent scalars the body still needs once `lf` holds the only borrow
    let ghoul2 = cent.ghoul2;
    let localAnimIndex = cent.localAnimIndex;
    let csNumber = cent.currentState.number;
    let csClientNum = cent.currentState.clientNum;
    let csFireflag = cent.currentState.fireflag;
    let csWeapon = cent.currentState.weapon;
    let csBrokenLimbs = cent.currentState.brokenLimbs;
    let csNPCclass = cent.currentState.NPC_class;
    let csTorsoAnim = cent.currentState.torsoAnim;
    let csLegsAnim = cent.currentState.legsAnim;
    let noLumbar = cent.noLumbar != qfalse;

    let lf: &mut lerpFrame_t = if torsoOnly {
        &mut cent.pe.torso
    } else {
        &mut cent.pe.legs
    };

    let oldSpeed = lf.animationSpeed;
    let oldAnim = lf.animationNumber;

    lf.animationNumber = newAnimation;

    if newAnimation < 0 || newAnimation >= animNumber_t::MAX_TOTALANIMATIONS as c_int {
        // Raven's `CG_Error` never returns; ours does, so the bad index is
        // stopped here instead of walking off the animation table.
        CG_Error(ctx, &format!("Bad animation number: {}", newAnimation));
        return;
    }

    // §F19: Raven indexes `bgAllAnims[cent->localAnimIndex]` unchecked — an
    // unparsed skeleton leaves the index at -1 and the table is a `Vec` here, so
    // an out-of-range index leaves the lerp frame exactly as it was.
    if localAnimIndex < 0 || localAnimIndex as usize >= ctx.world.bg_state.bgAllAnims.len() {
        return;
    }
    let animations = ctx.world.bg_state.bgAllAnims[localAnimIndex as usize].anims;
    if animations.is_null() {
        return;
    }

    // SAFETY: `anims` is the animation table `BG_ParseAnimationFile` allocated
    // for this skeleton; Raven indexes it unchecked and so do we.
    let anim = unsafe { animations.offset(newAnimation as isize) };
    let (animFirstFrame, animNumFrames, animFrameLerp, animLoopFrames) = unsafe {
        let a = &*anim;
        (
            a.firstFrame as c_int,
            a.numFrames as c_int,
            a.frameLerp as c_int,
            a.loopFrames as c_int,
        )
    };

    lf.animation = anim;
    lf.animationTime = lf.frameTime + animFrameLerp.abs();

    if localAnimIndex > 1 && animFirstFrame == 0 && animNumFrames == 0 {
        //We'll allow this for non-humanoids.
        return;
    }

    let cg_debugAnim = ctx.world.cvars.cg_debugAnim.integer;
    if cg_debugAnim != 0 && (cg_debugAnim < 0 || cg_debugAnim == csClientNum) {
        let time = ctx.world.cg.time;
        // `GetStringForID`'s only ported home is mp_game's `q_shared.rs`, and
        // mp_cgame does not depend on mp_game (DEC-32 keeps the one copy), so
        // its table walk is spelled out here: stop at the NULL-name sentinel,
        // answer the first id match. A miss can't happen past the range check
        // above, so the empty-string fallback is unreachable.
        let animName = animTable
            .iter()
            .take_while(|e| !e.name.is_null())
            .find(|e| e.id == newAnimation)
            // SAFETY: every non-sentinel `animTable` name is a static
            // NUL-terminated literal, the same read `bg_panimate.rs` makes.
            .map(|e| unsafe { cstr_to_str(e.name) })
            .unwrap_or_default();

        // PORT-NOTE: Raven's two labels are swapped — the legs lerp frame prints
        // "TORSO" and the torso one prints "LEGS". Kept.
        if !torsoOnly {
            CG_Printf(
                ctx,
                &format!(
                    "{}: {} TORSO Anim: {}, '{}'\n",
                    time, csClientNum, newAnimation, animName
                ),
            );
        } else {
            CG_Printf(
                ctx,
                &format!(
                    "{}: {} LEGS Anim: {}, '{}'\n",
                    time, csClientNum, newAnimation, animName
                ),
            );
        }
    }

    if ghoul2.is_null() {
        return;
    }

    let engine = ctx.engine;
    let time = ctx.world.cg.time;

    let mut resumeFrame = false;
    let mut beginFrame: c_int = -1;
    let firstFrame;
    let lastFrame;

    let mut animSpeed = 50.0f32 / animFrameLerp as f32;
    if animLoopFrames != -1 {
        flags = BONE_ANIM_OVERRIDE_LOOP;
    }

    if animSpeed < 0.0 {
        lastFrame = animFirstFrame;
        firstFrame = animFirstFrame + animNumFrames;
    } else {
        firstFrame = animFirstFrame;
        lastFrame = animFirstFrame + animNumFrames;
    }

    if ctx.world.cvars.cg_animBlend.integer != 0 {
        flags |= BONE_ANIM_BLEND;
    }

    if BG_InDeathAnim(newAnimation) != qfalse {
        flags &= !BONE_ANIM_BLEND;
    } else if oldAnim != -1 && BG_InDeathAnim(oldAnim) != qfalse {
        flags &= !BONE_ANIM_BLEND;
    }

    if flags & BONE_ANIM_BLEND != 0 {
        if BG_FlippingAnim(newAnimation) != qfalse {
            blendTime = 200;
        } else if oldAnim != -1 && BG_FlippingAnim(oldAnim) != qfalse {
            blendTime = 200;
        }
    }

    animSpeed *= animSpeedMult;

    let mut callbacks = CgGameCallbacks::new(engine);
    BG_SaberStartTransAnim(
        csNumber,
        csFireflag,
        csWeapon,
        newAnimation,
        &mut animSpeed as *mut f32,
        csBrokenLimbs,
        &mut callbacks,
    );

    if torsoOnly {
        if lf.animationTorsoSpeed != animSpeedMult
            && newAnimation == oldAnim
            && flipState == (lf.lastFlip != qfalse)
        {
            //same animation, but changing speed, so we will want to resume off the frame we're on.
            resumeFrame = true;
        }
        lf.animationTorsoSpeed = animSpeedMult;
    } else {
        if lf.animationSpeed != animSpeedMult
            && newAnimation == oldAnim
            && flipState == (lf.lastFlip != qfalse)
        {
            //same animation, but changing speed, so we will want to resume off the frame we're on.
            resumeFrame = true;
        }
        lf.animationSpeed = animSpeedMult;
    }

    //vehicles may have torso etc but we only want to animate the root bone
    if csNPCclass == class_t::CLASS_VEHICLE as c_int {
        trap::G2API_SetBoneAnim(
            engine,
            ghoul2,
            0,
            "model_root",
            firstFrame,
            lastFrame,
            flags,
            animSpeed,
            time,
            beginFrame as f32,
            blendTime,
        );
        return;
    }

    // PORT-NOTE: Raven passes NULL for `trap_G2API_GetBoneFrame`'s `modelList`;
    // the engine forwards it to `G2_Get_Bone_Anim_Index`, which never
    // dereferences it (`G2_bones.cpp:872-900`), so the wrapper's required slot
    // gets a throwaway int.
    let mut modelList: c_int = 0;

    if torsoOnly && !noLumbar {
        //rww - The guesswork based on the lerp frame figures is usually BS, so I've resorted to a call to get the frame of the bone directly.
        let mut GBAcFrame: f32 = 0.0;
        if resumeFrame {
            //we already checked, and this is the same anim, same flip state, but different speed, so we want to resume with the new speed off of the same frame.
            trap::G2API_GetBoneFrame(
                engine,
                ghoul2,
                "lower_lumbar",
                time,
                &mut GBAcFrame,
                &mut modelList,
                0,
            );
            beginFrame = GBAcFrame as c_int;
        }

        //even if resuming, also be sure to check if we are running the same frame on the legs. If so, we want to use their frame no matter what.
        trap::G2API_GetBoneFrame(
            engine,
            ghoul2,
            "model_root",
            time,
            &mut GBAcFrame,
            &mut modelList,
            0,
        );

        if csTorsoAnim == csLegsAnim
            && GBAcFrame >= animFirstFrame as f32
            && GBAcFrame <= (animFirstFrame + animNumFrames) as f32
        {
            //if the legs are already running this anim, pick up on the exact same frame to avoid the "wobbly spine" problem.
            beginFrame = GBAcFrame as c_int;
        }

        if firstFrame > lastFrame || ci.torsoAnim == newAnimation {
            //don't resume on backwards playing animations.. I guess.
            beginFrame = -1;
        }

        trap::G2API_SetBoneAnim(
            engine,
            ghoul2,
            0,
            "lower_lumbar",
            firstFrame,
            lastFrame,
            flags,
            animSpeed,
            time,
            beginFrame as f32,
            blendTime,
        );

        // Update the torso frame with the new animation (this arm's `lf` IS
        // `cent->pe.torso`, so Raven's explicit write lands here)
        lf.frame = firstFrame;

        ci.torsoAnim = newAnimation;
    } else {
        if resumeFrame {
            //we already checked, and this is the same anim, same flip state, but different speed, so we want to resume with the new speed off of the same frame.
            let mut GBAcFrame: f32 = 0.0;
            trap::G2API_GetBoneFrame(
                engine,
                ghoul2,
                "model_root",
                time,
                &mut GBAcFrame,
                &mut modelList,
                0,
            );
            beginFrame = GBAcFrame as c_int;
        }

        if beginFrame < firstFrame || beginFrame > lastFrame {
            //out of range, don't use it then.
            beginFrame = -1;
        }

        if csTorsoAnim == csLegsAnim && (ci.legsAnim != newAnimation || oldSpeed != animSpeed) {
            //alright, we are starting an anim on the legs, and that same anim is already playing on the toro, so pick up the frame.
            let mut GBAcFrame: f32 = 0.0;
            let oldBeginFrame = beginFrame;

            trap::G2API_GetBoneFrame(
                engine,
                ghoul2,
                "lower_lumbar",
                time,
                &mut GBAcFrame,
                &mut modelList,
                0,
            );
            beginFrame = GBAcFrame as c_int;
            if beginFrame < firstFrame || beginFrame > lastFrame {
                //out of range, don't use it then.
                beginFrame = oldBeginFrame;
            }
        }

        trap::G2API_SetBoneAnim(
            engine,
            ghoul2,
            0,
            "model_root",
            firstFrame,
            lastFrame,
            flags,
            animSpeed,
            time,
            beginFrame as f32,
            blendTime,
        );

        ci.legsAnim = newAnimation;
    }

    if localAnimIndex <= 1 && csTorsoAnim == newAnimation && !noLumbar {
        //make sure we're humanoid before we access the motion bone
        trap::G2API_SetBoneAnim(
            engine,
            ghoul2,
            0,
            "Motion",
            firstFrame,
            lastFrame,
            flags,
            animSpeed,
            time,
            beginFrame as f32,
            blendTime,
        );
    }
}

/// Raven `CG_RagDoll` — decides whether a corpse should go limp and, once it
/// has, feeds the engine's ragdoll solver a fresh position/velocity every frame.
/// `true` means we are ragging and the caller should skip its normal animation
/// work.
/// Source: `oracle/codemp/cgame/cg_players.c:3486-3911`
pub fn CG_RagDoll(ctx: &mut CgContext, cent: &mut centity_t, forcedAngles: &vec3_t) -> bool {
    let mut usedOrg: vec3_t = [0.0; 3];
    let mut inSomething = false;

    if ctx.world.cvars.cg_ragDoll.integer == 0 {
        return false;
    }

    if cent.localAnimIndex != 0 {
        //don't rag non-humanoids
        return false;
    }

    _VectorCopy(cent.lerpOrigin, &mut usedOrg);

    if cent.isRagging == qfalse {
        //If we're not in a ragdoll state, perform the checks.
        if cent.currentState.eFlags & EF_RAG != 0 {
            //want to go into it no matter what then
            inSomething = true;
        } else if cent.currentState.groundEntityNum == ENTITYNUM_NONE {
            let mut cVel: vec3_t = [0.0; 3];

            _VectorCopy(cent.currentState.pos.trDelta, &mut cVel);

            if VectorNormalize(&mut cVel) > 400.0 {
                //if he's flying through the air at a good enough speed, switch into ragdoll
                inSomething = true;
            }
        }

        if cent.currentState.eType == entityType_t::ET_BODY as c_int {
            //just rag bodies immediately if their own was ragging on respawn
            if cent.ownerRagging != qfalse {
                cent.isRagging = qtrue;
                return false;
            }
        }

        if ctx.world.cvars.cg_ragDoll.integer > 1 {
            inSomething = true;
        }

        if !inSomething {
            let anim = cent.currentState.legsAnim;
            let mut dur: c_int = 0;

            // §F19: same unchecked `bgAllAnims` read as elsewhere in this file —
            // with no parsed skeleton the duration stays 0, which just makes the
            // death anim read as finished.
            if let Some(loaded) = ctx
                .world
                .bg_state
                .bgAllAnims
                .get(cent.localAnimIndex as usize)
            {
                if !loaded.anims.is_null() {
                    // SAFETY: Raven indexes this table unchecked and so do we.
                    let (numFrames, frameLerp) = unsafe {
                        let a = &*loaded.anims.offset(anim as isize);
                        (a.numFrames as c_int, a.frameLerp as c_int)
                    };
                    dur = ((numFrames - 1) as f64 * (frameLerp as f32 as f64).abs()) as c_int;
                }
            }

            let mut i: usize = 0;
            let mut boltChecks: [c_int; 5] = [0; 5];
            let mut boltPoints: [vec3_t; 5] = [[0.0; 3]; 5];
            let mut trStart: vec3_t = [0.0; 3];
            let mut trEnd: vec3_t = [0.0; 3];
            let mut tAng: vec3_t = [0.0; 3];
            let mut deathDone = false;
            let mut tr = trace_t::zeroed();
            let mut boltMatrix = mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            };

            VectorSet(
                &mut tAng,
                cent.turAngles[PITCH],
                cent.turAngles[YAW],
                cent.turAngles[ROLL],
            );

            if cent.pe.legs.animationTime > 50
                && (ctx.world.cg.time - cent.pe.legs.animationTime) > dur
            {
                //Looks like the death anim is done playing
                deathDone = true;
            }

            let engine = ctx.engine;
            let ghoul2 = cent.ghoul2;
            let time = ctx.world.cg.time;
            let lerpOrigin = cent.lerpOrigin;
            let modelScale = cent.modelScale;
            let csNumber = cent.currentState.number;

            if deathDone {
                //only trace from the hands if the death anim is already done.
                boltChecks[0] = trap::G2API_AddBolt(engine, ghoul2, 0, "rhand");
                boltChecks[1] = trap::G2API_AddBolt(engine, ghoul2, 0, "lhand");
            } else {
                //otherwise start the trace loop at the cranium.
                i = 2;
            }
            boltChecks[2] = trap::G2API_AddBolt(engine, ghoul2, 0, "cranium");
            //boltChecks[3] = trap_G2API_AddBolt(cent->ghoul2, 0, "rtarsal");
            //boltChecks[4] = trap_G2API_AddBolt(cent->ghoul2, 0, "ltarsal");
            boltChecks[3] = trap::G2API_AddBolt(engine, ghoul2, 0, "rtalus");
            boltChecks[4] = trap::G2API_AddBolt(engine, ghoul2, 0, "ltalus");

            //This may seem bad, but since we have a bone cache now it should manage to not be too disgustingly slow.
            //Do the head first, because the hands reference it anyway.
            trap::G2API_GetBoltMatrix(
                engine,
                ghoul2,
                0,
                boltChecks[2],
                &mut boltMatrix,
                &tAng,
                &lerpOrigin,
                time,
                Some(&mut ctx.world.cgs.gameModels[0]),
                &modelScale,
            );
            BG_GiveMeVectorFromMatrix(
                &boltMatrix,
                Eorientations::ORIGIN as c_int,
                &mut boltPoints[2],
            );

            while i < 5 {
                if i < 2 {
                    //when doing hands, trace to the head instead of origin
                    trap::G2API_GetBoltMatrix(
                        engine,
                        ghoul2,
                        0,
                        boltChecks[i],
                        &mut boltMatrix,
                        &tAng,
                        &lerpOrigin,
                        time,
                        Some(&mut ctx.world.cgs.gameModels[0]),
                        &modelScale,
                    );
                    BG_GiveMeVectorFromMatrix(
                        &boltMatrix,
                        Eorientations::ORIGIN as c_int,
                        &mut boltPoints[i],
                    );
                    _VectorCopy(boltPoints[i], &mut trStart);
                    _VectorCopy(boltPoints[2], &mut trEnd);
                } else {
                    if i > 2 {
                        //2 is the head, which already has the bolt point.
                        trap::G2API_GetBoltMatrix(
                            engine,
                            ghoul2,
                            0,
                            boltChecks[i],
                            &mut boltMatrix,
                            &tAng,
                            &lerpOrigin,
                            time,
                            Some(&mut ctx.world.cgs.gameModels[0]),
                            &modelScale,
                        );
                        BG_GiveMeVectorFromMatrix(
                            &boltMatrix,
                            Eorientations::ORIGIN as c_int,
                            &mut boltPoints[i],
                        );
                    }
                    _VectorCopy(boltPoints[i], &mut trStart);
                    _VectorCopy(lerpOrigin, &mut trEnd);
                }

                //Now that we have all that sorted out, trace between the two points we desire.
                //
                // PORT-NOTE: Raven passes NULL mins/maxs; `CM_Trace` substitutes
                // `vec3_origin` for a NULL bound (`cm_trace.cpp:1603-1610`), so
                // the zero vectors are the same point trace.
                CG_Rag_Trace(
                    ctx,
                    &mut tr,
                    &trStart,
                    &vec3_origin,
                    &vec3_origin,
                    &trEnd,
                    csNumber,
                    MASK_SOLID,
                );

                if tr.fraction != 1.0 || tr.startsolid != 0 || tr.allsolid != 0 {
                    //Hit something or start in solid, so flag it and break.
                    //This is a slight hack, but if we aren't done with the death anim, we don't really want to
                    //go into ragdoll unless our body has a relatively "flat" pitch. (the pitch test itself is
                    //`#if 0`'d out in the oracle, `cg_players.c:3603-3617`)
                    inSomething = true;
                    break;
                }

                i += 1;
            }
        }

        if inSomething {
            cent.isRagging = qtrue;
            // VectorClear(cent->lerpOriginOffset) here is `#if 0`'d in the oracle
        }
    }

    if cent.isRagging != qfalse {
        //We're in a ragdoll state, so make the call to keep our positions updated and whatnot.
        let engine = ctx.engine;
        let ghoul2 = cent.ghoul2;
        let time = ctx.world.cg.time;

        let ragAnim = CG_RagAnimForPositioning(ctx, cent);

        if cent.ikStatus != qfalse {
            //ik must be reset before ragdoll is started, or you'll get some interesting results.
            trap::G2API_SetBoneIKState(
                engine,
                ghoul2,
                time,
                None,
                sharedEIKMoveState::IKS_NONE as c_int,
                None,
            );
            cent.ikStatus = qfalse;
        }

        //these will be used as "base" frames for the ragoll settling.
        let mut startFrame: c_int = 0;
        let mut endFrame: c_int = 0;
        // §F19: the unchecked `bgAllAnims` read again — no skeleton leaves the
        // settle range at 0..0, which the solver treats as "no base pose".
        if let Some(loaded) = ctx
            .world
            .bg_state
            .bgAllAnims
            .get(cent.localAnimIndex as usize)
        {
            if !loaded.anims.is_null() {
                // SAFETY: Raven indexes this table unchecked and so do we.
                let (firstFrame, numFrames) = unsafe {
                    let a = &*loaded.anims.offset(ragAnim as isize);
                    (a.firstFrame as c_int, a.numFrames as c_int)
                };
                startFrame = firstFrame;
                endFrame = firstFrame + numFrames;
            }
        }

        // Raven: with my new method of doing things I want it to continue the anim
        {
            let mut currentFrame: f32 = 0.0;
            let mut gbaStartFrame: c_int = 0;
            let mut gbaEndFrame: c_int = 0;
            let mut gbaFlags: c_int = 0;
            let mut gbaAnimSpeed: f32 = 0.0;

            if trap::G2API_GetBoneAnim(
                engine,
                ghoul2,
                "model_root",
                time,
                &mut currentFrame,
                &mut gbaStartFrame,
                &mut gbaEndFrame,
                &mut gbaFlags,
                &mut gbaAnimSpeed,
                Some(&mut ctx.world.cgs.gameModels[0]),
                0,
            ) {
                //lock the anim on the current frame.
                let blendTime: c_int = 500;

                let mut curFirstFrame: c_int = 0;
                let mut curNumFrames: c_int = 0;
                if let Some(loaded) = ctx
                    .world
                    .bg_state
                    .bgAllAnims
                    .get(cent.localAnimIndex as usize)
                {
                    if !loaded.anims.is_null() {
                        // SAFETY: Raven indexes this table unchecked and so do we.
                        let (ff, nf) = unsafe {
                            let a = &*loaded.anims.offset(cent.currentState.legsAnim as isize);
                            (a.firstFrame as c_int, a.numFrames as c_int)
                        };
                        curFirstFrame = ff;
                        curNumFrames = nf;
                    }
                }

                if currentFrame >= (curFirstFrame + curNumFrames - 1) as f32 {
                    //this is sort of silly but it works for now.
                    currentFrame = (curFirstFrame + curNumFrames - 2) as f32;
                }

                for bone in ["lower_lumbar", "model_root", "Motion"] {
                    trap::G2API_SetBoneAnim(
                        engine,
                        ghoul2,
                        0,
                        bone,
                        currentFrame as c_int,
                        (currentFrame + 1.0) as c_int,
                        gbaFlags,
                        gbaAnimSpeed,
                        time,
                        currentFrame,
                        blendTime,
                    );
                }
            }
        }

        // PORT-NOTE: Raven hands `cgs.gameModels` to these four; the borrow can't
        // ride beside a live `ctx`, and `G2_Set_Bone_Angles` never dereferences
        // `modelList` (`G2_bones.cpp:477-535`), so `None` is the same call.
        for bone in ["upper_lumbar", "lower_lumbar", "thoracic", "cervical"] {
            CG_G2SetBoneAngles(
                ctx,
                ghoul2,
                0,
                bone,
                &vec3_origin,
                BONE_ANGLES_POSTMULT,
                Eorientations::POSITIVE_X as c_int,
                Eorientations::NEGATIVE_Y as c_int,
                Eorientations::NEGATIVE_Z as c_int,
                None,
                0,
                time,
            );
        }

        // §F19: Raven's stack `tParms` is only partly filled — the pelvis
        // offsets, impact strength, begin flag and effector mask are read back
        // out by the engine, never in, so the port sends zeros rather than
        // whatever was on the stack.
        let mut tParms = sharedRagDollParams_t {
            angles: *forcedAngles,
            position: usedOrg,
            scale: cent.modelScale,
            pelvis_angles_offset: [0.0; 3],
            pelvis_position_offset: [0.0; 3],
            f_impact_strength: 0.0,
            f_shot_strength: 4.0,
            me: cent.currentState.number,
            start_frame: startFrame,
            end_frame: endFrame,
            collision_type: 1,
            call_rag_doll_begin: qfalse,
            rag_phase: sharedERagPhase::RP_DEATH_COLLISION as c_int,
            effectors_to_turn_off: 0,
        };

        trap::G2API_SetRagDoll(engine, ghoul2, Some(&mut tParms));

        let mut tuParms = sharedRagDollUpdateParams_t {
            angles: *forcedAngles,
            position: usedOrg,
            scale: cent.modelScale,
            velocity: [0.0; 3],
            me: cent.currentState.number,
            settle_frame: tParms.end_frame - 1,
        };

        if cent.currentState.groundEntityNum != ENTITYNUM_NONE {
            VectorClear(&mut tuParms.velocity);
        } else {
            _VectorScale(cent.currentState.pos.trDelta, 2.0, &mut tuParms.velocity);
        }

        trap::G2API_AnimateG2Models(engine, ghoul2, time, &mut tuParms);

        //So if we try to get a bolt point it's still correct
        cent.turAngles[YAW] = forcedAngles[YAW];
        cent.lerpAngles[YAW] = forcedAngles[YAW];
        cent.pe.torso.yawAngle = forcedAngles[YAW];
        cent.pe.legs.yawAngle = forcedAngles[YAW];

        if cent.currentState.ragAttach != 0
            && (cent.currentState.eType != entityType_t::ET_NPC as c_int
                || cent.currentState.NPC_class != class_t::CLASS_VEHICLE as c_int)
        {
            let grabNum = if cent.currentState.ragAttach == ENTITYNUM_NONE {
                //switch cl 0 and entitynum_none, so we can operate on the "if non-0" concept
                0usize
            } else {
                cent.currentState.ragAttach as usize
            };

            let grabEnt = ctx.world.entity(grabNum);
            let grabGhoul2 = grabEnt.ghoul2;
            let grabTurAngles = grabEnt.turAngles;
            let grabLerpOrigin = grabEnt.lerpOrigin;
            let grabModelScale = grabEnt.modelScale;

            if !grabGhoul2.is_null() {
                let mut matrix = mdxaBone_t {
                    matrix: [[0.0; 4]; 3],
                };
                let mut bOrg: vec3_t = [0.0; 3];
                let mut thisHand: vec3_t = [0.0; 3];
                let mut hands: vec3_t = [0.0; 3];
                let mut pcjMin: vec3_t = [0.0; 3];
                let mut pcjMax: vec3_t = [0.0; 3];
                let mut pDif: vec3_t = [0.0; 3];
                let mut thorPoint: vec3_t = [0.0; 3];

                let turAngles = cent.turAngles;
                let lerpOrigin = cent.lerpOrigin;
                let modelScale = cent.modelScale;

                //Get the person who is holding our hand's hand location
                trap::G2API_GetBoltMatrix(
                    engine,
                    grabGhoul2,
                    0,
                    0,
                    &mut matrix,
                    &grabTurAngles,
                    &grabLerpOrigin,
                    time,
                    Some(&mut ctx.world.cgs.gameModels[0]),
                    &grabModelScale,
                );
                BG_GiveMeVectorFromMatrix(&matrix, Eorientations::ORIGIN as c_int, &mut bOrg);

                //Get our hand's location
                trap::G2API_GetBoltMatrix(
                    engine,
                    ghoul2,
                    0,
                    0,
                    &mut matrix,
                    &turAngles,
                    &lerpOrigin,
                    time,
                    Some(&mut ctx.world.cgs.gameModels[0]),
                    &modelScale,
                );
                BG_GiveMeVectorFromMatrix(&matrix, Eorientations::ORIGIN as c_int, &mut thisHand);

                //Get the position of the thoracic bone for hinting its velocity later on
                let thorBolt = trap::G2API_AddBolt(engine, ghoul2, 0, "thoracic");
                trap::G2API_GetBoltMatrix(
                    engine,
                    ghoul2,
                    0,
                    thorBolt,
                    &mut matrix,
                    &turAngles,
                    &lerpOrigin,
                    time,
                    Some(&mut ctx.world.cgs.gameModels[0]),
                    &modelScale,
                );
                BG_GiveMeVectorFromMatrix(&matrix, Eorientations::ORIGIN as c_int, &mut thorPoint);

                _VectorSubtract(bOrg, thisHand, &mut hands);

                if VectorLength(hands) < 3.0 {
                    trap::G2API_RagForceSolve(engine, ghoul2, false);
                } else {
                    trap::G2API_RagForceSolve(engine, ghoul2, true);
                }

                //got the hand pos of him, now we want to make our hand go to it
                for bone in ["rhand", "rradius", "rradiusX", "rhumerusX", "rhumerus"] {
                    trap::G2API_RagEffectorGoal(engine, ghoul2, bone, Some(&bOrg));
                }

                //Make these two solve quickly so we can update decently
                trap::G2API_RagPCJGradientSpeed(engine, ghoul2, "rhumerus", 1.5);
                trap::G2API_RagPCJGradientSpeed(engine, ghoul2, "rradius", 1.5);

                //Break the constraints on them I suppose
                VectorSet(&mut pcjMin, -999.0, -999.0, -999.0);
                VectorSet(&mut pcjMax, 999.0, 999.0, 999.0);
                trap::G2API_RagPCJConstraint(engine, ghoul2, "rhumerus", &pcjMin, &pcjMax);
                trap::G2API_RagPCJConstraint(engine, ghoul2, "rradius", &pcjMin, &pcjMax);

                cent.overridingBones = time + 2000;

                //hit the thoracic velocity to the hand point
                _VectorSubtract(bOrg, thorPoint, &mut hands);
                VectorNormalize(&mut hands);
                let h = hands;
                _VectorScale(h, 2048.0, &mut hands);
                trap::G2API_RagEffectorKick(engine, ghoul2, "thoracic", &hands);
                trap::G2API_RagEffectorKick(engine, ghoul2, "ceyebrow", &hands);

                _VectorSubtract(cent.ragLastOrigin, cent.lerpOrigin, &mut pDif);
                let o = cent.lerpOrigin;
                _VectorCopy(o, &mut cent.ragLastOrigin);

                if cent.ragLastOriginTime >= time
                    && cent.currentState.groundEntityNum != ENTITYNUM_NONE
                {
                    //make sure it's reasonably updated
                    let difLen = VectorLength(pDif);
                    if difLen > 0.0 {
                        //if we're being dragged, then kick all the bones around a bit
                        if difLen < 12.0 {
                            let p = pDif;
                            _VectorScale(p, 12.0 / difLen, &mut pDif);
                            // Raven stores difLen = 12.0 here; nothing reads it again
                        }

                        for bone in cg_effectorStringTable {
                            let mut dVel: vec3_t = [0.0; 3];
                            let mut rVel: vec3_t = [0.0; 3];

                            _VectorCopy(pDif, &mut dVel);
                            dVel[2] = 0.0;

                            //Factor in a random velocity
                            let rx = ctx.world.bg_state.rng.flrand(-0.1, 0.1);
                            let ry = ctx.world.bg_state.rng.flrand(-0.1, 0.1);
                            let rz = ctx.world.bg_state.rng.flrand(0.1, 0.5);
                            VectorSet(&mut rVel, rx, ry, rz);
                            let r = rVel;
                            _VectorScale(r, 8.0, &mut rVel);

                            let d = dVel;
                            _VectorAdd(d, rVel, &mut dVel);
                            let d = dVel;
                            _VectorScale(d, 10.0, &mut dVel);

                            trap::G2API_RagEffectorKick(engine, ghoul2, bone, &dVel);
                        }
                    }
                }
                cent.ragLastOriginTime = time + 1000;
            }
        } else if cent.overridingBones != 0 {
            //reset things to their normal rag state
            let mut pcjMin: vec3_t = [0.0; 3];
            let mut pcjMax: vec3_t = [0.0; 3];
            let mut dVel: vec3_t = [0.0; 3];

            //got the hand pos of him, now we want to make our hand go to it
            // NULL clears the over-goal - the engine's "go back to none" arm
            // (`oracle/codemp/ghoul2/G2_API.cpp:1552-1555`)
            for bone in ["rhand", "rradius", "rradiusX", "rhumerusX", "rhumerus"] {
                trap::G2API_RagEffectorGoal(engine, ghoul2, bone, None);
            }

            VectorSet(&mut dVel, 0.0, 0.0, -64.0);
            trap::G2API_RagEffectorKick(engine, ghoul2, "rhand", &dVel);

            trap::G2API_RagPCJGradientSpeed(engine, ghoul2, "rhumerus", 0.0);
            trap::G2API_RagPCJGradientSpeed(engine, ghoul2, "rradius", 0.0);

            VectorSet(&mut pcjMin, -100.0, -40.0, -15.0);
            VectorSet(&mut pcjMax, -15.0, 80.0, 15.0);
            trap::G2API_RagPCJConstraint(engine, ghoul2, "rhumerus", &pcjMin, &pcjMax);

            VectorSet(&mut pcjMin, -25.0, -20.0, -20.0);
            VectorSet(&mut pcjMax, 90.0, 20.0, -20.0);
            trap::G2API_RagPCJConstraint(engine, ghoul2, "rradius", &pcjMin, &pcjMax);

            if cent.overridingBones < time {
                trap::G2API_RagForceSolve(engine, ghoul2, false);
                cent.overridingBones = 0;
            } else {
                trap::G2API_RagForceSolve(engine, ghoul2, true);
            }
        }

        return true;
    }

    false
}

/// Raven `CG_G2PlayerHeadAnims` — runs a humanoid's face: blinks, the odd
/// frown, and the talk pose driven by the voice channel's volume. `true` when a
/// face animation was actually started this frame.
///
/// Raven: Dead people close their eyes and don't make faces!
/// Source: `oracle/codemp/cgame/cg_players.c:4059-4189`
pub fn CG_G2PlayerHeadAnims(ctx: &mut CgContext, cent: &mut centity_t) -> bool {
    let mut anim: c_int = -1;

    if cent.localAnimIndex > 1 {
        //only do this for humanoids
        return false;
    }

    if cent.noFace != qfalse {
        // i don't have a face
        return false;
    }

    let number = cent.currentState.number;
    let isClient = number < MAX_CLIENTS_I32;

    // the three facial timers live either in the client table or in the NPC's
    // own info; they are pulled out here and written back at the bottom so the
    // blink/anim calls can hold `ctx` and `cent` at the same time
    let (mut facial_blink, mut facial_frown, mut facial_aux) = if isClient {
        let ci = &ctx.world.cgs.clientinfo[number as usize];
        (ci.facial_blink, ci.facial_frown, ci.facial_aux)
    } else {
        match cent.npcClient.as_ref() {
            Some(ci) => (ci.facial_blink, ci.facial_frown, ci.facial_aux),
            None => return false,
        }
    };

    let time = ctx.world.cg.time;

    if cent.currentState.eFlags & EF_DEAD != 0 {
        //Dead people close their eyes and don't make faces!
        anim = animNumber_t::FACE_DEAD as c_int;
        facial_blink = -1.0;
    } else {
        if facial_blink == 0.0 {
            // set the timers
            facial_blink = time as f32 + ctx.world.bg_state.rng.flrand(4000.0, 8000.0);
            facial_frown = time as f32 + ctx.world.bg_state.rng.flrand(6000.0, 10000.0);
            facial_aux = time as f32 + ctx.world.bg_state.rng.flrand(6000.0, 10000.0);
        }

        //are we blinking?
        if facial_blink < 0.0 {
            // yes, check if we are we done blinking ?
            if -facial_blink < time as f32 {
                // yes, so reset blink timer
                facial_blink = time as f32 + ctx.world.bg_state.rng.flrand(4000.0, 8000.0);
                CG_G2SetHeadBlink(ctx, cent, false); //stop the blink
            }
        } else {
            // no we aren't blinking
            if facial_blink < time as f32 {
                // but should we start ?
                CG_G2SetHeadBlink(ctx, cent, true);
                if facial_blink == 1.0 {
                    //requested to stay shut by SET_FACEEYESCLOSED
                    facial_blink = -(time as f32 + 99999999.0f32); // set blink timer
                } else {
                    facial_blink = -(time as f32 + 300.0f32); // set blink timer
                }
            }
        }

        let voiceVolume = trap::S_GetVoiceVolume(ctx.engine, number);

        if voiceVolume > 0 {
            // if we aren't talking, then it will be 0, -1 for talking but paused
            anim = animNumber_t::FACE_TALK1 as c_int + voiceVolume - 1;
        } else if voiceVolume == 0 {
            //don't do aux if in a slient part of speech
            //not talking
            if facial_aux < 0.0 {
                // are we auxing ? yes
                if -facial_aux < time as f32 {
                    // are we done auxing ? yes, reset aux timer
                    facial_aux = time as f32 + ctx.world.bg_state.rng.flrand(7000.0, 10000.0);
                } else {
                    // not yet, so choose aux
                    anim = animNumber_t::FACE_ALERT as c_int;
                }
            } else {
                // no we aren't auxing, but should we start ?
                if facial_aux < time as f32 {
                    //yes
                    anim = animNumber_t::FACE_ALERT as c_int;
                    // set aux timer
                    facial_aux = (-(time as f64 + 2000.0)) as f32;
                }
            }

            if anim != -1 {
                //we we are auxing, see if we should override with a frown
                if facial_frown < 0.0 {
                    // are we frowning ? yes,
                    if -facial_frown < time as f32 {
                        //are we done frowning ? yes, reset frown timer
                        facial_frown = time as f32 + ctx.world.bg_state.rng.flrand(7000.0, 10000.0);
                    } else {
                        // not yet, so choose frown
                        anim = animNumber_t::FACE_FROWN as c_int;
                    }
                } else {
                    // no we aren't frowning, but should we start ?
                    if facial_frown < time as f32 {
                        anim = animNumber_t::FACE_FROWN as c_int;
                        // set frown timer
                        facial_frown = (-(time as f64 + 2000.0)) as f32;
                    }
                }
            }
        } //talking
    } //dead

    if isClient {
        let ci = &mut ctx.world.cgs.clientinfo[number as usize];
        ci.facial_blink = facial_blink;
        ci.facial_frown = facial_frown;
        ci.facial_aux = facial_aux;
    } else if let Some(ci) = cent.npcClient.as_mut() {
        ci.facial_blink = facial_blink;
        ci.facial_frown = facial_frown;
        ci.facial_aux = facial_aux;
    }

    if anim != -1 {
        CG_G2SetHeadAnim(ctx, cent, anim);
        return true;
    }

    false
}

/// Raven `CG_PlayerFlag` — the CTF flag hanging off a carrier's lower lumbar,
/// canted back and to the right. Never drawn on your own body in first person.
/// Source: `oracle/codemp/cgame/cg_players.c:4367-4441`
pub fn CG_PlayerFlag(ctx: &mut CgContext, cent: &centity_t, hModel: qhandle_t) {
    let mut angles: vec3_t = [0.0; 3];
    let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];
    let mut boltOrg: vec3_t = [0.0; 3];
    let mut tAng: vec3_t = [0.0; 3];
    let mut getAng: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut boltMatrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };

    // PORT-NOTE: Raven derefs `cg.snap` unguarded; with no snapshot yet nobody
    // is "us", so the flag renders.
    let ourClientNum = ctx.world.cg.snap_ref().map_or(-1, |snap| snap.ps.clientNum);

    if cent.currentState.number == ourClientNum && ctx.world.cg.renderingThirdPerson == 0 {
        return;
    }

    if cent.ghoul2.is_null() {
        return;
    }

    let bolt_llumbar = if cent.currentState.eType == entityType_t::ET_NPC as c_int {
        match cent.npcClient.as_ref() {
            Some(ci) => ci.bolt_llumbar,
            None => {
                // §F19: Raven asserts here and then derefs the NULL anyway; a
                // flagless NPC just doesn't get a flag drawn.
                debug_assert!(false, "flag-carrying NPC with no npcClient");
                return;
            }
        }
    } else {
        ctx.world.cgs.clientinfo[cent.currentState.number as usize].bolt_llumbar
    };

    VectorSet(
        &mut tAng,
        cent.turAngles[PITCH],
        cent.turAngles[YAW],
        cent.turAngles[ROLL],
    );

    let engine = ctx.engine;
    let time = ctx.world.cg.time;
    trap::G2API_GetBoltMatrix(
        engine,
        cent.ghoul2,
        0,
        bolt_llumbar,
        &mut boltMatrix,
        &tAng,
        &cent.lerpOrigin,
        time,
        Some(&mut ctx.world.cgs.gameModels[0]),
        &cent.modelScale,
    );
    BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut boltOrg);

    BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::POSITIVE_X as c_int, &mut tAng);
    let t = tAng;
    vectoangles(t, &mut tAng);

    _VectorCopy(cent.lerpAngles, &mut angles);

    boltOrg[2] -= 12.0;
    VectorSet(&mut getAng, 0.0, cent.lerpAngles[1], 0.0);
    AngleVectors(getAng, None, Some(&mut right), None);
    boltOrg[0] += right[0] * 8.0;
    boltOrg[1] += right[1] * 8.0;
    boltOrg[2] += right[2] * 8.0;

    angles[PITCH] = -cent.lerpAngles[PITCH] / 2.0 - 30.0;
    angles[YAW] = tAng[YAW] + 270.0;

    AnglesToAxis(angles, axis.as_mut_ptr());

    let mut ent = refEntity_t::zeroed();
    _VectorMA(boltOrg, 24.0, axis[0], &mut ent.origin);

    angles[ROLL] += 20.0;
    AnglesToAxis(angles, ent.axis.as_mut_ptr());

    ent.hModel = hModel;

    ent.modelScale[0] = 0.5;
    ent.modelScale[1] = 0.5;
    ent.modelScale[2] = 0.5;
    ScaleModelAxis(&mut ent);

    // Raven's RF_FORCE_ENT_ALPHA pass for our own back is commented out
    // (`cg_players.c:4431-4437`): "Not doing this at the moment because sorting
    // totally messes up".

    trap::R_AddRefEntityToScene(engine, &ent);
}

/// Raven `CG_PlayerSprites` — the icon floating over a player's head: the
/// connection glitch beats everything, then a voice-chat balloon, then a plain
/// talk balloon. Mind-tricked entities draw nothing.
/// Source: `oracle/codemp/cgame/cg_players.c:4572-4600`
pub fn CG_PlayerSprites(ctx: &mut CgContext, cent: &centity_t) {
    if let Some(snap) = ctx.world.cg.snap_ref() {
        let ourClientNum = snap.ps.clientNum;
        if CG_IsMindTricked(
            ctx.world,
            cent.currentState.trickedentindex,
            cent.currentState.trickedentindex2,
            cent.currentState.trickedentindex3,
            cent.currentState.trickedentindex4,
            ourClientNum,
        ) {
            return; //this entity is mind-tricking the current client, so don't render it
        }
    }

    if cent.currentState.eFlags & EF_CONNECTION != 0 {
        let shader = ctx.world.cgs.media.connectionShader;
        CG_PlayerFloatSprite(ctx, cent, shader);
        return;
    }

    if cent.vChatTime > ctx.world.cg.time {
        let shader = ctx.world.cgs.media.vchatShader;
        CG_PlayerFloatSprite(ctx, cent, shader);
    } else if cent.currentState.eType != entityType_t::ET_NPC as c_int
        //don't draw talk balloons on NPCs
        && (cent.currentState.eFlags & EF_TALK) != 0
    {
        let shader = ctx.world.cgs.media.balloonShader;
        CG_PlayerFloatSprite(ctx, cent, shader);
    }
}

/// Raven `CG_AddRefEntityWithPowerups` — the one gate every player refEnt goes
/// through: an entity mind-tricking us is simply not submitted.
///
/// `team` is dead in Raven's body (the powerup shells it named are gone).
/// Source: `oracle/codemp/cgame/cg_players.c:5059-5071`
#[allow(unused_variables)]
pub fn CG_AddRefEntityWithPowerups(
    ctx: &mut CgContext,
    ent: &refEntity_t,
    state: &entityState_t,
    team: c_int,
) {
    // PORT-NOTE: Raven derefs `cg.snap` unguarded; with no snapshot there is
    // nobody to be tricked, so the entity renders.
    if let Some(snap) = ctx.world.cg.snap_ref() {
        let ourClientNum = snap.ps.clientNum;
        if CG_IsMindTricked(
            ctx.world,
            state.trickedentindex,
            state.trickedentindex2,
            state.trickedentindex3,
            state.trickedentindex4,
            ourClientNum,
        ) {
            return; //this entity is mind-tricking the current client, so don't render it
        }
    }

    trap::R_AddRefEntityToScene(ctx.engine, ent);
}

/// Raven `CG_PlayerHitFX` — the personal-shield flash on somebody who just took
/// a hit. Ships get nothing, and neither do you in first person.
///
/// Raven: only do the below fx if the cent in question is...uh...me, and it's
/// first person.
/// Source: `oracle/codemp/cgame/cg_players.c:5145-5158`
pub fn CG_PlayerHitFX(ctx: &mut CgContext, cent: &centity_t) {
    if cent.currentState.clientNum != ctx.world.cg.predictedPlayerState.clientNum
        || ctx.world.cg.renderingThirdPerson != 0
    {
        if cent.damageTime > ctx.world.cg.time
            && cent.currentState.NPC_class != class_t::CLASS_VEHICLE as c_int
        {
            CG_DrawPlayerShield(ctx, cent, &cent.lerpOrigin);
        }
    }
}

/// Raven `CG_DoSaberLight` — one dynamic light for a whole saber, averaging the
/// colour and midpoint of every lit blade and sizing the radius off how far
/// apart the tips are.
///
/// Raven: RGB combine all the colors of the sabers you're using into one
/// averaged color!
/// Source: `oracle/codemp/cgame/cg_players.c:5234-5318`
pub fn CG_DoSaberLight(ctx: &mut CgContext, saber: Option<&saberInfo_t>) {
    let mut positions: [vec3_t; MAX_BLADES * 2] = [[0.0; 3]; MAX_BLADES * 2];
    let mut mid: vec3_t = [0.0; 3];
    let mut rgbs: [vec3_t; MAX_BLADES * 2] = [[0.0; 3]; MAX_BLADES * 2];
    let mut rgb: vec3_t = [0.0; 3];
    let mut lengths: [f32; MAX_BLADES * 2] = [0.0; MAX_BLADES * 2];
    let mut totallength: f32 = 0.0;
    // Raven's counter is a float, and the average below divides by it as one
    let mut numpositions: f32 = 0.0;
    let mut diameter: f32 = 0.0;

    let saber = match saber {
        Some(saber) => saber,
        None => return,
    };

    if saber.saberFlags2 & SFL2_NO_DLIGHT != 0 {
        //no dlight!
        return;
    }

    for i in 0..saber.numBlades as usize {
        if saber.blade[i].length >= 0.5 {
            //FIXME: make RGB sabers
            CG_RGBForSaberColor(saber.blade[i].color, &mut rgbs[i]);
            lengths[i] = saber.blade[i].length;
            if saber.blade[i].length * 2.0 > diameter {
                diameter = saber.blade[i].length * 2.0;
            }
            totallength += saber.blade[i].length;
            _VectorMA(
                saber.blade[i].muzzlePoint,
                saber.blade[i].length,
                saber.blade[i].muzzleDir,
                &mut positions[i],
            );
            if numpositions == 0.0 {
                //first blade, store middle of that as midpoint
                _VectorMA(
                    saber.blade[i].muzzlePoint,
                    saber.blade[i].length * 0.5,
                    saber.blade[i].muzzleDir,
                    &mut mid,
                );
                _VectorCopy(rgbs[i], &mut rgb);
            }
            numpositions += 1.0;
        }
    }

    if totallength != 0.0 {
        //actually have something to do
        if numpositions == 1.0 {
            //only 1 blade, midpoint is already set (halfway between the start and end of that blade), rgb is already set, so it diameter
        } else {
            //multiple blades, calc averages
            VectorClear(&mut mid);
            VectorClear(&mut rgb);
            //now go through all the data and get the average RGB and middle position and the radius
            for i in 0..MAX_BLADES * 2 {
                if lengths[i] != 0.0 {
                    let r = rgb;
                    _VectorMA(r, lengths[i], rgbs[i], &mut rgb);
                    let m = mid;
                    _VectorAdd(m, positions[i], &mut mid);
                }
            }

            //get middle rgb
            let r = rgb;
            //get the average, normalized RGB
            _VectorScale(r, 1.0 / totallength, &mut rgb);
            //get mid position
            let m = mid;
            _VectorScale(m, 1.0 / numpositions, &mut mid);
            //find the farthest distance between the blade tips, this will be our diameter
            for i in 0..MAX_BLADES * 2 {
                if lengths[i] != 0.0 {
                    for j in 0..MAX_BLADES * 2 {
                        if lengths[j] != 0.0 {
                            let dist = Distance(positions[i], positions[j]);
                            if dist > diameter {
                                diameter = dist;
                            }
                        }
                    }
                }
            }
        }

        let jitter = ctx.world.bg_state.rng.random() * 8.0;
        trap::R_AddLightToScene(ctx.engine, &mid, diameter + jitter, rgb[0], rgb[1], rgb[2]);
    }
}

/// Raven `CG_DoSaber` — draws one blade: a sprite-based glow ref-ent plus the
/// white-hot line core, both radius-jittered per frame. Blades under half a unit
/// aren't worth submitting.
///
/// Raven: Jeff, I did this because I foolishly wished to have a bright halo as
/// the saber is unleashed. It's not quite what I'd hoped tho. If you have any
/// ideas, go for it! --Pat
/// Source: `oracle/codemp/cgame/cg_players.c:5320-5430`
#[allow(clippy::too_many_arguments)]
pub fn CG_DoSaber(
    ctx: &mut CgContext,
    origin: &vec3_t,
    dir: &vec3_t,
    length: f32,
    lengthMax: f32,
    radius: f32,
    color: saber_colors_t,
    rfx: c_int,
    doLight: bool,
) {
    let mut rfx = rfx;
    let mut mid: vec3_t = [0.0; 3];

    if length < 0.5 {
        // if the thing is so short, just forget even adding me.
        return;
    }

    // Find the midpoint of the saber for lighting purposes
    _VectorMA(*origin, length * 0.5, *dir, &mut mid);

    let media = &ctx.world.cgs.media;
    let (glow, blade) = match color {
        SABER_RED => (media.redSaberGlowShader, media.redSaberCoreShader),
        SABER_ORANGE => (media.orangeSaberGlowShader, media.orangeSaberCoreShader),
        SABER_YELLOW => (media.yellowSaberGlowShader, media.yellowSaberCoreShader),
        SABER_GREEN => (media.greenSaberGlowShader, media.greenSaberCoreShader),
        SABER_BLUE => (media.blueSaberGlowShader, media.blueSaberCoreShader),
        SABER_PURPLE => (media.purpleSaberGlowShader, media.purpleSaberCoreShader),
        _ => (media.blueSaberGlowShader, media.blueSaberCoreShader),
    };

    if doLight {
        // always add a light because sabers cast a nice glow before they slice you in half!!  or something...
        let mut rgb: vec3_t = [1.0, 1.0, 1.0];
        CG_RGBForSaberColor(color, &mut rgb);
        let jitter = ctx.world.bg_state.rng.random() * 3.0;
        trap::R_AddLightToScene(
            ctx.engine,
            &mid,
            (length * 1.4) + jitter,
            rgb[0],
            rgb[1],
            rgb[2],
        );
    }

    let mut saber = refEntity_t::zeroed();

    // Saber glow is it's own ref type because it uses a ton of sprites, otherwise it would eat up too many
    //	refEnts to do each glow blob individually
    saber.saberLength = length;

    let radiusmult: f32 = if length < lengthMax {
        // Note this creates a curve, and length cannot be < 0.5.
        (1.0f64 + (2.0f64 / length as f64)) as f32
    } else {
        1.0
    };

    if ctx.world.cvars.cg_saberTrail.integer == 2
        && ctx.world.cvars.cg_shadows.integer != 2
        && ctx.world.cgs.glconfig.stencilBits >= 4
    {
        //draw the blade as a post-render so it doesn't get in the cap...
        rfx |= RF_FORCEPOST;
    }

    let radiusRange = radius * 0.075;
    let mut radiusStart = radius - radiusRange;

    saber.radius = ((radiusStart as f64 + ctx.world.bg_state.rng.crandom() * radiusRange as f64)
        * radiusmult as f64) as f32;
    //saber.radius = (2.8f + crandom() * 0.2f)*radiusmult;

    _VectorCopy(*origin, &mut saber.origin);
    _VectorCopy(*dir, &mut saber.axis[0]);
    saber.reType = refEntityType_t::RT_SABER_GLOW;
    saber.customShader = glow;
    saber.shaderRGBA = [0xff; 4];
    saber.renderfx = rfx;

    trap::R_AddRefEntityToScene(ctx.engine, &saber);

    // Do the hot core
    _VectorMA(*origin, length, *dir, &mut saber.origin);
    _VectorMA(*origin, -1.0, *dir, &mut saber.oldorigin);

    saber.customShader = blade;
    saber.reType = refEntityType_t::RT_LINE;
    radiusStart = radius / 3.0;
    saber.radius = ((radiusStart as f64 + ctx.world.bg_state.rng.crandom() * radiusRange as f64)
        * radiusmult as f64) as f32;
    //	saber.radius = (1.0 + crandom() * 0.2f)*radiusmult;

    saber.shaderTexCoord[0] = 1.0;
    saber.shaderTexCoord[1] = 1.0;
    saber.shaderRGBA = [0xff; 4];

    trap::R_AddRefEntityToScene(ctx.engine, &saber);
}

/// Raven `CG_AddRandomLightning` — jitters both ends of a lightning bolt before
/// handing it to [`CG_AddLightningBeam`], so a held stream never draws the same
/// arc twice.
/// Source: `oracle/codemp/cgame/cg_players.c:6699-6740`
pub fn CG_AddRandomLightning(ctx: &mut CgContext, start: &vec3_t, end: &vec3_t) {
    let mut inOrg: vec3_t = [0.0; 3];
    let mut outOrg: vec3_t = [0.0; 3];

    _VectorCopy(*start, &mut inOrg);
    _VectorCopy(*end, &mut outOrg);

    if ctx.world.bg_state.rng.rand() & 1 != 0 {
        outOrg[0] += ctx.world.bg_state.rng.Q_irand(0, 24) as f32;
        inOrg[0] += ctx.world.bg_state.rng.Q_irand(0, 8) as f32;
    } else {
        outOrg[0] -= ctx.world.bg_state.rng.Q_irand(0, 24) as f32;
        inOrg[0] -= ctx.world.bg_state.rng.Q_irand(0, 8) as f32;
    }

    if ctx.world.bg_state.rng.rand() & 1 != 0 {
        outOrg[1] += ctx.world.bg_state.rng.Q_irand(0, 24) as f32;
        inOrg[1] += ctx.world.bg_state.rng.Q_irand(0, 8) as f32;
    } else {
        outOrg[1] -= ctx.world.bg_state.rng.Q_irand(0, 24) as f32;
        inOrg[1] -= ctx.world.bg_state.rng.Q_irand(0, 8) as f32;
    }

    if ctx.world.bg_state.rng.rand() & 1 != 0 {
        outOrg[2] += ctx.world.bg_state.rng.Q_irand(0, 50) as f32;
        inOrg[2] += ctx.world.bg_state.rng.Q_irand(0, 40) as f32;
    } else {
        outOrg[2] -= ctx.world.bg_state.rng.Q_irand(0, 64) as f32;
        inOrg[2] -= ctx.world.bg_state.rng.Q_irand(0, 40) as f32;
    }

    CG_AddLightningBeam(ctx, &inOrg, &outOrg);
}

/// Raven `CG_VehicleEffects` — a ship's per-frame effect pass: the hyperspace
/// star streak, the flyby whoosh, engine start, exhaust/trail plumes and the
/// damage smoke off broken surfaces.
///
/// DEFERRED past the hyperspace block: everything below it reads
/// `m_pVehicle->m_pVehicleInfo` (`soundFlyBy`/`soundFlyBy2`, `soundEngineStart`,
/// `type`, `surfDestruction`, `iTrailFX`/`iExhaustFX`/`iTurboFX`/`iInjureFX`/
/// `iDmgFX`) plus `m_iExhaustTag[]` and `m_iLastFXTime`, and DEC-46.2's
/// `Option<VehicleId>` carries only the vehicle cent's entity number until the
/// `Vehicle_t` referent pool lands ("ported code only tests presence" —
/// `local/vehicle_id.rs`). `CG_CreateSurfaceDebris`/`CG_CreateSurfaceSmoke` and
/// the traps those arms need all exist; only the vehicle data is missing.
/// Source: `oracle/codemp/cgame/cg_players.c:7981-8305`
pub fn CG_VehicleEffects(ctx: &mut CgContext, cent: &centity_t) {
    if cent.currentState.eType != entityType_t::ET_NPC as c_int
        || cent.currentState.NPC_class != class_t::CLASS_VEHICLE as c_int
        || cent.m_pVehicle.is_none()
    {
        return;
    }

    if cent.currentState.clientNum == ctx.world.cg.predictedPlayerState.m_iVehicleNum //my vehicle
        && (cent.currentState.eFlags2 & EF2_HYPERSPACE) != 0
    //hyperspacing
    {
        //in hyperspace!
        let hyperSpaceTime = ctx.world.cg.predictedVehicleState.hyperSpaceTime;
        let time = ctx.world.cg.time;

        if hyperSpaceTime != 0 && (time - hyperSpaceTime) < HYPERSPACE_TIME {
            let lastEffect = ctx.world.players.cg_lastHyperSpaceEffectTime;
            if lastEffect == 0 || (time - lastEffect) > HYPERSPACE_TIME + 500 {
                //can't be from the last time we were in hyperspace, so play the effect!
                trap::FX_PlayBoltedEffectID(
                    ctx.engine,
                    ctx.world.cgs.effects.mHyperspaceStars,
                    &cent.lerpOrigin,
                    cent.ghoul2,
                    0,
                    cent.currentState.number,
                    0,
                    0,
                    true,
                );
                ctx.world.players.cg_lastHyperSpaceEffectTime = time;
            }
        }
    }

    // DEFERRED: Vehicle_t referent pool — see the fn doc. The FLYBY sound, the
    // engine-start rev, and the whole `type != VH_ANIMAL` effect body (surface
    // destruction, exhaust, wing trails, death flames, damage smoke) all hang off
    // `pVehNPC->m_pVehicleInfo`, which `Option<VehicleId>` cannot reach yet.
}

/// Raven `CG_CustomSound` — resolves a `*`-prefixed custom sound reference
/// against a client's (or NPC's) per-model sound table, falling back to a
/// straight `trap_S_RegisterSound` for anything that isn't a custom
/// reference.
/// Source: `oracle/codemp/cgame/cg_players.c:170-304`
pub fn CG_CustomSound(ctx: &mut CgContext, clientNum: c_int, soundName: &str) -> sfxHandle_t {
    if !soundName.starts_with('*') {
        return trap::S_RegisterSound(ctx.engine, soundName);
    }

    let lSoundName = COM_StripExtension(soundName);

    let clientNum = if clientNum < 0 { 0 } else { clientNum };
    let isNpc = clientNum as usize >= MAX_CLIENTS;

    let ci = if isNpc {
        ctx.world.entities[clientNum as usize].npcClient.as_deref()
    } else {
        Some(&ctx.world.cgs.clientinfo[clientNum as usize])
    };

    let ci = match ci {
        Some(ci) => ci,
        None => return 0,
    };

    let numCSounds = cg_customSoundNames
        .iter()
        .position(Option::is_none)
        .unwrap_or(MAX_CUSTOM_SOUNDS);

    let mut numCComSounds = 0;
    let mut numCExSounds = 0;
    let mut numCJediSounds = 0;

    if isNpc {
        // these are only for npc's
        numCComSounds = cg_customCombatSoundNames
            .iter()
            .position(Option::is_none)
            .unwrap_or(MAX_CUSTOM_COMBAT_SOUNDS);
        numCExSounds = cg_customExtraSoundNames
            .iter()
            .position(Option::is_none)
            .unwrap_or(MAX_CUSTOM_EXTRA_SOUNDS);
        numCJediSounds = cg_customJediSoundNames
            .iter()
            .position(Option::is_none)
            .unwrap_or(MAX_CUSTOM_JEDI_SOUNDS);
    }

    let siege = ctx.world.cgs.gametype >= GT_TEAM || ctx.world.cvars.cg_buildScript.integer != 0;
    // PORT-NOTE: Raven's counting loop scans `bg_customSiegeSoundNames` up to
    // `MAX_CUSTOM_SOUNDS` (40), but the array itself is `MAX_CUSTOM_SIEGE_SOUNDS`
    // (30) long (own comment: "for now these must all be the same" — they
    // aren't) — an OOB C read past the real 30 entries. `.position()` only ever
    // walks the array's real length, so it can't reproduce that read; every
    // table has its `None` sentinel well inside 30, so the count is identical
    // either way.
    let mut numCSiegeSounds = 0;
    if siege {
        // siege only
        numCSiegeSounds = bg_customSiegeSoundNames
            .iter()
            .position(Option::is_none)
            .unwrap_or(bg_customSiegeSoundNames.len());
    }

    let duel = ctx.world.cgs.gametype == GT_DUEL
        || ctx.world.cgs.gametype == GT_POWERDUEL
        || ctx.world.cvars.cg_buildScript.integer != 0;
    let mut numCDuelSounds = 0;
    if duel {
        // Duel only
        numCDuelSounds = cg_customDuelSoundNames
            .iter()
            .position(Option::is_none)
            .unwrap_or(MAX_CUSTOM_DUEL_SOUNDS);
    }

    for i in 0..MAX_CUSTOM_SOUNDS {
        if i < numCSounds && cg_customSoundNames[i] == Some(lSoundName.as_str()) {
            return ci.sounds[i];
        } else if siege
            && i < numCSiegeSounds
            && bg_customSiegeSoundNames[i].and_then(|s| s.to_str().ok())
                == Some(lSoundName.as_str())
        {
            // siege only
            return ci.siegeSounds[i];
        } else if duel
            && i < numCDuelSounds
            && cg_customDuelSoundNames[i] == Some(lSoundName.as_str())
        {
            // siege only
            return ci.duelSounds[i];
        } else if isNpc
            && i < numCComSounds
            && cg_customCombatSoundNames[i] == Some(lSoundName.as_str())
        {
            // npc only
            return ci.combatSounds[i];
        } else if isNpc
            && i < numCExSounds
            && cg_customExtraSoundNames[i] == Some(lSoundName.as_str())
        {
            // npc only
            return ci.extraSounds[i];
        } else if isNpc
            && i < numCJediSounds
            && cg_customJediSoundNames[i] == Some(lSoundName.as_str())
        {
            // npc only
            return ci.jediSounds[i];
        }
    }

    //CG_Error( "Unknown custom sound: %s", lSoundName );
    //
    // PORT-NOTE: FINAL_BUILD is never defined for this port (release parity),
    // so Raven's `#ifndef FINAL_BUILD` diagnostic always fires.
    Com_Printf(ctx, &format!("Unknown custom sound: {}", lSoundName));
    0
}

/// Raven `CG_ParseSurfsFile` — loads `models/players/<model>/model_<skin>.surf`
/// and folds its `surfOff`/`surfOn` directives into two comma-joined lists.
///
/// Out-params become the return value: `Some((surfOff, surfOn))` for Raven's
/// `qtrue`, `None` for `qfalse`. Raven's
/// `memset( (char *)surfOff, 0, sizeof(surfOff) )` only clears
/// `sizeof(char*)` bytes of the caller's buffer (a sizeof-on-a-pointer-param
/// bug) — but that's enough to zero byte 0, the only byte either
/// `if (surfOff[0])` check ever reads, so the accumulators behave exactly as
/// if freshly cleared regardless of what a caller passed in. The port starts
/// both empty and hands them back instead of writing through caller buffers.
/// Source: `oracle/codemp/cgame/cg_players.c:314-409`
pub fn CG_ParseSurfsFile(
    ctx: &mut CgContext,
    modelName: &str,
    skinName: &str,
) -> Option<(String, String)> {
    // this is a multi-part skin, said skins do not support .surf files
    if skinName.contains('|') {
        return None;
    }

    // Load and parse .surf file
    let sfilename = format!("models/players/{}/model_{}.surf", modelName, skinName);

    // load the file
    let mut f: fileHandle_t = 0;
    let len = trap::FS_FOpenFile(ctx.engine, &sfilename, &mut f, FS_READ);
    if len <= 0 {
        // no file
        return None;
    }
    if len >= 20000 - 1 {
        Com_Printf(ctx, &format!("File {} too long\n", sfilename));
        return None;
    }

    let mut text = vec![0u8; len as usize];
    trap::FS_Read(ctx.engine, &mut text, f);
    trap::FS_FCloseFile(ctx.engine, f);

    // parse the text
    let mut text_p: Option<&[u8]> = Some(&text[..]);

    let mut surfOff = String::new();
    let mut surfOn = String::new();
    let mut qs = QSharedScratch::zeroed();

    // read information for surfOff and surfOn
    loop {
        let (token, rest) = COM_ParseExt(&mut qs, text_p, true);
        text_p = rest;
        if token.is_empty() {
            break;
        }

        // surfOff
        if Q_stricmp(&token, "surfOff") == 0 {
            // Raven's `COM_ParseString` guard tests the (always non-NULL)
            // token pointer, never the parsed value, so its `continue`-on-error
            // arm is dead (see `COM_ParseString`'s own doc); dropped here too.
            let (value, rest2) = COM_ParseString(&mut qs, text_p);
            text_p = rest2;

            if !surfOff.is_empty() {
                strcat_string(&mut surfOff, MAX_SURF_LIST_SIZE, ",");
                strcat_string(&mut surfOff, MAX_SURF_LIST_SIZE, &value);
            } else {
                surfOff = strncpyz_string(value.as_bytes(), MAX_SURF_LIST_SIZE);
            }
            continue;
        }

        // surfOn
        if Q_stricmp(&token, "surfOn") == 0 {
            let (value, rest2) = COM_ParseString(&mut qs, text_p);
            text_p = rest2;

            if !surfOn.is_empty() {
                strcat_string(&mut surfOn, MAX_SURF_LIST_SIZE, ",");
                strcat_string(&mut surfOn, MAX_SURF_LIST_SIZE, &value);
            } else {
                surfOn = strncpyz_string(value.as_bytes(), MAX_SURF_LIST_SIZE);
            }
            continue;
        }
    }

    Some((surfOff, surfOn))
}

/// Raven `CG_RunLerpFrame` — advances one lerp-frame's animation state
/// (torso or legs, chosen by `torsoOnly`) by one client render frame.
///
/// The port derives `lf` from `cent.pe.torso`/`cent.pe.legs` by `torsoOnly`
/// rather than threading Raven's `lerpFrame_t *lf` param — every oracle call
/// site passes exactly one of those two fields (`cg_players.c:3308,3326`),
/// matching the already-ported `CG_SetLerpFrameAnimation`'s own shape.
/// Source: `oracle/codemp/cgame/cg_players.c:3185-3235`
pub fn CG_RunLerpFrame(
    ctx: &mut CgContext,
    cent: &mut centity_t,
    ci: &mut clientInfo_t,
    flipState: bool,
    newAnimation: c_int,
    speedScale: f32,
    torsoOnly: bool,
) {
    // debugging tool to get no animations
    if ctx.world.cvars.cg_animSpeed.integer == 0 {
        let lf = if torsoOnly {
            &mut cent.pe.torso
        } else {
            &mut cent.pe.legs
        };
        lf.oldFrame = 0;
        lf.frame = 0;
        lf.backlerp = 0.0;
        return;
    }

    let ghoul2 = cent.ghoul2;
    let forceFrame = cent.currentState.forceFrame;
    let csBrokenLimbs = cent.currentState.brokenLimbs;
    let time = ctx.world.cg.time;

    // see if the animation sequence is switching
    if forceFrame != 0 {
        let lf = if torsoOnly {
            &mut cent.pe.torso
        } else {
            &mut cent.pe.legs
        };

        if lf.lastForcedFrame != forceFrame {
            let flags = BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND;
            let animSpeed = 1.0f32;
            trap::G2API_SetBoneAnim(
                ctx.engine,
                ghoul2,
                0,
                "lower_lumbar",
                forceFrame,
                forceFrame + 1,
                flags,
                animSpeed,
                time,
                -1.0,
                150,
            );
            trap::G2API_SetBoneAnim(
                ctx.engine,
                ghoul2,
                0,
                "model_root",
                forceFrame,
                forceFrame + 1,
                flags,
                animSpeed,
                time,
                -1.0,
                150,
            );
            trap::G2API_SetBoneAnim(
                ctx.engine,
                ghoul2,
                0,
                "Motion",
                forceFrame,
                forceFrame + 1,
                flags,
                animSpeed,
                time,
                -1.0,
                150,
            );
        }

        lf.lastForcedFrame = forceFrame;
        lf.animationNumber = 0;
    } else {
        let needsNewAnim = {
            let lf = if torsoOnly {
                &mut cent.pe.torso
            } else {
                &mut cent.pe.legs
            };
            lf.lastForcedFrame = -1;

            newAnimation != lf.animationNumber
                || csBrokenLimbs != ci.brokenLimbs
                || flipState != (lf.lastFlip != qfalse)
                || lf.animation.is_null()
                || CG_FirstAnimFrame(lf, torsoOnly, speedScale)
        };

        if needsNewAnim {
            CG_SetLerpFrameAnimation(
                ctx,
                cent,
                ci,
                newAnimation,
                speedScale,
                torsoOnly,
                flipState,
            );
        }
    }

    let lf = if torsoOnly {
        &mut cent.pe.torso
    } else {
        &mut cent.pe.legs
    };
    lf.lastFlip = if flipState { qtrue } else { qfalse };

    if lf.frameTime > time + 200 {
        lf.frameTime = time;
    }

    if lf.oldFrameTime > time {
        lf.oldFrameTime = time;
    }

    // calculate current lerp value
    if lf.frameTime == lf.oldFrameTime {
        lf.backlerp = 0.0;
    } else {
        lf.backlerp =
            1.0 - (time - lf.oldFrameTime) as f32 / (lf.frameTime - lf.oldFrameTime) as f32;
    }
}

/// Raven `CG_ClearLerpFrame` — snaps one lerp-frame (torso or legs, chosen by
/// `torsoOnly`) straight onto `animationNumber`'s first frame, no blend.
///
/// `lf` is derived from `cent.pe.torso`/`cent.pe.legs` the same way
/// `CG_RunLerpFrame` derives it — see that fn's doc.
/// Source: `oracle/codemp/cgame/cg_players.c:3243-3255`
pub fn CG_ClearLerpFrame(
    ctx: &mut CgContext,
    cent: &mut centity_t,
    ci: &mut clientInfo_t,
    animationNumber: c_int,
    torsoOnly: bool,
) {
    let time = ctx.world.cg.time;
    {
        let lf = if torsoOnly {
            &mut cent.pe.torso
        } else {
            &mut cent.pe.legs
        };
        lf.frameTime = time;
        lf.oldFrameTime = time;
    }

    CG_SetLerpFrameAnimation(ctx, cent, ci, animationNumber, 1.0, torsoOnly, false);

    let lf = if torsoOnly {
        &mut cent.pe.torso
    } else {
        &mut cent.pe.legs
    };

    if lf.animation.is_null() {
        // §F19: `CG_SetLerpFrameAnimation`'s ported early-return (unparsed
        // skeleton / out-of-range `localAnimIndex`) can leave `animation`
        // unset; Raven trusted `bgAllAnims[cent->localAnimIndex]` was always
        // valid and dereferenced it unconditionally. Leave the frame exactly
        // as `CG_SetLerpFrameAnimation` left it rather than deref null.
        return;
    }

    // SAFETY: `animation` was just resolved by `CG_SetLerpFrameAnimation` (or
    // is a still-valid earlier resolution); Raven dereferences it unchecked
    // and so do we.
    let (frameLerp, firstFrame, numFrames) = unsafe {
        let a = &*lf.animation;
        (a.frameLerp, a.firstFrame, a.numFrames)
    };

    if frameLerp < 0 {
        //Plays backwards
        let frame = firstFrame as c_int + numFrames as c_int;
        lf.oldFrame = frame;
        lf.frame = frame;
    } else {
        lf.oldFrame = firstFrame as c_int;
        lf.frame = firstFrame as c_int;
    }
}

/// Raven `CG_G2ServerBoneAngles` — replays the server's four packed
/// `boneIndex`/`boneAngles` overrides from `entityState_t` onto the client's
/// ghoul2 instance.
/// Source: `oracle/codemp/cgame/cg_players.c:3914-3962`
pub fn CG_G2ServerBoneAngles(ctx: &mut CgContext, cent: &centity_t) {
    let mut bone = cent.currentState.boneIndex1;
    let mut boneAngles: vec3_t = [0.0; 3];
    _VectorCopy(cent.currentState.boneAngles1, &mut boneAngles);

    let boneOrient = cent.currentState.boneOrient;
    let ghoul2 = cent.ghoul2;
    let engine = ctx.engine;
    let time = ctx.world.cg.time;

    for i in 0..4 {
        // cycle through the 4 bone index values on the entstate
        if bone != 0 {
            // if it's non-0 then it could have something in it.
            let boneName = CG_ConfigString(ctx, CS_G2BONES + bone);

            if !boneName.is_empty() {
                // got the bone, now set the angles from the corresponding
                // entitystate boneangles value.
                let flags = BONE_ANGLES_POSTMULT;

                // get the orientation out of our bit field
                let forward = boneOrient & 7; // 3 bits from bit 0
                let right = (boneOrient >> 3) & 7; // 3 bits from bit 3
                let up = (boneOrient >> 6) & 7; // 3 bits from bit 6

                trap::G2API_SetBoneAngles(
                    engine,
                    ghoul2,
                    0,
                    &boneName,
                    &boneAngles,
                    flags,
                    up,
                    right,
                    forward,
                    Some(&mut ctx.world.cgs.gameModels[0]),
                    100,
                    time,
                );
            }
        }

        match i {
            0 => {
                bone = cent.currentState.boneIndex2;
                _VectorCopy(cent.currentState.boneAngles2, &mut boneAngles);
            }
            1 => {
                bone = cent.currentState.boneIndex3;
                _VectorCopy(cent.currentState.boneAngles3, &mut boneAngles);
            }
            2 => {
                bone = cent.currentState.boneIndex4;
                _VectorCopy(cent.currentState.boneAngles4, &mut boneAngles);
            }
            _ => {}
        }
    }
}

/// Raven `CG_PlayerPowerups` — the powerup-driven dlights (quad/redflag/
/// blueflag/neutralflag) plus the flag models themselves.
///
/// `torso` is unused in Raven's own body (kept for signature parity with the
/// caller). `ci` resolves the owning `clientInfo_t` purely to mirror Raven's
/// `assert(ci)` NPC-sanity check — nothing downstream reads it, matching the
/// oracle body exactly.
/// Source: `oracle/codemp/cgame/cg_players.c:4449-4495`
pub fn CG_PlayerPowerups(ctx: &mut CgContext, cent: &centity_t, _torso: &refEntity_t) {
    let powerups = cent.currentState.powerups;
    if powerups == 0 {
        return;
    }

    // quad gives a dlight
    if powerups & (1 << PW_QUAD) != 0 {
        let intensity = 200 + (ctx.world.bg_state.rng.rand() & 31);
        trap::R_AddLightToScene(
            ctx.engine,
            &cent.lerpOrigin,
            intensity as f32,
            0.2,
            0.2,
            1.0,
        );
    }

    let ci = if cent.currentState.eType == entityType_t::ET_NPC as c_int {
        cent.npcClient.as_deref()
    } else {
        Some(&ctx.world.cgs.clientinfo[cent.currentState.clientNum as usize])
    };
    debug_assert!(ci.is_some(), "CG_PlayerPowerups: NPC with no npcClient");

    // redflag
    if powerups & (1 << PW_REDFLAG) != 0 {
        let hModel = ctx.world.cgs.media.redFlagModel;
        CG_PlayerFlag(ctx, cent, hModel);
        let intensity = 200 + (ctx.world.bg_state.rng.rand() & 31);
        trap::R_AddLightToScene(
            ctx.engine,
            &cent.lerpOrigin,
            intensity as f32,
            1.0,
            0.2,
            0.2,
        );
    }

    // blueflag
    if powerups & (1 << PW_BLUEFLAG) != 0 {
        let hModel = ctx.world.cgs.media.blueFlagModel;
        CG_PlayerFlag(ctx, cent, hModel);
        let intensity = 200 + (ctx.world.bg_state.rng.rand() & 31);
        trap::R_AddLightToScene(
            ctx.engine,
            &cent.lerpOrigin,
            intensity as f32,
            0.2,
            0.2,
            1.0,
        );
    }

    // neutralflag
    if powerups & (1 << PW_NEUTRALFLAG) != 0 {
        let intensity = 200 + (ctx.world.bg_state.rng.rand() & 31);
        trap::R_AddLightToScene(
            ctx.engine,
            &cent.lerpOrigin,
            intensity as f32,
            1.0,
            1.0,
            1.0,
        );
    }

    // haste leaves smoke trails
    //
    // if ( powerups & ( 1 << PW_HASTE ) ) {
    //     CG_HasteTrail( cent );
    // }
}
