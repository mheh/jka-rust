//! `ScriptDef` — Raven `scriptDef_t`.

/// Raven `#define MAX_SCRIPT_ARGS 12`.
///
/// Source: `oracle/codemp/ui/ui_shared.h:76`
pub const MAX_SCRIPT_ARGS: usize = 12;

/// Raven `scriptDef_t` — a UI script command plus its argument list.
///
/// PORT-NOTE: Raven's `const char *args[MAX_SCRIPT_ARGS]` slots were
/// `String_Alloc` pool pointers with the unused tail left NULL; the owned
/// `Vec<String>` carries the live arguments only.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:106-109`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[doc(alias = "scriptDef_t")]
pub struct ScriptDef {
    pub command: String,
    pub args: Vec<String>,
}
