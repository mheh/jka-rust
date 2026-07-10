// stringed-oracle stub for codemp/game/q_shared.h
//
// Minimal, self-contained replacement for the Raven q_shared.h closure — just
// the types, macros and string helpers the two unmodified StringEd TUs
// (stringed_ingame.cpp + stringed_interface.cpp) reference. This is a HARNESS
// stub; oracle/ is never edited. See README.md.
#ifndef STRINGED_ORACLE_Q_SHARED_H
#define STRINGED_ORACLE_Q_SHARED_H

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cctype>
#include <cassert>
#include <strings.h>   // strcasecmp / strncasecmp

// Windows-ism the TUs use; POSIX equivalent.
#define stricmp  strcasecmp
#define strnicmp strncasecmp

typedef enum { qfalse, qtrue } qboolean;

#define MAX_QPATH 64

// cvar flags — value irrelevant to the harness (cvar registry ignores them).
#define CVAR_ARCHIVE    1
#define CVAR_NORESTART  2
#define CVAR_ROM        4

// Com_Error level codes; only ERR_DROP is reached by these TUs.
#define ERR_FATAL   0
#define ERR_DROP    1
#define ERR_NEED_CD 2

// Z_Malloc tag; harness maps Z_Malloc→malloc so the value is inert.
#define TAG_TEMP_WORKSPACE 0

// Colour escape used in a Com_DPrintf format literal.
#define S_COLOR_YELLOW "^3"

// The one engine singleton these TUs touch by field: ->string, ->integer,
// ->modified. Minimal layout (harness-internal; StringEd is layout-free, §F).
typedef struct cvar_s {
    char     *string;
    int       integer;
    qboolean  modified;
} cvar_t;

// --- string helpers (defined in host.cpp) ---
#ifdef __cplusplus
extern "C" {
#endif
char *va(const char *format, ...);
char *Q_strupr(char *s1);
int   Q_stricmp(const char *s1, const char *s2);
int   Q_stricmpn(const char *s1, const char *s2, int n);
void  Q_strncpyz(char *dest, const char *src, int destsize);
#ifdef __cplusplus
}
#endif

#endif // STRINGED_ORACLE_Q_SHARED_H
