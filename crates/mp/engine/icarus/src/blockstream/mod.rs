//! MP ICARUS `blockstream.h` block-stream types (§F idiomatic reimplementation).
//!
//! The reader/build half ports in full; the **writer/duplicator half is
//! §20-dropped — exactly FIVE zero-caller methods** (porting-rules §20,
//! ICARUS-D1 / ruling 40; the dedicated server never writes `.IBI`):
//! - `CBlockStream::Create( char * )` (`blockstream.h:167`, def
//!   `BlockStream.cpp:525`) — zero callers.
//! - `CBlockStream::WriteBlock` (`blockstream.h:174`, def `:577`) — only callers
//!   in the out-of-set `Interpreter.cpp`.
//! - `CBlockMember::WriteMember` (`blockstream.h:47`, def `:133`) — only caller is
//!   the dropped `WriteBlock`.
//! - `CBlock::Duplicate` (`blockstream.h:138`, def `:359`) — zero callers.
//! - `CBlockMember::Duplicate` (`blockstream.h:74`, def `:148`) — only caller is the
//!   dead `CBlock::Duplicate`, so transitively caller-less.
//!
//! Not dropped (live, ICARUS-D1): `CBlockStream::Init` (`Open` calls it), the
//! `CBlock::Write` overloads, and `CBlockMember::SetData`/`WriteData`/
//! `WriteDataPointer` — all in-memory builders on the parse path.

pub mod cblock;
pub mod cblock_member;
pub mod cblock_stream;
pub mod file;
pub mod vector_t;
