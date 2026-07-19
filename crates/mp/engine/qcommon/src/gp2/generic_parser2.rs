//! `GenericParser2` — the GP2 document: parser plus group arena.

use native_string::gp2_tokenizer::Tokenizer;

use super::gp2_parse_error::Gp2ParseError;
use super::gp_group::{GpGroup, GpGroupId, GpGroupNode};
use super::gp_value::GpValue;

/// Raven `CGenericParser2` — parses Raven's `{`/`[` text format (effects,
/// ambient music sets, terrain/RMG instance files) into a group tree.
///
/// C++-track idiomatic reimplementation. This type owns every group node in an
/// arena ([`GpGroupId`] indices; [`GpGroup`] is a borrow of one node), so
/// Raven's parent pointers need no aliasing. `CTextPool` (bump allocation for
/// token text) has no counterpart — nodes own `String`s. Raven's write-only
/// `mWriteable` flag is read by nothing in either tree and is dropped, and
/// `CGPGroup::Duplicate`/`CGPValue::Duplicate` collapse to `Clone` of the
/// document. A failed parse keeps the partially built tree, as in Raven.
/// Class definition source: `oracle/codemp/qcommon/GenericParser2.h:136-158`
/// Method source: `oracle/codemp/qcommon/GenericParser2.cpp:860-914`
#[derive(Debug, Clone)]
pub struct GenericParser2 {
    pub(crate) groups: Vec<GpGroupNode>,
}

impl Default for GenericParser2 {
    fn default() -> Self {
        Self::new()
    }
}

impl GenericParser2 {
    pub fn new() -> Self {
        GenericParser2 {
            // Raven `CGPGroup`'s default group name.
            groups: vec![GpGroupNode {
                name: "Top Level".to_string(),
                parent: None,
                pairs: Vec::new(),
                subgroups: Vec::new(),
            }],
        }
    }

    pub(crate) fn node(&self, id: GpGroupId) -> &GpGroupNode {
        &self.groups[id.0 as usize]
    }

    fn node_mut(&mut self, id: GpGroupId) -> &mut GpGroupNode {
        &mut self.groups[id.0 as usize]
    }

    /// Raven `CGenericParser2::GetBaseParseGroup`.
    pub fn top_level(&self) -> GpGroup<'_> {
        GpGroup {
            doc: self,
            id: GpGroupId::TOP_LEVEL,
        }
    }

    /// Raven `CGenericParser2::Clean` — drop every group and pair; the top
    /// level survives, empty.
    pub fn clean(&mut self) {
        self.groups.truncate(1);
        let top = self.node_mut(GpGroupId::TOP_LEVEL);
        top.pairs.clear();
        top.subgroups.clear();
    }

    /// Raven `CGPGroup::AddGroup(name, textPool)` — appends a subgroup and
    /// returns its id. MP passes `this` as the new group's parent (SP does
    /// not — see the SP port).
    pub fn add_group(&mut self, parent: GpGroupId, name: impl Into<String>) -> GpGroupId {
        let id = GpGroupId(self.groups.len() as u32);
        self.groups.push(GpGroupNode {
            name: name.into(),
            parent: Some(parent),
            pairs: Vec::new(),
            subgroups: Vec::new(),
        });
        self.node_mut(parent).subgroups.push(id);
        id
    }

    /// Raven `CGPGroup::AddPair(name, value, textPool)` — appends a pair,
    /// with one initial value if given. Returns the pair's index within the
    /// group, in insertion order.
    pub fn add_pair(
        &mut self,
        group: GpGroupId,
        name: impl Into<String>,
        value: Option<&str>,
    ) -> usize {
        let mut pair = GpValue::new(name);
        if let Some(value) = value {
            pair.add_value(value);
        }
        let pairs = &mut self.node_mut(group).pairs;
        pairs.push(pair);
        pairs.len() - 1
    }

    /// Raven `CGenericParser2::Parse`. `clean_first` mirrors the `cleanFirst`
    /// parameter (`false` parses additional text into the existing tree); the
    /// no-op `writeable` parameter is dropped.
    pub fn parse(&mut self, text: &str, clean_first: bool) -> Result<(), Gp2ParseError> {
        if clean_first {
            self.clean();
        }

        let mut tokenizer = Tokenizer::new(text);
        self.parse_group(GpGroupId::TOP_LEVEL, &mut tokenizer)
    }

    /// Raven `CGPGroup::Parse`.
    fn parse_group(
        &mut self,
        group: GpGroupId,
        tokenizer: &mut Tokenizer,
    ) -> Result<(), Gp2ParseError> {
        loop {
            let token = tokenizer.get_token(true, false);

            if token.is_empty() {
                // end of data - error! (unless this is the top level: Raven
                // checks mParent, which parsing always sets here in MP)
                if self.node(group).parent.is_some() {
                    return Err(Gp2ParseError);
                }
                break;
            } else if token == "}" {
                // ending brace for this group
                break;
            }

            let last_token = token;

            // read ahead to see what we are doing
            let token = tokenizer.get_token(true, true);
            if token == "{" {
                // new sub group
                let sub_group = self.add_group(group, last_token);
                self.parse_group(sub_group, tokenizer)?;
            } else if token == "[" {
                // new pair list
                let pair = self.add_pair(group, last_token, None);
                self.parse_value_list(group, pair, tokenizer)?;
            } else {
                // new pair
                self.add_pair(group, last_token, Some(&token));
            }
        }

        Ok(())
    }

    /// Raven `CGPValue::Parse` — the `[`…`]` value list: one value per line,
    /// until a line reading `]`.
    fn parse_value_list(
        &mut self,
        group: GpGroupId,
        pair: usize,
        tokenizer: &mut Tokenizer,
    ) -> Result<(), Gp2ParseError> {
        loop {
            let token = tokenizer.get_token(true, true);

            if token.is_empty() {
                // end of data - error!
                return Err(Gp2ParseError);
            } else if token == "]" {
                // ending brace for this list
                break;
            }

            self.node_mut(group).pairs[pair].add_value(token);
        }

        Ok(())
    }

    /// Raven `CGenericParser2::Write` — regenerate the document text (CRLF
    /// line ends, tab indentation, values quoted when they contain a space or
    /// are empty). Raven streams into a `CTextPool`; this returns the text.
    pub fn write(&self) -> String {
        let mut out = String::new();
        self.write_group(GpGroupId::TOP_LEVEL, -1, &mut out);
        out
    }

    /// Raven `CGPGroup::Write` (depth -1 = top level: children only, no braces).
    fn write_group(&self, id: GpGroupId, depth: i32, out: &mut String) {
        let node = self.node(id);

        if depth >= 0 {
            write_tabs(out, depth);
            write_text(out, &node.name);
            out.push_str("\r\n");
            write_tabs(out, depth);
            out.push_str("{\r\n");
        }

        for pair in &node.pairs {
            write_pair(pair, depth + 1, out);
        }
        for &sub_group in &node.subgroups {
            self.write_group(sub_group, depth + 1, out);
        }

        if depth >= 0 {
            write_tabs(out, depth);
            out.push_str("}\r\n");
        }
    }
}

/// Raven `CGPValue::Write` — valueless pairs are skipped; a single value is
/// written inline, several as a `[`…`]` list.
fn write_pair(pair: &GpValue, depth: i32, out: &mut String) {
    let values: Vec<&str> = pair.values().collect();
    if values.is_empty() {
        return;
    }

    write_tabs(out, depth);
    write_text(out, pair.name());

    if let [value] = values[..] {
        out.push_str("\t\t");
        write_text(out, value);
        out.push_str("\r\n");
    } else {
        out.push_str("\r\n");
        write_tabs(out, depth);
        out.push_str("[\r\n");
        for value in values {
            write_tabs(out, depth + 1);
            write_text(out, value);
            out.push_str("\r\n");
        }
        write_tabs(out, depth);
        out.push_str("]\r\n");
    }
}

/// Raven `CGPObject::WriteText` — quote text containing a space (only a
/// space: `strchr(text, ' ')`) and empty text.
fn write_text(out: &mut String, text: &str) {
    if text.contains(' ') || text.is_empty() {
        out.push('"');
        out.push_str(text);
        out.push('"');
    } else {
        out.push_str(text);
    }
}

fn write_tabs(out: &mut String, depth: i32) {
    for _ in 0..depth {
        out.push('\t');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
// a comment\n\
outer\n\
{\n\
\tname\t\tvalue one\n\
\tcount\t3\n\
\tinner\n\
\t{\n\
\t\tlist\n\
\t\t[\n\
\t\t\tfirst value\n\
\t\t\tsecond\n\
\t\t]\n\
\t}\n\
}\n";

    fn parsed(text: &str) -> GenericParser2 {
        let mut parser = GenericParser2::new();
        parser.parse(text, true).expect("parse failed");
        parser
    }

    #[test]
    fn parses_groups_pairs_and_lists() {
        let parser = parsed(SAMPLE);
        let top = parser.top_level();
        assert_eq!(top.name(), "Top Level");
        assert_eq!(top.num_sub_groups(), 1);

        let outer = top.find_sub_group("OUTER").expect("outer");
        assert_eq!(outer.find_pair_value("name"), Some("value one"));
        assert_eq!(outer.find_pair_value("count"), Some("3"));
        assert_eq!(outer.find_pair_value("missing"), None);

        let inner = outer.find_sub_group("inner").expect("inner");
        let list = inner.find_pair("list").expect("list");
        assert!(list.is_list());
        assert_eq!(list.values().collect::<Vec<_>>(), ["first value", "second"]);
        assert_eq!(list.top_value(), Some("first value"));

        // MP sets parents while parsing.
        assert_eq!(inner.parent().expect("parent").name(), "outer");
        assert_eq!(outer.parent().expect("parent").id(), GpGroupId::TOP_LEVEL);
        assert!(parser.top_level().parent().is_none());
    }

    #[test]
    fn values_run_to_end_of_line() {
        let parser = parsed("key a b c // trailing comment\n");
        // The tokenizer keeps the space before the comment (Raven trims only
        // bytes below ' ').
        assert_eq!(parser.top_level().find_pair_value("key"), Some("a b c "));
    }

    #[test]
    fn find_pair_searches_multiple_keys() {
        let parser = parsed("beta 2\nalpha 1\n");
        let top = parser.top_level();
        assert_eq!(top.find_pair_value("alpha||beta"), Some("1"));
        assert_eq!(top.find_pair_value("missing||beta"), Some("2"));
        assert_eq!(top.find_pair_value("missing||absent"), None);
        assert_eq!(top.find_pair_value(""), None);
    }

    #[test]
    fn in_order_views_sort_case_insensitively_and_stably() {
        let parser = parsed("b 1\nA 2\na 3\nC 4\n");
        let names: Vec<&str> = parser
            .top_level()
            .pairs_in_order()
            .iter()
            .map(|p| p.name())
            .collect();
        assert_eq!(names, ["A", "a", "b", "C"]);
        // Insertion order is unaffected.
        let names: Vec<&str> = parser.top_level().pairs().map(|p| p.name()).collect();
        assert_eq!(names, ["b", "A", "a", "C"]);
    }

    #[test]
    fn unclosed_group_is_an_error_in_mp() {
        let mut parser = GenericParser2::new();
        assert_eq!(
            parser.parse("outer\n{\nkey value\n", true),
            Err(Gp2ParseError)
        );
        // The partial tree survives, as in Raven.
        assert_eq!(parser.top_level().num_sub_groups(), 1);
    }

    #[test]
    fn unclosed_list_is_an_error() {
        let mut parser = GenericParser2::new();
        assert_eq!(parser.parse("key\n[\nvalue\n", true), Err(Gp2ParseError));
    }

    #[test]
    fn clean_and_reparse() {
        let mut parser = parsed(SAMPLE);
        parser.parse("solo pair\n", true).expect("reparse");
        let top = parser.top_level();
        assert_eq!(top.num_sub_groups(), 0);
        assert_eq!(top.find_pair_value("solo"), Some("pair"));

        // clean_first = false appends instead.
        parser.parse("more stuff\n", false).expect("append");
        assert_eq!(parser.top_level().find_pair_value("solo"), Some("pair"));
        assert_eq!(parser.top_level().find_pair_value("more"), Some("stuff"));
    }

    #[test]
    fn write_regenerates_the_document() {
        let parser = parsed(SAMPLE);
        assert_eq!(
            parser.write(),
            "outer\r\n{\r\n\tname\t\t\"value one\"\r\n\tcount\t\t3\r\n\tinner\r\n\t{\r\n\t\tlist\r\n\t\t[\r\n\t\t\t\"first value\"\r\n\t\t\tsecond\r\n\t\t]\r\n\t}\r\n}\r\n"
        );
    }
}
