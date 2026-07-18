// rmg_host_stubs.cpp — the engine-service surface the standalone RMG/terrain TUs
// reach through Com_*/Cvar_*/FS/rand + the un-ported CM_GetShaderInfo extern.
// These are the harness's stand-in for the ported EngineHost services
// (print/error/fs_read_file/flrand/irand — see rmg-terrain.md "The host seam").
// The RNG + Info_ValueForKey bodies are transcribed VERBATIM from the oracle so
// the compiled behavior is bit-faithful; oracle/ is never edited (§18/§19).
#include <cstdio>
#include <cstdarg>
#include <cstring>
#include <cstdlib>
#include <string>
#include <vector>
#include <stdexcept>

#include "exe_headers.h"   // q_shared.h + qcommon.h (Com_*/Cvar_*/errorParm_t)
#include "cm_local.h"      // CCMShader + CM_GetShaderInfo
#include "GenericParser2.h" // CGenericParser2 for Com_ParseTextFile

#ifndef FIXTURE_ROOT
#define FIXTURE_ROOT "fixtures"
#endif

// --------------------------------------------------------------------------
// print / error  (EngineHost::print / EngineHost::error, fork-1 panic model)
// --------------------------------------------------------------------------
void Com_Printf( const char *fmt, ... ) {
	va_list ap; va_start( ap, fmt );
	vfprintf( stdout, fmt, ap );
	va_end( ap );
	fflush( stdout );
}

// Com_DPrintf is developer-only (com_developer 0 by default) — silent, matching
// the shipped dedicated server. Source: common.cpp:141
void Com_DPrintf( const char *, ... ) {}

// Com_Error diverts through the fork-1 panic/catch_unwind model: it never
// returns to its caller; main.cpp catches to observe the ctor's empty-heightMap
// ERR_FATAL. Source: common.cpp:249
void Com_Error( int code, const char *fmt, ... ) {
	char buf[2048];
	va_list ap; va_start( ap, fmt );
	vsnprintf( buf, sizeof( buf ), fmt, ap );
	va_end( ap );
	throw std::runtime_error( std::string( "ERR(" ) + std::to_string( code ) + "): " + buf );
}

void Com_sprintf( char *dest, int size, const char *fmt, ... ) {
	va_list ap; va_start( ap, fmt );
	vsnprintf( dest, size, fmt, ap );
	va_end( ap );
}

// Source: oracle/codemp/game/q_shared.c Com_Clamp
float Com_Clamp( float min, float max, float value ) {
	if ( value < min ) return min;
	if ( value > max ) return max;
	return value;
}

char *va( const char *format, ... ) {
	static char buf[4][8192];
	static int  idx = 0;
	idx = ( idx + 1 ) & 3;
	va_list ap; va_start( ap, format );
	vsnprintf( buf[idx], sizeof( buf[idx] ), format, ap );
	va_end( ap );
	return buf[idx];
}

// Unregistered cvars read empty, as Cvar_VariableStringBuffer does. Only reached
// past LoadMission's early-out (never executed under DEDICATED). Source: cvar.cpp
void Cvar_VariableStringBuffer( const char *, char *buffer, int bufsize ) {
	if ( bufsize > 0 ) buffer[0] = 0;
}

// --------------------------------------------------------------------------
// FS + GP2 parse  (EngineHost::fs_read_file feeding the real GenericParser2)
// --------------------------------------------------------------------------
// Com_ParseTextFile: FS_FOpenFileByMode + FS_Read + parser.Parse. Reads the
// committed fixture under FIXTURE_ROOT so golden #4 drives the REAL .terrain
// parse (ruling 47). Missing file -> false, driving the RMG->arioche fallback
// and the non-fatal "Could not open" double-miss. Source: common.cpp:2179-2202
bool Com_ParseTextFile( const char *file, CGenericParser2 &parser ) {
	std::string path = std::string( FIXTURE_ROOT ) + "/" + file;
	FILE *f = fopen( path.c_str(), "rb" );
	if ( !f ) return false;
	fseek( f, 0, SEEK_END );
	long size = ftell( f );
	fseek( f, 0, SEEK_SET );
	std::vector<char> buf( size + 64, 0 ); // zeroed tail like the oracle FS read
	if ( size > 0 && fread( buf.data(), 1, size, f ) != (size_t)size ) { fclose( f ); return false; }
	fclose( f );
	char *ptr = buf.data();
	return parser.Parse( &ptr, true, false );
}

void Com_ParseTextFileDestroy( CGenericParser2 &parser ) { parser.Clean(); }

// --------------------------------------------------------------------------
// CM_GetShaderInfo — the un-ported extern (RMG-D5). No CM_LoadMap runs here, so
// cmShaderTable is unpopulated; the harness supplies deterministic shader flags.
// CONTRACT: for a shader name it returns a stable CCMShader whose surfaceFlags /
// contentFlags are a documented FNV-1a-32 function of the name string:
//     h = 2166136261; for each byte c: h = (h ^ c) * 16777619
//     contentFlags =  (int)(h        & 0x0000ffff)
//     surfaceFlags =  (int)((h >> 16) & 0x0000ffff)
// The `name` overload never returns NULL (matches cm_shader.cpp:498). Pointers
// are pool-stable so repeated lookups of one name compare equal.
// --------------------------------------------------------------------------
CCMShader *CM_GetShaderInfo( const char *name ) {
	static std::vector<CCMShader *> pool;
	for ( CCMShader *s : pool ) if ( !strcmp( s->GetName(), name ) ) return s;

	unsigned int h = 2166136261u;
	for ( const char *p = name; *p; ++p ) { h ^= (unsigned char)*p; h *= 16777619u; }

	CCMShader *s = new CCMShader();
	memset( s, 0, sizeof( *s ) );
	strncpy( s->shader, name, MAX_QPATH - 1 );
	s->contentFlags = (int)( h & 0x0000ffff );
	s->surfaceFlags = (int)( ( h >> 16 ) & 0x0000ffff );
	s->mNext = 0;
	pool.push_back( s );
	return s;
}

// --------------------------------------------------------------------------
// Faithful q_math.c helpers (the determinism anchor for golden #1). Transcribed
// VERBATIM from oracle/codemp/game/q_math.c:1432-1470 and :SetPlaneSignbits.
// --------------------------------------------------------------------------
static unsigned int holdrand = 0x89abcdef; /* retail-win32 32-bit width (2026-07-17 ruling) */

void Rand_Init( int seed ) { holdrand = seed; }
unsigned long rng_state( void ) { return holdrand; }

float flrand( float min, float max ) {
	float result;
	holdrand = ( holdrand * 214013L ) + 2531011L;
	result = (float)( holdrand >> 17 );
	result = ( ( result * ( max - min ) ) / 32768.0F ) + min;
	return result;
}

int irand( int min, int max ) {
	int result;
	assert( ( max - min ) < 32768 );
	max++;
	holdrand = ( holdrand * 214013L ) + 2531011L;
	result = holdrand >> 17;
	result = ( ( result * ( max - min ) ) >> 15 ) + min;
	return result;
}

// Source: oracle/codemp/game/q_math.c SetPlaneSignbits
void SetPlaneSignbits( cplane_t *out ) {
	int bits = 0, j;
	for ( j = 0; j < 3; j++ ) if ( out->normal[j] < 0 ) bits |= 1 << j;
	out->signbits = bits;
}

// Faithful Info_ValueForKey. Source: oracle/codemp/game/q_shared.c
char *Info_ValueForKey( const char *s, const char *key ) {
	char        pkey[BIG_INFO_KEY];
	static char value[2][BIG_INFO_VALUE];
	static int  valueindex = 0;
	char       *o;

	if ( !s || !key ) return (char *)"";
	if ( strlen( s ) >= BIG_INFO_STRING ) Com_Error( ERR_DROP, "Info_ValueForKey: oversize infostring" );

	valueindex ^= 1;
	if ( *s == '\\' ) s++;
	while ( 1 ) {
		o = pkey;
		while ( *s != '\\' ) { if ( !*s ) return (char *)""; *o++ = *s++; }
		*o = 0; s++;
		o = value[valueindex];
		while ( *s != '\\' && *s ) *o++ = *s++;
		*o = 0;
		if ( !Q_stricmp( key, pkey ) ) return value[valueindex];
		if ( !*s ) break;
		s++;
	}
	return (char *)"";
}
