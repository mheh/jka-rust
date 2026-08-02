//! `snd_mem.cpp` — WAV parsing, resampling, and the sound-file loader.
//!
//! The loader adjusts the name for the language packs and for a WAV/MP3
//! substitution, reads the file, and resamples it to the `dma.speed` rate the
//! mixer runs at. A voice line that is worth keeping compressed stays MP3 and
//! streams; every other MP3 unpacks and takes the WAV path.
//!
//! Source: `oracle/codemp/client/snd_mem.cpp`

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_engine_qcommon::cmd_common::{Cmd_Argc, Cmd_Argv};
use mp_engine_qcommon::common::common::com_printf;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::common_fns::Com_DPrintf;
use mp_engine_qcommon::cvar_fns::Cvar_Get;
use mp_engine_qcommon::files_common::{
    FS_FCloseFile, FS_FOpenFileRead, FS_FOpenFileWrite, FS_ListFiles, FS_ReadFileVec, FS_Write,
};
use mp_qshared::shared::cvar::CVAR_ARCHIVE;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::fileHandle_t;
use mp_qshared::shared::MAX_QPATH;
use native_string::q_string::Q_stricmpn;

use crate::client_host::snd_from_view;
use crate::snd::sfx_sample_data::SfxSampleData;
use crate::snd::sound_compression_method_t::SoundCompressionMethod_t;
use crate::snd::sound_system::SoundSystem;
use crate::snd::wavinfo_t::wavinfo_t;
use crate::snd_dma::{
    S_FindName, S_StopAllSounds, SND_FreeOldestSound, SND_malloc, SND_TouchSFX,
};
use crate::snd_mp3::{
    MP3Stream_InitFromFile, MP3_FakeUpWAVInfo, MP3_GetUnpackedSize, MP3_IsValid,
    MP3_ReadSpecialTagInfo, MP3_UnpackRawPCM, ID3V1_BYTES, sKEY_MAXVOL, sKEY_UNCOMP,
};

/// Raven's IFF chunk cursor — the `data_p`/`iff_end`/`last_chunk`/`iff_data`
/// file-scope globals that `GetWavinfo` walks a WAV header with.
///
/// Every position is an offset into the loaded file, and `data_p` is `None`
/// where Raven sets the pointer to NULL.
/// Source: `oracle/codemp/client/snd_mem.cpp:21-24`
struct IffCursor<'a> {
    wav: &'a [u8],
    data_p: Option<usize>,
    iff_end: usize,
    last_chunk: usize,
    iff_data: usize,
    iff_chunk_len: c_int,
}

impl<'a> IffCursor<'a> {
    fn new(wav: &'a [u8]) -> IffCursor<'a> {
        IffCursor {
            wav,
            data_p: None,
            iff_end: wav.len(),
            last_chunk: 0,
            iff_data: 0,
            iff_chunk_len: 0,
        }
    }

    /// Raven `GetLittleShort`.
    /// Source: `oracle/codemp/client/snd_mem.cpp:34-41`
    fn GetLittleShort(&mut self) -> i16 {
        let p = self.data_p.expect("GetLittleShort past the chunk walker");
        let val = c_int::from(self.wav[p]) + (c_int::from(self.wav[p + 1]) << 8);
        self.data_p = Some(p + 2);
        val as i16
    }

    /// Raven `GetLittleLong`.
    /// Source: `oracle/codemp/client/snd_mem.cpp:43-52`
    fn GetLittleLong(&mut self) -> c_int {
        let p = self.data_p.expect("GetLittleLong past the chunk walker");
        let val = c_int::from(self.wav[p])
            + (c_int::from(self.wav[p + 1]) << 8)
            + (c_int::from(self.wav[p + 2]) << 16)
            + ((c_int::from(self.wav[p + 3]) << 24) as u32 as c_int);
        self.data_p = Some(p + 4);
        val
    }

    /// Raven `FindNextChunk` — walk forward until the four-letter name matches.
    ///
    /// Raven reads the eight-byte chunk header without checking that it fits, so
    /// a truncated file reads past the buffer. The port stops instead, which is
    /// the one defined behavior for that read (porting-rules §19).
    /// Source: `oracle/codemp/client/snd_mem.cpp:54-78`
    fn FindNextChunk(&mut self, name: &[u8; 4]) {
        loop {
            self.data_p = Some(self.last_chunk);

            let p = self.last_chunk;
            if p >= self.iff_end || p + 8 > self.wav.len() {
                self.data_p = None;
                return;
            }

            self.data_p = Some(p + 4);
            self.iff_chunk_len = self.GetLittleLong();
            if self.iff_chunk_len < 0 {
                self.data_p = None;
                return;
            }
            self.data_p = Some(p);
            self.last_chunk = p + 8 + ((self.iff_chunk_len as usize + 1) & !1);
            if &self.wav[p..p + 4] == name {
                return;
            }
        }
    }

    /// Raven `FindChunk` — restart the walk at the current chunk root.
    /// Source: `oracle/codemp/client/snd_mem.cpp:80-84`
    fn FindChunk(&mut self, name: &[u8; 4]) {
        self.last_chunk = self.iff_data;
        self.FindNextChunk(name);
    }
}

/// Raven `GetWavinfo` — read format, rate, width, and the data offset out of a
/// WAV header.
///
/// A header the walker cannot follow leaves the returned info all zero, and the
/// caller treats a zero `channels` as a load failure.
///
/// Raven's `if (!wav)` guard is a null-pointer test. A slice cannot be null, and
/// the one caller already returns before a missing file reaches here, so a
/// zero-byte file walks the chunk reader and prints the way Raven does.
/// Source: `oracle/codemp/client/snd_mem.cpp:108-176`
pub fn GetWavinfo(view: &mut EngineHostView, name: &str, wav: &[u8]) -> wavinfo_t {
    let mut info = wavinfo_t::default();

    let mut cursor = IffCursor::new(wav);

    // find "RIFF" chunk
    cursor.FindChunk(b"RIFF");
    let riff_is_wave = match cursor.data_p {
        Some(p) => p + 12 <= wav.len() && &wav[p + 8..p + 12] == b"WAVE",
        None => false,
    };
    if !riff_is_wave {
        com_printf(view.common, "Missing RIFF/WAVE chunks\n");
        return info;
    }

    // get "fmt " chunk
    cursor.iff_data = cursor.data_p.expect("RIFF chunk found") + 12;

    cursor.FindChunk(b"fmt ");
    if cursor.data_p.is_none() {
        com_printf(view.common, "Missing fmt chunk\n");
        return info;
    }
    cursor.data_p = Some(cursor.data_p.expect("fmt chunk found") + 8);
    info.format = c_int::from(cursor.GetLittleShort());
    info.channels = c_int::from(cursor.GetLittleShort());
    info.rate = cursor.GetLittleLong();
    cursor.data_p = Some(cursor.data_p.expect("fmt chunk body") + 4 + 2);
    info.width = c_int::from(cursor.GetLittleShort()) / 8;

    if info.format != 1 {
        com_printf(view.common, "Microsoft PCM format only\n");
        return info;
    }

    // find data chunk
    cursor.FindChunk(b"data");
    if cursor.data_p.is_none() {
        com_printf(view.common, "Missing data chunk\n");
        return info;
    }

    cursor.data_p = Some(cursor.data_p.expect("data chunk found") + 4);
    let samples = cursor.GetLittleLong() / info.width;

    if info.samples != 0 {
        if samples < info.samples {
            com_error(
                errorParm_t::ERR_DROP,
                format!("Sound {name} has a bad loop length"),
            );
        }
    } else {
        info.samples = samples;
    }

    info.dataofs = cursor.data_p.expect("data chunk body") as c_int;

    info
}

/// Raven `ResampleSfx` — resample or decimate the source block to `dma.speed`
/// and record the peak the lip-sync code reads.
///
/// Source: `oracle/codemp/client/snd_mem.cpp:186-229`
pub fn ResampleSfx(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    sfx: usize,
    iInRate: c_int,
    iInWidth: c_int,
    pData: &[u8],
) {
    let fStepScale = iInRate as f32 / snd.dma.speed as f32;

    // When stepscale is > 1 (we're downsampling), we really ought to run a low pass filter on the samples

    let iOutCount = (snd.s_knownSfx[sfx].iSoundLengthInSamples as f32 / fStepScale) as c_int;
    snd.s_knownSfx[sfx].iSoundLengthInSamples = iOutCount;

    SND_malloc(view, snd, iOutCount * 2, sfx);

    snd.s_knownSfx[sfx].fVolRange = 0.0;
    let mut uiSampleFrac: u32 = 0;
    let uiFracStep = (fStepScale * 256.0) as c_int as u32;

    for i in 0..iOutCount as usize {
        let iSrcSample = (uiSampleFrac >> 8) as usize;
        uiSampleFrac = uiSampleFrac.wrapping_add(uiFracStep);
        let mut iSample = if iInWidth == 2 {
            c_int::from(i16::from_le_bytes([
                pData[iSrcSample * 2],
                pData[iSrcSample * 2 + 1],
            ]))
        } else {
            (c_int::from(pData[iSrcSample]) - 128) << 8
        };

        snd.s_knownSfx[sfx]
            .pSoundData
            .as_mut()
            .and_then(SfxSampleData::pcm_mut)
            .expect("SND_malloc seated the sample block")[i] = iSample as i16;

        // work out max vol for this sample...
        if iSample < 0 {
            iSample = -iSample;
        }
        if snd.s_knownSfx[sfx].fVolRange < (iSample >> 8) as f32 {
            snd.s_knownSfx[sfx].fVolRange = (iSample >> 8) as f32;
        }
    }
}

/// Raven `COM_DefaultExtension` over an owned name.
///
/// The scan walks back to the first `/` and stops on a `.`, so a directory with
/// a dot in it never counts as an extension.
/// Source: `oracle/codemp/game/q_shared.c:112-131`
pub fn COM_DefaultExtension_str(path: &mut String, extension: &str) {
    // Raven's walk stops at index 0 without testing it, so index 0 is skipped here too.
    for &b in path.as_bytes()[1..].iter().rev() {
        if b == b'/' {
            break;
        }
        if b == b'.' {
            return; // it has an extension
        }
    }

    path.push_str(extension);
    path.truncate(MAX_QPATH - 1);
}

/// Raven `S_LoadSound_DirIsAllowedToKeepMP3s` — may a sound in this directory
/// stay compressed?
///
/// Raven: someone had got an ambient sound that was an MP3, then it tried to get
/// added as looping. "sound/ambience" he could check for, but doors and the like
/// could be anything. The name is the original un-languaged one.
/// Source: `oracle/codemp/client/snd_mem.cpp:692-708`
fn S_LoadSound_DirIsAllowedToKeepMP3s(psFilename: &str) -> bool {
    // Raven's `sound/chr_d/` entry and the other languages are commented out,
    // because the comparison always runs against the English name.
    const ALLOWED_DIRS: [&str; 1] = ["sound/chars/"];

    ALLOWED_DIRS
        .iter()
        .any(|dir| Q_stricmpn(psFilename, dir, dir.len()) == 0)
}

/// Raven `S_LoadSound_Finalize` — describe an unpacked block as a WAV and
/// resample it into the sfx slot.
///
/// Raven computes a `len` here that nothing reads.
/// Source: `oracle/codemp/client/snd_mem.cpp:233-245`
fn S_LoadSound_Finalize(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    info: &wavinfo_t,
    sfx: usize,
    data: &[u8],
) {
    snd.s_knownSfx[sfx].eSoundCompressionMethod = SoundCompressionMethod_t::ct_16;
    snd.s_knownSfx[sfx].iSoundLengthInSamples = info.samples;
    let dataofs = info.dataofs.max(0) as usize;
    ResampleSfx(view, snd, sfx, info.rate, info.width, &data[dataofs..]);
}

/// Opens a file only to pull it into the pak cache, then closes it.
/// This is the `com_buildScript` warm-up `S_LoadSound_FileLoadAndNameAdjuster`
/// runs three times, once per language.
/// Source: `oracle/codemp/client/snd_mem.cpp:570-583`
fn cache_foreign_voice(view: &mut EngineHostView, psFilename: &mut String, iNameStrlen: usize) {
    let mut hFile: fileHandle_t = 0;
    FS_FOpenFileRead(view, psFilename, &mut hFile, false);
    if hFile == 0 {
        psFilename.replace_range(iNameStrlen - 3.., "mp3");
        FS_FOpenFileRead(view, psFilename, &mut hFile, false);
    }
    if hFile != 0 {
        FS_FCloseFile(view.common, hFile);
    }
    psFilename.replace_range(iNameStrlen - 3.., "wav");
}

/// Raven `S_LoadSound_FileLoadAndNameAdjuster` — adjust the name for the
/// language packs and for the WAV/MP3 substitution, then read the file.
///
/// The returned name carries the extension the read actually succeeded on, so
/// the caller tests it to decide between the WAV and the MP3 arm.
/// Source: `oracle/codemp/client/snd_mem.cpp:561-680`
fn S_LoadSound_FileLoadAndNameAdjuster(
    view: &mut EngineHostView,
    snd: &SoundSystem,
    psFilename: &mut String,
    iNameStrlen: usize,
) -> Option<Vec<u8>> {
    // The offset of "chars" inside the name, and Raven's flag for "we
    // substituted a foreign version".
    let mut psVoice = psFilename.find("chars");
    if let Some(voice) = psVoice {
        // cache foreign voices...
        if view.common.com_buildScript.is_some()
            && view.common.cvar(view.common.com_buildScript).integer != 0
        {
            for language in ["chr_d", "chr_f", "chr_e"] {
                psFilename.replace_range(voice..voice + 5, language);
                cache_foreign_voice(view, psFilename, iNameStrlen);
            }
            psFilename.replace_range(voice..voice + 5, "chars");
        }

        // account for foreign voices...
        let language = view.common.cvar(snd.s_language).string.clone();
        if language.eq_ignore_ascii_case("DEUTSCH") {
            psFilename.replace_range(voice..voice + 5, "chr_d");
        } else if language.eq_ignore_ascii_case("FRANCAIS") {
            psFilename.replace_range(voice..voice + 5, "chr_f");
        } else if language.eq_ignore_ascii_case("ESPANOL") {
            psFilename.replace_range(voice..voice + 5, "chr_e");
        } else {
            // use this ptr as a flag as to whether or not we substituted with a foreign version
            psVoice = None;
        }
    }

    if let Some(data) = FS_ReadFileVec(view, psFilename) {
        return Some(data);
    }

    psFilename.replace_range(iNameStrlen - 3.., "mp3");
    if let Some(data) = FS_ReadFileVec(view, psFilename) {
        return Some(data);
    }

    // hmmm, not found, ok, maybe we were trying a foreign noise ("arghhhhh.mp3" that doesn't matter?) but it
    // was missing?   Can't tell really, since both types are now in sound/chars. Oh well, fall back to English for now...
    let voice = psVoice?;

    // yep, so fallback to re-try the english...
    // The `S_COLOR_YELLOW` warning above this line is a `!FINAL_BUILD` print, and
    // the retail build drops it (DEC-62.6).
    psFilename.replace_range(voice..voice + 5, "chars");

    psFilename.replace_range(iNameStrlen - 3.., "wav");
    if let Some(data) = FS_ReadFileVec(view, psFilename) {
        return Some(data);
    }

    psFilename.replace_range(iNameStrlen - 3.., "mp3");
    FS_ReadFileVec(view, psFilename)
}

/// Raven `S_LoadSound_Actual` — load one sound file into its `sfx_t` slot.
///
/// Raven allocates a `info.samples * 4` scratch block here and frees it without
/// reading it, so the port drops the block.
/// Source: `oracle/codemp/client/snd_mem.cpp:721-928`
fn S_LoadSound_Actual(view: &mut EngineHostView, snd: &mut SoundSystem, sfx: usize) -> bool {
    let mut len = snd.s_knownSfx[sfx].sSoundName.len();
    if len < 5 {
        return false;
    }

    // player specific sounds are never directly loaded...
    if snd.s_knownSfx[sfx].sSoundName.starts_with('*') {
        return false;
    }

    // make up a local filename to try wav/mp3 substitutes...
    let mut sLoadName = snd.s_knownSfx[sfx].sSoundName.to_ascii_lowercase();
    if sLoadName.len() >= MAX_QPATH {
        sLoadName.truncate(MAX_QPATH - 1);
    }

    // Ensure name has an extension (which it must have, but you never know), and get ptr to it...
    if sLoadName.as_bytes()[sLoadName.len() - 4] != b'.' {
        COM_DefaultExtension_str(&mut sLoadName, ".wav");
        len = sLoadName.len();
    }

    let Some(data) = S_LoadSound_FileLoadAndNameAdjuster(view, snd, &mut sLoadName, len) else {
        return false;
    };

    SND_TouchSFX(view, snd, sfx);

    if Q_stricmpn(&sLoadName[sLoadName.len() - 4..], ".mp3", 4) == 0 {
        return S_LoadSound_Mp3Arm(view, snd, sfx, &sLoadName, &data);
    }

    // loading a WAV, presumably...
    let info = GetWavinfo(view, &sLoadName, &data);
    if info.channels != 1 {
        com_printf(view.common, &format!("{sLoadName} is a stereo wav file\n"));
        return false;
    }

    snd.s_knownSfx[sfx].eSoundCompressionMethod = SoundCompressionMethod_t::ct_16;
    snd.s_knownSfx[sfx].iSoundLengthInSamples = info.samples;
    snd.s_knownSfx[sfx].pSoundData = None;
    ResampleSfx(
        view,
        snd,
        sfx,
        info.rate,
        info.width,
        &data[info.dataofs as usize..],
    );

    true
}

/// The MP3 arm of `S_LoadSound_Actual`.
///
/// A voice line in a directory allowed to keep MP3s, and big enough to be worth
/// it, stays compressed and streams. Everything else unpacks to PCM and takes
/// the normal post-load path, so the lip-sync volume still gets computed.
/// Source: `oracle/codemp/client/snd_mem.cpp:766-863`
fn S_LoadSound_Mp3Arm(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    sfx: usize,
    sLoadName: &str,
    data: &[u8],
) -> bool {
    let size = data.len() as c_int;

    if !MP3_IsValid(view.common, sLoadName, data, size, false) {
        // `MP3_IsValid` has already printed the reason.
        return false;
    }

    let iRawPCMDataSize = MP3_GetUnpackedSize(view.common, sLoadName, data, size, false);

    // NOT `sLoadName`: this test uses the original un-languaged name.
    let keepAsMp3 = S_LoadSound_DirIsAllowedToKeepMP3s(&snd.s_knownSfx[sfx].sSoundName) && {
        let overhead = Cvar_Get(view, "s_mp3overhead", "0", CVAR_ARCHIVE);
        let iMP3Overhead = view.common.cvar(overhead).integer;
        let dmaSpeed = snd.dma.speed;
        MP3Stream_InitFromFile(
            view.common,
            &mut snd.s_knownSfx[sfx],
            data,
            size,
            sLoadName,
            // Raven adds one MP3 frame, just in case.
            iRawPCMDataSize + 2304,
            iMP3Overhead,
            dmaSpeed,
            false,
        )
    };

    if keepAsMp3 {
        // Raven's "keeping file as MP3" note is a `Com_DPrintf` he commented out.
        snd.sndRawDataBytes += size;
        return true;
    }

    // A small file is not worth keeping as MP3, since the stream header would
    // make it bigger.
    Com_DPrintf(
        view.common,
        &format!("S_LoadSound: Unpacking MP3 file({size}) \"{sLoadName}\" to wav({iRawPCMDataSize}).\n"),
    );

    // Raven's scratch block is `iRawPCMDataSize + 10 + 2304`.
    let mut pbUnpackBuffer = vec![0u8; (iRawPCMDataSize.max(0) + 10 + 2304) as usize];
    let iResultBytes =
        MP3_UnpackRawPCM(view.common, sLoadName, data, size, &mut pbUnpackBuffer, false);

    if iResultBytes != iRawPCMDataSize {
        com_printf(
            view.common,
            &format!(
                "^3**** MP3 {sLoadName} final unpack size {iResultBytes} different to previous value {iRawPCMDataSize}\n"
            ),
        );
    }

    // Fake up a WAV structure so the other post-load sound code, such as the
    // lip-sync volume calc, runs unchanged. Raven calls this a bit crap.
    let mut info = wavinfo_t::default();
    MP3_FakeUpWAVInfo(
        view.common,
        sLoadName,
        data,
        size,
        iResultBytes,
        &mut info,
        false,
    );

    S_LoadSound_Finalize(view, snd, &info, sfx, &pbUnpackBuffer);

    true
}

/// The running totals of one `mp3_calcvols` pass.
///
/// Raven keeps these as file-scope globals beside `R_CheckMP3s`, which recurses.
/// Source: `oracle/codemp/client/snd_mem.cpp:290-295`
struct Mp3ScanTotals {
    iFilesFound: c_int,
    iFilesUpdated: c_int,
    iErrors: c_int,
    strErrors: String,
    qbForceRescan: bool,
    qbForceStereo: bool,
    /// Raven's `pSFX` function static: one reserved slot he re-uses for every
    /// file, restoring its name afterwards.
    pSFX: Option<usize>,
}

/// Raven's reserved sfx name for the `mp3_calcvols` scan.
///
/// Source: `oracle/codemp/client/snd_mem.cpp:389`
const sReservedSFXEntrynameForMP3: &str = "reserved_for_mp3";

/// Raven `R_CheckMP3s` — walk one directory and re-tag every MP3 under it.
///
/// The tag carries the peak volume the lip-sync code reads and the uncompressed
/// size the loader sizes its buffer with, so a file with no tag or a stale one
/// gets rewritten.
/// Source: `oracle/codemp/client/snd_mem.cpp:297-491`
fn R_CheckMP3s(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    totals: &mut Mp3ScanTotals,
    psDir: &str,
) {
    // Raven prints a dot so the useful info does not scroll off screen.
    com_printf(view.common, ".");

    let dirFiles = FS_ListFiles(view, psDir, "/");
    // The first two entries are "." and "..".
    for name in dirFiles.iter().skip(2) {
        let sDirName = format!("{psDir}\\{name}");
        R_CheckMP3s(view, snd, totals, &sDirName);
    }

    let sysFiles = FS_ListFiles(view, psDir, ".mp3");
    for (i, name) in sysFiles.iter().enumerate() {
        let sFilename = format!("{psDir}\\{name}");

        let lead = if i == 0 { "\n" } else { "" };
        com_printf(view.common, &format!("{lead}Found file: {sFilename}"));

        totals.iFilesFound += 1;

        let Some(pbData) = FS_ReadFileVec(view, &sFilename) else {
            continue;
        };
        let iSize = pbData.len() as c_int;

        let tag = MP3_ReadSpecialTagInfo(&pbData, iSize);
        let hadTag = tag_present(&pbData, iSize);
        if tag.is_some() && hadTag && !totals.qbForceRescan {
            com_printf(view.common, " ( OK )\n");
            continue;
        }

        com_printf(view.common, " ( Updating )\n");

        // Raven asks for one reserved slot the legal way and re-uses it for
        // every file, restoring its name after each one.
        if totals.pSFX.is_none() {
            totals.pSFX = Some(S_FindName(snd, sReservedSFXEntrynameForMP3));
        }
        let pSFX = totals.pSFX.expect("the reserved slot exists");

        if !MP3_IsValid(view.common, &sFilename, &pbData, iSize, totals.qbForceStereo) {
            com_printf(
                view.common,
                &format!("^1*********** File was not a valid MP3!: \"{sFilename}\"\n"),
            );
            totals.iErrors += 1;
            totals.strErrors += &format!("Not game-legal MP3 format: \"{sFilename}\"\n");
            continue;
        }

        let iRawPCMDataSize = MP3_GetUnpackedSize(
            view.common,
            &sFilename,
            &pbData,
            iSize,
            totals.qbForceStereo,
        );
        if iRawPCMDataSize == 0 {
            // Should only happen where the file is broken.
            com_error(
                errorParm_t::ERR_DROP,
                format!("******* This MP3 should be deleted: \"{sFilename}\"\n"),
            );
        }

        let mut fMaxVol = 128.0f32; // any old default
        let mut iActualUnpackedSize = iRawPCMDataSize;

        // No point for a stereo file, which is music and therefore has no lip sync.
        if !totals.qbForceStereo {
            let mut pbUnpackBuffer = vec![0u8; (iRawPCMDataSize + 10) as usize];
            iActualUnpackedSize = MP3_UnpackRawPCM(
                view.common,
                &sFilename,
                &pbData,
                iSize,
                &mut pbUnpackBuffer,
                false,
            );
            if iActualUnpackedSize != iRawPCMDataSize {
                com_error(errorParm_t::ERR_DROP, format!(
                    "******* Whoah! MP3 {sFilename} unpacked to {iActualUnpackedSize} bytes, but size calc said {iRawPCMDataSize}!\n"
                ));
            }

            let mut info = wavinfo_t::default();
            MP3_FakeUpWAVInfo(
                view.common,
                &sFilename,
                &pbData,
                iSize,
                iActualUnpackedSize,
                &mut info,
                false,
            );

            // All this just for lipsynch. Oh well.
            S_LoadSound_Finalize(view, snd, &info, pSFX, &pbUnpackBuffer);

            fMaxVol = snd.s_knownSfx[pSFX].fVolRange;

            // Force this to be the oldest sound file, so it can be disposed of.
            snd.s_knownSfx[pSFX].iLastTimeUsed = c_int::MIN;
            snd.s_knownSfx[pSFX].bInMemory = true;
            SND_FreeOldestSound(view, snd, None);

            // Put the reserved slot back to its default name so nothing else
            // accidentally uses it.
            snd.s_knownSfx[pSFX].sSoundName = sReservedSFXEntrynameForMP3.to_string();
            snd.s_knownSfx[pSFX].bDefaultSound = false;
        }

        // Time to update the file now.
        let f: fileHandle_t = FS_FOpenFileWrite(view.common, &sFilename);
        if f == 0 {
            com_printf(
                view.common,
                &format!("^1*********** Failed to re-open for write \"{sFilename}\"!\n"),
            );
            totals.iErrors += 1;
            totals.strErrors += &format!("Failed to re-open for write: \"{sFilename}\"\n");
            continue;
        }

        // Write the file back out, omitting the tag if there was one.
        let keep = iSize - if hadTag { ID3V1_BYTES as c_int } else { 0 };
        let iWritten = FS_Write(view.common, pbData.as_ptr() as *const (), keep, f);

        if iWritten != 0 {
            let title = Filename_WithoutPath(Filename_WithoutExt(&sFilename));
            let tagBytes = build_id3v1_tag(&title, iActualUnpackedSize, fMaxVol);
            if FS_Write(
                view.common,
                tagBytes.as_ptr() as *const (),
                ID3V1_BYTES as c_int,
                f,
            ) != 0
            {
                totals.iFilesUpdated += 1;
            } else {
                com_printf(
                    view.common,
                    &format!("^1*********** Failed write to file \"{sFilename}\"!\n"),
                );
                totals.iErrors += 1;
                totals.strErrors += &format!("Failed to write: \"{sFilename}\"\n");
            }
        } else {
            com_printf(
                view.common,
                &format!("^1*********** Failed write to file \"{sFilename}\"!\n"),
            );
            totals.iErrors += 1;
            totals.strErrors += &format!("Failed to write: \"{sFilename}\"\n");
        }

        FS_FCloseFile(view.common, f);
    }
}

/// Does this file end with an ID3v1 tag?
///
/// Raven reads the answer out of `MP3_ReadSpecialTagInfo`'s `ppTAG` out-param,
/// which the port drops (porting-rules §20), so the one caller that needs it
/// tests the marker itself.
fn tag_present(pbData: &[u8], iSize: c_int) -> bool {
    let len = (iSize.max(0) as usize).min(pbData.len());
    len >= ID3V1_BYTES && &pbData[len - ID3V1_BYTES..len - ID3V1_BYTES + 3] == b"TAG"
}

/// Build the 128-byte ID3v1 tag `mp3_calcvols` writes.
///
/// Source: `oracle/codemp/client/snd_mem.cpp:436-443`
fn build_id3v1_tag(title: &str, iUncompressedSize: c_int, fMaxVol: f32) -> Vec<u8> {
    fn field(out: &mut Vec<u8>, text: &str, size: usize) {
        let bytes = text.as_bytes();
        let take = bytes.len().min(size);
        out.extend_from_slice(&bytes[..take]);
        out.extend(core::iter::repeat(0u8).take(size - take));
    }

    let mut tag = Vec::with_capacity(ID3V1_BYTES);
    tag.extend_from_slice(b"TAG");
    field(&mut tag, title, 30);
    field(&mut tag, "Raven Software", 30);
    field(&mut tag, &format!("{sKEY_UNCOMP} {iUncompressedSize}"), 30);
    field(&mut tag, "2002", 4);
    field(&mut tag, &format!("{sKEY_MAXVOL} {}", fmt_g(fMaxVol)), 28);
    tag.extend_from_slice(&[0, 0, 0]);
    tag
}

/// C's `%g` over a float, which is what the tag carries.
fn fmt_g(value: f32) -> String {
    let text = format!("{value:.6}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text.is_empty() {
        "0".to_string()
    } else {
        text.to_string()
    }
}

/// Raven `Filename_WithoutPath` — everything past the last separator.
///
/// Source: `oracle/codemp/client/snd_mem.cpp:251-265`
fn Filename_WithoutPath(psFilename: String) -> String {
    match psFilename.rfind('\\') {
        Some(i) => psFilename[i + 1..].to_string(),
        None => psFilename,
    }
}

/// Raven `Filename_WithoutExt` — the name with its extension cut off.
///
/// Raven checks that the last dot is past the last separator, so a directory
/// with a dot in it is not mistaken for an extension.
/// Source: `oracle/codemp/client/snd_mem.cpp:269-286`
fn Filename_WithoutExt(psFilename: &str) -> String {
    let dot = psFilename.rfind('.');
    let sep = psFilename.rfind('\\');
    match (dot, sep) {
        (Some(p), None) => psFilename[..p].to_string(),
        (Some(p), Some(p2)) if p > p2 => psFilename[..p].to_string(),
        _ => psFilename.to_string(),
    }
}

/// Raven `S_MP3_CalcVols_f` — the `mp3_calcvols` development command.
///
/// It makes sure every `sound/*.mp3` carries a tag naming its peak volume and
/// its uncompressed size. The command lives in `snd_dma`'s command table, and
/// the body sits here beside the scan it drives.
/// Source: `oracle/codemp/client/snd_mem.cpp:495-545`
pub fn S_MP3_CalcVols_f_body(view: &mut EngineHostView) {
    const S_USAGE: &str = "Usage: mp3_calcvols [-rescan] <startdir>\ne.g. mp3_calcvols sound/chars";

    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let snd = unsafe { snd_from_view(view) };

    let argc = Cmd_Argc(view.common);
    if argc == 1 || argc > 4 {
        com_printf(view.common, S_USAGE);
        return;
    }

    S_StopAllSounds(view.common, snd);

    let mut totals = Mp3ScanTotals {
        iFilesFound: 0,
        iFilesUpdated: 0,
        iErrors: 0,
        strErrors: String::new(),
        qbForceRescan: false,
        qbForceStereo: false,
        pSFX: None,
    };

    let mut sStartDir = "sound".to_string();
    for i in 1..argc {
        let arg = Cmd_Argv(view.common, i).to_string();
        if arg.starts_with('-') {
            if arg.eq_ignore_ascii_case("-rescan") {
                totals.qbForceRescan = true;
            } else if arg.eq_ignore_ascii_case("-stereo") {
                totals.qbForceStereo = true;
            } else {
                com_printf(view.common, S_USAGE);
                return;
            }
            continue;
        }
        sStartDir = arg;
    }

    com_printf(
        view.common,
        &format!("Starting Scan for Updates in Dir: {sStartDir}\n"),
    );
    R_CheckMP3s(view, snd, &mut totals, &sStartDir);

    let (found, updated, errors) = (totals.iFilesFound, totals.iFilesUpdated, totals.iErrors);
    com_printf(view.common, &format!(
        "\n{found} files found/scanned, {updated} files updated      ( {errors} errors total)\n"
    ));

    if errors != 0 {
        let list = totals.strErrors;
        com_printf(view.common, &format!("\nBad Files:\n{list}\n"));
    }
}

/// Raven `S_LoadSound` — the wrapper that flags "a load is running", so a
/// memory-recovery pass never dumps audio out from under it.
///
/// Source: `oracle/codemp/client/snd_mem.cpp:934-943`
pub fn S_LoadSound(view: &mut EngineHostView, snd: &mut SoundSystem, sfx: usize) -> bool {
    snd.gbInsideLoadSound = true;

    let bReturn = S_LoadSound_Actual(view, snd, sfx);

    snd.gbInsideLoadSound = false;

    bReturn
}
