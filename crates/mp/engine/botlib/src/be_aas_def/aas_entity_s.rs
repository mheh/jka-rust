#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::botlib::aas_entityinfo_s::aas_entityinfo_s;

/// Raven `aas_entity_t` — an AAS view of a game entity: its info plus links
/// into the AAS areas and BSP leaves it occupies.
///
/// Type definition source: `oracle/codemp/botlib/be_aas_def.h:78-86`
#[repr(C)]
pub struct aas_entity_t {
    //entity info
    pub i: aas_entityinfo_s,
    //links into the AAS areas
    pub areas: *mut super::aas_link_s::aas_link_t,
    //links into the BSP leaves
    pub leaves: *mut super::bsp_link_s::bsp_link_s,
}

pub type aas_entity_s = aas_entity_t;

const _: () = assert!(core::mem::size_of::<aas_entity_t>() == 160);
const _: () = assert!(core::mem::offset_of!(aas_entity_t, i) == 0);
const _: () = assert!(core::mem::offset_of!(aas_entity_t, areas) == 144);
const _: () = assert!(core::mem::offset_of!(aas_entity_t, leaves) == 152);
