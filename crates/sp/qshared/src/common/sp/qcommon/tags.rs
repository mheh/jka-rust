//! SP qcommon memory tag definitions.
//!
//! Source: `oracle/oracle/code/qcommon/tags.h:1-53`
//! Type definition source: `oracle/oracle/code/game/q_shared.h:2682-2688`

#![allow(non_camel_case_types)]

// Filename:-	tags.h

// do NOT include-protect this file, or add any fields or labels, because it's included within enums and tables
//
// these macro args get "TAG_" prepended on them for enum purposes, and appear as literal strings for "meminfo" command

// Raven's `typedef char memtag_t` is 1 byte, not int-wide; `#[repr(i8)]` matches
// that width.
// Source: `oracle/oracle/code/game/q_shared.h:2682-2688`
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum memtag_t {
    TAG_ALL,
    TAG_HUNKALLOC, // mem that was formerly from the hunk AFTER the SetMark (ie discarded during vid_reset)
    TAG_HUNKMISCMODELS, // sub-hunk alloc to track misc models
    TAG_FILESYS,   // general filesystem usage
    TAG_EVENT,
    TAG_CLIPBOARD,
    TAG_LISTFILES, // for "*.blah" lists
    TAG_AMBIENTSET,
    TAG_G_ALLOC,          // used by G_Alloc()
    TAG_CLIENTS,          // Memory used for client info
    TAG_STATIC, // special usage for 1-byte allocations from 0..9 to avoid CopyString() slowdowns during cvar value copies
    TAG_SMALL,  // used by S_Malloc, but probably more of a hint now. Will be dumped later
    TAG_MODEL,  // general model usage), includes header-struct-only stuff like 'model_t'
    TAG_MODEL_MD3, // specific model types' disk images
    TAG_MODEL_GLM, //	   "
    TAG_MODEL_GLA, //	   "
    TAG_ICARUS, // Memory used internally by the Icarus scripting system
    TAG_IMAGE_T, // an image_t struct (no longer on the hunk because of cached texture stuff)
    TAG_TEMP_WORKSPACE, // anything like file loading or image workspace that's only temporary
    TAG_TEMP_TGA, // image workspace that's only temporary
    TAG_TEMP_JPG, // image workspace that's only temporary
    TAG_TEMP_PNG, // image workspace that's only temporary
    TAG_SND_MP3STREAMHDR, // specific MP3 struct for decoding (about 18..22K each?), not the actual MP3 binary
    TAG_SND_DYNAMICMUSIC, // in-mem MP3 files
    TAG_SND_RAWDATA,      // raw sound data, either MP3 or WAV
    TAG_GHOUL2,           // Ghoul2 stuff
    TAG_BSP,              // guess.
    TAG_BSP_DISKIMAGE, // temp during loading, to save both server and renderer fread()ing the same file. Only used if not low physical memory (currently 96MB)
    TAG_GP2,           // generic parser 2
    TAG_SPECIAL_MEM_TEST, // special usage in one function only!!!!!!
    TAG_ANIMATION_CFG, // may as well keep this seperate / readable

    TAG_SAVEGAME,   // used for allocating chunks during savegame file read
    TAG_SHADERTEXT, // used by cm_shader stuff
    TAG_CM_TERRAIN, // terrain
    TAG_R_TERRAIN,  // renderer side of terrain
    TAG_INFLATE,    // Temp memory used by zlib32
    TAG_DEFLATE,    // Temp memory used by zlib32
    TAG_POINTCACHE, // weather effects
    TAG_NEWDEL,
    // #ifdef _XBOX
    // #[cfg(target_vendor = "xbox")]
    // TAG_UI_ALLOC,
    // #[cfg(target_vendor = "xbox")]
    // TAG_BINK,
    // #endif
    TAG_COUNT,
}

//////////////// eof //////////////
