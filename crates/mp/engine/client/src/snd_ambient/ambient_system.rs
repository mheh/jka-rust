//! `AmbientSystem` — the owned home of every `snd_ambient.cpp` file-scope global.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;
use std::collections::BTreeMap;

use crate::snd_ambient::c_set_group::CSetGroup;

/// Raven's ambient-set globals: the set container, the precache list, the
/// cross-fade pair, and the parse cursor.
///
/// `aSets` is `None` until `AS_Init` runs, which is Raven's null pointer. The
/// parse cursor is here rather than local because Raven's parse helpers read it
/// out of file scope and `tempBuffer` carries the last token between them.
/// Source: `oracle/codemp/client/snd_ambient.cpp:22-43`
pub struct AmbientSystem {
    /// Raven `aSets` — the main ambient sound group.
    pub aSets: Option<CSetGroup>,
    /// Raven `pMap` — the precache list `AS_AddPrecacheEntry` fills.
    pub pMap: BTreeMap<String, u8>,
    /// Raven `currentSet` / `oldSet` — the cross-fading pair, -1 for none.
    pub currentSet: c_int,
    pub oldSet: c_int,
    /// Raven `crossDelay` — the fade length in milliseconds.
    pub crossDelay: c_int,
    /// Raven `currentSetTime` / `oldSetTime` — when each set last fired a subwave.
    pub currentSetTime: c_int,
    pub oldSetTime: c_int,
    /// Raven `numSets` — kept for debug purposes only.
    pub numSets: c_int,
    /// Raven `parseBuffer` / `parseSize` / `parsePos`, the file-scope parse cursor.
    pub parseBuffer: Vec<u8>,
    pub parseSize: c_int,
    pub parsePos: c_int,
    /// Raven `tempBuffer[1024]` — the last token a parse helper scanned.
    pub tempBuffer: String,
}

impl AmbientSystem {
    /// The C loader's zero fill, with Raven's three non-zero initialisers.
    /// Source: `oracle/codemp/client/snd_ambient.cpp:22-24`
    pub fn new() -> AmbientSystem {
        AmbientSystem {
            aSets: None,
            pMap: BTreeMap::new(),
            currentSet: -1,
            oldSet: -1,
            crossDelay: 1000,
            currentSetTime: 0,
            oldSetTime: 0,
            numSets: 0,
            parseBuffer: Vec::new(),
            parseSize: 0,
            parsePos: 0,
            tempBuffer: String::new(),
        }
    }
}

impl Default for AmbientSystem {
    fn default() -> AmbientSystem {
        AmbientSystem::new()
    }
}
