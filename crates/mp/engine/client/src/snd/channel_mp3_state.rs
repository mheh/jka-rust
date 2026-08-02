//! `ChannelMp3State` — the MP3 block Raven keeps inside `channel_t`.
//!
//! Raven embeds the stream header, a 50000-byte sliding decode window, and the
//! two window cursors in every one of the 32 channels. Keeping them in
//! `channel_t` would cost the port its `Copy`, which the mixer relies on, so
//! `SoundSystem` holds one of these per channel beside `s_channels` and the
//! background-music tracks hold their own.
//!
//! Type definition source: `oracle/codemp/client/snd_local.h:110-114`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use crate::mp3::mp3_stream_state::MP3StreamState;

/// Raven `MP3SlidingDecodeBuffer[50000]`.
///
/// Raven: typical back-request is -3072, so roughly double is 6000 for safety,
/// then doubled again so the 6K position is in the middle of the buffer.
/// Source: `oracle/codemp/client/snd_local.h:112`
pub const MP3_SLIDING_DECODE_BUFFER_BYTES: usize = 50000;

/// One channel's MP3 decode window.
#[derive(Clone)]
pub struct ChannelMp3State {
    pub MP3StreamHeader: MP3StreamState,
    pub MP3SlidingDecodeBuffer: Vec<u8>,
    pub iMP3SlidingDecodeWritePos: c_int,
    pub iMP3SlidingDecodeWindowPos: c_int,
}

impl Default for ChannelMp3State {
    fn default() -> ChannelMp3State {
        ChannelMp3State {
            MP3StreamHeader: MP3StreamState::default(),
            MP3SlidingDecodeBuffer: vec![0u8; MP3_SLIDING_DECODE_BUFFER_BYTES],
            iMP3SlidingDecodeWritePos: 0,
            iMP3SlidingDecodeWindowPos: 0,
        }
    }
}

impl ChannelMp3State {
    /// Raven `Channel_Clear` skips the sliding buffer and zeroes everything
    /// around it, so a cleared channel keeps its window allocation.
    /// Source: `oracle/codemp/client/snd_dma.cpp:321-330`
    pub fn clear(&mut self) {
        self.MP3StreamHeader = MP3StreamState::default();
        self.iMP3SlidingDecodeWritePos = 0;
        self.iMP3SlidingDecodeWindowPos = 0;
    }
}
