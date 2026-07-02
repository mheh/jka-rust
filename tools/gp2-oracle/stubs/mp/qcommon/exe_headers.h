// Stub exe_headers.h for compiling the UNMODIFIED codemp GenericParser2.cpp
// standalone. Provides only what that TU references. Q_stricmp/Q_stricmpn are
// ASCII case-insensitive in Raven; strcasecmp matches for the byte values the
// fixtures use.
#pragma once
#include <stdlib.h>
#include <string.h>
#include <strings.h>

#define Q_stricmp strcasecmp
#define Q_stricmpn strncasecmp
#define stricmp strcasecmp

typedef enum { qfalse = 0, qtrue } qboolean;

#define TAG_TEXTPOOL 0
#define TAG_GP2 0

static inline void *Z_Malloc(int size, int tag, qboolean zero)
{
	(void)tag;
	return zero ? calloc(1, size) : malloc(size);
}

static inline void Z_Free(void *ptr)
{
	free(ptr);
}
