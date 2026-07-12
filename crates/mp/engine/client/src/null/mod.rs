//! `oracle/codemp/null/` — the DEDICATED/no-renderer, no-sound stub
//! implementations of the client/renderer/input/sound-DMA entry points. Every
//! Raven body here is an intentional no-op (or a fixed sentinel return); this
//! module is the faithful transcription of that whole tree, one file per
//! oracle source file.

pub mod null_client;
pub mod null_glimp;
pub mod null_input;
pub mod null_renderer;
pub mod null_snddma;
