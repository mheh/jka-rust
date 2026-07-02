//! `GpGroup` — one GP2 parse-tree group node.

use super::generic_parser2::GenericParser2;
use super::gp_value::GpValue;

/// Index of a group in its owning [`GenericParser2`] arena (see
/// `docs/porting-rules.md` §B5 — entities by index/handle, not pointer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpGroupId(pub(crate) u32);

impl GpGroupId {
    /// The document's base group (Raven `CGenericParser2::mTopLevel`).
    pub const TOP_LEVEL: GpGroupId = GpGroupId(0);
}

/// Arena storage for one group. Only reachable through [`GpGroup`] /
/// [`GenericParser2`].
#[derive(Debug, Clone)]
pub(crate) struct GpGroupNode {
    pub(crate) name: String,
    pub(crate) parent: Option<GpGroupId>,
    pub(crate) pairs: Vec<GpValue>,
    pub(crate) subgroups: Vec<GpGroupId>,
}

/// Raven `CGPGroup` — a named group of key/value pairs and subgroups.
///
/// C++-track idiomatic reimplementation: Raven's intrusive sibling lists
/// (`mNext` insertion chain plus the eagerly sorted `mInOrder*` chain built by
/// `SortObject`) become insertion-ordered `Vec`s in an arena owned by
/// [`GenericParser2`], with the sorted view computed on demand. This type is a
/// copyable borrow of one node, so Raven pointer walks like
/// `group->GetParent()->FindPairValue(...)` translate directly.
/// Class definition source: `oracle/oracle/codemp/qcommon/GenericParser2.h:93-134`
/// Method source: `oracle/oracle/codemp/qcommon/GenericParser2.cpp:458-844`
#[derive(Clone, Copy)]
pub struct GpGroup<'a> {
    pub(crate) doc: &'a GenericParser2,
    pub(crate) id: GpGroupId,
}

/// Raven `Q_stricmp` ordering as used by `SortObject`/`FindSubGroup`
/// (ASCII case-insensitive; Raven's `tolower` is locale-dependent for
/// bytes >= 0x80, which no shipped file exercises).
fn stricmp_key(name: &str) -> impl Iterator<Item = u8> + '_ {
    name.bytes().map(|b| b.to_ascii_lowercase())
}

impl<'a> GpGroup<'a> {
    fn node(&self) -> &'a GpGroupNode {
        self.doc.node(self.id)
    }

    /// This group's arena id within its owning [`GenericParser2`].
    pub fn id(&self) -> GpGroupId {
        self.id
    }

    /// Raven `CGPObject::GetName`.
    pub fn name(&self) -> &'a str {
        &self.node().name
    }

    /// Raven `CGPGroup::GetParent`.
    ///
    /// `None` for the top-level group. MP's `AddGroup(name, textPool)` passes
    /// `this` as the parent, so parsed subgroups always have one.
    pub fn parent(&self) -> Option<GpGroup<'a>> {
        self.node().parent.map(|id| GpGroup { doc: self.doc, id })
    }

    /// Raven `CGPGroup::GetPairs` + `GetNext` chain — pairs in insertion order.
    pub fn pairs(&self) -> impl Iterator<Item = &'a GpValue> {
        self.node().pairs.iter()
    }

    /// Raven `CGPGroup::GetInOrderPairs` + `GetInOrderNext` chain — pairs in
    /// case-insensitive name order (stable: equal names keep insertion order,
    /// matching `SortObject`'s insertion sort).
    pub fn pairs_in_order(&self) -> Vec<&'a GpValue> {
        let mut pairs: Vec<&'a GpValue> = self.node().pairs.iter().collect();
        pairs.sort_by(|a, b| stricmp_key(a.name()).cmp(stricmp_key(b.name())));
        pairs
    }

    /// Raven `CGPGroup::GetSubGroups` + `GetNext` chain — subgroups in
    /// insertion order.
    pub fn subgroups(&self) -> impl Iterator<Item = GpGroup<'a>> + '_ {
        let doc = self.doc;
        self.node().subgroups.iter().map(move |&id| GpGroup { doc, id })
    }

    /// Raven `CGPGroup::GetInOrderSubGroups` + `GetInOrderNext` chain (see
    /// [`GpGroup::pairs_in_order`]).
    pub fn subgroups_in_order(&self) -> Vec<GpGroup<'a>> {
        let mut groups: Vec<GpGroup<'a>> = self.subgroups().collect();
        groups.sort_by(|a, b| stricmp_key(a.name()).cmp(stricmp_key(b.name())));
        groups
    }

    /// Raven `CGPGroup::GetNumPairs`.
    ///
    /// Raven counts with a `do..while`, dereferencing a null head when the
    /// group has no pairs (UB); this returns 0, the only defined behavior
    /// (and what SP's `while` loop does).
    pub fn num_pairs(&self) -> usize {
        self.node().pairs.len()
    }

    /// Raven `CGPGroup::GetNumSubGroups` (same null-head caveat as
    /// [`GpGroup::num_pairs`]).
    pub fn num_sub_groups(&self) -> usize {
        self.node().subgroups.len()
    }

    /// Raven `CGPGroup::FindSubGroup` — first subgroup whose name matches
    /// case-insensitively, in insertion order.
    pub fn find_sub_group(&self, name: &str) -> Option<GpGroup<'a>> {
        self.subgroups().find(|g| g.name().eq_ignore_ascii_case(name))
    }

    /// Raven `CGPGroup::FindPair`.
    ///
    /// Raven: multiple keys may be searched if you specify `"||"` inbetween
    /// each key name in the string; the first key to be found (from left to
    /// right) will be returned.
    pub fn find_pair(&self, key: &str) -> Option<&'a GpValue> {
        let mut pos = key;
        while !pos.is_empty() {
            let (segment, next) = match pos.find("||") {
                Some(i) => (&pos[..i], &pos[i + 2..]),
                None => (pos, &pos[pos.len()..]),
            };

            for pair in self.pairs() {
                if pair.name().eq_ignore_ascii_case(segment) {
                    return Some(pair);
                }
            }

            pos = next;
        }

        None
    }

    /// Raven `CGPGroup::FindPairValue`, minus the `defaultVal` parameter:
    /// callers write `.unwrap_or(default)`. Raven returns the found pair's
    /// `GetTopValue()` even when that is null (a valueless `key []` pair),
    /// handing callers a null they invariably dereference; folding that case
    /// into `None` is the only defined behavior.
    pub fn find_pair_value(&self, key: &str) -> Option<&'a str> {
        self.find_pair(key).and_then(GpValue::top_value)
    }
}
