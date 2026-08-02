// snd-oracle — deterministic host / link stubs for the Raven sound TUs.
//
// The unmodified oracle TUs (snd_dma.cpp, snd_mem.cpp, snd_mix.cpp,
// snd_music.cpp, snd_ambient.cpp) reference the engine seam. This file supplies
// the seam symbols the driven code paths reach. Everything the driver never
// reaches is dead-stripped at link, so only the live-path symbols need a body.
//
// Three rules keep every golden run-twice byte-identical:
//   1. No wall clock. `snd_oracle_clock_ms` only moves when the script says so.
//   2. No address is ever printed.
//   3. The file system reads from `fixtures/` and refuses every write.
//
// oracle/ is never edited.
#include "../qcommon/exe_headers.h"
#include "snd_local.h"
#include "host.h"

#include <cstdio>
#include <cstdlib>
#include <cstdarg>
#include <cstring>

// --- harness clock ----------------------------------------------------------

// The script advances this. `advance` moves it by the play time of the samples
// the device consumed, so the clock and the DMA cursor never disagree.
unsigned int snd_oracle_clock_ms = 0;

// The device read cursor, in the units SNDDMA_GetDMAPos returns.
int snd_oracle_dma_pos = 0;

// The console log. `dumpstate` prints the counts, never the text, because Raven
// prints file names and byte sizes that carry host detail.
int snd_oracle_print_count = 0;
int snd_oracle_dprint_count = 0;

extern "C" unsigned int timeGetTime(void) { return snd_oracle_clock_ms; }
int Com_Milliseconds(void) { return (int)snd_oracle_clock_ms; }
int Sys_Milliseconds(bool) { return (int)snd_oracle_clock_ms; }

// --- console ----------------------------------------------------------------

void Com_Printf(const char *fmt, ...)
{
	(void)fmt;
	snd_oracle_print_count++;
}

void Com_DPrintf(const char *fmt, ...)
{
	(void)fmt;
	snd_oracle_dprint_count++;
}

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

void snd_oracle_al_unreachable(const char *name)
{
	fprintf(stderr, "snd-oracle: the dropped or out-of-scope path %s ran\n", name);
	exit(1);
}

// --- cvars ------------------------------------------------------------------

// A flat table. The driver seeds values with `cvar` before `init`, and Cvar_Get
// then keeps the seeded value instead of the Raven default, the way a config
// file would.
#define SND_ORACLE_MAX_CVARS 128
static cvar_t s_oracleCvars[SND_ORACLE_MAX_CVARS];
static char s_oracleCvarNames[SND_ORACLE_MAX_CVARS][64];
static char s_oracleCvarStrings[SND_ORACLE_MAX_CVARS][128];
static int s_oracleNumCvars = 0;

static cvar_t *snd_oracle_cvar_find(const char *name)
{
	for (int i = 0; i < s_oracleNumCvars; i++) {
		if (strcmp(s_oracleCvarNames[i], name) == 0) {
			return &s_oracleCvars[i];
		}
	}
	return NULL;
}

static void snd_oracle_cvar_assign(cvar_t *cv, const char *value)
{
	int slot = (int)(cv - s_oracleCvars);
	strncpy(s_oracleCvarStrings[slot], value, sizeof(s_oracleCvarStrings[slot]) - 1);
	s_oracleCvarStrings[slot][sizeof(s_oracleCvarStrings[slot]) - 1] = 0;
	cv->string = s_oracleCvarStrings[slot];
	cv->value = (float)atof(value);
	cv->integer = atoi(value);
}

cvar_t *snd_oracle_cvar_set(const char *name, const char *value)
{
	cvar_t *cv = snd_oracle_cvar_find(name);
	if (!cv) {
		if (s_oracleNumCvars == SND_ORACLE_MAX_CVARS) {
			fprintf(stderr, "snd-oracle: the cvar table is full at %s\n", name);
			exit(1);
		}
		cv = &s_oracleCvars[s_oracleNumCvars];
		strncpy(s_oracleCvarNames[s_oracleNumCvars], name, sizeof(s_oracleCvarNames[0]) - 1);
		cv->name = s_oracleCvarNames[s_oracleNumCvars];
		s_oracleNumCvars++;
	}
	snd_oracle_cvar_assign(cv, value);
	return cv;
}

cvar_t *Cvar_Get(const char *var_name, const char *value, int flags)
{
	cvar_t *cv = snd_oracle_cvar_find(var_name);
	if (cv) {
		cv->flags |= flags;
		return cv;
	}
	cv = snd_oracle_cvar_set(var_name, value);
	cv->flags = flags;
	return cv;
}

void Cvar_Set(const char *var_name, const char *value) { snd_oracle_cvar_set(var_name, value); }

char *Cvar_VariableString(const char *var_name)
{
	static char empty[1] = "";
	cvar_t *cv = snd_oracle_cvar_find(var_name);
	return cv ? cv->string : empty;
}

// --- commands ---------------------------------------------------------------

// The console commands register and never run. The driver calls the sound API
// directly, so no command text is ever parsed.
void Cmd_AddCommand(const char *, xcommand_t) {}
void Cmd_RemoveCommand(const char *) {}
int Cmd_Argc(void) { return 1; }

char *Cmd_Argv(int)
{
	static char empty[1] = "";
	return empty;
}

// --- memory -----------------------------------------------------------------

// Raven tags every block and asks for the live size per tag. The harness keeps a
// header in front of each block, so Z_Size and Z_MemSize answer exactly.
struct SndOracleBlock {
	int size;
	int tag;
};

static int s_oracleTagBytes[TAG_COUNT];

void *Z_Malloc(int iSize, memtag_t eTag, qboolean bZeroit, int iAlign)
{
	(void)iAlign;
	SndOracleBlock *b = (SndOracleBlock *)malloc(sizeof(SndOracleBlock) + (size_t)iSize);
	if (!b) {
		Com_Error(ERR_FATAL, "Z_Malloc: out of memory (%d bytes)", iSize);
	}
	b->size = iSize;
	b->tag = (int)eTag;
	s_oracleTagBytes[(int)eTag] += iSize;
	void *p = (void *)(b + 1);
	if (bZeroit) {
		memset(p, 0, (size_t)iSize);
	}
	return p;
}

void Z_Free(void *ptr)
{
	if (!ptr) {
		return;
	}
	SndOracleBlock *b = ((SndOracleBlock *)ptr) - 1;
	s_oracleTagBytes[b->tag] -= b->size;
	free(b);
}

int Z_Size(void *pvAddress)
{
	if (!pvAddress) {
		return 0;
	}
	return (((SndOracleBlock *)pvAddress) - 1)->size;
}

int Z_MemSize(memtag_t eTag) { return s_oracleTagBytes[(int)eTag]; }

// --- file system ------------------------------------------------------------

// Every read resolves under fixtures/. Nothing writes: the MP3 re-tag pass
// (snd_mem.cpp R_CheckMP3s) is out of scope per DEC-57.3, and a write would make
// the harness state depend on the run order.
#define SND_ORACLE_MAX_HANDLES 8
static FILE *s_oracleFiles[SND_ORACLE_MAX_HANDLES];

static void snd_oracle_fixture_path(const char *qpath, char *out, size_t outSize)
{
	snprintf(out, outSize, "fixtures/%s", qpath);
	// Raven builds some names with the Windows separator. The fixture tree uses
	// forward slashes only.
	for (char *p = out; *p; p++) {
		if (*p == '\\') {
			*p = '/';
		}
	}
}

int FS_ReadFile(const char *qpath, void **buffer)
{
	char path[1024];
	snd_oracle_fixture_path(qpath, path, sizeof(path));

	FILE *f = fopen(path, "rb");
	if (!f) {
		if (buffer) {
			*buffer = NULL;
		}
		return -1;
	}
	fseek(f, 0, SEEK_END);
	long size = ftell(f);
	fseek(f, 0, SEEK_SET);

	if (!buffer) {
		fclose(f);
		return (int)size;
	}
	// Raven's file system zero-terminates every loaded file.
	byte *data = (byte *)Z_Malloc((int)size + 1, TAG_FILESYS, qtrue, 4);
	if (fread(data, 1, (size_t)size, f) != (size_t)size) {
		fclose(f);
		Com_Error(ERR_FATAL, "FS_ReadFile: short read on %s", path);
	}
	fclose(f);
	*buffer = data;
	return (int)size;
}

void FS_FreeFile(void *buffer) { Z_Free(buffer); }

int FS_FOpenFileRead(const char *qpath, fileHandle_t *file, qboolean uniqueFILE)
{
	(void)uniqueFILE;
	char path[1024];
	snd_oracle_fixture_path(qpath, path, sizeof(path));

	FILE *f = fopen(path, "rb");
	if (!f) {
		if (file) {
			*file = 0;
		}
		return -1;
	}
	fseek(f, 0, SEEK_END);
	long size = ftell(f);
	fseek(f, 0, SEEK_SET);

	for (int i = 1; i < SND_ORACLE_MAX_HANDLES; i++) {
		if (!s_oracleFiles[i]) {
			s_oracleFiles[i] = f;
			if (file) {
				*file = i;
			}
			return (int)size;
		}
	}
	fclose(f);
	Com_Error(ERR_FATAL, "FS_FOpenFileRead: out of handles for %s", path);
	return -1;
}

int FS_Read(void *buffer, int len, fileHandle_t f)
{
	if (f <= 0 || f >= SND_ORACLE_MAX_HANDLES || !s_oracleFiles[f]) {
		return 0;
	}
	return (int)fread(buffer, 1, (size_t)len, s_oracleFiles[f]);
}

void FS_FCloseFile(fileHandle_t f)
{
	if (f <= 0 || f >= SND_ORACLE_MAX_HANDLES || !s_oracleFiles[f]) {
		return;
	}
	fclose(s_oracleFiles[f]);
	s_oracleFiles[f] = NULL;
}

fileHandle_t FS_FOpenFileWrite(const char *qpath)
{
	fprintf(stderr, "snd-oracle: the harness refuses the write of %s\n", qpath);
	exit(1);
	return 0;
}

int FS_Write(const void *, int, fileHandle_t)
{
	fprintf(stderr, "snd-oracle: the harness refuses a file write\n");
	exit(1);
	return 0;
}

char **FS_ListFiles(const char *, const char *, int *numfiles)
{
	if (numfiles) {
		*numfiles = 0;
	}
	return NULL;
}

void FS_FreeFileList(char **) {}

// --- streamed reads ---------------------------------------------------------

// Raven streams the background music track through these. The driver never
// starts a background track, so a hit means the script left the covered set.
void Sys_BeginStreamedFile(fileHandle_t, int) { snd_oracle_al_unreachable("Sys_BeginStreamedFile"); }
int Sys_StreamedRead(void *, int, int, fileHandle_t) { snd_oracle_al_unreachable("Sys_StreamedRead"); return 0; }
void Sys_EndStreamedFile(fileHandle_t) { snd_oracle_al_unreachable("Sys_EndStreamedFile"); }

qboolean Sys_LowPhysicalMemory(void) { return qfalse; }

// --- renderer seam ----------------------------------------------------------

// SND_TouchSFX stamps each sfx with the level it was used on. The harness runs
// one level.
int RE_RegisterMedia_GetLevel(void) { return 1; }

// --- device seam ------------------------------------------------------------

// The five SNDDMA_* functions are the device end that DEC-57.1 dissolves into
// the cpal wrapper. The harness models the retail DirectSound secondary buffer:
// 0x10000 bytes, stereo, 16 bit, at the rate `s_khz` picks.
// Source: `oracle/codemp/win32/win_snd.cpp:12,183-250`
#define SND_ORACLE_BUFFER_BYTES 0x10000

qboolean SNDDMA_Init(void)
{
	dma.channels = 2;
	dma.samplebits = 16;

	if (s_khz->integer == 44) {
		dma.speed = 44100;
	} else if (s_khz->integer == 22) {
		dma.speed = 22050;
	} else {
		dma.speed = 11025;
	}

	dma.samples = SND_ORACLE_BUFFER_BYTES / (dma.samplebits / 8);
	dma.submission_chunk = 1;
	dma.buffer = (byte *)calloc(1, SND_ORACLE_BUFFER_BYTES);
	snd_oracle_dma_pos = 0;
	return qtrue;
}

void SNDDMA_Shutdown(void)
{
	free(dma.buffer);
	dma.buffer = NULL;
}

// DirectSound reports the hardware play cursor. The harness reports the cursor
// the script set, so the mix window is scripted and not timed.
int SNDDMA_GetDMAPos(void) { return snd_oracle_dma_pos & (dma.samples - 1); }

// The retail pair locks and unlocks the DirectSound buffer. The harness owns the
// buffer outright, so both are empty.
void SNDDMA_BeginPainting(void) {}
void SNDDMA_Submit(void) {}

// --- MP3 seam ---------------------------------------------------------------

// DEC-57.3 keeps the decoder outside the byte gate: MP3 content enters as
// decoded PCM fixtures. Nothing below runs, and a hit means the script fed the
// harness an .mp3 name.
cvar_t *cv_MP3overhead = NULL;
const char sKEY_MAXVOL[] = "#MAXVOL=";
const char sKEY_UNCOMP[] = "#UNCOMP=";

void MP3_InitCvars(void) { cv_MP3overhead = Cvar_Get("s_mp3overhead", "0", 0); }
sboolean MP3_IsValid(const char *, void *, int, sboolean) { snd_oracle_al_unreachable("MP3_IsValid"); return qfalse; }
int MP3_GetUnpackedSize(const char *, void *, int, sboolean, sboolean) { snd_oracle_al_unreachable("MP3_GetUnpackedSize"); return 0; }
sboolean MP3_UnpackRawPCM(const char *, void *, int, byte *, sboolean) { snd_oracle_al_unreachable("MP3_UnpackRawPCM"); return qfalse; }
sboolean MP3_FakeUpWAVInfo(const char *, void *, int, int, int &, int &, int &, int &, int &, int &, sboolean) { snd_oracle_al_unreachable("MP3_FakeUpWAVInfo"); return qfalse; }
sboolean MP3_ReadSpecialTagInfo(byte *, int, id3v1_1 **, int *, float *) { snd_oracle_al_unreachable("MP3_ReadSpecialTagInfo"); return qfalse; }
sboolean MP3Stream_InitFromFile(sfx_t *, byte *, int, const char *, int, sboolean) { snd_oracle_al_unreachable("MP3Stream_InitFromFile"); return qfalse; }
int MP3Stream_GetSamples(channel_t *, int, int, short *, sboolean) { snd_oracle_al_unreachable("MP3Stream_GetSamples"); return 0; }
sboolean MP3Stream_Rewind(channel_t *) { snd_oracle_al_unreachable("MP3Stream_Rewind"); return qfalse; }
sboolean MP3Stream_SeekTo(channel_t *, float) { snd_oracle_al_unreachable("MP3Stream_SeekTo"); return qfalse; }
float MP3Stream_GetPlayingTimeInSeconds(MP3STREAM *) { snd_oracle_al_unreachable("MP3Stream_GetPlayingTimeInSeconds"); return 0.0f; }
float MP3Stream_GetRemainingTimeInSeconds(MP3STREAM *) { snd_oracle_al_unreachable("MP3Stream_GetRemainingTimeInSeconds"); return 0.0f; }
sboolean MP3Stream_InitPlayingTimeFields(MP3STREAM *, const char *, void *, int, sboolean) { snd_oracle_al_unreachable("MP3Stream_InitPlayingTimeFields"); return qfalse; }
int MP3Stream_Decode(MP3STREAM *, sboolean) { snd_oracle_al_unreachable("MP3Stream_Decode"); return 0; }

extern "C" char *C_MP3Stream_DecodeInit(LP_MP3STREAM, void *, int, int, int, int)
{
	snd_oracle_al_unreachable("C_MP3Stream_DecodeInit");
	return NULL;
}

extern "C" unsigned int C_MP3Stream_Decode(LP_MP3STREAM, int)
{
	snd_oracle_al_unreachable("C_MP3Stream_Decode");
	return 0;
}

// --- misc engine seam -------------------------------------------------------

// Raven builds release pak files under this cvar. The harness never does.
cvar_t *com_buildScript = NULL;
