//! `Mp3Decoder` — the replaced MPEG audio decoder (DEC-03, DEC-57.3).
//!
//! Raven's `codemp/mp3code` C decoder does not port. This type is the one place
//! the replacement crate is called, so `snd_mp3` keeps Raven's wrap logic and
//! knows nothing about the decoder behind it.

#![allow(non_snake_case)]

use core::ffi::c_int;

use symphonia_bundle_mp3::MpaDecoder;
use symphonia_core::codecs::audio::well_known::CODEC_ID_MP3;
use symphonia_core::codecs::audio::{AudioCodecParameters, AudioDecoder, AudioDecoderOptions};
use symphonia_core::packet::PacketRef;
use symphonia_core::units::{Duration, Timestamp};

use crate::mp3::mp3_frame_header::Mp3FrameHeader;

/// One decoded frame: interleaved 16-bit samples at the source rate.
pub struct Mp3DecodedFrame<'a> {
    pub samples: &'a [i16],
    pub sampleRate: c_int,
    pub channels: c_int,
}

/// The replaced decoder, one instance per live MP3 stream.
///
/// The decoder carries the bit-reservoir state between frames, so a stream that
/// restarts builds a fresh one. `Clone` therefore yields an empty decoder, which
/// is what Raven's `MP3Stream_Rewind` does anyway when it copies the pristine
/// header back over the working one.
pub struct Mp3Decoder {
    decoder: Option<MpaDecoder>,
    pcm: Vec<i16>,
}

impl Mp3Decoder {
    pub fn new() -> Mp3Decoder {
        Mp3Decoder {
            decoder: None,
            pcm: Vec::new(),
        }
    }

    /// Throw the decode state away, so the next frame starts a fresh stream.
    pub fn reset(&mut self) {
        self.decoder = None;
        self.pcm.clear();
    }

    /// Decode exactly one frame.
    ///
    /// `frame` must start on the sync word and be exactly `frameBytes` long,
    /// which is what the caller's `Mp3FrameHeader` walk hands over. A frame the
    /// decoder rejects answers `None`, and Raven's wrap reads that as end of
    /// stream.
    pub fn decode(&mut self, frame: &[u8]) -> Option<Mp3DecodedFrame<'_>> {
        let head = Mp3FrameHeader::parse(frame)?;
        if frame.len() != head.frameBytes {
            return None;
        }

        if self.decoder.is_none() {
            let mut params = AudioCodecParameters::new();
            params.for_codec(CODEC_ID_MP3);
            let opts = AudioDecoderOptions::default();
            self.decoder = Some(MpaDecoder::try_new(&params, &opts).ok()?);
        }
        let decoder = self.decoder.as_mut()?;

        let packet = PacketRef::new(
            0,
            Timestamp::ZERO,
            Duration::from(head.samplesPerFrame as u64),
            frame,
        );
        let buffer = decoder.decode_ref(&packet).ok()?;

        self.pcm.clear();
        buffer.copy_to_vec_interleaved(&mut self.pcm);

        Some(Mp3DecodedFrame {
            samples: &self.pcm,
            sampleRate: buffer.spec().rate() as c_int,
            channels: buffer.num_planes() as c_int,
        })
    }
}

impl Default for Mp3Decoder {
    fn default() -> Mp3Decoder {
        Mp3Decoder::new()
    }
}

impl Clone for Mp3Decoder {
    /// A copied stream starts its decode over. See the type doc.
    fn clone(&self) -> Mp3Decoder {
        Mp3Decoder::new()
    }
}
