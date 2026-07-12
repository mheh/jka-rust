// trmodel-oracle — deterministic engine host for the unmodified tr_model.cpp.
//
// Implements the qcommon/q_shared seam the loader calls with fully deterministic
// behaviour: a fixture-backed filesystem (files under fixtures/, checksums from a
// PAK map honouring the 1/-1 convention), a zone allocator that tracks per-tag
// byte sums so Z_MemSize == sum of live iAllocSize (GetModelDataAllocSize parity,
// TRM-D3), a cvar registry, and captured console output (Com_Printf -> stdout,
// the golden; Com_DPrintf a no-op, matching non-developer). oracle/ is untouched.
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cstdarg>
#include <cctype>
#include <cmath>
#include <string>
#include <map>

#include "tr_local.h"   // -> q_shared.h + qcommon.h + mdx_format.h (build/ stubs)
#include "host.h"

// ---------------------------------------------------------------------------
// Renderer + loader globals the TU expects (normally in tr_main/tr_init).
// ---------------------------------------------------------------------------
trGlobals_t tr;
cvar_t *sv_pure          = nullptr;
cvar_t *r_modelpoolmegs  = nullptr;
cvar_t *r_noServerGhoul2 = nullptr;
cvar_t *r_lodbias        = nullptr;

// ---------------------------------------------------------------------------
// string helpers (q_shared.c equivalents)
// ---------------------------------------------------------------------------
extern "C" {

int Q_stricmp(const char *s1, const char *s2) { return strcasecmp(s1, s2); }
int Q_stricmpn(const char *s1, const char *s2, int n) { return strncasecmp(s1, s2, n); }

char *Q_strlwr(char *s1) {
	for (char *s = s1; *s; ++s) *s = (char)tolower((unsigned char)*s);
	return s1;
}

void Q_strncpyz(char *dest, const char *src, int destsize) {
	if (destsize <= 0) return;
	strncpy(dest, src, destsize - 1);
	dest[destsize - 1] = 0;
}

// §20 client tag/bounds math link stub (R_LerpTag, never called live).
vec_t VectorNormalize(vec3_t v) {
	float len = (float)sqrt(v[0]*v[0] + v[1]*v[1] + v[2]*v[2]);
	if (len) { v[0]/=len; v[1]/=len; v[2]/=len; }
	return len;
}

void Com_sprintf(char *dest, int size, const char *fmt, ...) {
	va_list ap; va_start(ap, fmt);
	vsnprintf(dest, size, fmt, ap);
	va_end(ap);
}

char *va(const char *format, ...) {
	static char buf[4][1024];
	static int idx = 0;
	char *out = buf[idx++ & 3];
	va_list ap; va_start(ap, format);
	vsnprintf(out, sizeof(buf[0]), format, ap);
	va_end(ap);
	return out;
}

// ---------------------------------------------------------------------------
// console
// ---------------------------------------------------------------------------
void Com_Printf(const char *fmt, ...) {
	va_list ap; va_start(ap, fmt);
	vprintf(fmt, ap);
	va_end(ap);
}
void Com_DPrintf(const char *, ...) { /* developer print: silent, non-developer */ }
void Com_Error(int level, const char *fmt, ...) {
	printf("Com_Error(%d): ", level);
	va_list ap; va_start(ap, fmt);
	vprintf(fmt, ap);
	va_end(ap);
	printf("\n");
	exit(1);
}

} // extern "C"

// ---------------------------------------------------------------------------
// cvar registry
// ---------------------------------------------------------------------------
static std::map<std::string, cvar_t*> g_cvars;

extern "C" cvar_t *Cvar_Get(const char *name, const char *value, int /*flags*/) {
	auto it = g_cvars.find(name);
	if (it != g_cvars.end()) return it->second;
	cvar_t *c = (cvar_t*)calloc(1, sizeof(cvar_t));
	c->name    = strdup(name);
	c->string  = strdup(value ? value : "");
	c->integer = value ? atoi(value) : 0;
	c->value   = value ? (float)atof(value) : 0.0f;
	g_cvars[name] = c;
	return c;
}

void host_cvar_set(const char *name, int value) {
	cvar_t *c = Cvar_Get(name, "0", 0);
	free(c->string);
	char tmp[32]; snprintf(tmp, sizeof(tmp), "%d", value);
	c->string  = strdup(tmp);
	c->integer = value;
	c->value   = (float)value;
}

// ---------------------------------------------------------------------------
// zone allocator — per-tag byte sums (Z_MemSize parity with iAllocSize)
// ---------------------------------------------------------------------------
struct ZBlock { int size; int tag; };
static std::map<void*, ZBlock> g_zone;
static long g_tagsum[8] = {0};

extern "C" {

void *Z_Malloc(int iSize, memtag_t eTag, qboolean bZeroit) {
	void *p = malloc(iSize > 0 ? iSize : 1);
	if (bZeroit) memset(p, 0, iSize > 0 ? iSize : 1);
	g_zone[p] = ZBlock{ iSize, (int)eTag };
	g_tagsum[(unsigned char)eTag] += iSize;
	return p;
}

void Z_Free(void *ptr) {
	if (!ptr) return;
	auto it = g_zone.find(ptr);
	if (it != g_zone.end()) {
		g_tagsum[(unsigned char)it->second.tag] -= it->second.size;
		g_zone.erase(it);
	}
	free(ptr);
}

void Z_MorphMallocTag(void *pvBuffer, memtag_t eDesiredTag) {
	auto it = g_zone.find(pvBuffer);
	if (it == g_zone.end()) return;
	g_tagsum[(unsigned char)it->second.tag] -= it->second.size;
	it->second.tag = (int)eDesiredTag;
	g_tagsum[(unsigned char)eDesiredTag] += it->second.size;
}

int Z_MemSize(memtag_t eTag) { return (int)g_tagsum[(unsigned char)eTag]; }

void *Hunk_Alloc(int size, int /*preference*/) {
	void *p = malloc(size > 0 ? size : 1);
	memset(p, 0, size > 0 ? size : 1);   // Raven's hunk is zeroed
	return p;
}

} // extern "C"

// ---------------------------------------------------------------------------
// filesystem — files served from fixtures/, checksums from a PAK map
// ---------------------------------------------------------------------------
static std::map<std::string, int> g_pak;
int host_fs_reads = 0;

void host_pak_add(const char *lc_path, int checksum) { g_pak[lc_path] = checksum; }

extern "C" {

int FS_ReadFile(const char *qpath, void **buffer) {
	std::string path = std::string("fixtures/") + qpath;
	FILE *f = fopen(path.c_str(), "rb");
	if (!f) { if (buffer) *buffer = nullptr; return -1; }
	fseek(f, 0, SEEK_END);
	long len = ftell(f);
	fseek(f, 0, SEEK_SET);
	void *buf = Z_Malloc((int)len, TAG_FILESYS, qfalse);
	if (fread(buf, 1, len, f) != (size_t)len) { fclose(f); Z_Free(buf); if (buffer) *buffer = nullptr; return -1; }
	fclose(f);
	if (buffer) *buffer = buf;
	host_fs_reads++;
	return (int)len;
}

void FS_FreeFile(void *buffer) { Z_Free(buffer); }

// Ruling 59: 1 (with checksum) only for a pure-pak hit, else -1.
int FS_FileIsInPAK(const char *filename, int *pChecksum) {
	auto it = g_pak.find(filename);
	if (it == g_pak.end()) return -1;
	if (pChecksum) *pChecksum = it->second;
	return 1;
}

} // extern "C"

// C++-linkage link stubs (qboolean& params / declared in tr_local.h as C++).
// ghoul2 client loaders (tr_ghoul2.cpp) — §20, never called live.
qboolean R_LoadMDXA(model_t *, void *, const char *, qboolean &) { return qfalse; }
qboolean R_LoadMDXM(model_t *, void *, const char *, qboolean &) { return qfalse; }
// shader hash teardown (tr_shader.cpp) — §20; R_HunkClearCrap calls it.
void KillTheShaderHashTable(void) {}
