//! Raven `CSetGroup` — the container every parsed ambient set lands in.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;
use std::collections::BTreeMap;

use crate::snd_ambient::ambient_set_s::{ambientSet_t, MAX_SET_NAME_LENGTH};

/// The ambient sets of one sound file, by insertion order and by name.
///
/// Raven holds `vector<ambientSet_t *>` beside `map<sstring_t, ambientSet_t *>`
/// with both pointing at the same blocks, so a name lookup and an id lookup
/// answer the same set. The port owns the sets in the vector and keeps slot
/// indices in the map (porting-rules §B5).
/// Class definition source: `oracle/codemp/client/snd_ambient.h:80-104`
/// Method source: `oracle/codemp/client/snd_ambient.cpp:76-180`
#[derive(Default)]
pub struct CSetGroup {
    m_numSets: c_int,
    m_ambientSets: Vec<ambientSet_t>,
    m_setMap: BTreeMap<String, usize>,
}

impl CSetGroup {
    pub fn new() -> CSetGroup {
        CSetGroup::default()
    }

    /// Raven `CSetGroup::Free` — drop every set and start the id run over.
    ///
    /// Source: `oracle/codemp/client/snd_ambient.cpp:95-111`
    pub fn Free(&mut self) {
        self.m_ambientSets.clear();
        self.m_setMap.clear();
        self.m_numSets = 0;
    }

    /// Raven `CSetGroup::AddSet` — append a set under `name` and answer its slot.
    ///
    /// A repeated name overwrites the map entry and leaves the earlier set in
    /// the vector, so its id stays reachable by number and not by name.
    /// Source: `oracle/codemp/client/snd_ambient.cpp:119-145`
    pub fn AddSet(&mut self, name: &str) -> usize {
        let mut set = ambientSet_t {
            name: name.to_string(),
            ..Default::default()
        };
        set.name.truncate(MAX_SET_NAME_LENGTH - 1);

        let slot = self.m_ambientSets.len();
        set.id = self.m_numSets;
        self.m_numSets += 1;
        let key = set.name.clone();
        self.m_ambientSets.push(set);
        self.m_setMap.insert(key, slot);

        slot
    }

    /// Raven `CSetGroup::GetSet(const char *)` — the slot one name holds.
    ///
    /// Source: `oracle/codemp/client/snd_ambient.cpp:153-166`
    pub fn GetSetByName(&self, name: &str) -> Option<usize> {
        self.m_setMap.get(name).copied()
    }

    /// Raven `CSetGroup::GetSet(int)` — the slot one id holds.
    ///
    /// Source: `oracle/codemp/client/snd_ambient.cpp:168-180`
    pub fn GetSetById(&self, ID: c_int) -> Option<usize> {
        if self.m_ambientSets.is_empty() || ID < 0 || ID >= self.m_numSets {
            return None;
        }
        Some(ID as usize)
    }

    pub fn set(&self, slot: usize) -> &ambientSet_t {
        &self.m_ambientSets[slot]
    }

    pub fn set_mut(&mut self, slot: usize) -> &mut ambientSet_t {
        &mut self.m_ambientSets[slot]
    }

    /// The set names in map order, which is the order `AS_ParseSets` reports
    /// missing precache entries in.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.m_setMap.keys().map(|k| k.as_str())
    }
}
