//! GP2 ("Generic Parser 2") — Raven's `{`/`[` text format parser, used by the
//! FX system, ambient music sets, and terrain/RMG instance files. SP's copy
//! lives in `code/game/` and is included by SP `q_shared.h`, hence this crate.
//!
//! C++-track idiomatic reimplementation of `oracle/code/game/genericparser2.{h,cpp}`:
//!
//! - `CGenericParser2` → [`generic_parser2::GenericParser2`] (owns all nodes in
//!   an arena), `CGPGroup` → [`gp_group::GpGroup`] (id + borrow wrapper),
//!   `CGPValue` → [`gp_value::GpValue`].
//! - `CTextPool` (bump allocator for token text) and `CGPObject` (intrusive
//!   linked-list base) have no counterpart — nodes own `String`s and `Vec`s.
//! - The C handle interface (`GP_*`/`GPG_*`/`GPV_*`,
//!   `genericparser2.h:170-202`) has zero callers in either tree and is not
//!   ported; its opaque handle typedefs stay in `tgeneric_parser2`/`tgpgroup`/
//!   `tgpvalue` as the record of that seam.
//!
//! SP diverges from MP in three observable ways, each preserved here:
//! `FindPair` is a plain case-insensitive match (no `"||"` multi-key search),
//! `AddGroup` never sets the subgroup's parent (so `GetParent` is always null
//! and unclosed groups parse successfully), and `GetNumPairs`/
//! `GetNumSubGroups` tolerate empty lists (MP's `do..while` does not).

pub mod generic_parser2;
pub mod gp2_parse_error;
pub mod gp_group;
pub mod gp_value;

pub mod tgeneric_parser2;
pub mod tgpgroup;
pub mod tgpvalue;
