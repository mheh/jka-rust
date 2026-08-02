//! `snd_mem.cpp` — WAV parsing, resampling, and the sound-file loader.
//!
//! The loader adjusts the name for the language packs and for a WAV/MP3
//! substitution, reads the file, and resamples it to the `dma.speed` rate the
//! mixer runs at. MP3 content is gh#25 under DEC-57.3, so the MP3 arm of the
//! loader is a loud stub and the WAV arm carries every gh#24 path.
//!
//! Source: `oracle/codemp/client/snd_mem.cpp`

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_engine_qcommon::common::common::com_printf;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::files_common::{FS_FCloseFile, FS_FOpenFileRead, FS_ReadFileVec};
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::fileHandle_t;
use mp_qshared::shared::MAX_QPATH;
use native_string::q_string::Q_stricmpn;

use crate::snd::sound_compression_method_t::SoundCompressionMethod_t;
use crate::snd::sound_system::SoundSystem;
use crate::snd::wavinfo_t::wavinfo_t;
use crate::snd_dma::{SND_malloc, SND_TouchSFX};

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
/// Source: `oracle/codemp/client/snd_mem.cpp:108-176`
pub fn GetWavinfo(view: &mut EngineHostView, name: &str, wav: &[u8]) -> wavinfo_t {
    let mut info = wavinfo_t::default();

    if wav.is_empty() {
        return info;
    }

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

        snd.s_knownSfx[sfx].pSoundData[i] = iSample as i16;

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
fn COM_DefaultExtension_str(path: &mut String, extension: &str) {
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

//TODO: Port S_LoadSound_DirIsAllowedToKeepMP3s
// Source: oracle/codemp/client/snd_mem.cpp:692. Only the MP3 arm calls it, and
// that arm is gh#25 under DEC-57.3.

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
        //TODO: Port S_LoadSound_Actual MP3 arm
        // Source: oracle/codemp/client/snd_mem.cpp:766-863. The decoder is gh#25
        // under DEC-57.3, so no gh#24 path may reach an .mp3 name.
        todo!("Port S_LoadSound_Actual MP3 arm — oracle/codemp/client/snd_mem.cpp:766 (gh#25)")
    }

    // loading a WAV, presumably...
    let info = GetWavinfo(view, &sLoadName, &data);
    if info.channels != 1 {
        com_printf(view.common, &format!("{sLoadName} is a stereo wav file\n"));
        return false;
    }

    snd.s_knownSfx[sfx].eSoundCompressionMethod = SoundCompressionMethod_t::ct_16;
    snd.s_knownSfx[sfx].iSoundLengthInSamples = info.samples;
    snd.s_knownSfx[sfx].pSoundData = Vec::new();
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
