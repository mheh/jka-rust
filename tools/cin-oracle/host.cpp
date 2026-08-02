// cin-oracle — deterministic host / link stubs for the Raven RoQ decoder TU.
//
// `main.cpp` compiles the unmodified `cl_cin.cpp` inside its own translation
// unit, so the whole file's engine seam has to link. This file supplies it. The
// decode core reaches exactly one seam symbol, `glConfig`, and one console call,
// `Com_Printf`. Everything else belongs to the playback shell that the byte gate
// leaves out, so it gets an aborting body: a hit means the driver walked off the
// covered set, and the run stops loudly.
//
// Two rules keep every golden run-twice byte-identical: no wall clock, and no
// address is ever printed.
//
// oracle/ is never edited.
#include "../qcommon/exe_headers.h"
#include "client.h"
#include "snd_local.h"
#include "host.h"

#include <cstdio>
#include <cstdlib>
#include <cstdarg>
#include <cstring>

// --- the one live seam object -----------------------------------------------

// `readQuadInfo` reads `glConfig.maxTextureSize`. The driver holds it above 256
// in every scenario, so the Rage Pro clamp and its print never run.
// Source: `oracle/codemp/client/cl_cin.cpp:40,812-822`
glconfig_t glConfig;

// `cl_cin.cpp` declares these three sound globals extern. Nothing the gate
// drives reads them.
// Source: `oracle/codemp/client/cl_cin.cpp:41-42,122-123`
int s_paintedtime = 0;
int s_rawend = 0;
int s_soundtime = 0;

// The harness clock never moves.
extern "C" unsigned int timeGetTime(void) { return 0; }

void cin_oracle_unreachable(const char *name)
{
	fprintf(stderr, "cin-oracle: the out-of-gate path %s ran\n", name);
	exit(1);
}

// --- console ----------------------------------------------------------------

// The only `Com_Printf` inside the gated set is the Rage Pro hack message, and
// the driver keeps `maxTextureSize` above 256 so it never fires.
void Com_Printf(const char *, ...) { cin_oracle_unreachable("Com_Printf"); }
void Com_DPrintf(const char *, ...) { cin_oracle_unreachable("Com_DPrintf"); }

void Com_Error(int level, const char *fmt, ...)
{
	va_list ap;
	va_start(ap, fmt);
	fprintf(stderr, "Com_Error(%d): ", level);
	vfprintf(stderr, fmt, ap);
	fprintf(stderr, "\n");
	va_end(ap);
	exit(1);
}

// --- the playback shell the byte gate leaves out ----------------------------

// `CIN_PlayCinematic`, `RoQInterrupt`, `RoQShutdown`, `CIN_DrawCinematic` and
// `CL_PlayCinematic_f` reach these. The driver replicates `RoQInterrupt`'s chunk
// dispatch over an in-memory fixture instead, so none of them runs.
int Sys_Milliseconds(bool) { cin_oracle_unreachable("Sys_Milliseconds"); return 0; }
void Sys_BeginStreamedFile(fileHandle_t, int) { cin_oracle_unreachable("Sys_BeginStreamedFile"); }
int Sys_StreamedRead(void *, int, int, fileHandle_t) { cin_oracle_unreachable("Sys_StreamedRead"); return 0; }
void Sys_EndStreamedFile(fileHandle_t) { cin_oracle_unreachable("Sys_EndStreamedFile"); }

int FS_FOpenFileRead(const char *, fileHandle_t *, qboolean) { cin_oracle_unreachable("FS_FOpenFileRead"); return -1; }
int FS_Read(void *, int, fileHandle_t) { cin_oracle_unreachable("FS_Read"); return 0; }
int FS_Seek(fileHandle_t, long, int) { cin_oracle_unreachable("FS_Seek"); return -1; }
void FS_FCloseFile(fileHandle_t) { cin_oracle_unreachable("FS_FCloseFile"); }

void S_RawSamples(int, int, int, int, const byte *, float, int) { cin_oracle_unreachable("S_RawSamples"); }
void S_Update(void) { cin_oracle_unreachable("S_Update"); }
void S_StopAllSounds(void) { cin_oracle_unreachable("S_StopAllSounds"); }

void Con_Close(void) { cin_oracle_unreachable("Con_Close"); }
void Cbuf_ExecuteText(int, const char *) { cin_oracle_unreachable("Cbuf_ExecuteText"); }
void Cvar_Set(const char *, const char *) { cin_oracle_unreachable("Cvar_Set"); }
char *Cvar_VariableString(const char *) { cin_oracle_unreachable("Cvar_VariableString"); return NULL; }
char *Cmd_Argv(int) { cin_oracle_unreachable("Cmd_Argv"); return NULL; }
void *Hunk_AllocateTempMemory(int) { cin_oracle_unreachable("Hunk_AllocateTempMemory"); return NULL; }
void Hunk_FreeTempMemory(void *) { cin_oracle_unreachable("Hunk_FreeTempMemory"); }
