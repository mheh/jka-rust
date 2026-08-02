//! `MP3StreamState` — the live state of one streaming MP3.
//!
//! Raven's `MP3STREAM` is the decoder's own working struct with a handful of
//! wrap fields bolted on the end. DEC-03 and DEC-57.3 replace the decoder, so
//! this type carries the wrap fields plus the replacement decoder, and the
//! layout twin `MP3STREAM` stays only as the `s_mp3overhead` size reference.
//!
//! Raven's `iCopyOffset` and the `iRewind_Final*Code` pair have no reader in
//! either tree (porting-rules §20), so no field carries them.
//!
//! Source: `oracle/codemp/mp3code/mp3struct.h:17-128`,
//! `oracle/codemp/mp3code/towave.c:573-712`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use crate::mp3::mp3_decoder::Mp3Decoder;
use crate::mp3::mp3_frame_header::Mp3FrameHeader;

/// Raven `bDecodeBuffer[2304*2]` — one decoded packet, stereo-sized.
///
/// Source: `oracle/codemp/mp3code/mp3struct.h:120`
pub const DECODE_BUFFER_BYTES: usize = 2304 * 2;

/// One streaming MP3: where the read has reached, what the last packet decoded
/// to, and the playing-time fields the dynamic-music player queries.
///
/// The source bytes live with the owner (an `sfx_t` block or the music disk
/// window), so `decode` takes them as an argument instead of the raw
/// `pbSourceData` pointer Raven kept here.
#[derive(Clone)]
pub struct MP3StreamState {
    /// Raven `iSourceBytesRemaining` / `iSourceReadIndex` / `iSourceFrameBytes`.
    pub iSourceBytesRemaining: c_int,
    pub iSourceReadIndex: c_int,
    pub iSourceFrameBytes: c_int,
    /// Raven `iBytesDecodedTotal` / `iBytesDecodedThisPacket`.
    pub iBytesDecodedTotal: c_int,
    pub iBytesDecodedThisPacket: c_int,
    /// Raven `bDecodeBuffer` — the last decoded packet, 16-bit little-endian.
    pub bDecodeBuffer: Vec<u8>,
    /// Raven `iRewind_SourceReadIndex` / `iRewind_SourceBytesRemaining`.
    pub iRewind_SourceReadIndex: c_int,
    pub iRewind_SourceBytesRemaining: c_int,
    /// Raven's `iTimeQuery_*` block, zero until
    /// `MP3Stream_InitPlayingTimeFields` fills it.
    pub iTimeQuery_UnpackedLength: c_int,
    pub iTimeQuery_SampleRate: c_int,
    pub iTimeQuery_Channels: c_int,
    pub iTimeQuery_Width: c_int,
    /// The rate the game mixer runs at, which the packet output is reduced to.
    pub iGameAudioSampleRate: c_int,
    /// Raven's `convert_code`: false is the mono downmix every sound effect
    /// takes, true is the stereo pair music takes.
    pub bStereoDesired: bool,
    /// Raven's reduction code as the divisor it means: 1, 2, or 4.
    pub iReduction: c_int,
    decoder: Mp3Decoder,
}

impl Default for MP3StreamState {
    fn default() -> MP3StreamState {
        MP3StreamState {
            iSourceBytesRemaining: 0,
            iSourceReadIndex: 0,
            iSourceFrameBytes: 0,
            iBytesDecodedTotal: 0,
            iBytesDecodedThisPacket: 0,
            bDecodeBuffer: vec![0u8; DECODE_BUFFER_BYTES],
            iRewind_SourceReadIndex: 0,
            iRewind_SourceBytesRemaining: 0,
            iTimeQuery_UnpackedLength: 0,
            iTimeQuery_SampleRate: 0,
            iTimeQuery_Channels: 0,
            iTimeQuery_Width: 0,
            iGameAudioSampleRate: 0,
            bStereoDesired: false,
            iReduction: 1,
            decoder: Mp3Decoder::new(),
        }
    }
}

impl MP3StreamState {
    /// Raven `C_MP3Stream_DecodeInit` — find the first frame, work out the
    /// reduction the game rate needs, and record the rewind point.
    ///
    /// Answers Raven's error string, or `None` for a stream that is ready.
    /// Raven skips the trailing-tag adjustment for the stereo case, because a
    /// streamed music file only has its first few kilobytes in memory here.
    /// Source: `oracle/codemp/mp3code/towave.c:573-664`
    pub fn DecodeInit(
        &mut self,
        pvSourceData: &[u8],
        iSourceBytesRemaining: c_int,
        iGameAudioSampleRate: c_int,
        bStereoDesired: bool,
    ) -> Option<&'static str> {
        *self = MP3StreamState::default();

        self.iSourceBytesRemaining = iSourceBytesRemaining;
        self.iGameAudioSampleRate = iGameAudioSampleRate;
        self.bStereoDesired = bStereoDesired;

        let scanLimit = (iSourceBytesRemaining.max(0) / 2) as usize;
        let Some((offset, head)) = Mp3FrameHeader::find(pvSourceData, scanLimit) else {
            return Some("MP3ERR: Errr.... something's broken with this MP3 file");
        };
        self.iSourceReadIndex = offset as c_int;
        self.iSourceFrameBytes = head.frameBytes as c_int;

        if !bStereoDesired {
            self.iSourceBytesRemaining =
                bytes_before_rear_tag(pvSourceData, self.iSourceBytesRemaining);
            self.iSourceBytesRemaining -= self.iSourceReadIndex;
        }

        self.iRewind_SourceReadIndex = self.iSourceReadIndex;
        self.iRewind_SourceBytesRemaining = self.iSourceBytesRemaining;

        // The decoder offers whole-number rate reduction only, so a game rate
        // that is not the source rate, half of it, or a quarter of it fails.
        self.iReduction = if iGameAudioSampleRate == head.sampleRate {
            1
        } else if iGameAudioSampleRate == head.sampleRate >> 1 {
            2
        } else if iGameAudioSampleRate == head.sampleRate >> 2 {
            4
        } else {
            return Some("MP3ERR: Decoder unable to convert to current game audio settings");
        };

        None
    }

    /// Raven `C_MP3Stream_Decode` — decode one packet into `bDecodeBuffer`.
    ///
    /// The byte at stream index `i` is `source[i - sourceOrigin]`, which is how
    /// Raven's music streamer slides a disk window under a stream that keeps
    /// counting in whole-file offsets.
    /// Returns the decoded byte count, and 0 for a finished stream.
    /// Source: `oracle/codemp/mp3code/towave.c:669-712`
    pub fn Decode(&mut self, source: &[u8], sourceOrigin: c_int) -> c_int {
        if self.iSourceBytesRemaining == 0 {
            return 0;
        }

        let start = (self.iSourceReadIndex - sourceOrigin).max(0) as usize;
        if start >= source.len() {
            return 0;
        }
        let Some(head) = Mp3FrameHeader::parse(&source[start..]) else {
            return 0;
        };
        if start + head.frameBytes > source.len() {
            return 0;
        }

        let Some(frame) = self.decoder.decode(&source[start..start + head.frameBytes]) else {
            return 0;
        };

        let outChannels = if self.bStereoDesired { 2 } else { 1 };
        let iBytes = pack_packet(
            frame.samples,
            frame.channels,
            outChannels,
            self.iReduction,
            &mut self.bDecodeBuffer,
        );

        self.iSourceReadIndex += head.frameBytes as c_int;
        self.iSourceBytesRemaining -= head.frameBytes as c_int;
        self.iBytesDecodedTotal += iBytes;
        self.iBytesDecodedThisPacket = iBytes;

        iBytes
    }

    /// Raven `C_MP3Stream_Rewind` — put the read back on the first frame and
    /// start the decode over.
    ///
    /// Source: `oracle/codemp/mp3code/towave.c:717-756`
    pub fn Rewind(&mut self) {
        self.iSourceReadIndex = self.iRewind_SourceReadIndex;
        self.iSourceBytesRemaining = self.iRewind_SourceBytesRemaining;
        self.iBytesDecodedTotal = 0;
        self.iBytesDecodedThisPacket = 0;
        self.decoder.reset();
    }
}

/// Raven `BYTESREMAINING_ACCOUNT_FOR_REAR_TAG` — drop a trailing ID3v1 tag from
/// the byte count, so the decoder never treats the tag as audio.
///
/// Source: `oracle/codemp/mp3code/towave.c:170-185`
fn bytes_before_rear_tag(data: &[u8], iBytesRemaining: c_int) -> c_int {
    const TAG_BYTES: usize = 128;
    let len = (iBytesRemaining.max(0) as usize).min(data.len());
    if len >= TAG_BYTES && &data[len - TAG_BYTES..len - TAG_BYTES + 3] == b"TAG" {
        return iBytesRemaining - TAG_BYTES as c_int;
    }
    iBytesRemaining
}

/// Fold one decoded frame into the packet shape the mixer reads: `outChannels`
/// interleaved 16-bit samples, reduced by `iReduction`.
///
/// The downmix and the rate reduction both average, so the replacement decoder
/// behaves the way Raven's `convert_code`/`reduction_code` pair did. Returns the
/// byte count written.
fn pack_packet(
    samples: &[i16],
    inChannels: c_int,
    outChannels: c_int,
    iReduction: c_int,
    dest: &mut [u8],
) -> c_int {
    let inChannels = inChannels.max(1) as usize;
    let outChannels = outChannels.max(1) as usize;
    let step = iReduction.max(1) as usize;

    let inFrames = samples.len() / inChannels;
    let outFrames = inFrames / step;

    let mut written = 0usize;
    for frame in 0..outFrames {
        for lane in 0..outChannels {
            let mut total: i32 = 0;
            let mut count: i32 = 0;
            for sub in 0..step {
                let index = (frame * step + sub) * inChannels;
                if inChannels == outChannels {
                    total += i32::from(samples[index + lane]);
                    count += 1;
                } else if inChannels > outChannels {
                    // stereo source, mono output: average the pair
                    for ch in 0..inChannels {
                        total += i32::from(samples[index + ch]);
                        count += 1;
                    }
                } else {
                    // mono source, stereo output: both lanes carry the sample
                    total += i32::from(samples[index]);
                    count += 1;
                }
            }
            let value = (total / count.max(1)) as i16;
            if written + 2 > dest.len() {
                return written as c_int;
            }
            dest[written..written + 2].copy_from_slice(&value.to_le_bytes());
            written += 2;
        }
    }

    written as c_int
}
