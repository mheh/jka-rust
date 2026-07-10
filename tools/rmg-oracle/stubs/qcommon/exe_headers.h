// Stub exe_headers.h — the umbrella header every codemp TU opens with. The real
// one pulls the entire engine; this provides only what the RMG/terrain +
// GenericParser2 TUs reference: the shared types/math (q_shared.h) and the Z_
// memory shim. oracle/ is never edited (§18).
#pragma once
#include "q_shared.h"
#include "qcommon.h"

// Raven memory tags the terrain/GP2 TUs pass to Z_Malloc. Values are irrelevant
// to the stub allocator; kept as distinct symbols so the sources compile 1:1.
#define TAG_TEXTPOOL       0
#define TAG_GP2            0
#define TAG_CM_TERRAIN     0
#define TAG_CM_TERRAIN_TEMP 0
#define TAG_RESAMPLE       0

// Raven `Z_Malloc(size, tag, bZeroit=qfalse)`: the non-zeroing overload default
// matches qcommon.h:787 (bZeroit defaults qfalse) — the §F.19 fact that
// mHeightMap is allocated uninitialized. cm_terrain calls the 2-arg form;
// GenericParser2 calls the 3-arg form. Both resolve here.
// Source: oracle/codemp/qcommon/qcommon.h:787
static inline void *Z_Malloc( int size, int tag, qboolean bZeroit = qfalse ) {
	(void)tag;
	return bZeroit ? calloc( 1, size ) : malloc( size );
}
static inline void Z_Free( void *ptr ) { free( ptr ); }
