#![allow(non_camel_case_types)]

/// Raven `CRMAutomapSymbol` — automap symbol drawn for an RMG instance.
///
/// Type definition source: `oracle/oracle/code/Rmg/RM_Instance.h:13-23`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CRMAutomapSymbol {
    AUTOMAP_NONE = 0,
    AUTOMAP_BLD = 1,
    AUTOMAP_OBJ = 2,
    AUTOMAP_START = 3,
    AUTOMAP_END = 4,
    AUTOMAP_ENEMY = 5,
    AUTOMAP_FRIEND = 6,
    AUTOMAP_WALL = 7,
}

const _: () = assert!(core::mem::size_of::<CRMAutomapSymbol>() == 4);
