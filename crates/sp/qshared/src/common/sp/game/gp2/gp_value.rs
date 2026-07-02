//! `GpValue` — one GP2 key/value(s) pair (SP).

/// Raven `CGPValue` — a named pair holding one value or a `[`…`]` value list.
///
/// C++-track idiomatic reimplementation: Raven chains `CGPObject` string nodes
/// through `mList`/`mNext` (with `mInOrderNext` doubling as a tail cache in
/// `AddValue`); the values are simply an insertion-ordered `Vec<String>` here.
/// Class definition source: `oracle/oracle/code/game/genericparser2.h:76-97`
/// Method source: `oracle/oracle/code/game/genericparser2.cpp:276-450`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpValue {
    name: String,
    values: Vec<String>,
}

impl GpValue {
    /// Raven `CGPValue::CGPValue` (without the optional initial value).
    pub fn new(name: impl Into<String>) -> Self {
        GpValue {
            name: name.into(),
            values: Vec::new(),
        }
    }

    /// Raven `CGPObject::GetName`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Raven `CGPValue::IsList` — true only for two or more values.
    pub fn is_list(&self) -> bool {
        self.values.len() >= 2
    }

    /// Raven `CGPValue::GetTopValue` — the first value, if any.
    pub fn top_value(&self) -> Option<&str> {
        self.values.first().map(String::as_str)
    }

    /// Raven `CGPValue::GetList` — all values, in insertion order.
    pub fn values(&self) -> impl Iterator<Item = &str> {
        self.values.iter().map(String::as_str)
    }

    /// Raven `CGPValue::AddValue`.
    pub fn add_value(&mut self, value: impl Into<String>) {
        self.values.push(value.into());
    }
}
