#![allow(non_camel_case_types, non_snake_case)]

use core::mem::ManuallyDrop;

use super::decode_function::DECODE_FUNCTION;
use super::sample::SAMPLE;
use super::sbt_function::SBT_FUNCTION;
use super::xform_function::XFORM_FUNCTION;

/// `NBUF` — MP3 decode ring-buffer size (Raven `#define NBUF (8*1024)`).
///
/// Source: `oracle/oracle/codemp/client/../mp3code/mp3struct.h:48`
const NBUF: usize = 8 * 1024;

/// Anonymous struct for `MP3STREAM`'s top union, layer-1/2 branch (Raven names it in a
/// trailing comment: `};//L1_2;`).
///
/// Type definition source: `oracle/oracle/codemp/client/../mp3code/mp3struct.h:21-36`
#[repr(C)]
pub struct MP3STREAM_L1_2 {
    pub sbt: SBT_FUNCTION,

    /// 768 bytes
    pub cs_factor: [[f32; 64]; 3],

    pub nbat: [i32; 4],
    pub bat: [[i32; 16]; 4],
    pub max_sb: i32,
    pub stereo_sb: i32,
    pub bit_skip: i32,

    pub cs_factorL1: *mut f32,
    pub nbatL1: i32,
}

/// Anonymous struct for `MP3STREAM`'s top union, layer-3 branch (Raven names it in a
/// trailing comment: `};//L3;`).
///
/// Raven: `sample` — if this isn't kept per stream then the decode breaks up. `buf` — the
/// 4k version of `NBUF`/`BUF_TRIGGER` seems to work for everything, but reverting to the
/// original 8k for safety just in case.
/// Type definition source: `oracle/oracle/codemp/client/../mp3code/mp3struct.h:38-74`
#[repr(C)]
pub struct MP3STREAM_L3 {
    pub sbt_L3: SBT_FUNCTION,
    pub Xform: XFORM_FUNCTION,
    pub decode_function: DECODE_FUNCTION,

    /// if this isn't kept per stream then the decode breaks up
    pub sample: [[[SAMPLE; 576]; 2]; 2],

    pub buf: [u8; NBUF],
    pub buf_ptr0: i32,
    pub buf_ptr1: i32,
    pub main_pos_bit: i32,

    pub band_limit_nsb: i32,
    /// `[long/short][cb]`
    pub nBand: [[i32; 22]; 2],
    /// `[long/short][cb]`
    pub sfBandIndex: [[i32; 22]; 2],
    pub half_outbytes: i32,
    pub crcbytes: i32,
    pub nchan: i32,
    pub ms_mode: i32,
    pub is_mode: i32,
    pub zero_level_pcm: u32,
    pub mpeg25_flag: i32,
    pub band_limit: i32,
    pub band_limit21: i32,
    pub band_limit12: i32,
    pub gain_adjust: i32,
    pub ncbl_mixed: i32,
}

/// Anonymous union for `MP3STREAM`'s top member (no Raven name — anonymous in the header;
/// only one of the `L1_2`/`L3` branches is live per stream, chosen by MPEG layer).
///
/// Type definition source: `oracle/oracle/codemp/client/../mp3code/mp3struct.h:19-75`
#[repr(C)]
pub union MP3STREAM_u {
    pub l1_2: ManuallyDrop<MP3STREAM_L1_2>,
    pub l3: ManuallyDrop<MP3STREAM_L3>,
}

/// Raven `MP3STREAM` (`LP_MP3STREAM` is `*mut MP3STREAM`) — per-stream MP3 decoder state,
/// owned by the `sfx_t` that streams it.
///
/// Type definition source: `oracle/oracle/codemp/client/../mp3code/mp3struct.h:17-128`
#[repr(C)]
pub struct MP3STREAM {
    pub u: MP3STREAM_u,

    // from csbt.c... if this isn't kept per stream then the decode breaks up.
    pub vb_ptr: i32,
    pub vb2_ptr: i32,
    pub vbuf: [f32; 512],
    /// this can be lost if we stick to mono samples
    pub vbuf2: [f32; 512],

    // L3 only...
    /// L3 only (99%)
    pub sr_index: i32,
    pub id: i32,

    // any type...
    pub outvalues: i32,
    pub outbytes: i32,
    pub framebytes: i32,
    pub pad: i32,
    pub nsb_limit: i32,

    // stuff added now that the game uses streaming MP3s...
    /// a useful dup ptr only, this whole struct will be owned by an `sfx_t` struct that
    /// has the actual data ptr field
    pub pbSourceData: *mut u8,
    pub iSourceBytesRemaining: i32,
    pub iSourceReadIndex: i32,
    pub iSourceFrameBytes: i32,
    // Raven's `iSourceFrameCounter` is `#ifdef _DEBUG` only (not really important); the
    // release build omits it, confirmed by `iBytesDecodedTotal` landing directly after
    // `iSourceFrameBytes` with no gap.
    pub iBytesDecodedTotal: i32,
    /// not sure how useful this will be, it's only per-frame, so will always be full frame
    /// size (eg 2304 or below for mono) except possibly for the last frame?
    pub iBytesDecodedThisPacket: i32,

    pub iRewind_FinalReductionCode: i32,
    pub iRewind_FinalConvertCode: i32,
    pub iRewind_SourceBytesRemaining: i32,
    pub iRewind_SourceReadIndex: i32,
    /// *2 to allow for stereo now
    pub bDecodeBuffer: [u8; 2304 * 2],
    /// used for painting to DMA-feeder, since 2304 won't match the size it wants
    pub iCopyOffset: i32,

    // some new stuff added for dynamic music, to allow "how many seconds left to play"
    // queries... ( m_lengthInSeconds = ((iUnpackedDataLength / iRate) / iChannels) / iWidth; )
    /// Only valid/initialised if `MP3Stream_InitPlayingTimeFields()` was called; if not,
    /// `iTimeQuery_UnpackedLength` will be zero.
    pub iTimeQuery_UnpackedLength: i32,
    pub iTimeQuery_SampleRate: i32,
    pub iTimeQuery_Channels: i32,
    pub iTimeQuery_Width: i32,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<MP3STREAM>() == 26656);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, vb_ptr) == 17848);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, vb2_ptr) == 17852);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, vbuf) == 17856);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, vbuf2) == 19904);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, sr_index) == 21952);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, id) == 21956);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, outvalues) == 21960);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, outbytes) == 21964);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, framebytes) == 21968);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, pad) == 21972);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, nsb_limit) == 21976);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, pbSourceData) == 21984);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, iSourceBytesRemaining) == 21992);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, iSourceReadIndex) == 21996);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, iSourceFrameBytes) == 22000);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, iBytesDecodedTotal) == 22004);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, iBytesDecodedThisPacket) == 22008);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, iRewind_FinalReductionCode) == 22012);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, iRewind_FinalConvertCode) == 22016);
#[cfg(target_pointer_width = "64")]
const _: () =
    assert!(core::mem::offset_of!(MP3STREAM, iRewind_SourceBytesRemaining) == 22020);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, iRewind_SourceReadIndex) == 22024);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, bDecodeBuffer) == 22028);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, iCopyOffset) == 26636);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, iTimeQuery_UnpackedLength) == 26640);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, iTimeQuery_SampleRate) == 26644);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, iTimeQuery_Channels) == 26648);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(MP3STREAM, iTimeQuery_Width) == 26652);
