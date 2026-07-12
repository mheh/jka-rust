#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

use crate::mp3::mp3_stream::MP3STREAM;
use crate::snd::sfx_s::sfx_t;
use crate::snd::streamingbuffer::STREAMINGBUFFER;

/// `NUM_STREAMING_BUFFERS` — OpenAL streaming buffers per channel.
///
/// Source: `oracle/codemp/client/snd_local.h:87`
pub const NUM_STREAMING_BUFFERS: usize = 4;

/// Raven `channel_t` — a live sound-playback channel (mixer slot / OpenAL source).
///
/// Raven: back-indented fields new in TA codebase, will re-format when MP3 code finished -ste;
/// note: field missing in TA: `sboolean loopSound;` // from an `S_AddLoopSound` call, cleared
/// each frame.
/// Type definition source: `oracle/codemp/client/snd_local.h:94-129`
#[repr(C)]
pub struct channel_t {
    /// START_SAMPLE_IMMEDIATE = set immediately on next mix
    pub startSample: u32,
    /// to allow overriding a specific sound
    pub entnum: i32,
    /// to allow overriding a specific sound
    pub entchannel: i32,
    /// 0-255 volume after spatialization
    pub leftvol: i32,
    /// 0-255 volume after spatialization
    pub rightvol: i32,
    /// 0-255 volume before spatialization
    pub master_vol: i32,

    /// only use if fixed_origin is set
    pub origin: vec3_t,

    /// use origin instead of fetching entnum's origin
    pub fixed_origin: i32,
    /// sfx structure
    pub thesfx: *mut sfx_t,
    /// from an S_AddLoopSound call, cleared each frame
    pub loopSound: i32,
    //
    pub MP3StreamHeader: MP3STREAM,
    /// typical back-request = -3072, so roughly double is 6000 (safety), then doubled again so
    /// the 6K pos is in the middle of the buffer)
    pub MP3SlidingDecodeBuffer: [u8; 50000],
    pub iMP3SlidingDecodeWritePos: i32,
    pub iMP3SlidingDecodeWindowPos: i32,

    // Open AL specific
    /// Signifies if this channel / source is playing a looping sound
    pub bLooping: bool,
    // bAmbient: Signifies if this channel / source is playing a looping ambient sound
    /// Signifies if this channel / source has been processed
    pub bProcessed: bool,
    /// Set to true if the data needs to be streamed (MP3 or dialogue)
    pub bStreaming: bool,
    /// AL Buffers for streaming
    pub buffers: [STREAMINGBUFFER; NUM_STREAMING_BUFFERS],
    /// Open AL Source
    pub alSource: u32,
    /// Set to true when a sound is playing on this channel / source
    pub bPlaying: bool,
    /// Time playback of Source begins
    pub iStartTime: i32,
    /// ID of Slot rendering Source's environment (enables a send to this FXSlot)
    pub lSlotID: i32,
}

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<channel_t>() == 76808);
    assert!(core::mem::offset_of!(channel_t, startSample) == 0);
    assert!(core::mem::offset_of!(channel_t, entnum) == 4);
    assert!(core::mem::offset_of!(channel_t, entchannel) == 8);
    assert!(core::mem::offset_of!(channel_t, leftvol) == 12);
    assert!(core::mem::offset_of!(channel_t, rightvol) == 16);
    assert!(core::mem::offset_of!(channel_t, master_vol) == 20);
    assert!(core::mem::offset_of!(channel_t, origin) == 24);
    assert!(core::mem::offset_of!(channel_t, fixed_origin) == 36);
    assert!(core::mem::offset_of!(channel_t, thesfx) == 40);
    assert!(core::mem::offset_of!(channel_t, loopSound) == 48);
    assert!(core::mem::offset_of!(channel_t, MP3StreamHeader) == 56);
    assert!(core::mem::offset_of!(channel_t, MP3SlidingDecodeBuffer) == 26712);
    assert!(core::mem::offset_of!(channel_t, iMP3SlidingDecodeWritePos) == 76712);
    assert!(core::mem::offset_of!(channel_t, iMP3SlidingDecodeWindowPos) == 76716);
    assert!(core::mem::offset_of!(channel_t, bLooping) == 76720);
    assert!(core::mem::offset_of!(channel_t, bProcessed) == 76721);
    assert!(core::mem::offset_of!(channel_t, bStreaming) == 76722);
    assert!(core::mem::offset_of!(channel_t, buffers) == 76728);
    assert!(core::mem::offset_of!(channel_t, alSource) == 76792);
    assert!(core::mem::offset_of!(channel_t, bPlaying) == 76796);
    assert!(core::mem::offset_of!(channel_t, iStartTime) == 76800);
    assert!(core::mem::offset_of!(channel_t, lSlotID) == 76804);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<channel_t>() == 76760);
    assert!(core::mem::offset_of!(channel_t, startSample) == 0);
    assert!(core::mem::offset_of!(channel_t, entnum) == 4);
    assert!(core::mem::offset_of!(channel_t, entchannel) == 8);
    assert!(core::mem::offset_of!(channel_t, leftvol) == 12);
    assert!(core::mem::offset_of!(channel_t, rightvol) == 16);
    assert!(core::mem::offset_of!(channel_t, master_vol) == 20);
    assert!(core::mem::offset_of!(channel_t, origin) == 24);
    assert!(core::mem::offset_of!(channel_t, fixed_origin) == 36);
    assert!(core::mem::offset_of!(channel_t, thesfx) == 40);
    assert!(core::mem::offset_of!(channel_t, loopSound) == 44);
    assert!(core::mem::offset_of!(channel_t, MP3StreamHeader) == 48);
    assert!(core::mem::offset_of!(channel_t, MP3SlidingDecodeBuffer) == 26684);
    assert!(core::mem::offset_of!(channel_t, iMP3SlidingDecodeWritePos) == 76684);
    assert!(core::mem::offset_of!(channel_t, iMP3SlidingDecodeWindowPos) == 76688);
    assert!(core::mem::offset_of!(channel_t, bLooping) == 76692);
    assert!(core::mem::offset_of!(channel_t, bProcessed) == 76693);
    assert!(core::mem::offset_of!(channel_t, bStreaming) == 76694);
    assert!(core::mem::offset_of!(channel_t, buffers) == 76696);
    assert!(core::mem::offset_of!(channel_t, alSource) == 76744);
    assert!(core::mem::offset_of!(channel_t, bPlaying) == 76748);
    assert!(core::mem::offset_of!(channel_t, iStartTime) == 76752);
    assert!(core::mem::offset_of!(channel_t, lSlotID) == 76756);
};
