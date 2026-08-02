//! `snd_music.cpp` — the dynamic-music description and the transition rules.
//!
//! `ext_data/dms.dat` names, per level, which music piece plays for explore,
//! action, boss, and death, where each piece may be entered, and where it may
//! be left. This file parses that description and answers the four questions the
//! background player asks: which file a state plays, whether a state may be
//! interrupted, whether the play position has reached an exit point, and which
//! entry time a fresh action piece starts at.
//!
//! Every `snd_music.cpp` file-scope global lives in `SoundSystem.music`
//! (porting-rules §B3), and the parser is the ported GP2 (`mp_engine_qcommon`).
//!
//! Source: `oracle/codemp/client/snd_music.cpp`

#![allow(non_snake_case)]

use core::ffi::c_int;
use std::collections::BTreeMap;

use mp_engine_qcommon::common::common::com_printf;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::QRand;
use mp_engine_qcommon::files_common::FS_ReadFileVec;
use mp_engine_qcommon::gp2::generic_parser2::GenericParser2;
use mp_engine_qcommon::gp2::gp_group::GpGroup;
use mp_qshared::shared::MAX_QPATH;
use native_string::q_string::Q_stricmp;

use crate::snd::music_data_t::MusicData_t;
use crate::snd::music_exit_point_t::MusicExitPoint_t;
use crate::snd::music_exit_time_t::MusicExitTime_t;
use crate::snd::music_file_t::MusicFile_t;
use crate::snd::music_state_e::MusicState_e;
use crate::snd::sound_system::SoundSystem;
use crate::snd_dma::S_FileExists;

/// Raven's `dms.dat` keys.
///
/// Source: `oracle/codemp/client/snd_music.cpp:37-54`
const sKEY_MUSICFILES: &str = "musicfiles";
const sKEY_ENTRY: &str = "entry";
const sKEY_EXIT: &str = "exit";
const sKEY_TIME: &str = "time";
const sKEY_NEXTFILE: &str = "nextfile";
const sKEY_NEXTMARK: &str = "nextmark";
const sKEY_LEVELMUSIC: &str = "levelmusic";
const sKEY_EXPLORE: &str = "explore";
const sKEY_ACTION: &str = "action";
const sKEY_BOSS: &str = "boss";
const sKEY_DEATH: &str = "death";
const sKEY_USES: &str = "uses";
const sKEY_USEBOSS: &str = "useboss";
/// Raven ignores any entry whose value is this.
const sKEY_PLACEHOLDER: &str = "placeholder";

/// Raven `sFILENAME_DMS`.
///
/// Source: `oracle/codemp/client/snd_music.cpp:54`
pub const sFILENAME_DMS: &str = "ext_data/dms.dat";

/// Raven `iMAX_ACTION_TRANSITIONS` / `iMAX_EXPLORE_TRANSITIONS`.
///
/// Source: `oracle/codemp/client/snd_music.h:38-39`
const iMAX_ACTION_TRANSITIONS: usize = 4;
const iMAX_EXPLORE_TRANSITIONS: usize = 4;

// Raven's `Music_Free` and `Music_SetLevelName` have no caller in either tree
// (porting-rules §20), so neither ports. `gsLevelNameFromServer` therefore stays
// empty in MP, and `Music_DynamicDataAvailable` always works off the label its
// caller hands it.
// Source: `oracle/codemp/client/snd_music.cpp:100-104,430-433`

/// Raven `Music_Parse_Error` — report a broken description and throw it away.
///
/// The retail build prints only under `s_debugdynamic`, and the data is gone
/// once this returns, so a caller must leave its loop at once.
/// Source: `oracle/codemp/client/snd_music.cpp:108-120`
fn Music_Parse_Error(view: &mut EngineHostView, snd: &mut SoundSystem, psError: &str) {
    if snd.s_debugdynamic.is_some() && view.common.cvar(snd.s_debugdynamic).integer != 0 {
        com_printf(
            view.common,
            &format!("^1Error parsing music data ( in \"{sFILENAME_DMS}\" ):\n{psError}\n"),
        );
    }
    snd.music.MusicData = Some(BTreeMap::new());
}

/// Raven `Music_Parse_Warning` — mention something, under the same gate.
///
/// Source: `oracle/codemp/client/snd_music.cpp:124-135`
fn Music_Parse_Warning(view: &mut EngineHostView, snd: &SoundSystem, psError: &str) {
    if snd.s_debugdynamic.is_some() && view.common.cvar(snd.s_debugdynamic).integer != 0 {
        com_printf(view.common, &format!("^3{psError}"));
    }
}

/// Raven `Music_BuildFileName` — the pak path of one music piece.
///
/// Raven calls the death case a hack: every level shares one death piece.
/// Source: `oracle/codemp/client/snd_music.cpp:141-155`
fn Music_BuildFileName(
    music: &MusicData_t,
    psFileNameBase: &str,
    eMusicState: MusicState_e,
) -> String {
    if eMusicState == MusicState_e::eBGRNDTRACK_DEATH {
        return "music/death_music.mp3".to_string();
    }

    let psDirName = if eMusicState == MusicState_e::eBGRNDTRACK_BOSS {
        &music.gsLevelNameForBossLoad
    } else {
        &music.gsLevelNameForLoad
    };

    format!("music/{psDirName}/{psFileNameBase}.mp3")
}

/// Raven `Music_BaseStateToString` — the `dms.dat` key of a base state.
///
/// Answers `None` for a non-base state unless `bDebugPrintQuery` is set, because
/// only a base state is a map key. Raven's switch has no `break` on the
/// transition cases, so a non-debug query falls through every one of them and
/// out of the switch.
/// Source: `oracle/codemp/client/snd_music.cpp:158-182`
pub fn Music_BaseStateToString(
    eMusicState: MusicState_e,
    bDebugPrintQuery: bool,
) -> Option<&'static str> {
    match eMusicState {
        MusicState_e::eBGRNDTRACK_EXPLORE => Some("explore"),
        MusicState_e::eBGRNDTRACK_ACTION => Some("action"),
        MusicState_e::eBGRNDTRACK_BOSS => Some("boss"),
        // Raven: not used in this module, but snd_dma uses it now it is de-static'd.
        MusicState_e::eBGRNDTRACK_SILENCE => Some("silence"),
        MusicState_e::eBGRNDTRACK_DEATH => Some("death"),

        // Info only, not map lookup keys.
        MusicState_e::eBGRNDTRACK_ACTIONTRANS0 if bDebugPrintQuery => Some("action_tr0"),
        MusicState_e::eBGRNDTRACK_ACTIONTRANS1 if bDebugPrintQuery => Some("action_tr1"),
        MusicState_e::eBGRNDTRACK_ACTIONTRANS2 if bDebugPrintQuery => Some("action_tr2"),
        MusicState_e::eBGRNDTRACK_ACTIONTRANS3 if bDebugPrintQuery => Some("action_tr3"),
        MusicState_e::eBGRNDTRACK_EXPLORETRANS0 if bDebugPrintQuery => Some("explore_tr0"),
        MusicState_e::eBGRNDTRACK_EXPLORETRANS1 if bDebugPrintQuery => Some("explore_tr1"),
        MusicState_e::eBGRNDTRACK_EXPLORETRANS2 if bDebugPrintQuery => Some("explore_tr2"),
        MusicState_e::eBGRNDTRACK_EXPLORETRANS3 if bDebugPrintQuery => Some("explore_tr3"),
        MusicState_e::eBGRNDTRACK_FADE if bDebugPrintQuery => Some("fade"),

        _ => None,
    }
}

/// Raven `Music_ParseMusic` — read one piece's entry and exit blocks.
///
/// Entry points are read first, so an exit point can be checked against them.
/// Returns false where the piece is missing or its blocks do not add up, and
/// every failure path has already thrown the description away.
/// Source: `oracle/codemp/client/snd_music.cpp:184-357`
fn Music_ParseMusic(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    pgMusicFiles: GpGroup<'_>,
    psMusicName: &str,
    psMusicNameKey: &str,
    eMusicState: MusicState_e,
) -> bool {
    let mut MusicFile = MusicFile_t::default();

    let Some(pgMusicFile) = pgMusicFiles.find_sub_group(psMusicName) else {
        Music_Parse_Error(
            view,
            snd,
            &format!("Unable to find musicfiles entry \"{psMusicName}\"\n"),
        );
        return false;
    };

    let mut bEntryFound = false;
    let mut bExitFound = false;

    // Read entry points first, so exit points can be checked against them.
    if let Some(pEntryGroup) = pgMusicFile.find_sub_group(sKEY_ENTRY) {
        for pValue in pEntryGroup.pairs() {
            // Raven's marker-name test is commented out: anything is a marker.
            let psKey = pValue.name().to_string();
            let psValue = pValue.top_value().unwrap_or("");
            MusicFile.MusicEntryTimes.insert(psKey, atof(psValue) as f32);
            bEntryFound = true; // harmless to keep setting
        }
    }

    for pGroup in pgMusicFile.subgroups() {
        let psGroupName = pGroup.name();

        if psGroupName == sKEY_ENTRY {
            // skip entry points, already read above
            continue;
        }
        if psGroupName != sKEY_EXIT {
            continue;
        }

        // Must be read before the push below, so it is unaffected by it.
        let iThisExitPointIndex = MusicFile.MusicExitPoints.len() as c_int;

        let mut MusicExitPoint = MusicExitPoint_t::default();
        for pValue in pGroup.pairs() {
            let psKey = pValue.name();
            let psValue = pValue.top_value().unwrap_or("");

            if psKey == sKEY_NEXTFILE {
                MusicExitPoint.sNextFile = psValue.to_string();
                bExitFound = true; // harmless to keep setting
            } else if psKey == sKEY_NEXTMARK {
                MusicExitPoint.sNextMark = psValue.to_string();
            } else if psKey.starts_with(sKEY_TIME) {
                let MusicExitTime = MusicExitTime_t {
                    fTime: atof(psValue) as f32,
                    iExitPoint: iThisExitPointIndex,
                };

                // Raven's "too close to an entry point" reject is commented
                // out, so the loop below finds nothing and every time is kept.
                MusicFile.MusicExitTimes.push(MusicExitTime);
            }
        }

        MusicFile.MusicExitPoints.push(MusicExitPoint);
        let iNumExitPoints = MusicFile.MusicExitPoints.len();

        match eMusicState {
            MusicState_e::eBGRNDTRACK_EXPLORE => {
                if iNumExitPoints > iMAX_EXPLORE_TRANSITIONS {
                    Music_Parse_Error(view, snd, &format!(
                        "\"{psMusicName}\" has > {iMAX_EXPLORE_TRANSITIONS} {psMusicNameKey} transitions defined!\n"
                    ));
                    return false;
                }
            }
            MusicState_e::eBGRNDTRACK_ACTION => {
                if iNumExitPoints > iMAX_ACTION_TRANSITIONS {
                    Music_Parse_Error(view, snd, &format!(
                        "\"{psMusicName}\" has > {iMAX_ACTION_TRANSITIONS} {psMusicNameKey} transitions defined!\n"
                    ));
                    return false;
                }
            }
            MusicState_e::eBGRNDTRACK_BOSS | MusicState_e::eBGRNDTRACK_DEATH => {
                // Raven reports and keeps going: there is no `return` here.
                Music_Parse_Error(view, snd, &format!(
                    "\"{psMusicName}\" has {psMusicNameKey} transitions defined, this is not allowed!\n"
                ));
            }
            _ => {}
        }
    }

    // For now, assume everything was ok unless some obvious things are missing.
    let mut bReturn = true;

    // Boss and death pieces can omit entry and exit blocks.
    if eMusicState != MusicState_e::eBGRNDTRACK_BOSS
        && eMusicState != MusicState_e::eBGRNDTRACK_DEATH
    {
        if !bEntryFound {
            Music_Parse_Error(
                view,
                snd,
                &format!("Unable to find subgroup \"{sKEY_ENTRY}\" in group \"{psMusicName}\"\n"),
            );
            bReturn = false;
        }
        if !bExitFound {
            Music_Parse_Error(
                view,
                snd,
                &format!("Unable to find subgroup \"{sKEY_EXIT}\" in group \"{psMusicName}\"\n"),
            );
            bReturn = false;
        }
    }

    if bReturn {
        MusicFile.sFileNameBase = psMusicName.to_string();
        snd.music
            .MusicData
            .get_or_insert_with(BTreeMap::new)
            .insert(psMusicNameKey.to_string(), MusicFile);
    }

    bReturn
}

/// Raven `StripTrailingWhiteSpaceOnEveryLine` — GP2 cannot cope with trailing
/// whitespace, so every line is trimmed and the output is newline separated.
///
/// Raven copies at most 1023 bytes of a line before it forces a break, so a
/// longer line is split rather than truncated.
/// Source: `oracle/codemp/client/snd_music.cpp:368-422`
fn StripTrailingWhiteSpaceOnEveryLine(pText: &str) -> String {
    const LINE_BYTES: usize = 1024;

    let bytes = pText.as_bytes();
    let mut strNewText = String::new();
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        // Find the end of this line, and stop at the copy limit either way.
        let mut end = cursor;
        while end < bytes.len() && bytes[end] != b'\r' && (end - cursor) < LINE_BYTES - 1 {
            end += 1;
        }

        let mut sOneLine = String::from_utf8_lossy(&bytes[cursor..end]).into_owned();
        cursor = end;
        while cursor < bytes.len() && (bytes[cursor] == b'\n' || bytes[cursor] == b'\r') {
            cursor += 1;
        }

        while sOneLine.ends_with('\t') || sOneLine.ends_with(' ') {
            sOneLine.pop();
        }

        strNewText.push_str(&sOneLine);
        strNewText.push('\n');
    }

    strNewText
}

/// Raven `Music_ParseLeveldata` — read `dms.dat` and keep this level's pieces.
///
/// The description is cached, so a repeat call for the same level answers at
/// once. A "uses" chain redirects one level's music to another, up to ten deep.
/// Source: `oracle/codemp/client/snd_music.cpp:435-764`
fn Music_ParseLeveldata(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    psLevelName: &str,
) -> bool {
    if snd.music.MusicData.is_none() {
        snd.music.MusicData = Some(BTreeMap::new());
    }

    // Already got this data?
    let cached = snd
        .music
        .MusicData
        .as_ref()
        .is_some_and(|data| !data.is_empty());
    if cached && Q_stricmp(psLevelName, &snd.music.gsLevelNameForCompare) == 0 {
        return true;
    }

    snd.music.MusicData = Some(BTreeMap::new());

    let mut sLevelName = psLevelName.to_string();
    sLevelName.truncate(MAX_QPATH - 1);

    // Harmless to init here even if we fail to parse the dms.dat file.
    snd.music.gsLevelNameForLoad = sLevelName.clone();
    snd.music.gsLevelNameForCompare = sLevelName.clone();
    snd.music.gsLevelNameForBossLoad = sLevelName.clone();

    let Some(pText) = FS_ReadFileVec(view, sFILENAME_DMS) else {
        // The file name is in the error message.
        Music_Parse_Error(view, snd, "Unable to even read main file\n");
        return false;
    };

    let psStrippedText = StripTrailingWhiteSpaceOnEveryLine(&String::from_utf8_lossy(&pText));
    let mut parser = GenericParser2::new();
    if parser.parse(&psStrippedText, true).is_err() {
        Music_Parse_Error(view, snd, "Error using GP to parse file\n");
        return false;
    }

    let pFileGroup = parser.top_level();
    let Some(pgMusicFiles) = pFileGroup.find_sub_group(sKEY_MUSICFILES) else {
        Music_Parse_Error(
            view,
            snd,
            &format!("Unable to find subgroup \"{sKEY_MUSICFILES}\"\n"),
        );
        return false;
    };

    let Some(pgLevelMusic) = pFileGroup.find_sub_group(sKEY_LEVELMUSIC) else {
        Music_Parse_Error(
            view,
            snd,
            &format!("Unable to find subgroup \"{sKEY_LEVELMUSIC}\"\n"),
        );
        return false;
    };

    // Follow the "uses" chain to the set this level actually plays.
    let mut pgThisLevelMusic = None;
    let mut iSanityLimit = 0;
    let mut sSearchName = sLevelName.clone();

    while !sSearchName.is_empty() && iSanityLimit < 10 {
        snd.music.gsLevelNameForLoad = sSearchName.clone();
        snd.music.gsLevelNameForBossLoad = sSearchName.clone();
        pgThisLevelMusic = pgLevelMusic.find_sub_group(&sSearchName);

        let Some(group) = pgThisLevelMusic else {
            // level entry not found
            break;
        };
        match group.find_pair_value(sKEY_USES) {
            Some(value) => {
                sSearchName = value.to_string();
                iSanityLimit += 1;
            }
            None => sSearchName = String::new(),
        }
    }

    let (Some(thisLevelMusic), true) = (pgThisLevelMusic, iSanityLimit < 10) else {
        Music_Parse_Warning(
            view,
            snd,
            &format!("Unable to find entry for \"{sLevelName}\" in \"{sFILENAME_DMS}\"\n"),
        );
        return finish_parse(view, snd, false);
    };

    // These are optional fields, so see which ones we find.
    let mut psName_Explore: Option<String> = None;
    let mut psName_Action: Option<String> = None;
    let mut psName_Boss: Option<String> = None;
    let mut psName_UseBoss: Option<String> = None;

    for pValue in thisLevelMusic.pairs() {
        let psKey = pValue.name();
        let psValue = pValue.top_value().unwrap_or("");

        if Q_stricmp(psValue, sKEY_PLACEHOLDER) == 0 {
            continue; // ignore "placeholder" items
        }
        if Q_stricmp(psKey, sKEY_EXPLORE) == 0 {
            psName_Explore = Some(psValue.to_string());
        } else if Q_stricmp(psKey, sKEY_ACTION) == 0 {
            psName_Action = Some(psValue.to_string());
        } else if Q_stricmp(psKey, sKEY_USEBOSS) == 0 {
            psName_UseBoss = Some(psValue.to_string());
        } else if Q_stricmp(psKey, sKEY_BOSS) == 0 {
            psName_Boss = Some(psValue.to_string());
        } else if Q_stricmp(psKey, sKEY_DEATH) == 0 {
            // Raven reads the death name and then never uses it, see below.
        }
    }

    // Default to ON now, so it can be turned off if "useboss" fails.
    let mut bReturn = true;

    if let Some(useBoss) = psName_UseBoss.clone() {
        match pgLevelMusic.find_sub_group(&useBoss) {
            Some(pgLevelMusicOfBoss) => match pgLevelMusicOfBoss.find_pair_value(sKEY_BOSS) {
                Some(value) => {
                    psName_Boss = Some(value.to_string());
                    snd.music.gsLevelNameForBossLoad = useBoss;
                }
                None => {
                    Music_Parse_Error(
                        view,
                        snd,
                        &format!("'useboss' \"{useBoss}\" has no \"boss\" entry!\n"),
                    );
                    bReturn = false;
                }
            },
            None => {
                Music_Parse_Error(
                    view,
                    snd,
                    &format!("Unable to find 'useboss' entry \"{useBoss}\"\n"),
                );
                bReturn = false;
            }
        }
    }

    if bReturn {
        if let Some(name) = psName_Explore {
            bReturn = Music_ParseMusic(
                view,
                snd,
                pgMusicFiles,
                &name,
                sKEY_EXPLORE,
                MusicState_e::eBGRNDTRACK_EXPLORE,
            );
        }
    }
    if bReturn {
        if let Some(name) = psName_Action {
            bReturn = Music_ParseMusic(
                view,
                snd,
                pgMusicFiles,
                &name,
                sKEY_ACTION,
                MusicState_e::eBGRNDTRACK_ACTION,
            );
        }
    }
    if bReturn {
        if let Some(name) = psName_Boss {
            bReturn = Music_ParseMusic(
                view,
                snd,
                pgMusicFiles,
                &name,
                sKEY_BOSS,
                MusicState_e::eBGRNDTRACK_BOSS,
            );
        }
    }
    if bReturn {
        // Raven calls this a last-minute hack: death music is always forced in,
        // and the parsed death name is ignored.
        let mut m = MusicFile_t::default();
        m.sFileNameBase = "death_music".to_string();
        snd.music
            .MusicData
            .get_or_insert_with(BTreeMap::new)
            .insert(sKEY_DEATH.to_string(), m);
    }

    finish_parse(view, snd, bReturn)
}

/// The post-parse pass: sort every exit-time list and check every named file
/// exists.
///
/// A failure here has already destroyed the description, so the caller must not
/// look at it again.
/// Source: `oracle/codemp/client/snd_music.cpp:648-716`
fn finish_parse(view: &mut EngineHostView, snd: &mut SoundSystem, bReturn: bool) -> bool {
    if !bReturn {
        return false;
    }

    let keys: Vec<String> = snd
        .music
        .MusicData
        .as_ref()
        .map(|data| data.keys().cloned().collect())
        .unwrap_or_default();

    for psMusicStateType in keys {
        // Raven kludges up an enum here, and only cares about boss or not.
        let eMusicState = if psMusicStateType.eq_ignore_ascii_case("boss") {
            MusicState_e::eBGRNDTRACK_BOSS
        } else if psMusicStateType.eq_ignore_ascii_case("death") {
            MusicState_e::eBGRNDTRACK_DEATH
        } else {
            MusicState_e::eBGRNDTRACK_EXPLORE
        };

        if let Some(file) = snd
            .music
            .MusicData
            .as_mut()
            .and_then(|data| data.get_mut(&psMusicStateType))
        {
            if !file.MusicExitTimes.is_empty() {
                file.MusicExitTimes
                    .sort_by(|a, b| a.fTime.partial_cmp(&b.fTime).unwrap_or(core::cmp::Ordering::Equal));
            }
        }

        let (sFileNameBase, exitPoints) = {
            let file = snd
                .music
                .MusicData
                .as_ref()
                .and_then(|data| data.get(&psMusicStateType));
            let Some(file) = file else { continue };
            (file.sFileNameBase.clone(), file.MusicExitPoints.clone())
        };

        // check music exists...
        let psMusicFileName = Music_BuildFileName(&snd.music, &sFileNameBase, eMusicState);
        if !S_FileExists(view, &psMusicFileName) {
            Music_Parse_Error(
                view,
                snd,
                &format!("Music file \"{psMusicFileName}\" not found!\n"),
            );
            return false; // have to return, because music data destroyed now
        }

        // Check every transition piece exists, and that the entry point each
        // one names exists in the explore piece.
        for MusicExitPoint in exitPoints {
            let psTransitionFileName =
                Music_BuildFileName(&snd.music, &MusicExitPoint.sNextFile, eMusicState);
            if !S_FileExists(view, &psTransitionFileName) {
                let next = MusicExitPoint.sNextFile;
                Music_Parse_Error(view, snd, &format!(
                    "Transition file \"{psTransitionFileName}\" (entry \"{next}\" ) not found!\n"
                ));
                return false;
            }

            let psNextMark = MusicExitPoint.sNextMark;
            if psNextMark.is_empty() {
                continue;
            }

            // Then this must be "action" music under current rules, and the
            // marker must exist in the explore piece.
            let explore = snd
                .music
                .MusicData
                .as_ref()
                .and_then(|data| data.get(sKEY_EXPLORE));
            match explore {
                Some(MusicFile_Explore) => {
                    if !MusicFile_Explore.MusicEntryTimes.contains_key(&psNextMark) {
                        let base = MusicFile_Explore.sFileNameBase.clone();
                        Music_Parse_Error(view, snd, &format!(
                            "Unable to find entry point \"{psNextMark}\" in description for \"{base}\"\n"
                        ));
                        return false;
                    }
                }
                None => {
                    Music_Parse_Error(view, snd, &format!(
                        "Unable to find {sKEY_EXPLORE} piece to match \"{sFileNameBase}\"\n"
                    ));
                    return false;
                }
            }
        }
    }

    true
}

/// Raven `Music_GetBaseMusicFile` — the piece one base state plays.
///
/// Source: `oracle/codemp/client/snd_music.cpp:769-790`
fn Music_GetBaseMusicFile<'a>(
    music: &'a MusicData_t,
    eMusicState: MusicState_e,
) -> Option<&'a MusicFile_t> {
    let psMusicStateString = Music_BaseStateToString(eMusicState, false)?;
    music.MusicData.as_ref()?.get(psMusicStateString)
}

/// Raven `Music_DynamicDataAvailable` — does this level have both an explore and
/// an action piece?
///
/// The label is a level name; a blank one falls back to the name the server sent.
/// Source: `oracle/codemp/client/snd_music.cpp:795-812`
pub fn Music_DynamicDataAvailable(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    psDynamicMusicLabel: &str,
) -> bool {
    let source = if psDynamicMusicLabel.is_empty() {
        snd.music.gsLevelNameFromServer.clone()
    } else {
        psDynamicMusicLabel.to_string()
    };
    let mut sLevelName = COM_SkipPath(&source).to_string();
    sLevelName.truncate(MAX_QPATH - 1);
    sLevelName.make_ascii_lowercase();

    // Avoid error messages when there is no music waiting to be played.
    if sLevelName.is_empty() {
        return false;
    }

    if Music_ParseLeveldata(view, snd, &sLevelName) {
        return Music_GetBaseMusicFile(&snd.music, MusicState_e::eBGRNDTRACK_EXPLORE).is_some()
            && Music_GetBaseMusicFile(&snd.music, MusicState_e::eBGRNDTRACK_ACTION).is_some();
    }

    false
}

/// Raven `Music_GetFileNameForState` — the pak path one state plays.
///
/// A transition state reads its file off the base piece's exit-point list, and a
/// state with no piece answers `None`.
/// Source: `oracle/codemp/client/snd_music.cpp:814-872`
pub fn Music_GetFileNameForState(
    music: &MusicData_t,
    eMusicState: MusicState_e,
) -> Option<String> {
    match eMusicState {
        MusicState_e::eBGRNDTRACK_EXPLORE
        | MusicState_e::eBGRNDTRACK_ACTION
        | MusicState_e::eBGRNDTRACK_BOSS
        | MusicState_e::eBGRNDTRACK_DEATH => {
            let pMusicFile = Music_GetBaseMusicFile(music, eMusicState)?;
            Some(Music_BuildFileName(
                music,
                &pMusicFile.sFileNameBase,
                eMusicState,
            ))
        }

        MusicState_e::eBGRNDTRACK_ACTIONTRANS0
        | MusicState_e::eBGRNDTRACK_ACTIONTRANS1
        | MusicState_e::eBGRNDTRACK_ACTIONTRANS2
        | MusicState_e::eBGRNDTRACK_ACTIONTRANS3 => {
            let pMusicFile = Music_GetBaseMusicFile(music, MusicState_e::eBGRNDTRACK_ACTION)?;
            let iTransNum = eMusicState as usize - MusicState_e::eBGRNDTRACK_ACTIONTRANS0 as usize;
            let point = pMusicFile.MusicExitPoints.get(iTransNum)?;
            Some(Music_BuildFileName(music, &point.sNextFile, eMusicState))
        }

        MusicState_e::eBGRNDTRACK_EXPLORETRANS0
        | MusicState_e::eBGRNDTRACK_EXPLORETRANS1
        | MusicState_e::eBGRNDTRACK_EXPLORETRANS2
        | MusicState_e::eBGRNDTRACK_EXPLORETRANS3 => {
            let pMusicFile = Music_GetBaseMusicFile(music, MusicState_e::eBGRNDTRACK_EXPLORE)?;
            let iTransNum = eMusicState as usize - MusicState_e::eBGRNDTRACK_EXPLORETRANS0 as usize;
            let point = pMusicFile.MusicExitPoints.get(iTransNum)?;
            Some(Music_BuildFileName(music, &point.sNextFile, eMusicState))
        }

        // Raven's default arm is a `!FINAL_BUILD` assert and print.
        _ => None,
    }
}

/// Raven `Music_StateIsTransition` — is this state one of the eight transitions?
///
/// Source: `oracle/codemp/client/snd_music.cpp:876-881`
pub fn Music_StateIsTransition(eMusicState: MusicState_e) -> bool {
    eMusicState as c_int >= MusicState_e::eBGRNDTRACK_ACTIONTRANS0 as c_int
        && eMusicState as c_int <= MusicState_e::eBGRNDTRACK_EXPLORETRANS3 as c_int
}

/// Raven `Music_StateCanBeInterrupted` — may the proposed state take over now?
///
/// Death interrupts anything and nothing interrupts it. Boss interrupts anything
/// but death, and only silence or death interrupts it. Action interrupts
/// anything left, and nothing interrupts a transition.
/// Source: `oracle/codemp/client/snd_music.cpp:884-928`
pub fn Music_StateCanBeInterrupted(
    eMusicState: MusicState_e,
    eProposedMusicState: MusicState_e,
) -> bool {
    if eProposedMusicState == MusicState_e::eBGRNDTRACK_DEATH {
        return true;
    }
    if eMusicState == MusicState_e::eBGRNDTRACK_DEATH {
        return false;
    }

    if eProposedMusicState == MusicState_e::eBGRNDTRACK_BOSS {
        return true;
    }
    if eMusicState == MusicState_e::eBGRNDTRACK_BOSS {
        // ...except by silence (or death, but that is handled above)
        return eProposedMusicState == MusicState_e::eBGRNDTRACK_SILENCE;
    }

    if eProposedMusicState == MusicState_e::eBGRNDTRACK_ACTION {
        return true;
    }

    if Music_StateIsTransition(eMusicState) {
        return false;
    }

    true
}

/// Raven `Music_AllowedToTransition` — has the play position reached an exit
/// point of the state being queried?
///
/// Answers the transition track to switch to and the entry time of the track
/// after it. Raven searches the sorted exit-time array with `equal_range` and
/// then widens the range by one either way, so a point missed by less than the
/// epsilon is still caught.
/// Source: `oracle/codemp/client/snd_music.cpp:945-1091`
pub fn Music_AllowedToTransition(
    view: &mut EngineHostView,
    music: &MusicData_t,
    fPlayingTimeElapsed: f32,
    eMusicState: MusicState_e,
) -> Option<(MusicState_e, f32)> {
    // Raven: arbitrary, how close we have to be to an exit point to take it. Too
    // high and the music change is sloppy, too low and a poor frame rate misses
    // the exit.
    const F_TIME_EPSILON: f32 = 0.3;

    let pMusicFile = Music_GetBaseMusicFile(music, eMusicState)?;
    if pMusicFile.MusicExitTimes.is_empty() {
        return None;
    }

    // `equal_range` over a time-sorted array, widened by one on each side.
    let times = &pMusicFile.MusicExitTimes;
    let mut first = times.partition_point(|t| t.fTime < fPlayingTimeElapsed);
    let mut last = times.partition_point(|t| t.fTime <= fPlayingTimeElapsed);
    if first != 0 {
        first -= 1;
    }
    if last != times.len() {
        last += 1;
    }

    for pExitTime in &times[first..last] {
        if (pExitTime.fTime - fPlayingTimeElapsed).abs() > F_TIME_EPSILON {
            continue;
        }

        // Got an exit point, so work out the feedback parameters.
        let iExitPoint = pExitTime.iExitPoint;

        // Check legality in case of crap data.
        let Some(ExitPoint) = pMusicFile.MusicExitPoints.get(iExitPoint.max(0) as usize) else {
            // Raven's arm is a `!FINAL_BUILD` assert and print.
            return None;
        };

        match eMusicState {
            MusicState_e::eBGRNDTRACK_EXPLORE => {
                // Explore transitions go to silence, so there is no entry time.
                let eFeedBackTransition = transition_state(
                    MusicState_e::eBGRNDTRACK_EXPLORETRANS0 as c_int + iExitPoint,
                );
                return Some((eFeedBackTransition, 0.0));
            }

            MusicState_e::eBGRNDTRACK_ACTION => {
                if ExitPoint.sNextMark.is_empty() {
                    return Some((MusicState_e::eBGRNDTRACK_ACTIONTRANS0, 0.0));
                }

                // Find the explore piece, then the marker time inside it.
                let explore = music
                    .MusicData
                    .as_ref()
                    .and_then(|data| data.get(sKEY_EXPLORE));
                let Some(MusicFile_Explore) = explore else {
                    let base = &pMusicFile.sFileNameBase;
                    com_printf(view.common, &format!(
                        "^1Music_AllowedToTransition() unable to find {sKEY_EXPLORE} version of \"{base}\"\n"
                    ));
                    return None;
                };
                let Some(&fEntryTime) = MusicFile_Explore.MusicEntryTimes.get(&ExitPoint.sNextMark)
                else {
                    let mark = &ExitPoint.sNextMark;
                    let base = &MusicFile_Explore.sFileNameBase;
                    com_printf(view.common, &format!(
                        "^1Music_AllowedToTransition() unable to find entry marker \"{mark}\" in \"{base}\""
                    ));
                    return None;
                };
                let eFeedBackTransition = transition_state(
                    MusicState_e::eBGRNDTRACK_ACTIONTRANS0 as c_int + iExitPoint,
                );
                return Some((eFeedBackTransition, fEntryTime));
            }

            _ => {
                // Raven's arm is a `!FINAL_BUILD` assert and print.
                return None;
            }
        }
    }

    None
}

/// Raven's `(MusicState_e)(eBGRNDTRACK_xTRANS0 + iExitPoint)` cast.
///
/// The exit-point count is capped at four during the parse, so the sum always
/// lands inside the transition block. A value outside it reads as the first
/// explore transition, Raven's own default.
fn transition_state(value: c_int) -> MusicState_e {
    match value {
        4 => MusicState_e::eBGRNDTRACK_ACTIONTRANS0,
        5 => MusicState_e::eBGRNDTRACK_ACTIONTRANS1,
        6 => MusicState_e::eBGRNDTRACK_ACTIONTRANS2,
        7 => MusicState_e::eBGRNDTRACK_ACTIONTRANS3,
        8 => MusicState_e::eBGRNDTRACK_EXPLORETRANS0,
        9 => MusicState_e::eBGRNDTRACK_EXPLORETRANS1,
        10 => MusicState_e::eBGRNDTRACK_EXPLORETRANS2,
        11 => MusicState_e::eBGRNDTRACK_EXPLORETRANS3,
        _ => MusicState_e::eBGRNDTRACK_EXPLORETRANS0,
    }
}

/// Raven `Music_GetRandomEntryTime` — a predefined random entry point, usually
/// for the action piece.
///
/// Raven calls Quake's generator poor, so it adds a call counter and refuses to
/// answer the same entry twice running. Defaults to 0 where there is no info.
/// Source: `oracle/codemp/client/snd_music.cpp:1097-1136`
pub fn Music_GetRandomEntryTime(
    music: &mut MusicData_t,
    qrand: &mut QRand,
    eMusicState: MusicState_e,
) -> f32 {
    let Some(key) = Music_BaseStateToString(eMusicState, false) else {
        return 0.0;
    };
    let Some(MusicFile) = music.MusicData.as_ref().and_then(|data| data.get(key)) else {
        return 0.0;
    };
    // Make sure at least one is defined, else default to the start.
    let count = MusicFile.MusicEntryTimes.len();
    if count == 0 {
        return 0.0;
    }

    // The draw happens here and nowhere earlier: a state with no entry times
    // never touches the generator.
    music.iCallCount += 1;
    let mut iRandomEntryNum = (qrand.rand() + music.iCallCount) as usize % count;
    if iRandomEntryNum == music.iPrevRandomNumber.max(0) as usize
        && music.iPrevRandomNumber >= 0
        && count > 1
    {
        iRandomEntryNum += 1;
        iRandomEntryNum %= count;
    }
    music.iPrevRandomNumber = iRandomEntryNum as c_int;

    MusicFile
        .MusicEntryTimes
        .values()
        .nth(iRandomEntryNum)
        .copied()
        .unwrap_or(0.0)
}

/// Raven `Music_GetLevelSetName` — the set name the `soundinfo` command prints.
///
/// A remapped level shows both names.
/// Source: `oracle/codemp/client/snd_music.cpp:1140-1150`
pub fn Music_GetLevelSetName(music: &MusicData_t) -> String {
    if Q_stricmp(&music.gsLevelNameForCompare, &music.gsLevelNameForLoad) != 0 {
        let from = &music.gsLevelNameForCompare;
        let to = &music.gsLevelNameForLoad;
        return format!("{from} -> {to}");
    }

    music.gsLevelNameForLoad.clone()
}

/// Raven `COM_SkipPath` over an owned name — everything past the last separator.
///
/// Source: `oracle/codemp/game/q_shared.c:96-110`
fn COM_SkipPath(pathname: &str) -> &str {
    match pathname.rfind('/') {
        Some(i) => &pathname[i + 1..],
        None => pathname,
    }
}

/// C's `atof` over a leading decimal prefix, the way `dms.dat` values are read.
fn atof(text: &str) -> f64 {
    let text = text.trim_start();
    let end = text
        .find(|c: char| !c.is_ascii_digit() && c != '-' && c != '+' && c != '.')
        .unwrap_or(text.len());
    text[..end].parse().unwrap_or(0.0)
}
