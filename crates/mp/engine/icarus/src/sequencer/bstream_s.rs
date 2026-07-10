//! Raven `bstream_t` — an ICARUS block-stream stack node.

use crate::blockstream::cblock_stream::BlockStream;

/// Raven `bstream_t` → `Bstream` (§F idiomatic, ICARUS-D1 naming).
///
/// A stream-stack node. `stream` owns Raven's `CBlockStream *stream`; Raven's
/// intrusive `bstream_s *last` back-pointer folds to an **index** (`last`) into
/// the sequencer's owned `Vec<Bstream>` (`m_streamsCreated`) — `None` mirrors
/// Raven's top-level `last == NULL` (the fold the frozen shape called for; the
/// stream stack cannot be reconstructed from Vec order alone once `DeleteStream`
/// removes an interior node).
/// Type definition source: `oracle/codemp/icarus/sequencer.h:42-46`
#[derive(Default)]
pub struct Bstream {
    /// Raven `CBlockStream *stream` — owned here.
    pub stream: BlockStream,
    /// Raven `bstream_s *last` — index of the previous stream node in
    /// `m_streamsCreated`, or `None` at the top level (`last == NULL`).
    pub last: Option<usize>,
}
