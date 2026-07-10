// Stub qcommon.h — the common-services prototypes the terrain/RMG TUs reach
// through Com_*/Cvar_*/va. The real bodies are the harness's EngineHost-shaped
// stubs (src/rmg_host_stubs.cpp): print/error/FS(GP2 parse)/cvar. oracle never
// edited (§18).
#pragma once
#include "q_shared.h"

class CGenericParser2; // Com_ParseTextFile takes it by reference (incomplete OK)

// errorParm_t — Raven's Com_Error code (enum fidelity over int).
// Source: oracle/codemp/qcommon/qcommon.h (errorParm_t)
typedef enum { ERR_FATAL = 0, ERR_DROP, ERR_SERVERDISCONNECT, ERR_DISCONNECT, ERR_NEED_CD } errorParm_t;

void  Com_Printf( const char *fmt, ... );
void  Com_DPrintf( const char *fmt, ... );
void  Com_Error( int code, const char *fmt, ... );
void  Com_sprintf( char *dest, int size, const char *fmt, ... );
float Com_Clamp( float min, float max, float value );

// Raven Com_ParseTextFile / Com_ParseTextFileDestroy — FS read + GP2 parse.
// Source: oracle/codemp/qcommon/common.cpp:2179-2202
bool  Com_ParseTextFile( const char *file, CGenericParser2 &parser );
void  Com_ParseTextFileDestroy( CGenericParser2 &parser );

void  Cvar_VariableStringBuffer( const char *var_name, char *buffer, int bufsize );
char *va( const char *format, ... );

// Source: oracle/codemp/qcommon/qcommon.h:1094
static inline int Round( float value ) { return (int)floorf( value + 0.5f ); }
