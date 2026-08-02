//! `SfxSampleData` — the block Raven hangs off `sfx_t::pSoundData`.
//!
//! Raven types the block `short *` and then puts one of two things in it: the
//! resampled 16-bit samples of a WAV, or the raw file image of an MP3 the stream
//! decoder reads. `Z_Size` tells the two apart by byte count only. The port
//! names the two cases instead, which keeps the byte count exact for both.
//!
//! Source: `oracle/codemp/client/snd_local.h:47`,
//! `oracle/codemp/client/snd_mp3.cpp:269-270`

/// The sample block of one sound.
///
/// - `Pcm`: the `ct_16` arm, resampled to `dma.speed`, one `i16` per sample.
/// - `Mp3`: the `ct_MP3` arm, the raw MP3 file image the stream decoder reads.
#[derive(Clone)]
pub enum SfxSampleData {
    Pcm(Vec<i16>),
    Mp3(Vec<u8>),
}

impl SfxSampleData {
    /// Raven `Z_Size(sfx->pSoundData)` — the bytes the block holds.
    pub fn byte_len(&self) -> usize {
        match self {
            SfxSampleData::Pcm(data) => data.len() * 2,
            SfxSampleData::Mp3(data) => data.len(),
        }
    }

    /// The 16-bit samples, and `None` on the MP3 arm, which has none until the
    /// stream decoder unpacks a packet.
    pub fn pcm(&self) -> Option<&[i16]> {
        match self {
            SfxSampleData::Pcm(data) => Some(data),
            SfxSampleData::Mp3(_) => None,
        }
    }

    /// The 16-bit samples for writing.
    pub fn pcm_mut(&mut self) -> Option<&mut [i16]> {
        match self {
            SfxSampleData::Pcm(data) => Some(data),
            SfxSampleData::Mp3(_) => None,
        }
    }

    /// The raw MP3 file image, and `None` on the PCM arm.
    pub fn mp3(&self) -> Option<&[u8]> {
        match self {
            SfxSampleData::Pcm(_) => None,
            SfxSampleData::Mp3(data) => Some(data),
        }
    }
}
