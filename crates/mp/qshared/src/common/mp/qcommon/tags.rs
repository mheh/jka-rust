//! MP qcommon memory tag definitions.
//!
//! Source: `oracle/oracle/codemp/qcommon/tags.h:1-74`
//! Type definition source: `oracle/oracle/codemp/game/q_shared.h:3101-3107`

#![allow(non_camel_case_types)]

// Filename:-	tags.h

// do NOT include-protect this file, or add any fields or labels, because it's included within enums and tables
//
// these macro args get "TAG_" prepended on them for enum purposes, and appear as literal strings for "meminfo" command

// Raven's `typedef char memtag_t` is 1 byte, not int-wide; `#[repr(i8)]` matches
// that width.
// Source: `oracle/oracle/codemp/game/q_shared.h:3101-3107`
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum memtag_t {
    TAG_ALL,
    TAG_BOTLIB,
    TAG_CLIENTS, // Memory used for client info
    // #ifndef _XBOX
    // #[cfg(not(target_vendor = "xbox"))]
    TAG_BOTGAME,
    // #[cfg(not(target_vendor = "xbox"))]
    TAG_DOWNLOAD, // used by the downloading system
    // #[cfg(not(target_vendor = "xbox"))]
    TAG_GENERAL,
    // #[cfg(not(target_vendor = "xbox"))]
    TAG_CLIPBOARD,
    // #[cfg(not(target_vendor = "xbox"))]
    TAG_SND_MP3STREAMHDR, // specific MP3 struct for decoding (about 18..22K each?), not the actual MP3 binary
    // #[cfg(not(target_vendor = "xbox"))]
    TAG_SND_DYNAMICMUSIC, // in-mem MP3 files
    // #[cfg(not(target_vendor = "xbox"))]
    TAG_BSP_DISKIMAGE, // temp during loading, to save both server and renderer fread()ing the same file. Only used if not low physical memory (currently 96MB)
    // #[cfg(not(target_vendor = "xbox"))]
    TAG_VM, // stuff for VM, may be zapped later?
    // #[cfg(not(target_vendor = "xbox"))]
    TAG_SPECIAL_MEM_TEST, // special usage for testing z_malloc recover only
    // #endif
    TAG_HUNK_MARK1, //hunk allocations before the mark is set
    TAG_HUNK_MARK2, //hunk allocations after the mark is set
    TAG_EVENT,
    TAG_FILESYS,     // general filesystem usage
    TAG_GHOUL2,      // Ghoul2 stuff
    TAG_GHOUL2_GORE, // Ghoul2 gore stuff
    TAG_LISTFILES,   // for "*.blah" lists
    TAG_AMBIENTSET,
    TAG_STATIC, // special usage for 1-byte allocations from 0..9 to avoid CopyString() slowdowns during cvar value copies
    TAG_SMALL,  // used by S_Malloc, but probably more of a hint now. Will be dumped later
    TAG_MODEL_MD3, // specific model types' disk images
    TAG_MODEL_GLM, //	   "
    TAG_MODEL_GLA, //	   "
    TAG_ICARUS, // Memory used internally by the Icarus scripting system
    //sorry, I don't want to have to keep adding these and recompiling, so there may be more than I need
    TAG_ICARUS2, //for debugging mem leaks in icarus -rww
    TAG_ICARUS3, //for debugging mem leaks in icarus -rww
    TAG_ICARUS4, //for debugging mem leaks in icarus -rww
    TAG_ICARUS5, //for debugging mem leaks in icarus -rww
    TAG_SHADERTEXT,
    TAG_SND_RAWDATA,    // raw sound data, either MP3 or WAV
    TAG_TEMP_WORKSPACE, // anything like file loading or image workspace that's only temporary
    TAG_TEMP_PNG,       // image workspace that's only temporary
    TAG_TEXTPOOL,       // for some special text-pool class thingy
    TAG_IMAGE_T,        // an image_t struct (no longer on the hunk because of cached texture stuff)
    TAG_INFLATE,        // Temp memory used by zlib32
    TAG_DEFLATE, // Temp memory used by zlib32//	TAGDEF(SOUNDPOOL),					// pool of mem for the sound system
    TAG_BSP,     // guess.
    TAG_GRIDMESH, // some specific temp workspace that only seems to be in the MP codebase

    //rwwRMG - following:
    TAG_POINTCACHE,      // weather system
    TAG_TERRAIN,         // RMG terrain management
    TAG_R_TERRAIN,       // terrain renderer
    TAG_RESAMPLE,        // terrain heightmap resampling (I think)
    TAG_CM_TERRAIN,      // common terrain data management
    TAG_CM_TERRAIN_TEMP, // temporary terrain allocations
    TAG_TEMP_IMAGE,      // temporary allocations for image manipulation

    TAG_VM_ALLOCATED, // allocated by game or cgame via memory shifting

    TAG_TEMP_HUNKALLOC,
    // #ifdef _XBOX
    // #[cfg(target_vendor = "xbox")]
    // TAG_NEWDEL, // new / delete -> Z_Malloc on Xbox
    // #[cfg(target_vendor = "xbox")]
    // TAG_UI_ALLOC, // UI DLL calls to UI_Alloc
    // #[cfg(target_vendor = "xbox")]
    // TAG_CG_UI_ALLOC, // Cgame DLL calls to UI_Alloc
    // #[cfg(target_vendor = "xbox")]
    // TAG_BG_ALLOC,
    // #[cfg(target_vendor = "xbox")]
    // TAG_BINK,
    // #[cfg(target_vendor = "xbox")]
    // TAG_XBL_FRIENDS, // friends list
    // #endif
    TAG_COUNT,
}

//////////////// eof //////////////
