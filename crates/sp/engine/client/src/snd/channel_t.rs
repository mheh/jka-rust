#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::common::sp::game::sound_channel_t::soundChannel_t;
use sp_qshared::shared::{qboolean, vec3_t};

use crate::mp3::mp3_stream::MP3STREAM;
use crate::snd::sfx_s::sfx_t;
use crate::snd::streamingbuffer::STREAMINGBUFFER;

/// `NUM_STREAMING_BUFFERS` — OpenAL streaming buffers per channel.
///
/// Source: `oracle/code/client/snd_local.h:87`
pub const NUM_STREAMING_BUFFERS: usize = 4;

/// Raven `channel_t` — a live sound-playback channel (mixer slot / OpenAL source).
///
/// Raven: back-indented fields new in TA codebase, will re-format when MP3 code finished -ste;
/// note: field missing in TA: `qboolean loopSound;` // from an `S_AddLoopSound` call, cleared
/// each frame.
/// Type definition source: `oracle/code/client/snd_local.h:94-129`
#[repr(C)]
pub struct channel_t {
    /// START_SAMPLE_IMMEDIATE = set immediately on next mix
    pub startSample: i32,
    /// to allow overriding a specific sound
    pub entnum: i32,
    /// to allow overriding a specific sound
    pub entchannel: soundChannel_t,
    /// 0-255 volume after spatialization
    pub leftvol: i32,
    /// 0-255 volume after spatialization
    pub rightvol: i32,
    /// 0-255 volume before spatialization
    pub master_vol: i32,

    /// only use if fixed_origin is set
    pub origin: vec3_t,

    /// use origin instead of fetching entnum's origin
    pub fixed_origin: qboolean,
    /// sfx structure
    pub thesfx: *mut sfx_t,
    /// from an S_AddLoopSound call, cleared each frame
    pub loopSound: qboolean,
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

const _: () = assert!(core::mem::size_of::<channel_t>() == 76808);
const _: () = assert!(core::mem::offset_of!(channel_t, startSample) == 0);
const _: () = assert!(core::mem::offset_of!(channel_t, entnum) == 4);
const _: () = assert!(core::mem::offset_of!(channel_t, entchannel) == 8);
const _: () = assert!(core::mem::offset_of!(channel_t, leftvol) == 12);
const _: () = assert!(core::mem::offset_of!(channel_t, rightvol) == 16);
const _: () = assert!(core::mem::offset_of!(channel_t, master_vol) == 20);
const _: () = assert!(core::mem::offset_of!(channel_t, origin) == 24);
const _: () = assert!(core::mem::offset_of!(channel_t, fixed_origin) == 36);
const _: () = assert!(core::mem::offset_of!(channel_t, thesfx) == 40);
const _: () = assert!(core::mem::offset_of!(channel_t, loopSound) == 48);
const _: () = assert!(core::mem::offset_of!(channel_t, MP3StreamHeader) == 56);
const _: () = assert!(core::mem::offset_of!(channel_t, MP3SlidingDecodeBuffer) == 26712);
const _: () = assert!(core::mem::offset_of!(channel_t, iMP3SlidingDecodeWritePos) == 76712);
const _: () = assert!(core::mem::offset_of!(channel_t, iMP3SlidingDecodeWindowPos) == 76716);
const _: () = assert!(core::mem::offset_of!(channel_t, bLooping) == 76720);
const _: () = assert!(core::mem::offset_of!(channel_t, bProcessed) == 76721);
const _: () = assert!(core::mem::offset_of!(channel_t, bStreaming) == 76722);
const _: () = assert!(core::mem::offset_of!(channel_t, buffers) == 76728);
const _: () = assert!(core::mem::offset_of!(channel_t, alSource) == 76792);
const _: () = assert!(core::mem::offset_of!(channel_t, bPlaying) == 76796);
const _: () = assert!(core::mem::offset_of!(channel_t, iStartTime) == 76800);
const _: () = assert!(core::mem::offset_of!(channel_t, lSlotID) == 76804);
