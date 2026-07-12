// MockHost engine services for the end-to-end ICARUS golden (icarus.md
// § Verification strategy, unit 3). This is the C++ oracle-side equivalent of
// ruling 32's fixture-backed MockHost (crates/mp/host-interface/src/mock.rs):
// it stands up just enough engine surface to drive ICARUS_Init -> InitEnt ->
// RunScript -> per-frame Update against the UNMODIFIED oracle icarus TUs, and
// records the ordered VM_Call(gvm, GAME_ICARUS_*) trace the sequencer/taskmanager
// emit. Every engine service the icarus link references is defined here; nothing
// here is ported (it is the test harness's stand-in for the real Engine).
#include "exe_headers.h"
#include "../game/g_public.h"
#include "../server/server.h"
#include "interface.h"
#include "GameInterface.h"
#include "../qcommon/RoffSystem.h"

#include <cstdio>
#include <cstdlib>
#include <cstdarg>
#include <cstring>
#include <vector>

// ---- recorded VM_Call trace (the golden's core signal) -------------------
std::vector<int> g_vmTrace;

// ---- engine singletons the icarus link reads -----------------------------
vm_t          *gvm  = (vm_t *)1;          // dummy non-null game VM handle
server_t       sv;                         // sv.mSharedMemory = the arg window
serverStatic_t svs;                        // svs.time = the mock clock
static cvar_t  s_developer;                // com_developer->integer == 0 (dedicated default)
static cvar_t  s_timescale;                // com_timescale->value == 1.0
cvar_t        *com_developer = &s_developer;
cvar_t        *com_timescale = &s_timescale;
CROFFSystem    theROFFSystem;              // ROFF cache (never exercised by the fixtures)

// ROFF is out of the icarus link set; these two symbols only need to resolve.
int      CROFFSystem::Cache(const char *, qboolean) { return 0; }
qboolean CROFFSystem::Restart() { return qtrue; }

// ---- entity arena (SV_GentityNum service) --------------------------------
sharedEntity_t g_entities[MAX_GENTITIES];

sharedEntity_t *SV_GentityNum(int num) { return &g_entities[num]; }

// ---- Zone / console ------------------------------------------------------
void *Z_Malloc(int iSize, memtag_t, qboolean, int) { return calloc(1, iSize ? iSize : 1); }
void  Z_Free(void *ptr) { free(ptr); }

void QDECL Com_Printf(const char *fmt, ...)
{ va_list ap; va_start(ap, fmt); vfprintf(stderr, fmt, ap); va_end(ap); }

void QDECL Com_Error(int code, const char *fmt, ...)
{ va_list ap; va_start(ap, fmt); fprintf(stderr, "Com_Error(%d): ", code); vfprintf(stderr, fmt, ap); va_end(ap); exit(3); }

void QDECL SV_SendServerCommand(client_t *, const char *, ...) {}

// ---- the vmcall seam: record every callnum, return 0 ---------------------
int QDECL VM_Call(vm_t *, int callNum, ...) { g_vmTrace.push_back(callNum); return 0; }

// ---- filesystem: serve fixtures/<name>.IBI from disk ---------------------
int FS_ReadFile(const char *qpath, void **buffer)
{
	FILE *f = fopen(qpath, "rb");
	if (!f) { if (buffer) *buffer = NULL; return -1; }
	fseek(f, 0, SEEK_END); long len = ftell(f); fseek(f, 0, SEEK_SET);
	char *buf = (char *)malloc(len + 1);
	if (fread(buf, 1, len, f) != (size_t)len) { fclose(f); free(buf); if (buffer) *buffer = NULL; return -1; }
	buf[len] = 0; fclose(f);
	if (buffer) *buffer = buf;
	return (int)len;
}
void FS_FreeFile(void *buffer) { free(buffer); }

// ---- string helpers (route MSVC/Raven forms to libc) ---------------------
int   Q_stricmp(const char *a, const char *b) { return strcasecmp(a, b); }
int   Q_stricmpn(const char *a, const char *b, int n) { return strncasecmp(a, b, n); }
char *Q_strupr(char *s) { for (char *p = s; *p; ++p) *p = (char)toupper((unsigned char)*p); return s; }
void  Q_strncpyz(char *dst, const char *src, int n) { if (n <= 0) return; strncpy(dst, src, n - 1); dst[n - 1] = 0; }
void  COM_StripExtension(const char *in, char *out)
{ const char *dot = strrchr(in, '.'); size_t n = dot ? (size_t)(dot - in) : strlen(in); memcpy(out, in, n); out[n] = 0; }

char *QDECL va(const char *fmt, ...)
{
	static char buf[4][2048]; static int idx = 0;
	char *out = buf[idx = (idx + 1) & 3];
	va_list ap; va_start(ap, fmt); vsnprintf(out, sizeof(buf[0]), fmt, ap); va_end(ap);
	return out;
}

// Deterministic RNG. Interface_Init wires I_Random = Q_flrand (Q3_Interface.cpp).
// The committed fixtures use no random() block, so the value is inert here, but a
// fixed-seed LCG keeps any future random fixture reproducible (mirrors the Rust
// MockHost replicating Raven's holdrand off a fixed seed).
static unsigned long s_hold = 0x89abcdefUL;
static float nextrand() { s_hold = s_hold * 1103515245UL + 12345UL; return (float)((s_hold >> 16) & 0x7fff) / (float)0x7fff; }
float Q_flrand(float min, float max) { return min + nextrand() * (max - min); }
float flrand(float min, float max)   { return Q_flrand(min, max); }
int   Q_irand(int a, int b)          { return a + (int)(nextrand() * (float)(b - a + 1)); }

// Behavior-state name->id lookup (GameInterface BSTable). Not exercised by the
// fixtures (no behaviorSet parsing); resolve to "not found".
int GetIDForString(stringID_table_t *, const char *) { return -1; }
