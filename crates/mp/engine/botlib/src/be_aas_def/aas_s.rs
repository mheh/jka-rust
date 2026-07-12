#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use mp_qshared::shared::{MAX_CONFIGSTRINGS, MAX_QPATH};

use super::aas_entity_s::aas_entity_t;
use super::aas_link_s::aas_link_t;
use super::aas_reachabilityareas_s::aas_reachabilityareas_t;
use super::aas_reversedreachability_s::aas_reversedreachability_t;
use super::aas_routingcache_s::aas_routingcache_t;
use super::aas_routingupdate_s::aas_routingupdate_t;
use crate::aasfile::aas_area_s::aas_area_t;
use crate::aasfile::aas_areasettings_s::aas_areasettings_t;
use crate::aasfile::aas_bbox_s::aas_bbox_t;
use crate::aasfile::aas_cluster_s::aas_cluster_t;
use crate::aasfile::aas_edge_s::aas_edge_t;
use crate::aasfile::aas_edgeindex_t::aas_edgeindex_t;
use crate::aasfile::aas_face_s::aas_face_t;
use crate::aasfile::aas_faceindex_t::aas_faceindex_t;
use crate::aasfile::aas_node_s::aas_node_t;
use crate::aasfile::aas_plane_s::aas_plane_t;
use crate::aasfile::aas_portal_s::aas_portal_t;
use crate::aasfile::aas_portalindex_t::aas_portalindex_t;
use crate::aasfile::aas_reachability_s::aas_reachability_t;
use crate::aasfile::aas_vertex_t::aas_vertex_t;

/// `MAX_TRAVELTYPES`.
///
/// Source: `oracle/codemp/botlib/aasfile.h:16`
pub const MAX_TRAVELTYPES: usize = 32;

/// Raven `aas_t` — the whole in-memory AAS (area awareness system) file plus
/// derived routing state for one loaded map.
///
/// Type definition source: `oracle/codemp/botlib/be_aas_def.h:183-276`
#[repr(C)]
pub struct aas_t {
    /// true when an AAS file is loaded
    pub loaded: c_int,
    /// true when AAS has been initialized
    pub initialized: c_int,
    /// set true when file should be saved
    pub savefile: c_int,
    pub bspchecksum: c_int,
    //current time
    pub time: f32,
    pub numframes: c_int,
    //name of the aas file
    pub filename: [c_char; MAX_QPATH as usize],
    pub mapname: [c_char; MAX_QPATH as usize],
    //bounding boxes
    pub numbboxes: c_int,
    pub bboxes: *mut aas_bbox_t,
    //vertexes
    pub numvertexes: c_int,
    pub vertexes: *mut aas_vertex_t,
    //planes
    pub numplanes: c_int,
    pub planes: *mut aas_plane_t,
    //edges
    pub numedges: c_int,
    pub edges: *mut aas_edge_t,
    //edge index
    pub edgeindexsize: c_int,
    pub edgeindex: *mut aas_edgeindex_t,
    //faces
    pub numfaces: c_int,
    pub faces: *mut aas_face_t,
    //face index
    pub faceindexsize: c_int,
    pub faceindex: *mut aas_faceindex_t,
    //convex areas
    pub numareas: c_int,
    pub areas: *mut aas_area_t,
    //convex area settings
    pub numareasettings: c_int,
    pub areasettings: *mut aas_areasettings_t,
    //reachablity list
    pub reachabilitysize: c_int,
    pub reachability: *mut aas_reachability_t,
    //nodes of the bsp tree
    pub numnodes: c_int,
    pub nodes: *mut aas_node_t,
    //cluster portals
    pub numportals: c_int,
    pub portals: *mut aas_portal_t,
    //cluster portal index
    pub portalindexsize: c_int,
    pub portalindex: *mut aas_portalindex_t,
    //clusters
    pub numclusters: c_int,
    pub clusters: *mut aas_cluster_t,
    //
    pub numreachabilityareas: c_int,
    pub reachabilitytime: f32,
    //enities linked in the areas
    /// heap with link structures
    pub linkheap: *mut aas_link_t,
    /// size of the link heap
    pub linkheapsize: c_int,
    /// first free link
    pub freelinks: *mut aas_link_t,
    /// entities linked into areas
    pub arealinkedentities: *mut *mut aas_link_t,
    //entities
    pub maxentities: c_int,
    pub maxclients: c_int,
    pub entities: *mut aas_entity_t,
    //string indexes
    pub configstrings: [*mut c_char; MAX_CONFIGSTRINGS],
    pub indexessetup: c_int,
    //index to retrieve travel flag for a travel type
    pub travelflagfortype: [c_int; MAX_TRAVELTYPES],
    //travel flags for each area based on contents
    pub areacontentstravelflags: *mut c_int,
    //routing update
    pub areaupdate: *mut aas_routingupdate_t,
    pub portalupdate: *mut aas_routingupdate_t,
    //number of routing updates during a frame (reset every frame)
    pub frameroutingupdates: c_int,
    //reversed reachability links
    pub reversedreachability: *mut aas_reversedreachability_t,
    //travel times within the areas
    pub areatraveltimes: *mut *mut *mut u16,
    //array of size numclusters with cluster cache
    pub clusterareacache: *mut *mut *mut aas_routingcache_t,
    pub portalcache: *mut *mut aas_routingcache_t,
    //cache list sorted on time
    /// start of cache list sorted on time
    pub oldestcache: *mut aas_routingcache_t,
    /// end of cache list sorted on time
    pub newestcache: *mut aas_routingcache_t,
    //maximum travel time through portal areas
    pub portalmaxtraveltimes: *mut c_int,
    //areas the reachabilities go through
    pub reachabilityareaindex: *mut c_int,
    pub reachabilityareas: *mut aas_reachabilityareas_t,
}

/// Raven's C tag is `aas_s`; the typedef name `aas_t` is house style for the
/// struct itself.
pub type aas_s = aas_t;

const _: () = assert!(core::mem::size_of::<aas_t>() == 14272);
const _: () = assert!(core::mem::offset_of!(aas_t, loaded) == 0);
const _: () = assert!(core::mem::offset_of!(aas_t, initialized) == 4);
const _: () = assert!(core::mem::offset_of!(aas_t, savefile) == 8);
const _: () = assert!(core::mem::offset_of!(aas_t, bspchecksum) == 12);
const _: () = assert!(core::mem::offset_of!(aas_t, time) == 16);
const _: () = assert!(core::mem::offset_of!(aas_t, numframes) == 20);
const _: () = assert!(core::mem::offset_of!(aas_t, filename) == 24);
const _: () = assert!(core::mem::offset_of!(aas_t, mapname) == 88);
const _: () = assert!(core::mem::offset_of!(aas_t, numbboxes) == 152);
const _: () = assert!(core::mem::offset_of!(aas_t, bboxes) == 160);
const _: () = assert!(core::mem::offset_of!(aas_t, numvertexes) == 168);
const _: () = assert!(core::mem::offset_of!(aas_t, vertexes) == 176);
const _: () = assert!(core::mem::offset_of!(aas_t, numplanes) == 184);
const _: () = assert!(core::mem::offset_of!(aas_t, planes) == 192);
const _: () = assert!(core::mem::offset_of!(aas_t, numedges) == 200);
const _: () = assert!(core::mem::offset_of!(aas_t, edges) == 208);
const _: () = assert!(core::mem::offset_of!(aas_t, edgeindexsize) == 216);
const _: () = assert!(core::mem::offset_of!(aas_t, edgeindex) == 224);
const _: () = assert!(core::mem::offset_of!(aas_t, numfaces) == 232);
const _: () = assert!(core::mem::offset_of!(aas_t, faces) == 240);
const _: () = assert!(core::mem::offset_of!(aas_t, faceindexsize) == 248);
const _: () = assert!(core::mem::offset_of!(aas_t, faceindex) == 256);
const _: () = assert!(core::mem::offset_of!(aas_t, numareas) == 264);
const _: () = assert!(core::mem::offset_of!(aas_t, areas) == 272);
const _: () = assert!(core::mem::offset_of!(aas_t, numareasettings) == 280);
const _: () = assert!(core::mem::offset_of!(aas_t, areasettings) == 288);
const _: () = assert!(core::mem::offset_of!(aas_t, reachabilitysize) == 296);
const _: () = assert!(core::mem::offset_of!(aas_t, reachability) == 304);
const _: () = assert!(core::mem::offset_of!(aas_t, numnodes) == 312);
const _: () = assert!(core::mem::offset_of!(aas_t, nodes) == 320);
const _: () = assert!(core::mem::offset_of!(aas_t, numportals) == 328);
const _: () = assert!(core::mem::offset_of!(aas_t, portals) == 336);
const _: () = assert!(core::mem::offset_of!(aas_t, portalindexsize) == 344);
const _: () = assert!(core::mem::offset_of!(aas_t, portalindex) == 352);
const _: () = assert!(core::mem::offset_of!(aas_t, numclusters) == 360);
const _: () = assert!(core::mem::offset_of!(aas_t, clusters) == 368);
const _: () = assert!(core::mem::offset_of!(aas_t, numreachabilityareas) == 376);
const _: () = assert!(core::mem::offset_of!(aas_t, reachabilitytime) == 380);
const _: () = assert!(core::mem::offset_of!(aas_t, linkheap) == 384);
const _: () = assert!(core::mem::offset_of!(aas_t, linkheapsize) == 392);
const _: () = assert!(core::mem::offset_of!(aas_t, freelinks) == 400);
const _: () = assert!(core::mem::offset_of!(aas_t, arealinkedentities) == 408);
const _: () = assert!(core::mem::offset_of!(aas_t, maxentities) == 416);
const _: () = assert!(core::mem::offset_of!(aas_t, maxclients) == 420);
const _: () = assert!(core::mem::offset_of!(aas_t, entities) == 424);
const _: () = assert!(core::mem::offset_of!(aas_t, configstrings) == 432);
const _: () = assert!(core::mem::offset_of!(aas_t, indexessetup) == 14032);
const _: () = assert!(core::mem::offset_of!(aas_t, travelflagfortype) == 14036);
const _: () = assert!(core::mem::offset_of!(aas_t, areacontentstravelflags) == 14168);
const _: () = assert!(core::mem::offset_of!(aas_t, areaupdate) == 14176);
const _: () = assert!(core::mem::offset_of!(aas_t, portalupdate) == 14184);
const _: () = assert!(core::mem::offset_of!(aas_t, frameroutingupdates) == 14192);
const _: () = assert!(core::mem::offset_of!(aas_t, reversedreachability) == 14200);
const _: () = assert!(core::mem::offset_of!(aas_t, areatraveltimes) == 14208);
const _: () = assert!(core::mem::offset_of!(aas_t, clusterareacache) == 14216);
const _: () = assert!(core::mem::offset_of!(aas_t, portalcache) == 14224);
const _: () = assert!(core::mem::offset_of!(aas_t, oldestcache) == 14232);
const _: () = assert!(core::mem::offset_of!(aas_t, newestcache) == 14240);
const _: () = assert!(core::mem::offset_of!(aas_t, portalmaxtraveltimes) == 14248);
const _: () = assert!(core::mem::offset_of!(aas_t, reachabilityareaindex) == 14256);
const _: () = assert!(core::mem::offset_of!(aas_t, reachabilityareas) == 14264);
