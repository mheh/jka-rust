//! `Gp2ParseError` — GP2 parse failure (SP).

/// The one way SP GP2 parsing fails: the data ran out inside an unclosed
/// `[`…`]` value list (Raven `CGPValue::Parse` returning `false`, "end of
/// data - error!"). Unlike MP, an unclosed *group* is not an error in SP —
/// `CGPGroup::Parse` checks `mParent`, which SP's `AddGroup` never sets, so
/// end of data always terminates group parsing successfully. The partially
/// built tree is kept, as in Raven.
/// Source: `oracle/oracle/code/game/genericparser2.cpp:686-741,371-394`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gp2ParseError;

impl std::fmt::Display for Gp2ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GP2 parse error: unexpected end of data")
    }
}

impl std::error::Error for Gp2ParseError {}
