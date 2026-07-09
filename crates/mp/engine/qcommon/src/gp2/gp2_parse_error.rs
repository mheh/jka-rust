//! `Gp2ParseError` — GP2 parse failure.

/// The one way GP2 parsing fails: the data ran out inside an unclosed group or
/// `[`…`]` value list (Raven `CGPGroup::Parse` / `CGPValue::Parse` returning
/// `false`, "end of data - error!"). The partially built tree is kept, as in
/// Raven.
/// Source: `oracle/codemp/qcommon/GenericParser2.cpp:679-734,362-385`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gp2ParseError;

impl std::fmt::Display for Gp2ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GP2 parse error: unexpected end of data")
    }
}

impl std::error::Error for Gp2ParseError {}
