#![allow(non_camel_case_types, non_snake_case)]

/// Raven `q3print_t` — print message types.
///
/// Raven: bk001201 - warning: useless keyword or type name in empty declaration.
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:2373-2377`
#[repr(i32)]
pub enum q3print_t {
    SYSTEM_PRINT = 0,
    CHAT_PRINT = 1,
    TEAMCHAT_PRINT = 2,
}
