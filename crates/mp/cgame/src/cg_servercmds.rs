//! Port of `oracle/codemp/cgame/cg_servercmds.c` — server command and configstring handling. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

use mp_abi::ui::public::ui_menu_command_t::{UIMENU_CLASSSEL, UIMENU_PLAYERCONFIG};
use mp_bg::bg_panimate::BG_InDeathAnim;
use mp_bg::local::bg_customSiegeSoundNames;
use mp_bg::public::anim_number::animNumber_t;
use mp_bg::public::configstring::{
    CS_CLIENT_DUELHEALTHS, CS_CLIENT_DUELISTS, CS_CLIENT_DUELWINNER, CS_CLIENT_JEDIMASTER,
    CS_EFFECTS, CS_FLAGSTATUS, CS_INTERMISSION, CS_LEVEL_START_TIME, CS_LIGHT_STYLES, CS_MODELS,
    CS_MUSIC, CS_PLAYERS, CS_SCORES1, CS_SCORES2, CS_SERVERINFO, CS_SHADERSTATE,
    CS_SIEGE_OBJECTIVES, CS_SIEGE_STATE, CS_SIEGE_TIMEOVERRIDE, CS_SIEGE_WINTEAM, CS_SOUNDS,
    CS_TEAMVOTE_NO, CS_TEAMVOTE_STRING, CS_TEAMVOTE_TIME, CS_TEAMVOTE_YES, CS_TERRAINS, CS_VOTE_NO,
    CS_VOTE_STRING, CS_VOTE_TIME, CS_VOTE_YES, CS_WARMUP, MAX_FX, MAX_MODELS, MAX_SOUNDS,
};
use mp_bg::public::gametype::{GT_CTF, GT_CTY, GT_POWERDUEL, GT_SIEGE};
use mp_bg::weapons::weapon_t::WP_BRYAR_PISTOL;
use mp_engine_select::Engine;
use mp_qshared::common::mp::ghoul2::bone_flags::BONE_ANIM_OVERRIDE_FREEZE;
use mp_qshared::common::mp::qcommon::saber::saber_info::MAX_SABERS;
use mp_qshared::shared::keycatch::KEYCATCH_UI;
use mp_qshared::shared::limits::{MAX_SAY_TEXT, MAX_STRING_TOKENS};
use mp_qshared::shared::q_math::VectorClear;
use mp_qshared::shared::sound_channel::{CHAN_ANNOUNCER, CHAN_LOCAL_SOUND};
use mp_qshared::shared::{
    qfalse, qtrue, sfxHandle_t, BIGCHAR_WIDTH, GIANTCHAR_WIDTH, MAX_CLIENTS, MAX_CLIENTS_I32,
    MAX_GENTITIES, MAX_QPATH, MAX_STRING_CHARS, SCREEN_HEIGHT,
};
use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::menu_system::MenuSystem;
use native_string::{atoi, strcat_string, Info_ValueForKey, Q_stricmp, Q_strncpyz};

use crate::cg_draw::{CG_CenterPrint, CG_ChatBox_AddString};
use crate::cg_ents::CG_S_StopLoopingSound;
use crate::cg_event::CG_ReattachLimb;
use crate::cg_light::CG_SetLightstyle;
use crate::cg_localents::CG_InitLocalEntities;
use crate::cg_main::{
    CG_Argv, CG_BuildSpectatorString, CG_ConfigString, CG_GetStringEdString, CG_ParseSiegeState,
    CG_ParseWeatherEffect, CG_Printf, CG_SetScoreSelection, CG_StartMusic,
};
use crate::cg_marks::{CG_ClearParticles, CG_InitMarkPolys};
use crate::cg_players::{
    cg_customCombatSoundNames, cg_customDuelSoundNames, cg_customExtraSoundNames,
    cg_customJediSoundNames, cg_customSoundNames, CG_CacheG2AnimInfo, CG_DestroyNPCClient,
    CG_HandleAppendedSkin, CG_LoadDeferredPlayers, CG_NewClientInfo,
};
use crate::cg_saga::{
    CG_ParseSiegeExtendedData, CG_ParseSiegeObjectiveStatus, CG_SetSiegeTimerCvar,
    CG_SiegeBriefingDisplay,
};
use crate::cg_weapons::CG_G2WeaponInstance;
use crate::local::client_info_t::{
    clientInfo_t, MAX_CUSTOM_COMBAT_SOUNDS, MAX_CUSTOM_EXTRA_SOUNDS, MAX_CUSTOM_JEDI_SOUNDS,
    MAX_CUSTOM_SOUNDS,
};
use crate::local::score_t::score_t;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::CgWorld;

/// Raven `MAX_STRINGED_SV_STRING` — this is an quake-engine limit, not a StringEd limit.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:1057`
pub const MAX_STRINGED_SV_STRING: usize = 1024;

/// Raven `MAX_LIGHT_STYLES` — `mp_bg::public::configstring` never re-exported
/// this one (only its derived `CS_*` offsets), so it's spelled here at its
/// only cgame call site.
///
/// Source: `oracle/codemp/game/q_shared.h:424`
const MAX_LIGHT_STYLES: c_int = 64;

/// Raven `MAX_CLIENT_SCORE_SEND` — the score rows one `scores` command carries.
///
/// `bg_public.h` is the owning header but `mp_bg` never ported the define;
/// `mp_game`'s `g_cmds.rs` spells its own copy the same way.
/// Source: `oracle/codemp/game/bg_public.h:51`
const MAX_CLIENT_SCORE_SEND: c_int = 20;

/// Raven `GetCustomSoundForType` — looks up a custom sound name table by
/// `cg_customSoundSet_e`-ish `setType`, indexed by `index`.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:310-330`
pub fn GetCustomSoundForType(setType: i32, index: usize) -> Option<&'static str> {
    match setType {
        1 => cg_customSoundNames[index],
        2 => cg_customCombatSoundNames[index],
        3 => cg_customExtraSoundNames[index],
        4 => cg_customJediSoundNames[index],
        // Raven returns `bg_customSiegeSoundNames[index]` unconditionally
        // (`cg_servercmds.c:322-323`), no encoding check; `to_str()` maps a
        // non-UTF-8 slot to `None` instead (§F19) - unreachable today since
        // every entry is a UTF-8 string literal.
        5 => bg_customSiegeSoundNames[index].and_then(|s| s.to_str().ok()),
        6 => cg_customDuelSoundNames[index],
        _ => {
            debug_assert!(false, "GetCustomSoundForType: unhandled setType {setType}");
            None
        }
    }
}

/// Raven `SetCustomSoundForType` — writes `sfx` into the client's custom
/// sound table selected by `setType`.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:332-358`
pub fn SetCustomSoundForType(ci: &mut clientInfo_t, setType: i32, index: usize, sfx: sfxHandle_t) {
    match setType {
        1 => ci.sounds[index] = sfx,
        2 => ci.combatSounds[index] = sfx,
        3 => ci.extraSounds[index] = sfx,
        4 => ci.jediSounds[index] = sfx,
        5 => ci.siegeSounds[index] = sfx,
        6 => ci.duelSounds[index] = sfx,
        _ => debug_assert!(false, "SetCustomSoundForType: unhandled setType {setType}"),
    }
}

/// Raven `SetDuelistHealthsFromConfigString` — parses the `|`-delimited
/// duelist-health configstring into `cgs.duelist{1,2,3}health`.
///
/// Raven: we only have 2 duelists, apparently (a leading `!` on the third
/// field means power-duel is off and `duelist3health` is forced to -1).
///
/// PORT-NOTE: Raven copies each field into a fixed `char buf[64]` with no
/// bounds check, so a field over 64 bytes overflows the stack (UB); the port's
/// `String` grows instead, so there's no cap to replicate (§F19).
/// Source: `oracle/codemp/cgame/cg_servercmds.c:612-656`
pub fn SetDuelistHealthsFromConfigString(world: &mut CgWorld, s: &str) {
    let bytes = s.as_bytes();
    let mut buf = String::new();
    let mut i = 0usize;

    while i < bytes.len() && bytes[i] != b'|' {
        buf.push(bytes[i] as char);
        i += 1;
    }
    world.cgs.duelist1health = atoi(&buf);

    buf.clear();
    // §F19: with no `|` in the string Raven's `i++` steps past the terminator
    // and reads OOB; the length bound stops at the end and yields `atoi("")`.
    i += 1;
    while i < bytes.len() && bytes[i] != b'|' {
        buf.push(bytes[i] as char);
        i += 1;
    }
    world.cgs.duelist2health = atoi(&buf);

    buf.clear();
    i += 1;
    if i < bytes.len() && bytes[i] == b'!' {
        // we only have 2 duelists, apparently.
        world.cgs.duelist3health = -1;
        return;
    }

    while i < bytes.len() && bytes[i] != b'|' {
        buf.push(bytes[i] as char);
        i += 1;
    }
    world.cgs.duelist3health = atoi(&buf);
}

/// Raven `CG_RemoveChatEscapeChar` — strips the `\x19` chat-escape marker out
/// of `text`, returning the compacted copy (Raven mutates the buffer in
/// place; the out-param becomes a return per porting-rules #7).
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:1045-1055`
pub fn CG_RemoveChatEscapeChar(text: &str) -> String {
    text.chars().filter(|&c| c != '\u{19}').collect()
}

/// The zero fill Raven gets from its `memset(cg.scores, 0, sizeof(cg.scores))`.
/// `score_t` is plain ints with no `Default`, so the fill is spelled out once.
fn zeroed_score() -> score_t {
    score_t {
        client: 0,
        score: 0,
        ping: 0,
        time: 0,
        scoreFlags: 0,
        powerUps: 0,
        accuracy: 0,
        impressiveCount: 0,
        excellentCount: 0,
        guantletCount: 0,
        defendCount: 0,
        assistCount: 0,
        captures: 0,
        perfect: 0,
        team: 0,
    }
}

/// Raven `CG_ParseScores` — decodes the `scores` server command into
/// `cg.scores` and mirrors each row's score/powerups back onto `cgs.clientinfo`.
///
/// PORT-NOTE: Raven's `cg.numScores > MAX_CLIENTS` clamp is dead - the very
/// next line overwrites `cg.numScores` with `readScores`. Kept as written.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:21-69`
pub fn CG_ParseScores(
    ctx: &mut CgContext,
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
) {
    let arg = CG_Argv(ctx, 1);
    ctx.world.cg.numScores = atoi(&arg);

    let mut readScores = ctx.world.cg.numScores;

    if readScores > MAX_CLIENT_SCORE_SEND {
        readScores = MAX_CLIENT_SCORE_SEND;
    }

    if ctx.world.cg.numScores > MAX_CLIENTS_I32 {
        ctx.world.cg.numScores = MAX_CLIENTS_I32;
    }

    ctx.world.cg.numScores = readScores;

    let arg = CG_Argv(ctx, 2);
    ctx.world.cg.teamScores[0] = atoi(&arg);
    let arg = CG_Argv(ctx, 3);
    ctx.world.cg.teamScores[1] = atoi(&arg);

    for score in ctx.world.cg.scores.iter_mut() {
        *score = zeroed_score();
    }

    let mut i: c_int = 0;
    while i < readScores {
        let idx = i as usize;

        let arg = CG_Argv(ctx, i * 14 + 4);
        ctx.world.cg.scores[idx].client = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 5);
        ctx.world.cg.scores[idx].score = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 6);
        ctx.world.cg.scores[idx].ping = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 7);
        ctx.world.cg.scores[idx].time = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 8);
        ctx.world.cg.scores[idx].scoreFlags = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 9);
        let powerups = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 10);
        ctx.world.cg.scores[idx].accuracy = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 11);
        ctx.world.cg.scores[idx].impressiveCount = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 12);
        ctx.world.cg.scores[idx].excellentCount = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 13);
        ctx.world.cg.scores[idx].guantletCount = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 14);
        ctx.world.cg.scores[idx].defendCount = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 15);
        ctx.world.cg.scores[idx].assistCount = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 16);
        ctx.world.cg.scores[idx].perfect = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 17);
        ctx.world.cg.scores[idx].captures = atoi(&arg);

        if ctx.world.cg.scores[idx].client < 0 || ctx.world.cg.scores[idx].client >= MAX_CLIENTS_I32
        {
            ctx.world.cg.scores[idx].client = 0;
        }

        let client = ctx.world.cg.scores[idx].client as usize;
        let score = ctx.world.cg.scores[idx].score;
        ctx.world.cgs.clientinfo[client].score = score;
        ctx.world.cgs.clientinfo[client].powerups = powerups;

        ctx.world.cg.scores[idx].team = ctx.world.cgs.clientinfo[client].team;

        i += 1;
    }

    CG_SetScoreSelection(ctx.world, menus, ds, dc, None);
}

/// Raven `CG_ParseTeamInfo` — decodes the `tinfo` server command into the team
/// overlay's sorted client list and the per-teammate HUD fields.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:77-94`
pub fn CG_ParseTeamInfo(ctx: &mut CgContext) {
    let arg = CG_Argv(ctx, 1);
    ctx.world.draw.numSortedTeamPlayers = atoi(&arg);

    let mut i: c_int = 0;
    while i < ctx.world.draw.numSortedTeamPlayers {
        let client = atoi(&CG_Argv(ctx, i * 6 + 2));

        // §F19: Raven bounds neither `i` against `TEAM_MAXOVERLAY` nor `client`
        // against `MAX_CLIENTS` and smashes memory past either end; we take the
        // five `CG_Argv` reads either way (same syscall trace) and drop the
        // out-of-range stores.
        if (i as usize) < ctx.world.draw.sortedTeamPlayers.len() {
            ctx.world.draw.sortedTeamPlayers[i as usize] = client;
        }

        let location = atoi(&CG_Argv(ctx, i * 6 + 3));
        let health = atoi(&CG_Argv(ctx, i * 6 + 4));
        let armor = atoi(&CG_Argv(ctx, i * 6 + 5));
        let curWeapon = atoi(&CG_Argv(ctx, i * 6 + 6));
        let powerups = atoi(&CG_Argv(ctx, i * 6 + 7));

        if client >= 0 && client < MAX_CLIENTS_I32 {
            let ci = &mut ctx.world.cgs.clientinfo[client as usize];
            ci.location = location;
            ci.health = health;
            ci.armor = armor;
            ci.curWeapon = curWeapon;
            ci.powerups = powerups;
        }

        i += 1;
    }
}

/// Raven `CG_RegisterCustomSounds` — registers one of a client's custom sound
/// sets out of `sound/chars/<psDir>/misc/`, retrying a missing numbered variant
/// as its `…1.wav` form.
///
/// PORT-NOTE: Raven's `case 5:` has no `break`, so it falls into
/// `default: assert(0); return;` — the siege set never registers through here.
/// Quirk preserved.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:360-430`
pub fn CG_RegisterCustomSounds(
    ctx: &mut CgContext,
    ci: &mut clientInfo_t,
    setType: c_int,
    psDir: &str,
) {
    let iTableEntries = match setType {
        1 => MAX_CUSTOM_SOUNDS,
        2 => MAX_CUSTOM_COMBAT_SOUNDS,
        3 => MAX_CUSTOM_EXTRA_SOUNDS,
        4 => MAX_CUSTOM_JEDI_SOUNDS,
        _ => {
            debug_assert!(
                false,
                "CG_RegisterCustomSounds: unhandled setType {setType}"
            );
            return;
        }
    };

    for i in 0..iTableEntries {
        let Some(s) = GetCustomSoundForType(setType, i) else {
            break;
        };

        // step past the leading `*`
        let s = &s[1..];
        let mut hSFX = trap::S_RegisterSound(ctx.engine, &format!("sound/chars/{psDir}/misc/{s}"));

        if hSFX == 0 {
            let mut modifiedSound = String::from(s);

            // §F19: Raven backs the cursor up one char from the `.` without
            // checking, reading before the buffer when the name starts with it;
            // a leading `.` just skips the retry here.
            if let Some(p) = modifiedSound.find('.').filter(|&p| p > 0) {
                let p = p - 1;

                // before we destroy it.. we want to see if this is actually a number.
                // If it isn't a number then don't try decrementing and registering as
                // it will only cause a disk hit (we don't try precaching such files)
                let testNumber = &modifiedSound[p..p + 1];
                if atoi(testNumber) != 0 {
                    modifiedSound.truncate(p);
                    modifiedSound.push_str("1.wav");

                    hSFX = trap::S_RegisterSound(
                        ctx.engine,
                        &format!("sound/chars/{psDir}/misc/{modifiedSound}"),
                    );
                }
            }
        }

        SetCustomSoundForType(ci, setType, i, hSFX);
    }
}

/// Raven `CG_PrecacheNPCSounds` — blind-registers every base custom sound name
/// under an NPC's sound folder, muting the sound system's missing-file spam
/// while it does.
///
/// Raven takes the folder off the configstring at `str[2]`, past the leading
/// index marker.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:432-482`
pub fn CG_PrecacheNPCSounds(ctx: &mut CgContext, s: &str) {
    // §F19: Raven walks from `str[2]` without a length check and reads past the
    // end of a shorter string; a string under 2 chars gives an empty folder here.
    let pEnd: String = s.as_bytes().iter().skip(2).map(|&b| b as char).collect();

    let mut i: c_int = 0;
    while i < 4 {
        // 4 types
        // It would be better if we knew what type this actually was (extra, combat, jedi, etc).
        // But that would require extra configstring indexing and that is a bad thing.
        let mut j = 0usize;
        while j < MAX_CUSTOM_SOUNDS {
            let sound = GetCustomSoundForType(i + 1, j);

            match sound.filter(|name| !name.is_empty()) {
                // whatever it is, try registering it under this folder.
                Some(sound) => {
                    let sEnd: String = sound
                        .as_bytes()
                        .iter()
                        .skip(1)
                        .map(|&b| b as char)
                        .collect();

                    trap::S_ShutUp(ctx.engine, true);
                    trap::S_RegisterSound(ctx.engine, &format!("sound/chars/{pEnd}/misc/{sEnd}"));
                    trap::S_ShutUp(ctx.engine, false);
                }
                // move onto the next set
                None => break,
            }

            j += 1;
        }

        i += 1;
    }
}

/// Frees a client's custom-saber weapon instances — Raven's `MAX_SABERS` loop
/// inside `CG_KillCEntityG2`, spelled once for both of that fn's `ci` arms.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:894-905`
fn CG_CleanCIWeaponInstances(engine: &Engine, ci: &mut clientInfo_t) {
    // Clean up any weapon instances for custom saber stuff
    let mut j = 0;
    while j < MAX_SABERS {
        if !ci.ghoul2Weapons[j].is_null()
            && trap::G2_HaveWeGhoul2Models(engine, ci.ghoul2Weapons[j])
        {
            trap::G2API_CleanGhoul2Models(engine, &mut ci.ghoul2Weapons[j]);
            ci.ghoul2Weapons[j] = null_mut();
        }

        j += 1;
    }
}

/// Raven `CG_KillCEntityG2` — tears down every ghoul2 instance an entity slot
/// owns (its own, the grip arm, the frame hold, its client's model and saber
/// instances) and resets the rag/ik/anim latches.
///
/// Raven picks `ci` as the real client table below `MAX_CLIENTS` and the
/// entity's own `npcClient` above it, then compares the two pointers; the two
/// arms are spelled separately here because the owned `npcClient` makes that
/// comparison a fact of which branch we are in, not a runtime test.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:863-935`
pub fn CG_KillCEntityG2(ctx: &mut CgContext, entNum: usize) {
    let engine = ctx.engine;
    let ghoul2 = ctx.world.entity(entNum).ghoul2;

    if entNum < MAX_CLIENTS {
        let ci = &mut ctx.world.cgs.clientinfo[entNum];

        if ci.ghoul2Model == ghoul2 {
            ci.ghoul2Model = null_mut();
        } else if !ci.ghoul2Model.is_null() && trap::G2_HaveWeGhoul2Models(engine, ci.ghoul2Model) {
            trap::G2API_CleanGhoul2Models(engine, &mut ci.ghoul2Model);
            ci.ghoul2Model = null_mut();
        }

        CG_CleanCIWeaponInstances(engine, ci);
    } else if let Some(ci) = ctx.world.entity_mut(entNum).npcClient.as_deref_mut() {
        // never going to be != cent->ghoul2, unless cent->ghoul2 has already
        // been removed (and then this ptr is not valid)
        ci.ghoul2Model = null_mut();

        CG_CleanCIWeaponInstances(engine, ci);
    }

    if !ghoul2.is_null() && trap::G2_HaveWeGhoul2Models(engine, ghoul2) {
        trap::G2API_CleanGhoul2Models(engine, &mut ctx.world.entity_mut(entNum).ghoul2);
        ctx.world.entity_mut(entNum).ghoul2 = null_mut();
    }

    let grip_arm = ctx.world.entity(entNum).grip_arm;
    if !grip_arm.is_null() && trap::G2_HaveWeGhoul2Models(engine, grip_arm) {
        trap::G2API_CleanGhoul2Models(engine, &mut ctx.world.entity_mut(entNum).grip_arm);
        ctx.world.entity_mut(entNum).grip_arm = null_mut();
    }

    let frame_hold = ctx.world.entity(entNum).frame_hold;
    if !frame_hold.is_null() && trap::G2_HaveWeGhoul2Models(engine, frame_hold) {
        trap::G2API_CleanGhoul2Models(engine, &mut ctx.world.entity_mut(entNum).frame_hold);
        ctx.world.entity_mut(entNum).frame_hold = null_mut();
    }

    if ctx.world.entity(entNum).npcClient.is_some() {
        CG_DestroyNPCClient(&mut ctx.world.entity_mut(entNum).npcClient);
    }

    // just in case.
    ctx.world.entity_mut(entNum).isRagging = qfalse;
    ctx.world.entity_mut(entNum).ikStatus = qfalse;

    ctx.world.entity_mut(entNum).localAnimIndex = 0;
}

/// Raven `CG_CheckSVStringEdRef` — expands every `@@@KEY` reference in a
/// server-supplied string through the `MP_SVGAME` StringEd package. Raven's
/// `buf` out-param is the return value (porting-rules #7).
///
/// Raven: I don't really like doing this. But it utilizes the system that was
/// already in place.
///
/// PORT-NOTE: Raven's `gotStrip` is set once and never raised, so the copy-out
/// arm it guards always runs. The caller's `buf` is a fixed 1024 bytes that a
/// StringEd expansion can overrun (UB); the `String` grows instead, so only the
/// `Q_strcat` bound is replicated (§F19).
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:1059-1126`
pub fn CG_CheckSVStringEdRef(ctx: &mut CgContext, s: &str) -> String {
    let bytes = s.as_bytes();
    let strLen = bytes.len();

    if strLen == 0 {
        return String::new();
    }

    if strLen >= MAX_STRINGED_SV_STRING {
        return s.to_string();
    }

    let mut buf = String::new();
    let mut i = 0usize;

    while i < strLen && bytes[i] != 0 {
        if bytes[i] == b'@'
            && (i + 1) < strLen
            && bytes[i + 1] == b'@'
            && (i + 2) < strLen
            && bytes[i + 2] == b'@'
            && (i + 3) < strLen
        {
            // @@@ should mean to insert a StringEd reference here, so insert it
            // into buf at the current place
            let mut stringRef = String::new();

            while i < strLen && bytes[i] == b'@' {
                i += 1;
            }

            while i < strLen
                && bytes[i] != 0
                && bytes[i] != b' '
                && bytes[i] != b':'
                && bytes[i] != b'.'
                && bytes[i] != b'\n'
            {
                stringRef.push(bytes[i] as char);
                i += 1;
            }

            let stringEd = CG_GetStringEdString(ctx, "MP_SVGAME", &stringRef);
            strcat_string(&mut buf, MAX_STRINGED_SV_STRING, &stringEd);
        }

        // Raven copies `str[i]` here even when the ref scan ran off the end,
        // which appends the terminator and ends the C string; we just stop.
        if i < strLen {
            buf.push(bytes[i] as char);
        }
        i += 1;
    }

    buf
}

/// Raven `CG_BodyQueueCopy` — clones a dying client's ghoul2 instance into a
/// body-queue entity, freezes it on the last frame of its death animation and
/// hands the weapon model (or drops it) per `knownWeapon`.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:1128-1244`
pub fn CG_BodyQueueCopy(ctx: &mut CgContext, centNum: usize, clientNum: c_int, knownWeapon: c_int) {
    let engine = ctx.engine;
    let flags: c_int = BONE_ANIM_OVERRIDE_FREEZE;

    if !ctx.world.entity(centNum).ghoul2.is_null() {
        trap::G2API_CleanGhoul2Models(engine, &mut ctx.world.entity_mut(centNum).ghoul2);
    }

    if clientNum < 0 || clientNum >= MAX_CLIENTS_I32 {
        return;
    }
    let sourceNum = clientNum as usize;

    // Raven's `if (!source)` null-check on an array element drops — an owned
    // array slot can't be null (§B5).
    let source_ghoul2 = ctx.world.entity(sourceNum).ghoul2;
    if source_ghoul2.is_null() {
        return;
    }

    // reset in case it's still set from another body that was in this cent slot.
    ctx.world.entity_mut(centNum).isRagging = qfalse;
    // if the owner was in ragdoll state, then we want to go into it too right away.
    let source_isRagging = ctx.world.entity(sourceNum).isRagging;
    ctx.world.entity_mut(centNum).ownerRagging = source_isRagging;

    ctx.world.entity_mut(centNum).bodyFadeTime = 0;
    ctx.world.entity_mut(centNum).bodyHeight = 0.0;

    let source_dustTrailTime = ctx.world.entity(sourceNum).dustTrailTime;
    ctx.world.entity_mut(centNum).dustTrailTime = source_dustTrailTime;

    trap::G2API_DuplicateGhoul2Instance(
        engine,
        source_ghoul2,
        &mut ctx.world.entity_mut(centNum).ghoul2,
    );

    if source_isRagging != qfalse {
        // just reset it now.
        ctx.world.entity_mut(sourceNum).isRagging = qfalse;
        // NULL params is the engine's "reset to no ragdoll" arm
        // (`oracle/codemp/client/cl_cgame.cpp:1539-1542`)
        // Source: `oracle/codemp/cgame/cg_servercmds.c:1176`
        let source_ghoul2 = ctx.world.entity(sourceNum).ghoul2;
        trap::G2API_SetRagDoll(ctx.engine, source_ghoul2, None);
    }

    // either force the weapon from when we died or remove it if it was a dropped weapon
    // `HasGhoul2ModelOnIndex`/`RemoveGhoul2Model` take the ADDRESS of the
    // instance slot, not the token: Raven passes `&(cent->ghoul2)` and the
    // engine casts the word to `CGhoul2Info_v **` (`cl_cgame.cpp:1434`).
    if knownWeapon > WP_BRYAR_PISTOL
        && trap::G2API_HasGhoul2ModelOnIndex(
            engine,
            &mut ctx.world.entity_mut(centNum).ghoul2 as *mut *mut c_void as *mut c_void,
            1,
        )
    {
        trap::G2API_RemoveGhoul2Model(
            engine,
            &mut ctx.world.entity_mut(centNum).ghoul2 as *mut *mut c_void as *mut c_void,
            1,
        );
    } else if trap::G2API_HasGhoul2ModelOnIndex(
        engine,
        &mut ctx.world.entity_mut(centNum).ghoul2 as *mut *mut c_void as *mut c_void,
        1,
    ) {
        let world = &*ctx.world;
        let instance = CG_G2WeaponInstance(world, world.entity(centNum), knownWeapon);
        let cent_ghoul2 = world.entity(centNum).ghoul2;
        trap::G2API_CopySpecificGhoul2Model(engine, instance, 0, cent_ghoul2, 1);
    }

    if ctx.world.entity(centNum).ownerRagging == qfalse {
        let source_localAnimIndex = ctx.world.entity(sourceNum).localAnimIndex;
        let source_torsoAnim = ctx.world.entity(sourceNum).currentState.torsoAnim;

        //anim = &bgAllAnims[cent->localAnimIndex].anims[ cent->currentState.torsoAnim ];
        let mut fallBack = false;
        let animNum = if BG_InDeathAnim(source_torsoAnim) == qfalse {
            // then just snap the corpse into a default
            fallBack = true;
            animNumber_t::BOTH_DEAD1 as c_int
        } else {
            source_torsoAnim
        };

        // §F19: a skeleton whose animation.cfg never parsed leaves
        // `localAnimIndex` at -1 with a null table, which Raven indexes one
        // entry before `bgAllAnims`; we skip the anim block instead.
        let animations = if source_localAnimIndex >= 0
            && (source_localAnimIndex as usize) < ctx.world.bg_state.bgAllAnims.len()
        {
            ctx.world.bg_state.bgAllAnims[source_localAnimIndex as usize].anims
        } else {
            null_mut()
        };

        if !animations.is_null() {
            // SAFETY: `anims` is the animation table `BG_ParseAnimationFile`
            // allocated for this skeleton; Raven indexes it unchecked and so do we
            // (same read `cg_players::CG_G2SetHeadAnim` does).
            let (firstFrame, numFrames, frameLerp) = unsafe {
                let a = &*animations.offset(animNum as isize);
                (a.firstFrame as c_int, a.numFrames as c_int, a.frameLerp)
            };

            let animSpeed = 50.0f32 / frameLerp as f32;

            let mut aNum;
            if !fallBack {
                // this will just set us to the last frame of the animation, in theory
                let source_number = ctx.world.entity(sourceNum).currentState.number;
                aNum = ctx.world.cgs.clientinfo[source_number as usize].frame + 1;

                while aNum >= firstFrame + numFrames {
                    aNum -= 1;
                }

                if aNum < firstFrame - 1 {
                    // wrong animation...?
                    aNum = (firstFrame + numFrames) - 1;
                }
            } else {
                aNum = firstFrame;
            }

            let eFrame = firstFrame + numFrames;

            //if (!cgs.clientinfo[source->currentState.number].frame || (cent->currentState.torsoAnim) != (source->currentState.torsoAnim) )
            //{
            //	aNum = (anim->firstFrame+anim->numFrames)-1;
            //}

            let cent_ghoul2 = ctx.world.entity(centNum).ghoul2;
            let time = ctx.world.cg.time;
            for bone in ["upper_lumbar", "model_root", "Motion"] {
                trap::G2API_SetBoneAnim(
                    engine,
                    cent_ghoul2,
                    0,
                    bone,
                    aNum,
                    eFrame,
                    flags,
                    animSpeed,
                    time,
                    -1.0,
                    150,
                );
            }
        }
    }

    // After we create the bodyqueue, regenerate any limbs on the real instance
    if ctx.world.entity(sourceNum).torsoBolt != 0 {
        CG_ReattachLimb(ctx, sourceNum);
    }
}

/// Raven `CG_ParseServerinfo` — pulls the gameplay cvars off the
/// `CS_SERVERINFO` configstring into `cgs`/`cg` and mirrors the "about"/RMG
/// cvars the UI reads back out.
///
/// Raven: rww - You must do this one here, Info_ValueForKey always uses the
/// same memory pointer.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:105-187`
pub fn CG_ParseServerinfo(ctx: &mut CgContext) {
    let info = CG_ConfigString(ctx, CS_SERVERINFO);

    ctx.world.cgs.debugMelee = atoi(&Info_ValueForKey(&info, "g_debugMelee"));
    ctx.world.cgs.stepSlideFix = atoi(&Info_ValueForKey(&info, "g_stepSlideFix"));

    ctx.world.cgs.noSpecMove = atoi(&Info_ValueForKey(&info, "g_noSpecMove"));

    trap::Cvar_Set(
        ctx.engine,
        "bg_fighterAltControl",
        &Info_ValueForKey(&info, "bg_fighterAltControl"),
    );

    ctx.world.cgs.siegeTeamSwitch = atoi(&Info_ValueForKey(&info, "g_siegeTeamSwitch"));

    ctx.world.cgs.showDuelHealths = atoi(&Info_ValueForKey(&info, "g_showDuelHealths"));

    ctx.world.cgs.gametype = atoi(&Info_ValueForKey(&info, "g_gametype"));
    trap::Cvar_Set(
        ctx.engine,
        "g_gametype",
        &format!("{}", ctx.world.cgs.gametype),
    );
    ctx.world.cgs.needpass = atoi(&Info_ValueForKey(&info, "needpass"));
    ctx.world.cgs.jediVmerc = atoi(&Info_ValueForKey(&info, "g_jediVmerc"));
    ctx.world.cgs.wDisable = atoi(&Info_ValueForKey(&info, "wdisable"));
    ctx.world.cgs.fDisable = atoi(&Info_ValueForKey(&info, "fdisable"));
    ctx.world.cgs.dmflags = atoi(&Info_ValueForKey(&info, "dmflags"));
    ctx.world.cgs.teamflags = atoi(&Info_ValueForKey(&info, "teamflags"));
    ctx.world.cgs.fraglimit = atoi(&Info_ValueForKey(&info, "fraglimit"));
    ctx.world.cgs.duel_fraglimit = atoi(&Info_ValueForKey(&info, "duel_fraglimit"));
    ctx.world.cgs.capturelimit = atoi(&Info_ValueForKey(&info, "capturelimit"));
    ctx.world.cgs.timelimit = atoi(&Info_ValueForKey(&info, "timelimit"));
    ctx.world.cgs.maxclients = atoi(&Info_ValueForKey(&info, "sv_maxclients"));
    let mapname = Info_ValueForKey(&info, "mapname");

    // rww - You must do this one here, Info_ValueForKey always uses the same memory pointer.
    trap::Cvar_Set(ctx.engine, "ui_about_mapname", &mapname);

    Q_strncpyz(
        &mut ctx.world.cgs.mapname,
        &format!("maps/{mapname}.bsp"),
        MAX_QPATH,
    );

    trap::Cvar_Set(
        ctx.engine,
        "ui_about_gametype",
        &format!("{}", ctx.world.cgs.gametype),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_about_fraglimit",
        &format!("{}", ctx.world.cgs.fraglimit),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_about_duellimit",
        &format!("{}", ctx.world.cgs.duel_fraglimit),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_about_capturelimit",
        &format!("{}", ctx.world.cgs.capturelimit),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_about_timelimit",
        &format!("{}", ctx.world.cgs.timelimit),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_about_maxclients",
        &format!("{}", ctx.world.cgs.maxclients),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_about_dmflags",
        &format!("{}", ctx.world.cgs.dmflags),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_about_hostname",
        &Info_ValueForKey(&info, "sv_hostname"),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_about_needpass",
        &Info_ValueForKey(&info, "g_needpass"),
    );
    trap::Cvar_Set(
        ctx.engine,
        "ui_about_botminplayers",
        &Info_ValueForKey(&info, "bot_minplayers"),
    );

    // Set the siege teams based on what the server has for overrides.
    trap::Cvar_Set(
        ctx.engine,
        "cg_siegeTeam1",
        &Info_ValueForKey(&info, "g_siegeTeam1"),
    );
    trap::Cvar_Set(
        ctx.engine,
        "cg_siegeTeam2",
        &Info_ValueForKey(&info, "g_siegeTeam2"),
    );

    let tinfo = CG_ConfigString(ctx, CS_TERRAINS + 1);
    if tinfo.is_empty() {
        ctx.world.cg.mInRMG = qfalse;
    } else {
        ctx.world.cg.mInRMG = qtrue;
        trap::Cvar_Set(ctx.engine, "RMG", "1");

        let weather = atoi(&Info_ValueForKey(&info, "RMG_weather"));

        trap::Cvar_Set(ctx.engine, "RMG_weather", &format!("{weather}"));

        if weather == 1 || weather == 2 {
            ctx.world.cg.mRMGWeather = qtrue;
        } else {
            ctx.world.cg.mRMGWeather = qfalse;
        }
    }
}

/// Raven `CG_ParseWarmup` — reads the `CS_WARMUP` configstring into `cg.warmup`
/// and resets the countdown display.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:194-204`
pub fn CG_ParseWarmup(ctx: &mut CgContext) {
    let info = CG_ConfigString(ctx, CS_WARMUP);

    let warmup = atoi(&info);
    ctx.world.cg.warmupCount = -1;

    ctx.world.cg.warmup = warmup;
}

/// Raven `CG_SetConfigValues` — refreshes the small scoreboard/warmup/duel
/// fields cgame re-reads out of their configstrings on every relevant
/// `CS_*_CHANGED` event, rather than caching them once.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:213-262`
pub fn CG_SetConfigValues(ctx: &mut CgContext) {
    let s = CG_ConfigString(ctx, CS_SCORES1);
    ctx.world.cgs.scores1 = atoi(&s);
    let s = CG_ConfigString(ctx, CS_SCORES2);
    ctx.world.cgs.scores2 = atoi(&s);
    let s = CG_ConfigString(ctx, CS_LEVEL_START_TIME);
    ctx.world.cgs.levelStartTime = atoi(&s);

    if ctx.world.cgs.gametype == GT_CTF || ctx.world.cgs.gametype == GT_CTY {
        let s = CG_ConfigString(ctx, CS_FLAGSTATUS);
        let bytes = s.as_bytes();

        // §F19: Raven indexes `s[0]`/`s[1]` unchecked; a short/empty
        // configstring here leaves the flag fields at 0 instead of reading OOB.
        ctx.world.cgs.redflag = bytes.first().map(|&b| b as i32 - '0' as i32).unwrap_or(0);
        ctx.world.cgs.blueflag = bytes.get(1).map(|&b| b as i32 - '0' as i32).unwrap_or(0);
    }

    let s = CG_ConfigString(ctx, CS_WARMUP);
    ctx.world.cg.warmup = atoi(&s);

    // Track who the jedi master is
    let s = CG_ConfigString(ctx, CS_CLIENT_JEDIMASTER);
    ctx.world.cgs.jediMaster = atoi(&s);
    let s = CG_ConfigString(ctx, CS_CLIENT_DUELWINNER);
    ctx.world.cgs.duelWinner = atoi(&s);

    let s = CG_ConfigString(ctx, CS_CLIENT_DUELISTS);

    if !s.is_empty() {
        let bytes = s.as_bytes();
        let mut buf = String::new();
        let mut i = 0usize;

        while i < bytes.len() && bytes[i] != b'|' {
            buf.push(bytes[i] as char);
            i += 1;
        }
        ctx.world.cgs.duelist1 = atoi(&buf);

        buf.clear();
        i += 1;

        while i < bytes.len() {
            buf.push(bytes[i] as char);
            i += 1;
        }
        ctx.world.cgs.duelist2 = atoi(&buf);
    }
}

/// Raven `CG_ShaderStateChanged` — walks the `=`/`:`/`@`-delimited
/// `CS_SHADERSTATE` configstring and remaps each `original=new:timeOffset`
/// triple through the renderer.
///
/// PORT-NOTE: a `:` that never turns up after a `=` (or a trailing entry with
/// no `@`) ends the whole scan, not just that entry - Raven's `break`/dropped-
/// pointer behavior. Kept as written.
///
/// §F19: Raven's `strncpy` fills fixed `originalShader[MAX_QPATH]`/
/// `newShader[MAX_QPATH]`/`timeOffset[16]` stack buffers with no length check
/// against the segment it found, so an oversized segment overflows the stack
/// (UB); the port slices the live `String` instead, so there's no buffer to
/// overrun.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:269-302`
pub fn CG_ShaderStateChanged(ctx: &mut CgContext) {
    let o = CG_ConfigString(ctx, CS_SHADERSTATE);
    let mut rest: &str = &o;

    while !rest.is_empty() {
        let Some(eq) = rest.find('=') else { break };
        let originalShader = &rest[..eq];
        let afterEq = &rest[eq + 1..];

        let Some(colon) = afterEq.find(':') else {
            break;
        };
        let newShader = &afterEq[..colon];
        let afterColon = &afterEq[colon + 1..];

        // Raven's `o = strstr(t, "@")` going NULL here just fails the outer
        // `while (o && *o)` check on the next spin; breaking straight away is
        // the same outcome without a dead re-check.
        let Some(at) = afterColon.find('@') else {
            break;
        };
        let timeOffset = &afterColon[..at];

        trap::R_RemapShader(ctx.engine, originalShader, newShader, timeOffset);

        rest = &afterColon[at + 1..];
    }
}

/// Raven `CG_HandleNPCSounds` — mirrors an NPC's four server-driven custom
/// sound sets (standard/combat/extra/jedi) onto its `npcClient`, registering
/// whatever set the current configstring names or clearing the table when the
/// server stopped naming one.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:484-606`
pub fn CG_HandleNPCSounds(ctx: &mut CgContext, centNum: usize) {
    if ctx.world.entity(centNum).npcClient.is_none() {
        return;
    }

    // standard
    let csSounds_Std = ctx.world.entity(centNum).currentState.csSounds_Std;
    if csSounds_Std != 0 {
        let s = CG_ConfigString(ctx, CS_SOUNDS + csSounds_Std);

        if !s.is_empty() {
            // Parse past the initial "*" which indicates this is a custom
            // sound, and the "$" which indicates it is an NPC custom sound dir.
            let sEnd: String = s.as_bytes().iter().skip(2).map(|&b| b as char).collect();

            let mut npc = ctx.world.entity_mut(centNum).npcClient.take();
            if let Some(ci) = npc.as_deref_mut() {
                CG_RegisterCustomSounds(ctx, ci, 1, &sEnd);
            }
            ctx.world.entity_mut(centNum).npcClient = npc;
        }
    } else if let Some(ci) = ctx.world.entity_mut(centNum).npcClient.as_deref_mut() {
        ci.sounds = [0; MAX_CUSTOM_SOUNDS];
    }

    // combat
    let csSounds_Combat = ctx.world.entity(centNum).currentState.csSounds_Combat;
    if csSounds_Combat != 0 {
        let s = CG_ConfigString(ctx, CS_SOUNDS + csSounds_Combat);

        if !s.is_empty() {
            let sEnd: String = s.as_bytes().iter().skip(2).map(|&b| b as char).collect();

            let mut npc = ctx.world.entity_mut(centNum).npcClient.take();
            if let Some(ci) = npc.as_deref_mut() {
                CG_RegisterCustomSounds(ctx, ci, 2, &sEnd);
            }
            ctx.world.entity_mut(centNum).npcClient = npc;
        }
    } else if let Some(ci) = ctx.world.entity_mut(centNum).npcClient.as_deref_mut() {
        ci.combatSounds = [0; MAX_CUSTOM_COMBAT_SOUNDS];
    }

    // extra
    let csSounds_Extra = ctx.world.entity(centNum).currentState.csSounds_Extra;
    if csSounds_Extra != 0 {
        let s = CG_ConfigString(ctx, CS_SOUNDS + csSounds_Extra);

        if !s.is_empty() {
            let sEnd: String = s.as_bytes().iter().skip(2).map(|&b| b as char).collect();

            let mut npc = ctx.world.entity_mut(centNum).npcClient.take();
            if let Some(ci) = npc.as_deref_mut() {
                CG_RegisterCustomSounds(ctx, ci, 3, &sEnd);
            }
            ctx.world.entity_mut(centNum).npcClient = npc;
        }
    } else if let Some(ci) = ctx.world.entity_mut(centNum).npcClient.as_deref_mut() {
        ci.extraSounds = [0; MAX_CUSTOM_EXTRA_SOUNDS];
    }

    // jedi
    let csSounds_Jedi = ctx.world.entity(centNum).currentState.csSounds_Jedi;
    if csSounds_Jedi != 0 {
        let s = CG_ConfigString(ctx, CS_SOUNDS + csSounds_Jedi);

        if !s.is_empty() {
            let sEnd: String = s.as_bytes().iter().skip(2).map(|&b| b as char).collect();

            let mut npc = ctx.world.entity_mut(centNum).npcClient.take();
            if let Some(ci) = npc.as_deref_mut() {
                CG_RegisterCustomSounds(ctx, ci, 4, &sEnd);
            }
            ctx.world.entity_mut(centNum).npcClient = npc;
        }
    } else if let Some(ci) = ctx.world.entity_mut(centNum).npcClient.as_deref_mut() {
        ci.jediSounds = [0; MAX_CUSTOM_JEDI_SOUNDS];
    }
}

/// Raven `CG_KillCEntityInstances` — resets every entity slot's ghoul2-adjacent
/// bolt/trail/anim latches for a fresh map/snapshot, additionally tearing down
/// non-client slots' live ghoul2 instances (client slots keep theirs - they're
/// constant across the connection).
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:937-981`
pub fn CG_KillCEntityInstances(ctx: &mut CgContext) {
    let mut i: usize = 0;

    while i < MAX_GENTITIES {
        if i >= MAX_CLIENTS && ctx.world.entity(i).currentState.number == i as c_int {
            // do not clear G2 instances on client ents, they are constant
            CG_KillCEntityG2(ctx, i);
        }

        let cent = ctx.world.entity_mut(i);

        cent.bolt1 = 0;
        cent.bolt2 = 0;
        cent.bolt3 = 0;
        cent.bolt4 = 0;

        cent.bodyHeight = 0.0; //SABER_LENGTH_MAX;
                               //cent->saberExtendTime = 0;

        cent.boltInfo = 0;

        cent.frame_minus1_refreshed = 0;
        cent.frame_minus2_refreshed = 0;
        cent.dustTrailTime = 0;
        cent.ghoul2weapon = null_mut();
        //cent->torsoBolt = 0;
        cent.trailTime = 0;
        cent.frame_hold_time = 0;
        cent.frame_hold_refreshed = 0;
        cent.trickAlpha = 0;
        cent.trickAlphaTime = 0;
        VectorClear(&mut cent.turAngles);
        cent.weapon = 0;
        cent.teamPowerEffectTime = 0;
        cent.teamPowerType = 0;
        cent.numLoopingSounds = 0;

        cent.localAnimIndex = 0;

        i += 1;
    }
}

/// Raven `CG_MapRestart` - clears the local-entity/mark/particle pools and
/// per-entity ghoul2 latches on a mid-match map restart, resets the frag/time
/// limit warning latches so they play again, and (skipping siege/powerduel)
/// plays the "fight" announcer sound + centerprint when the restart isn't
/// coming out of warmup.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:994-1038`
pub fn CG_MapRestart(ctx: &mut CgContext) {
    if ctx.world.cvars.cg_showmiss.integer != 0 {
        CG_Printf(ctx, "CG_MapRestart\n");
    }

    trap::R_ClearDecals(ctx.engine);
    //FIXME: trap_FX_Reset?

    CG_InitLocalEntities(ctx.world);
    CG_InitMarkPolys(ctx.world);
    CG_ClearParticles(ctx.world);
    CG_KillCEntityInstances(ctx);

    // make sure the "3 frags left" warnings play again
    ctx.world.cg.fraglimitWarnings = 0;

    ctx.world.cg.timelimitWarnings = 0;

    ctx.world.cg.intermissionStarted = qfalse;

    ctx.world.cgs.voteTime = 0;

    ctx.world.cg.mapRestart = qtrue;

    CG_StartMusic(ctx, true);

    trap::S_ClearLoopingSounds(ctx.engine);

    // we really should clear more parts of cg here and stop sounds

    // play the "fight" sound if this is a restart without warmup
    if ctx.world.cg.warmup == 0
        && ctx.world.cgs.gametype != GT_SIEGE
        && ctx.world.cgs.gametype != GT_POWERDUEL
    /* && cgs.gametype == GT_DUEL */
    {
        trap::S_StartLocalSound(
            ctx.engine,
            ctx.world.cgs.media.countFightSound,
            CHAN_ANNOUNCER,
        );
        let s = CG_GetStringEdString(ctx, "MP_SVGAME", "BEGIN_DUEL");
        CG_CenterPrint(ctx.world, &s, 120, GIANTCHAR_WIDTH * 2);
    }
    /*
    if (cg_singlePlayerActive.integer) {
        trap_Cvar_Set("ui_matchStartTime", va("%i", cg.time));
        if (cg_recordSPDemo.integer && cg_recordSPDemoName.string && *cg_recordSPDemoName.string) {
            trap_SendConsoleCommand(va("set g_synchronousclients 1 ; record %s \n", cg_recordSPDemoName.string));
        }
    }
    */
    trap::Cvar_Set(ctx.engine, "cg_thirdPerson", "0");
}

/// Raven `CG_ConfigStringModified` — dispatches one changed configstring index
/// onto the right piece of `cg`/`cgs`, re-fetching the whole `gameState` first
/// since the client system already folded the new string in.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:671-860`
pub fn CG_ConfigStringModified(ctx: &mut CgContext) {
    let arg = CG_Argv(ctx, 1);
    let num = atoi(&arg);

    // get the gamestate from the client system, which will have the
    // new configstring already integrated
    trap::GetGameState(ctx.engine, &mut ctx.world.cgs.gameState);

    // look up the individual string that was modified
    let s = CG_ConfigString(ctx, num);

    // do something with it if necessary
    if num == CS_MUSIC {
        CG_StartMusic(ctx, true);
    } else if num == CS_SERVERINFO {
        CG_ParseServerinfo(ctx);
    } else if num == CS_WARMUP {
        CG_ParseWarmup(ctx);
    } else if num == CS_SCORES1 {
        ctx.world.cgs.scores1 = atoi(&s);
    } else if num == CS_SCORES2 {
        ctx.world.cgs.scores2 = atoi(&s);
    } else if num == CS_CLIENT_JEDIMASTER {
        ctx.world.cgs.jediMaster = atoi(&s);
    } else if num == CS_CLIENT_DUELWINNER {
        ctx.world.cgs.duelWinner = atoi(&s);
    } else if num == CS_CLIENT_DUELISTS {
        let bytes = s.as_bytes();
        let mut buf = String::new();
        let mut i = 0usize;

        while i < bytes.len() && bytes[i] != b'|' {
            buf.push(bytes[i] as char);
            i += 1;
        }
        ctx.world.cgs.duelist1 = atoi(&buf);

        buf.clear();
        // §F19: same OOB-after-terminator quirk `SetDuelistHealthsFromConfigString`
        // above documents - a string with no `|` steps `i` past the end and the
        // length bound below just yields the rest as empty.
        i += 1;
        while i < bytes.len() && bytes[i] != b'|' {
            buf.push(bytes[i] as char);
            i += 1;
        }
        ctx.world.cgs.duelist2 = atoi(&buf);

        if i < bytes.len() {
            buf.clear();
            i += 1;

            while i < bytes.len() {
                buf.push(bytes[i] as char);
                i += 1;
            }
            ctx.world.cgs.duelist3 = atoi(&buf);
        }
    } else if num == CS_CLIENT_DUELHEALTHS {
        // nmckenzie: DUEL_HEALTH
        SetDuelistHealthsFromConfigString(ctx.world, &s);
    } else if num == CS_LEVEL_START_TIME {
        ctx.world.cgs.levelStartTime = atoi(&s);
    } else if num == CS_VOTE_TIME {
        ctx.world.cgs.voteTime = atoi(&s);
        ctx.world.cgs.voteModified = qtrue;
    } else if num == CS_VOTE_YES {
        ctx.world.cgs.voteYes = atoi(&s);
        ctx.world.cgs.voteModified = qtrue;
    } else if num == CS_VOTE_NO {
        ctx.world.cgs.voteNo = atoi(&s);
        ctx.world.cgs.voteModified = qtrue;
    } else if num == CS_VOTE_STRING {
        Q_strncpyz(&mut ctx.world.cgs.voteString, &s, MAX_STRING_TOKENS);
    } else if num >= CS_TEAMVOTE_TIME && num <= CS_TEAMVOTE_TIME + 1 {
        let idx = (num - CS_TEAMVOTE_TIME) as usize;
        ctx.world.cgs.teamVoteTime[idx] = atoi(&s);
        ctx.world.cgs.teamVoteModified[idx] = qtrue;
    } else if num >= CS_TEAMVOTE_YES && num <= CS_TEAMVOTE_YES + 1 {
        let idx = (num - CS_TEAMVOTE_YES) as usize;
        ctx.world.cgs.teamVoteYes[idx] = atoi(&s);
        ctx.world.cgs.teamVoteModified[idx] = qtrue;
    } else if num >= CS_TEAMVOTE_NO && num <= CS_TEAMVOTE_NO + 1 {
        let idx = (num - CS_TEAMVOTE_NO) as usize;
        ctx.world.cgs.teamVoteNo[idx] = atoi(&s);
        ctx.world.cgs.teamVoteModified[idx] = qtrue;
    } else if num >= CS_TEAMVOTE_STRING && num <= CS_TEAMVOTE_STRING + 1 {
        let idx = (num - CS_TEAMVOTE_STRING) as usize;
        // §F19: Raven's sizeof here is the whole 2D array (2048), so a >1023
        // char string overruns row 0 into row 1 (UB); the port caps at the row.
        Q_strncpyz(
            &mut ctx.world.cgs.teamVoteString[idx],
            &s,
            MAX_STRING_TOKENS,
        );
    } else if num == CS_INTERMISSION {
        ctx.world.cg.intermissionStarted = atoi(&s);
    } else if num >= CS_MODELS && num < CS_MODELS + MAX_MODELS {
        // Raven `strcpy(modelName, str)` into a fixed `MAX_QPATH` stack buffer
        // with no length check (UB on overflow); the port's `String` just grows.
        let mut modelName = s.clone();

        if modelName.contains(".glm") || modelName.starts_with('$') {
            // Check to see if it has a custom skin attached.
            CG_HandleAppendedSkin(ctx, &mut modelName);
            CG_CacheG2AnimInfo(ctx, &modelName);
        }

        let idx = (num - CS_MODELS) as usize;
        if !modelName.starts_with('$') && !modelName.starts_with('@') {
            // don't register vehicle names and saber names as models.
            ctx.world.cgs.gameModels[idx] = trap::R_RegisterModel(ctx.engine, &modelName);
        } else {
            ctx.world.cgs.gameModels[idx] = 0;
        }
        // GHOUL2 Insert start
        // cgs.skins[ num-CS_CHARSKINS ] = trap_R_RegisterSkin( str );
        // rww - removed and replaced with CS_G2BONES
        // Ghoul2 Insert end
    } else if num >= CS_SOUNDS && num < CS_SOUNDS + MAX_SOUNDS {
        let bytes = s.as_bytes();
        if bytes.first().copied().unwrap_or(0) != b'*' {
            // player specific sounds don't register here
            let idx = (num - CS_SOUNDS) as usize;
            ctx.world.cgs.gameSounds[idx] = trap::S_RegisterSound(ctx.engine, &s);
        } else if bytes.get(1) == Some(&b'$') {
            // an NPC soundset
            CG_PrecacheNPCSounds(ctx, &s);
        }
    } else if num >= CS_EFFECTS && num < CS_EFFECTS + MAX_FX {
        let idx = (num - CS_EFFECTS) as usize;
        if s.as_bytes().first() == Some(&b'*') {
            // it's a special global weather effect
            CG_ParseWeatherEffect(ctx, &s);
            ctx.world.cgs.gameEffects[idx] = 0;
        } else {
            ctx.world.cgs.gameEffects[idx] = trap::FX_RegisterEffect(ctx.engine, &s);
        }
    } else if num >= CS_SIEGE_STATE && num < CS_SIEGE_STATE + 1 {
        if !s.is_empty() {
            CG_ParseSiegeState(ctx.world, &s);
        }
    } else if num >= CS_SIEGE_WINTEAM && num < CS_SIEGE_WINTEAM + 1 {
        if !s.is_empty() {
            ctx.world.scoreboard.cg_siegeWinTeam = atoi(&s);
        }
    } else if num >= CS_SIEGE_OBJECTIVES && num < CS_SIEGE_OBJECTIVES + 1 {
        CG_ParseSiegeObjectiveStatus(ctx, &s);
    } else if num >= CS_SIEGE_TIMEOVERRIDE && num < CS_SIEGE_TIMEOVERRIDE + 1 {
        ctx.world.draw.cg_beatingSiegeTime = atoi(&s);
        let msec = ctx.world.draw.cg_beatingSiegeTime;
        CG_SetSiegeTimerCvar(ctx, msec);
    } else if num >= CS_PLAYERS && num < CS_PLAYERS + MAX_CLIENTS_I32 {
        CG_NewClientInfo(ctx, num - CS_PLAYERS, true);
        CG_BuildSpectatorString(ctx);
    } else if num == CS_FLAGSTATUS {
        if ctx.world.cgs.gametype == GT_CTF || ctx.world.cgs.gametype == GT_CTY {
            // format is rb where its red/blue, 0 is at base, 1 is taken, 2 is dropped
            let bytes = s.as_bytes();
            // Raven reads through the NUL on a short string: an empty
            // configstring gives `0 - '0'` = -48 (matches no flag-status arm,
            // draws nothing). Reproduced; only a true end-of-gamestate overrun
            // diverges (§F19: bound instead of reading past our String).
            ctx.world.cgs.redflag = bytes.first().map_or(-48, |&b| b as i32 - '0' as i32);
            ctx.world.cgs.blueflag = bytes.get(1).map_or(-48, |&b| b as i32 - '0' as i32);
        }
    } else if num == CS_SHADERSTATE {
        CG_ShaderStateChanged(ctx);
    } else if num >= CS_LIGHT_STYLES && num < CS_LIGHT_STYLES + (MAX_LIGHT_STYLES * 3) {
        CG_SetLightstyle(ctx, num - CS_LIGHT_STYLES);
    }
}

/// Raven's `char text[MAX_SAY_TEXT]` copy discipline - `Q_strncpyz`/`Com_sprintf`
/// into that fixed buffer stop at `MAX_SAY_TEXT-1` bytes; one Latin-1 char is one
/// byte, so we cap by char count.
fn cap_say_text(s: String) -> String {
    if s.chars().count() >= MAX_SAY_TEXT {
        s.chars().take(MAX_SAY_TEXT - 1).collect()
    } else {
        s
    }
}

/// Raven `CG_ServerCommand` — the client-game side of a reliable server command:
/// dispatches the leading token onto siege/menu/ghoul2-reset/chat/configstring
/// handlers, printing "Unknown client game command" for anything unrecognized.
///
/// PORT-NOTE: the `remapShader` arm has no `return`, so a `remapShader` command
/// falls through and prints "Unknown client game command" after remapping -
/// Raven's own quirk, preserved.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:1257-1670`
pub fn CG_ServerCommand(
    ctx: &mut CgContext,
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
) {
    let cmd = CG_Argv(ctx, 0);

    if cmd.is_empty() {
        // server claimed the command
        return;
    }

    // The `#if 0` "spd" arm (`cg_servercmds.c:1269-1294`) never shipped -Ste; dropped.

    if cmd == "sxd" {
        //siege extended data, contains extra info certain classes may want to know about other clients
        CG_ParseSiegeExtendedData(ctx);
        return;
    }

    if cmd == "sb" {
        //siege briefing display
        let team = atoi(&CG_Argv(ctx, 1));
        CG_SiegeBriefingDisplay(ctx, team, 0);
        return;
    }

    if cmd == "scl" {
        //if (!( trap_Key_GetCatcher() & KEYCATCH_UI ))
        //Well, I want it to come up even if the briefing display is up.
        trap::OpenUIMenu(ctx.engine, UIMENU_CLASSSEL); //UIMENU_CLASSSEL
        return;
    }

    if cmd == "spc" {
        trap::Cvar_Set(ctx.engine, "ui_myteam", "3");
        trap::OpenUIMenu(ctx.engine, UIMENU_PLAYERCONFIG); //UIMENU_CLASSSEL
        return;
    }

    if cmd == "nfr" {
        //"nfr" == "new force rank" (want a short string)
        if trap::Argc(ctx.engine) < 3 {
            // _DEBUG-only "Invalid newForceRank string" warning is compiled out.
            return;
        }

        let newRank = atoi(&CG_Argv(ctx, 1));
        let doMenu = atoi(&CG_Argv(ctx, 2));
        let setTeam = atoi(&CG_Argv(ctx, 3));

        trap::Cvar_Set(ctx.engine, "ui_rankChange", &format!("{newRank}"));

        trap::Cvar_Set(ctx.engine, "ui_myteam", &format!("{setTeam}"));

        if (trap::Key_GetCatcher(ctx.engine) & KEYCATCH_UI) == 0 && doMenu != 0 {
            trap::OpenUIMenu(ctx.engine, UIMENU_PLAYERCONFIG);
        }

        return;
    }

    if cmd == "kg2" {
        //Kill a ghoul2 instance in this slot.
        //If it has been occupied since this message was sent somehow, the worst that can (should) happen
        //is the instance will have to reinit with its current info.
        let argNum = trap::Argc(ctx.engine);
        let mut i = 1;

        if argNum < 1 {
            return;
        }

        while i < argNum {
            let indexNum = atoi(&CG_Argv(ctx, i));

            // §F19: `indexNum` is server-supplied; Raven indexes `cg_entities`
            // unchecked - out of range we skip instead of reading OOB.
            if indexNum >= 0 && (indexNum as usize) < MAX_GENTITIES {
                let idx = indexNum as usize;
                let ghoul2 = ctx.world.entity(idx).ghoul2;

                if !ghoul2.is_null() && trap::G2_HaveWeGhoul2Models(ctx.engine, ghoul2) {
                    if idx < MAX_CLIENTS {
                        //You try to do very bad thing!
                        // _DEBUG-only warning compiled out; the return is not.
                        return;
                    }

                    CG_KillCEntityG2(ctx, idx);
                }
            }

            i += 1;
        }

        return;
    }

    if cmd == "kls" {
        //kill looping sounds
        let argNum = trap::Argc(ctx.engine);

        if argNum < 1 {
            // Raven's assert(0) here is _DEBUG-only - compiled out in retail
            return;
        }

        let indexNum = atoi(&CG_Argv(ctx, 1));

        // §F19: Raven filters only -1 and reads any other index unchecked; we
        // guard the whole OOB range (server-supplied) and keep the -1 skip.
        let clentNum: Option<usize> =
            (indexNum >= 0 && (indexNum as usize) < MAX_GENTITIES).then_some(indexNum as usize);

        let mut trackerentNum: Option<usize> = None;
        if argNum >= 2 {
            let indexNum = atoi(&CG_Argv(ctx, 2));

            if indexNum >= 0 && (indexNum as usize) < MAX_GENTITIES {
                trackerentNum = Some(indexNum as usize);
            }
        }

        if let Some(clentNum) = clentNum {
            let number = ctx.world.entity(clentNum).currentState.number;
            CG_S_StopLoopingSound(ctx.world, number as usize, -1);
        }
        if let Some(trackerentNum) = trackerentNum {
            let number = ctx.world.entity(trackerentNum).currentState.number;
            CG_S_StopLoopingSound(ctx.world, number as usize, -1);
        }

        return;
    }

    let mut IRCG = false;
    if cmd == "ircg" {
        //this means param 2 is the body index and we want to copy to bodyqueue on it
        IRCG = true;
    }

    if cmd == "rcg" || IRCG {
        //rcg - Restore Client Ghoul (make sure limbs are reattached and ragdoll state is reset - this must be done reliably)
        let argNum = trap::Argc(ctx.engine);

        if argNum < 1 {
            // Raven's assert(0) here is _DEBUG-only - compiled out in retail
            return;
        }

        let indexNum = atoi(&CG_Argv(ctx, 1));
        if indexNum < 0 || indexNum >= MAX_CLIENTS_I32 {
            // server-supplied client index; Raven asserts, we just bail (§F19).
            return;
        }
        let idx = indexNum as usize;

        //assert(clent->ghoul2);
        let ghoul2 = ctx.world.entity(idx).ghoul2;
        if ghoul2.is_null() {
            //this can happen while connecting as a client
            return;
        }

        if IRCG {
            let bodyIndex = atoi(&CG_Argv(ctx, 2));
            let weaponIndex = atoi(&CG_Argv(ctx, 3));
            let side = atoi(&CG_Argv(ctx, 4));

            // §F19: `bodyIndex` is server-supplied; the guard covers the OOB
            // `cg_entities` read AND Raven's unconditional CG_BodyQueueCopy
            // call, which only ran usefully on the in-bounds path anyway.
            if bodyIndex >= 0 && (bodyIndex as usize) < MAX_GENTITIES {
                let bodyNum = bodyIndex as usize;

                ctx.world.entity_mut(bodyNum).teamPowerType = if side != 0 {
                    qtrue //light side
                } else {
                    qfalse //dark side
                };

                let clientNum = ctx.world.entity(idx).currentState.number;
                CG_BodyQueueCopy(ctx, bodyNum, clientNum, weaponIndex);
            }
        }

        //reattach any missing limbs
        if ctx.world.entity(idx).torsoBolt != 0 {
            CG_ReattachLimb(ctx, idx);
        }

        //make sure ragdoll state is reset
        if ctx.world.entity(idx).isRagging != qfalse {
            ctx.world.entity_mut(idx).isRagging = qfalse;
            //calling with null parms resets to no ragdoll.
            let ghoul2 = ctx.world.entity(idx).ghoul2;
            trap::G2API_SetRagDoll(ctx.engine, ghoul2, None);
        }

        //clear all the decals as well
        let ghoul2 = ctx.world.entity(idx).ghoul2;
        trap::G2API_ClearSkinGore(ctx.engine, ghoul2);

        ctx.world.entity_mut(idx).weapon = 0;
        ctx.world.entity_mut(idx).ghoul2weapon = null_mut(); //force a weapon reinit

        return;
    }

    if cmd == "cp" {
        let arg = CG_Argv(ctx, 1);
        let strEd = CG_CheckSVStringEdRef(ctx, &arg);
        CG_CenterPrint(
            ctx.world,
            &strEd,
            (SCREEN_HEIGHT as f64 * 0.30) as c_int,
            BIGCHAR_WIDTH,
        );
        return;
    }

    if cmd == "cps" {
        let arg = CG_Argv(ctx, 1);
        let x = arg.strip_prefix('@').unwrap_or(&arg);
        let strEd = trap::SP_GetStringTextString(ctx.engine, x, MAX_STRINGED_SV_STRING)
            .unwrap_or_else(|| format!("??{x}"));
        CG_CenterPrint(
            ctx.world,
            &strEd,
            (SCREEN_HEIGHT as f64 * 0.20) as c_int,
            BIGCHAR_WIDTH,
        );
        return;
    }

    if cmd == "cs" {
        CG_ConfigStringModified(ctx);
        return;
    }

    if cmd == "print" {
        let arg = CG_Argv(ctx, 1);
        let strEd = CG_CheckSVStringEdRef(ctx, &arg);
        CG_Printf(ctx, &strEd);
        return;
    }

    if cmd == "chat" {
        if ctx.world.cvars.cg_teamChatsOnly.integer == 0 {
            let talkSound = ctx.world.cgs.media.talkSound;
            trap::S_StartLocalSound(ctx.engine, talkSound, CHAN_LOCAL_SOUND);
            let text = cap_say_text(CG_Argv(ctx, 1));
            let text = CG_RemoveChatEscapeChar(&text);
            CG_ChatBox_AddString(ctx, ds, &text);
            CG_Printf(ctx, &format!("*{text}\n"));
        }
        return;
    }

    if cmd == "tchat" {
        let talkSound = ctx.world.cgs.media.talkSound;
        trap::S_StartLocalSound(ctx.engine, talkSound, CHAN_LOCAL_SOUND);
        let text = cap_say_text(CG_Argv(ctx, 1));
        let text = CG_RemoveChatEscapeChar(&text);
        CG_ChatBox_AddString(ctx, ds, &text);
        CG_Printf(ctx, &format!("*{text}\n"));

        return;
    }

    //chat with location, possibly localized.
    if cmd == "lchat" {
        if ctx.world.cvars.cg_teamChatsOnly.integer == 0 {
            if trap::Argc(ctx.engine) < 4 {
                return;
            }

            let name = CG_Argv(ctx, 1);
            // §F19: Raven strcpy's this into char color[8] - a server token
            // past 7 bytes overruns his stack; the owned String just holds it.
            let color = CG_Argv(ctx, 3);
            let message = CG_Argv(ctx, 4);
            let mut loc = CG_Argv(ctx, 2);

            if loc.starts_with('@') {
                //get localized text
                loc = trap::SP_GetStringTextString(ctx.engine, &loc[1..], MAX_STRING_CHARS)
                    .unwrap_or_else(|| format!("??{}", &loc[1..]));
            }

            let talkSound = ctx.world.cgs.media.talkSound;
            trap::S_StartLocalSound(ctx.engine, talkSound, CHAN_LOCAL_SOUND);
            let text = cap_say_text(format!("{name}<{loc}>^{color}{message}"));
            let text = CG_RemoveChatEscapeChar(&text);
            CG_ChatBox_AddString(ctx, ds, &text);
            CG_Printf(ctx, &format!("*{text}\n"));
        }
        return;
    }
    if cmd == "ltchat" {
        if trap::Argc(ctx.engine) < 4 {
            return;
        }

        let name = CG_Argv(ctx, 1);
        // §F19: same char color[8] strcpy overrun as the lchat arm above
        let color = CG_Argv(ctx, 3);
        let message = CG_Argv(ctx, 4);
        let mut loc = CG_Argv(ctx, 2);

        if loc.starts_with('@') {
            //get localized text
            loc = trap::SP_GetStringTextString(ctx.engine, &loc[1..], MAX_STRING_CHARS)
                .unwrap_or_else(|| format!("??{}", &loc[1..]));
        }

        let talkSound = ctx.world.cgs.media.talkSound;
        trap::S_StartLocalSound(ctx.engine, talkSound, CHAN_LOCAL_SOUND);
        let text = cap_say_text(format!("{name}<{loc}> ^{color}{message}"));
        let text = CG_RemoveChatEscapeChar(&text);
        CG_ChatBox_AddString(ctx, ds, &text);
        CG_Printf(ctx, &format!("*{text}\n"));

        return;
    }

    if cmd == "scores" {
        CG_ParseScores(ctx, menus, ds, dc);
        return;
    }

    if cmd == "tinfo" {
        CG_ParseTeamInfo(ctx);
        return;
    }

    if cmd == "map_restart" {
        CG_MapRestart(ctx);
        return;
    }

    if Q_stricmp(&cmd, "remapShader") == 0 {
        if trap::Argc(ctx.engine) == 4 {
            let old = CG_Argv(ctx, 1);
            let new = CG_Argv(ctx, 2);
            let timeOffset = CG_Argv(ctx, 3);
            trap::R_RemapShader(ctx.engine, &old, &new, &timeOffset);
        }
    }

    // loaddeferred can be both a servercmd and a consolecmd
    if cmd == "loaddefered" {
        // FIXME: spelled wrong, but not changing for demo
        CG_LoadDeferredPlayers(ctx.world);
        return;
    }

    // clientLevelShot is sent before taking a special screenshot for
    // the menu system during development
    if cmd == "clientLevelShot" {
        ctx.world.cg.levelShot = qtrue;
        return;
    }

    CG_Printf(ctx, &format!("Unknown client game command: {cmd}\n"));
}
