//! GP2 ("Generic Parser 2") — Raven's `{`/`[` text format parser, used by the
//! FX system, ambient music sets, and terrain/RMG instance files.
//!
//! C++-track idiomatic reimplementation of `oracle/oracle/codemp/qcommon/GenericParser2.{h,cpp}`:
//!
//! - `CGenericParser2` → [`generic_parser2::GenericParser2`] (owns all nodes in
//!   an arena), `CGPGroup` → [`gp_group::GpGroup`] (id + borrow wrapper),
//!   `CGPValue` → [`gp_value::GpValue`].
//! - `CTextPool` (bump allocator for token text) and `CGPObject` (intrusive
//!   linked-list base) have no counterpart — nodes own `String`s and `Vec`s.
//! - The C handle interface (`GP_*`/`GPG_*`/`GPV_*`,
//!   `GenericParser2.h:162-201`) has zero callers in either tree and is not
//!   ported; its opaque handle typedefs stay in `tgeneric_parser2`/`tgpgroup`/
//!   `tgpvalue` as the record of that seam.

pub mod generic_parser2;
pub mod gp2_parse_error;
pub mod gp_group;
pub mod gp_value;
pub(crate) mod tokenizer;

pub mod tgeneric_parser2;
pub mod tgpgroup;
pub mod tgpvalue;
