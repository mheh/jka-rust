//! `snd_mp3.cpp` — the interface between the MP3 decoder and the game.
//!
//! DEC-03 and DEC-57.3 replace Raven's `codemp/mp3code` C decoder, so this file
//! ports the wrap logic only: validation, the unpacked-size walk, the ID3v1 tag
//! reader, the sliding-window sample feeder, and the playing-time queries. The
//! decoder itself sits behind `mp3::mp3_decoder::Mp3Decoder`, and the layer
//! between them is `mp3::mp3_stream_state::MP3StreamState`.
//!
//! The decoder is outside the byte gate, so the parity rig takes MP3 content as
//! decoded PCM fixtures and the decoder gets pinned decode fixtures of its own
//! (`tools/snd-oracle/gen_mp3_fixtures.py`).
//!
//! Source: `oracle/codemp/client/snd_mp3.cpp`,
//! `oracle/codemp/mp3code/towave.c`

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_engine_qcommon::common::common::com_printf;
use mp_engine_qcommon::common::Common;

use crate::mp3::mp3_frame_header::Mp3FrameHeader;
use crate::mp3::mp3_stream_state::MP3StreamState;
use crate::snd::channel_mp3_state::ChannelMp3State;
use crate::snd::sfx_s::sfx_t;
use crate::snd::sfx_sample_data::SfxSampleData;
use crate::snd::sound_compression_method_t::SoundCompressionMethod_t;
use crate::snd::wavinfo_t::wavinfo_t;

/// Raven `sKEY_MAXVOL` / `sKEY_UNCOMP` — the two keys the tagger writes into an
/// ID3v1 comment and album field.
///
/// Source: `oracle/codemp/client/snd_mp3.cpp:158-159`
pub const sKEY_MAXVOL: &str = "#MAXVOL";
pub const sKEY_UNCOMP: &str = "#UNCOMP";

/// The ID3v1 tag is 128 bytes at the end of the file.
///
/// Source: `oracle/codemp/client/snd_mp3.h:15-25`
pub const ID3V1_BYTES: usize = 128;

/// Raven's `C_MP3_IsValid` rejects a stereo source only past this file size.
///
/// Raven: we'll allow it for small files even if stereo.
/// Source: `oracle/codemp/mp3code/towave.c:274`
const STEREO_REJECT_BYTES: c_int = 98000;

/// The largest packet the mixer accepts, mono and stereo.
///
/// Source: `oracle/codemp/mp3code/towave.c:284-297`
const MAX_PACKET_BYTES_MONO: usize = 2304;
const MAX_PACKET_BYTES_STEREO: usize = 4608;

// ===========================================================================
// The decoder-facing half, Raven's `C_MP3_*` entry points
// ===========================================================================

/// Raven `C_MP3_IsValid` — the header checks that keep a bad file out of the mixer.
///
/// Answers Raven's error string, or `None` for a file the mixer accepts.
/// Source: `oracle/codemp/mp3code/towave.c:248-320`
fn C_MP3_IsValid(pvData: &[u8], iDataLen: c_int, bStereoDesired: bool) -> Option<&'static str> {
    let scanLimit = (iDataLen.max(0) / 2) as usize;
    let Some((_, head)) = Mp3FrameHeader::find(pvData, scanLimit) else {
        return Some("MP3ERR: Bad or unsupported file!");
    };

    // Although the decoder can convert stereo to mono, a stereo source is a
    // waste of space for an effect: every effect is mono and moved by panning.
    if head.channels != 1 && !bStereoDesired && iDataLen > STEREO_REJECT_BYTES {
        return Some("MP3ERR: Sound file is stereo!");
    }

    let outChannels = if bStereoDesired {
        head.channels as usize
    } else {
        1
    };
    let outbytes = head.samplesPerFrame * outChannels * 2;
    if bStereoDesired {
        if outbytes > MAX_PACKET_BYTES_STEREO {
            return Some("MP3ERR: Source file has output packet size > 2304 (*2 for stereo) bytes!");
        }
    } else if outbytes > MAX_PACKET_BYTES_MONO {
        return Some("MP3ERR: Source file has output packet size > 2304 bytes!");
    }

    if head.sampleRate != 44100 {
        return Some("MP3ERR: Source file is not sampled @ 44100!");
    }

    if bStereoDesired && head.channels != 2 {
        return Some("MP3ERR: Source file is not stereo!");
    }

    None
}

/// Raven `C_MP3_GetHeaderData` — the rate, the sample width, and the channel
/// count the decoder answers for this file.
///
/// Source: `oracle/codemp/mp3code/towave.c:329-360`
fn C_MP3_GetHeaderData(
    pvData: &[u8],
    iDataLen: c_int,
    bStereoDesired: bool,
) -> Result<(c_int, c_int, c_int), &'static str> {
    let scanLimit = (iDataLen.max(0) / 2) as usize;
    let Some((_, head)) = Mp3FrameHeader::find(pvData, scanLimit) else {
        return Err("MP3ERR: Bad or unsupported file!");
    };
    // The mono conversion is inside the decoder, so it answers one channel.
    let channels = if bStereoDesired { head.channels } else { 1 };
    Ok((head.sampleRate, 2, channels))
}

/// Walk the whole stream frame by frame and answer the frame count plus the
/// packet shape, without decoding anything.
///
/// This is Raven's `bFastEstimateOnly` pass: the packet size is a property of
/// the header, so counting frames is enough to size the unpacked output.
/// Source: `oracle/codemp/mp3code/towave.c:374-470`
fn walk_frames(
    pvData: &[u8],
    iSourceBytesRemaining: c_int,
    bStereoDesired: bool,
) -> Option<(usize, usize)> {
    let scanLimit = (iSourceBytesRemaining.max(0) / 2) as usize;
    let (start, head) = Mp3FrameHeader::find(pvData, scanLimit)?;

    let mut remaining = trim_rear_tag(pvData, iSourceBytesRemaining) - start as c_int;
    let mut read = start;
    let mut frames = 0usize;

    while remaining > 0 && remaining >= head.frameBytes as c_int {
        let Some(frame) = Mp3FrameHeader::parse(&pvData[read.min(pvData.len())..]) else {
            break;
        };
        read += frame.frameBytes;
        remaining -= frame.frameBytes as c_int;
        frames += 1;
        if read >= pvData.len() {
            break;
        }
    }

    let outChannels = if bStereoDesired {
        head.channels as usize
    } else {
        1
    };
    Some((frames, head.samplesPerFrame * outChannels * 2))
}

/// Raven `BYTESREMAINING_ACCOUNT_FOR_REAR_TAG` — the byte count with a trailing
/// ID3v1 tag taken off.
///
/// Source: `oracle/codemp/mp3code/towave.c:170-185`
fn trim_rear_tag(pvData: &[u8], iBytesRemaining: c_int) -> c_int {
    let len = (iBytesRemaining.max(0) as usize).min(pvData.len());
    if len >= ID3V1_BYTES && &pvData[len - ID3V1_BYTES..len - ID3V1_BYTES + 3] == b"TAG" {
        return iBytesRemaining - ID3V1_BYTES as c_int;
    }
    iBytesRemaining
}

// ===========================================================================
// Raven's `snd_mp3.cpp` surface
// ===========================================================================

/// Raven `MP3_IsValid` — validate a loaded file, and print why it failed.
///
/// The file name is for the error message only; the data is already loaded.
/// Source: `oracle/codemp/client/snd_mp3.cpp:18-28`
pub fn MP3_IsValid(
    common: &mut Common,
    psLocalFilename: &str,
    pvData: &[u8],
    iDataLen: c_int,
    bStereoDesired: bool,
) -> bool {
    let psError = C_MP3_IsValid(pvData, iDataLen, bStereoDesired);

    if let Some(error) = psError {
        com_printf(common, &format!("^1{error}({psLocalFilename})\n"));
    }

    psError.is_none()
}

/// Raven `MP3_GetUnpackedSize` — the byte count the whole file decodes to.
///
/// Raven always measures rather than trusting a tag, because a tag may have been
/// edited or the file re-saved. Answers 0 for an error, which is printed here.
/// Source: `oracle/codemp/client/snd_mp3.cpp:36-56`
pub fn MP3_GetUnpackedSize(
    common: &mut Common,
    psLocalFilename: &str,
    pvData: &[u8],
    iDataLen: c_int,
    bStereoDesired: bool,
) -> c_int {
    let Some((frames, packetBytes)) = walk_frames(pvData, iDataLen, bStereoDesired) else {
        com_printf(
            common,
            &format!("^1MP3ERR: Bad or Unsupported MP3 file!\n(File: {psLocalFilename})\n"),
        );
        return 0;
    };

    (frames * packetBytes) as c_int
}

/// Raven `MP3_UnpackRawPCM` — decode the whole file into `pbUnpackBuffer`.
///
/// Raven decodes at the source rate here, and the caller resamples afterwards.
/// Answers the byte count written, which reads as a success flag.
/// Source: `oracle/codemp/client/snd_mp3.cpp:64-76`
pub fn MP3_UnpackRawPCM(
    common: &mut Common,
    psLocalFilename: &str,
    pvData: &[u8],
    iDataLen: c_int,
    pbUnpackBuffer: &mut [u8],
    bStereoDesired: bool,
) -> c_int {
    let scanLimit = (iDataLen.max(0) / 2) as usize;
    let Some((_, head)) = Mp3FrameHeader::find(pvData, scanLimit) else {
        com_printf(
            common,
            &format!("^1MP3ERR: Bad or Unsupported MP3 file!\n(File: {psLocalFilename})\n"),
        );
        return 0;
    };

    let mut stream = MP3StreamState::default();
    // The source rate keeps the reduction at one, which is Raven's
    // `reduction_code = 0` on this path.
    if let Some(error) = stream.DecodeInit(pvData, iDataLen, head.sampleRate, bStereoDesired) {
        com_printf(
            common,
            &format!("^1{error}\n(File: {psLocalFilename})\n"),
        );
        return 0;
    }

    let mut written = 0usize;
    loop {
        let iBytes = stream.Decode(pvData, 0);
        if iBytes == 0 {
            break;
        }
        let iBytes = iBytes as usize;
        if written + iBytes > pbUnpackBuffer.len() {
            break;
        }
        pbUnpackBuffer[written..written + iBytes]
            .copy_from_slice(&stream.bDecodeBuffer[..iBytes]);
        written += iBytes;
    }

    written as c_int
}

/// Raven `MP3Stream_InitPlayingTimeFields` — fill the four fields the
/// playing-time queries read.
///
/// Source: `oracle/codemp/client/snd_mp3.cpp:81-108`
pub fn MP3Stream_InitPlayingTimeFields(
    common: &mut Common,
    lpMP3Stream: &mut MP3StreamState,
    psLocalFilename: &str,
    pvData: &[u8],
    iDataLen: c_int,
    bStereoDesired: bool,
) -> bool {
    let mut bRetval = false;

    match C_MP3_GetHeaderData(pvData, iDataLen, bStereoDesired) {
        Err(error) => {
            com_printf(
                common,
                &format!(
                    "^1MP3Stream_InitPlayingTimeFields(): {error}\n(File: {psLocalFilename})\n"
                ),
            );
        }
        Ok((iRate, iWidth, iChannels)) => {
            let iUnpackLength =
                MP3_GetUnpackedSize(common, psLocalFilename, pvData, iDataLen, bStereoDesired);
            if iUnpackLength != 0 {
                lpMP3Stream.iTimeQuery_UnpackedLength = iUnpackLength;
                lpMP3Stream.iTimeQuery_SampleRate = iRate;
                lpMP3Stream.iTimeQuery_Channels = iChannels;
                lpMP3Stream.iTimeQuery_Width = iWidth;

                bRetval = true;
            }
        }
    }

    bRetval
}

/// Raven `MP3Stream_GetPlayingTimeInSeconds` — the whole track length.
///
/// Answers 0 where `MP3Stream_InitPlayingTimeFields` never ran.
/// Source: `oracle/codemp/client/snd_mp3.cpp:110-116`
pub fn MP3Stream_GetPlayingTimeInSeconds(lpMP3Stream: &MP3StreamState) -> f32 {
    if lpMP3Stream.iTimeQuery_UnpackedLength != 0 {
        return ((f64::from(lpMP3Stream.iTimeQuery_UnpackedLength)
            / f64::from(lpMP3Stream.iTimeQuery_SampleRate))
            / f64::from(lpMP3Stream.iTimeQuery_Channels))
            as f32
            / lpMP3Stream.iTimeQuery_Width as f32;
    }

    0.0
}

/// Raven `MP3Stream_GetRemainingTimeInSeconds` — the track time still to play.
///
/// Raven's `(iTimeQuery_SampleRate / dma.speed)` is integer division between two
/// ints, so it steps 2 at the default rate and 1 at 44 kHz.
/// Source: `oracle/codemp/client/snd_mp3.cpp:118-124`
pub fn MP3Stream_GetRemainingTimeInSeconds(lpMP3Stream: &MP3StreamState, dmaSpeed: c_int) -> f32 {
    if lpMP3Stream.iTimeQuery_UnpackedLength != 0 {
        let consumed = lpMP3Stream.iBytesDecodedTotal * (lpMP3Stream.iTimeQuery_SampleRate / dmaSpeed);
        return (((f64::from(lpMP3Stream.iTimeQuery_UnpackedLength - consumed)
            / f64::from(lpMP3Stream.iTimeQuery_SampleRate))
            / f64::from(lpMP3Stream.iTimeQuery_Channels))
            / f64::from(lpMP3Stream.iTimeQuery_Width)) as f32;
    }

    0.0
}

/// Raven `MP3_FakeUpWAVInfo` — describe the unpacked block as a WAV, so the
/// loader's post-load code runs unchanged.
///
/// Source: `oracle/codemp/client/snd_mp3.cpp:131-154`
#[allow(clippy::too_many_arguments)]
pub fn MP3_FakeUpWAVInfo(
    common: &mut Common,
    psLocalFilename: &str,
    pvData: &[u8],
    iDataLen: c_int,
    iUnpackedDataLength: c_int,
    info: &mut wavinfo_t,
    bStereoDesired: bool,
) -> bool {
    // Some things can be done instantly.
    info.format = 1; // 1 for MS format
    info.dataofs = 0; // will be 0 for me (since there's no header in the unpacked data)

    let ok = match C_MP3_GetHeaderData(pvData, iDataLen, bStereoDesired) {
        Ok((rate, width, channels)) => {
            info.rate = rate;
            info.width = width;
            info.channels = channels;
            true
        }
        Err(error) => {
            com_printf(
                common,
                &format!("^1{error}\n(File: {psLocalFilename})\n"),
            );
            false
        }
    };

    // and some stuff needs calculating...
    info.samples = iUnpackedDataLength / info.width.max(1);

    ok
}

/// Raven `MP3_ReadSpecialTagInfo` — read the uncompressed size and the peak
/// volume the tagger wrote into an ID3v1 tag.
///
/// Answers `None` where the file carries no tag or the tag misses a key, which
/// is Raven's `qfalse` return. Raven also hands the caller the tag itself, and
/// no call site in either tree asks for it, so the port drops that out-param
/// (porting-rules §20).
/// Source: `oracle/codemp/client/snd_mp3.cpp:163-218`
pub fn MP3_ReadSpecialTagInfo(pbLoadedFile: &[u8], iLoadedFileLen: c_int) -> Option<(c_int, f32)> {
    let len = (iLoadedFileLen.max(0) as usize).min(pbLoadedFile.len());
    if len < ID3V1_BYTES {
        return None;
    }
    let tag = &pbLoadedFile[len - ID3V1_BYTES..len];
    if &tag[..3] != b"TAG" {
        return None;
    }

    // The layout is fixed: 3 id, 30 title, 30 artist, 30 album, 4 year, 28 comment.
    let album = field_str(&tag[93..123]);
    let comment = field_str(&tag[127 - 28 + 0..127]);

    let fMaxVol = comment.strip_prefix(sKEY_MAXVOL)?;
    let iUncompressedSize = album.strip_prefix(sKEY_UNCOMP)?;

    Some((atoi(iUncompressedSize), atof(fMaxVol) as f32))
}

/// One ID3v1 text field as a string, cut at the first NUL the way C reads it.
fn field_str(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

/// C's `atoi` over a leading-space-tolerant decimal prefix.
fn atoi(text: &str) -> c_int {
    let text = text.trim_start();
    let end = text
        .find(|c: char| !c.is_ascii_digit() && c != '-' && c != '+')
        .unwrap_or(text.len());
    text[..end].parse().unwrap_or(0)
}

/// C's `atof` over a leading-space-tolerant decimal prefix.
fn atof(text: &str) -> f64 {
    let text = text.trim_start();
    let end = text
        .find(|c: char| !c.is_ascii_digit() && c != '-' && c != '+' && c != '.')
        .unwrap_or(text.len());
    text[..end].parse().unwrap_or(0.0)
}

/// Raven `MP3Stream_InitFromFile` — decide whether a loaded file stays MP3, and
/// set the `sfx_t` up for streaming where it does.
///
/// Raven keeps the file as MP3 only where the raw bytes plus the per-stream
/// overhead still come out smaller than the unpacked PCM. Returns true where the
/// sound is now a stream.
/// Source: `oracle/codemp/client/snd_mp3.cpp:238-309`
pub fn MP3Stream_InitFromFile(
    common: &mut Common,
    sfx: &mut sfx_t,
    pbSrcData: &[u8],
    iSrcDatalen: c_int,
    psSrcDataFilename: &str,
    iMP3UnPackedSize: c_int,
    iMP3Overhead: c_int,
    dmaSpeed: c_int,
    bStereoDesired: bool,
) -> bool {
    // First, decide on size: the MP3 buffer space makes a small file bigger, so
    // a small file is best left as a WAV.
    if iSrcDatalen + iMP3Overhead >= iMP3UnPackedSize {
        return false;
    }

    // Raven: seems to be a reasonable typical default for maxvol (for lip
    // synch). Naturally there is no #define to use instead.
    let mut fMaxVol = 128.0f32;
    if let Some((_, tagMaxVol)) = MP3_ReadSpecialTagInfo(pbSrcData, iSrcDatalen) {
        fMaxVol = tagMaxVol;
    }

    sfx.eSoundCompressionMethod = SoundCompressionMethod_t::ct_MP3;
    sfx.fVolRange = fMaxVol;
    sfx.iSoundLengthInSamples = ((iMP3UnPackedSize / 2) / (44100 / dmaSpeed))
        / if bStereoDesired { 2 } else { 1 };

    // The raw MP3 stays in the sfx block, and the stream decoder reads it there.
    sfx.pSoundData = Some(SfxSampleData::Mp3(
        pbSrcData[..(iSrcDatalen.max(0) as usize).min(pbSrcData.len())].to_vec(),
    ));

    let mut stream = MP3StreamState::default();
    if let Some(error) = stream.DecodeInit(pbSrcData, iSrcDatalen, dmaSpeed, bStereoDesired) {
        // Raven: this should never happen, since any problem with the MP3 file
        // would have stopped us getting to this whole function.
        com_printf(
            common,
            &format!("^3File \"{psSrcDataFilename}\": {error}\n"),
        );
        return false;
    }

    sfx.pMP3StreamHeader = Some(Box::new(stream));
    true
}

/// Raven `MP3Stream_Decode` — decode one packet of MP3 data.
///
/// Raven's SOF2 link-list arm is `#if 0`, so only the direct call survives.
/// Returns the decoded byte count, and 0 for a finished stream.
/// Source: `oracle/codemp/client/snd_mp3.cpp:317-400`
pub fn MP3Stream_Decode(
    lpMP3Stream: &mut MP3StreamState,
    source: &[u8],
    sourceOrigin: c_int,
) -> c_int {
    lpMP3Stream.Decode(source, sourceOrigin)
}

/// Raven `MP3Stream_Rewind` — put the channel's stream back on the first frame.
///
/// Raven copies the pristine header off the `sfx_t` rather than re-reading the
/// file, which it calls a speed optimisation.
/// Source: `oracle/codemp/client/snd_mp3.cpp:440-462`
pub fn MP3Stream_Rewind(chMp3: &mut ChannelMp3State, pristine: &MP3StreamState) {
    chMp3.iMP3SlidingDecodeWritePos = 0;
    chMp3.iMP3SlidingDecodeWindowPos = 0;
    chMp3.MP3StreamHeader = pristine.clone();
}

/// Raven `MP3Stream_SeekTo` — decode forward until the play position lands
/// within a twentieth of a second of `fTimeToSeekTo`.
///
/// Source: `oracle/codemp/client/snd_mp3.cpp:403-435`
pub fn MP3Stream_SeekTo(
    chMp3: &mut ChannelMp3State,
    pristine: &MP3StreamState,
    source: &[u8],
    sourceOrigin: c_int,
    dmaSpeed: c_int,
    fTimeToSeekTo: f32,
) -> bool {
    // Raven: accurate to 1/50 of a second, but plus or minus this gives 1/10.
    const F_EPSILON: f32 = 0.05;

    MP3Stream_Rewind(chMp3, pristine);

    let fTrackLengthInSeconds = MP3Stream_GetPlayingTimeInSeconds(&chMp3.MP3StreamHeader);
    let fTimeToSeekTo = if fTimeToSeekTo > fTrackLengthInSeconds {
        fTrackLengthInSeconds
    } else {
        fTimeToSeekTo
    };

    loop {
        let fPlayingTimeElapsed = MP3Stream_GetPlayingTimeInSeconds(&chMp3.MP3StreamHeader)
            - MP3Stream_GetRemainingTimeInSeconds(&chMp3.MP3StreamHeader, dmaSpeed);
        let fAbsTimeDiff = (fTimeToSeekTo - fPlayingTimeElapsed).abs();

        if fAbsTimeDiff <= F_EPSILON {
            return true;
        }

        // Raven fast-forwards until within 3 seconds, then slow-decodes. The
        // replaced decoder has one decode path, so both run the same call.
        let iBytesDecodedThisPacket = chMp3.MP3StreamHeader.Decode(source, sourceOrigin);
        if iBytesDecodedThisPacket == 0 {
            break; // EOS
        }
    }

    false
}

/// Raven `MP3Stream_GetSamples` — feed `count` samples out of the sliding decode
/// window, decoding forward until the window covers the request.
///
/// Returns true while the stream is still playing, and false for either a
/// finished stream or a request that asks to go backwards.
/// Source: `oracle/codemp/client/snd_mp3.cpp:467-549`
pub fn MP3Stream_GetSamples(
    chMp3: &mut ChannelMp3State,
    source: &[u8],
    sourceOrigin: c_int,
    startingSampleNum: c_int,
    count: c_int,
    buf: &mut [i16],
    bStereo: bool,
) -> bool {
    let mut qbStreamStillGoing = true;

    let bufferBytes = chMp3.MP3SlidingDecodeBuffer.len();
    let iQuarterOfSlidingBuffer = bufferBytes / 4;
    let iThreeQuartersOfSlidingBuffer = (bufferBytes * 3) / 4;

    // The count arg was words, so double it for bytes.
    let countBytes = (count * 2) as usize;

    // Convert the sample number into a byte offset.
    let startingSampleNum = startingSampleNum * 2 * if bStereo { 2 } else { 1 };

    if startingSampleNum < chMp3.iMP3SlidingDecodeWindowPos {
        // Raven: what?!?!?! smegging time travel needed or something?, forget it
        buf[..countBytes / 2].fill(0);
        return false;
    }

    let countBytesInt = countBytes as c_int;
    while !(startingSampleNum >= chMp3.iMP3SlidingDecodeWindowPos
        && startingSampleNum + countBytesInt
            < chMp3.iMP3SlidingDecodeWindowPos + chMp3.iMP3SlidingDecodeWritePos)
    {
        // Raven passes `bStereo` as the "doing music" flag, which is safe
        // because only music decodes in stereo.
        let iBytesDecoded = chMp3.MP3StreamHeader.Decode(source, sourceOrigin);
        if iBytesDecoded == 0 {
            // No more source data left, so clear the rest of the buffer.
            let from = chMp3.iMP3SlidingDecodeWritePos.max(0) as usize;
            chMp3.MP3SlidingDecodeBuffer[from.min(bufferBytes)..].fill(0);
            qbStreamStillGoing = false;
            break;
        }

        let write = chMp3.iMP3SlidingDecodeWritePos.max(0) as usize;
        let iBytesDecoded = (iBytesDecoded as usize).min(bufferBytes - write);
        let packet = chMp3.MP3StreamHeader.bDecodeBuffer[..iBytesDecoded].to_vec();
        chMp3.MP3SlidingDecodeBuffer[write..write + iBytesDecoded].copy_from_slice(&packet);
        chMp3.iMP3SlidingDecodeWritePos += iBytesDecoded as c_int;

        // Once past three quarters of the buffer, backscroll the window by one
        // quarter.
        if chMp3.iMP3SlidingDecodeWritePos > iThreeQuartersOfSlidingBuffer as c_int {
            chMp3
                .MP3SlidingDecodeBuffer
                .copy_within(iQuarterOfSlidingBuffer..bufferBytes, 0);
            chMp3.iMP3SlidingDecodeWritePos -= iQuarterOfSlidingBuffer as c_int;
            chMp3.iMP3SlidingDecodeWindowPos += iQuarterOfSlidingBuffer as c_int;
        }
    }

    let from = (startingSampleNum - chMp3.iMP3SlidingDecodeWindowPos).max(0) as usize;
    for i in 0..countBytes / 2 {
        let byte = from + i * 2;
        buf[i] = if byte + 2 <= bufferBytes {
            i16::from_le_bytes([
                chMp3.MP3SlidingDecodeBuffer[byte],
                chMp3.MP3SlidingDecodeBuffer[byte + 1],
            ])
        } else {
            // Raven reads past the window here; the port answers silence (§19).
            0
        };
    }

    qbStreamStillGoing
}
