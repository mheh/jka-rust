//! `Mp3FrameHeader` — one MPEG audio frame header, the framing Raven's
//! `head_info3` reads.

#![allow(non_snake_case)]

use core::ffi::c_int;

/// Bitrates in kbit/s, indexed by `[version_is_mpeg1][layer][bitrate_index]`.
/// Index 0 is the free-format slot and index 15 is the reserved slot, and both
/// read as 0 here, which makes the header invalid.
/// Source: ISO/IEC 11172-3 table 8, ISO/IEC 13818-3 table 8
const BITRATES_MPEG1: [[u32; 16]; 3] = [
    // Layer I
    [
        0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448, 0,
    ],
    // Layer II
    [
        0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384, 0,
    ],
    // Layer III
    [
        0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0,
    ],
];

/// The MPEG-2 and MPEG-2.5 bitrate table, in the same layer order.
/// Source: ISO/IEC 13818-3 table 8, and the MPEG-2.5 extension
const BITRATES_MPEG2: [[u32; 16]; 3] = [
    // Layer I
    [
        0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256, 0,
    ],
    // Layer II and Layer III share this row
    [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0],
    [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0],
];

/// Sample rates by version, then by the two-bit rate index. Index 3 is reserved
/// and reads as 0, which makes the header invalid.
const SAMPLE_RATES: [[u32; 4]; 3] = [
    // MPEG-1
    [44100, 48000, 32000, 0],
    // MPEG-2
    [22050, 24000, 16000, 0],
    // MPEG-2.5
    [11025, 12000, 8000, 0],
];

/// One MPEG audio frame header.
///
/// The whole frame, header bytes included, is `frameBytes` long, so a walker
/// steps the stream by that count. `channels` is the decoded channel count, and
/// `samplesPerFrame` is per channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mp3FrameHeader {
    /// 1 for MPEG-1, 2 for MPEG-2, 3 for MPEG-2.5.
    pub version: u8,
    /// 1, 2, or 3.
    pub layer: u8,
    pub sampleRate: c_int,
    pub channels: c_int,
    pub frameBytes: usize,
    pub samplesPerFrame: usize,
}

impl Mp3FrameHeader {
    /// The four header bytes every MPEG audio frame opens with.
    pub const BYTES: usize = 4;

    /// Read the header the four bytes at the start of `data` hold.
    ///
    /// Answers `None` where the sync word, the version, the layer, the bitrate,
    /// or the sample rate is not a shipped combination.
    pub fn parse(data: &[u8]) -> Option<Mp3FrameHeader> {
        if data.len() < Mp3FrameHeader::BYTES {
            return None;
        }
        let word = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);

        // The 11-bit sync word opens every frame.
        if word & 0xffe0_0000 != 0xffe0_0000 {
            return None;
        }

        let version = match (word >> 19) & 0x3 {
            0 => 3, // MPEG-2.5
            2 => 2, // MPEG-2
            3 => 1, // MPEG-1
            _ => return None,
        };
        let layer = match (word >> 17) & 0x3 {
            1 => 3,
            2 => 2,
            3 => 1,
            _ => return None,
        };

        let bitrateIndex = ((word >> 12) & 0xf) as usize;
        let rateIndex = ((word >> 10) & 0x3) as usize;
        let padding = ((word >> 9) & 0x1) as usize;
        let mode = (word >> 6) & 0x3;

        let table = if version == 1 {
            &BITRATES_MPEG1
        } else {
            &BITRATES_MPEG2
        };
        let bitrate = table[layer as usize - 1][bitrateIndex] * 1000;
        let sampleRate = SAMPLE_RATES[version as usize - 1][rateIndex];
        if bitrate == 0 || sampleRate == 0 {
            return None;
        }

        // Mode 3 is single channel; the other three carry two.
        let channels = if mode == 3 { 1 } else { 2 };

        let (samplesPerFrame, frameBytes) = match layer {
            1 => (384, (12 * bitrate as usize / sampleRate as usize + padding) * 4),
            2 => (1152, 144 * bitrate as usize / sampleRate as usize + padding),
            _ if version == 1 => (1152, 144 * bitrate as usize / sampleRate as usize + padding),
            _ => (576, 72 * bitrate as usize / sampleRate as usize + padding),
        };

        if frameBytes <= Mp3FrameHeader::BYTES {
            return None;
        }

        Some(Mp3FrameHeader {
            version,
            layer,
            sampleRate: sampleRate as c_int,
            channels,
            frameBytes,
            samplesPerFrame,
        })
    }

    /// Raven `head_info3` — scan forward for the first frame header, and answer
    /// the offset it sits at.
    ///
    /// Raven limits the scan to half the supplied length, and the callers pass
    /// `iDataLen/2` for that reason. An ID3v2 tag at the head of the file is
    /// skipped the way any sync scan skips it, by walking past it.
    /// Source: `oracle/codemp/mp3code/mhead.c` (`head_info3`)
    pub fn find(data: &[u8], scanLimit: usize) -> Option<(usize, Mp3FrameHeader)> {
        let limit = scanLimit.min(data.len());
        for offset in 0..limit {
            let Some(head) = Mp3FrameHeader::parse(&data[offset..]) else {
                continue;
            };
            // One sync word alone is a weak match, so the next frame must sync
            // too whenever the stream is long enough to hold it.
            let next = offset + head.frameBytes;
            if next + Mp3FrameHeader::BYTES <= data.len()
                && Mp3FrameHeader::parse(&data[next..]).is_none()
            {
                continue;
            }
            return Some((offset, head));
        }
        None
    }
}
