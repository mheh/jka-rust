// roff-oracle — deterministic engine host for the unmodified RoffSystem.cpp.
//
// Implements the q_shared / qcommon / server seam ROFF calls with fully
// deterministic behaviour: a fixture-backed FS (files under fixtures/), a mock
// gentity array (SV_GentityNum), a controllable svs clock, a note-track VM_Call
// log, and console capture (Com_Printf -> stdout, part of the golden). The two
// vec math helpers (AngleVectors, COM_StripExtension) are copied faithfully from
// q_math.c / q_shared.c so the translated-playback golden is bit-exact.
// oracle/ is untouched.
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cstdarg>
#include <cmath>
#include <string>
#include <vector>

#include "game/q_shared.h"
#include "server/server.h"
#include "host.h"

// ---------------------------------------------------------------------------
// server globals the TU expects (normally in sv_main / vm)
// ---------------------------------------------------------------------------
serverStatic_t svs = { 0 };
vm_t *gvm = (vm_t *)0x1;   // non-null sentinel; VM_Call ignores the pointer value

// ---------------------------------------------------------------------------
// clock
// ---------------------------------------------------------------------------
void host_set_time(int t) { svs.time = t; }
int  host_get_time(void)  { return svs.time; }

// ---------------------------------------------------------------------------
// mock gentity array
// ---------------------------------------------------------------------------
#define HOST_MAX_ENTS 64
static sharedEntity_t g_entities[HOST_MAX_ENTS];

void host_reset_entities(void) { memset(g_entities, 0, sizeof(g_entities)); }

void host_set_ent_angles(int num, float pitch, float yaw, float roll) {
	if (num < 0 || num >= HOST_MAX_ENTS) return;
	g_entities[num].s.apos.trBase[PITCH] = pitch;
	g_entities[num].s.apos.trBase[YAW]   = yaw;
	g_entities[num].s.apos.trBase[ROLL]  = roll;
}

extern "C" sharedEntity_t *SV_GentityNum(int num) {
	if (num < 0 || num >= HOST_MAX_ENTS) return 0;
	return &g_entities[num];
}

// ---------------------------------------------------------------------------
// note-track VM_Call log
// ---------------------------------------------------------------------------
struct NoteEmit { int callNum; int entnum; std::string text; };
static std::vector<NoteEmit> g_notes;

int         host_note_count(void)        { return (int)g_notes.size(); }
int         host_note_callnum(int i)     { return g_notes[i].callNum; }
int         host_note_entnum(int i)      { return g_notes[i].entnum; }
const char *host_note_text(int i)        { return g_notes[i].text.c_str(); }
void        host_note_clear(void)        { g_notes.clear(); }

// VM_Call — ProcessNote's server arm is VM_Call(gvm, GAME_ROFF_NOTETRACK_CALLBACK,
// entnum, char *note). Record it; return 0.
extern "C" int VM_Call(vm_t * /*vm*/, int callNum, ...) {
	va_list ap; va_start(ap, callNum);
	int entnum = va_arg(ap, int);
	const char *note = va_arg(ap, const char *);
	va_end(ap);
	g_notes.push_back(NoteEmit{ callNum, entnum, note ? note : "" });
	return 0;
}

// ---------------------------------------------------------------------------
// ILP32<->LP64 header shim (ROFF-D4).
//
// The oracle's TROFFHeader/TROFF2Header spell mVersion as `long`, so on the
// shipped 32-bit WinDed target the v1 header is 12 bytes / v2 header 20 bytes and
// the parse matches the on-disk (4-byte-version) format. On this LP64 host `long`
// is 8 bytes: the unmodified oracle would read mVersion (and every following
// field) at the wrong offset and reject every valid fixture. Since no 32-bit
// toolchain is available here (`g++-16 -m32` unsupported on arm64), and oracle/
// must stay unedited, this shim re-lays the committed *ship-format* 4-byte-header
// fixture into the host's `long`-width struct layout before handing it to the
// oracle. The parsed VALUES (version, count, frameRate, entries, notes) are
// identical to the ship parse, so the golden is ship-faithful; on a real ILP32
// build sizeof(long)==4 and this shim is a no-op. See README.md § LP64 shim.
// ---------------------------------------------------------------------------
static unsigned char *roff_to_host_layout(const unsigned char *in, int len, int *outlen) {
	if (len < 8 || memcmp(in, "ROFF", 4) != 0) {          // not a roff: verbatim
		unsigned char *o = (unsigned char *)malloc(len > 0 ? len : 1);
		memcpy(o, in, len > 0 ? len : 0);
		*outlen = len;
		return o;
	}
	int32_t version; memcpy(&version, in + 4, 4);
	long widened = (long)version;                          // sign-extend to host long
	if (version == 2) {                                    // v2: file 20B hdr -> host 32B
		int body = len - 20; if (body < 0) body = 0;
		int olen = 32 + body;
		unsigned char *o = (unsigned char *)calloc(olen, 1);
		// mHeader[4] + the ship version bytes in the pad slot (4..7): IsROFF's
		// strcmp(mHeader, "ROFF") reads past mHeader into these bytes, and the
		// nonzero version low byte is exactly what makes valid files pass (ROFF-V1).
		memcpy(o, in, 8);
		memcpy(o + 8,  &widened, sizeof(long));            // mVersion (long @8)
		memcpy(o + 16, in + 8,  4);                        // mCount    @16
		memcpy(o + 20, in + 12, 4);                        // mFrameRate@20
		memcpy(o + 24, in + 16, 4);                        // mNumNotes @24
		if (body) memcpy(o + 32, in + 20, body);           // entries + note blob
		*outlen = olen;
		return o;
	}
	// v1 (and reject versions): file 12B hdr -> host 24B
	int body = len - 12; if (body < 0) body = 0;
	int olen = 24 + body;
	unsigned char *o = (unsigned char *)calloc(olen, 1);
	// mHeader[4] + ship version bytes in the pad slot (4..7) — preserves the
	// ROFF-V1 strcmp-past-header quirk (see the v2 branch note).
	memcpy(o, in, 8);
	memcpy(o + 8,  &widened, sizeof(long));                // mVersion (long @8)
	memcpy(o + 16, in + 8,  4);                            // mCount (float) @16
	if (body) memcpy(o + 24, in + 12, body);               // entries
	*outlen = olen;
	return o;
}

// ---------------------------------------------------------------------------
// filesystem — files served from fixtures/ (ship-format 4-byte headers)
// ---------------------------------------------------------------------------
extern "C" {

int FS_ReadFile(const char *qpath, void **buffer) {
	std::string path = std::string("fixtures/") + qpath;
	FILE *f = fopen(path.c_str(), "rb");
	if (!f) { if (buffer) *buffer = 0; return -1; }
	fseek(f, 0, SEEK_END);
	long flen = ftell(f);
	fseek(f, 0, SEEK_SET);
	unsigned char *raw = (unsigned char *)malloc(flen > 0 ? flen : 1);
	if (fread(raw, 1, flen, f) != (size_t)flen) { fclose(f); free(raw); if (buffer) *buffer = 0; return -1; }
	fclose(f);
	if (sizeof(long) == 4) {                               // real ILP32 build: verbatim
		if (buffer) *buffer = raw;
		return (int)flen;
	}
	int olen = 0;
	unsigned char *host = roff_to_host_layout(raw, (int)flen, &olen);
	free(raw);
	if (buffer) *buffer = host;
	return olen;                                           // length the oracle sees
}

void FS_FreeFile(void *buffer) { free(buffer); }

// ---------------------------------------------------------------------------
// console
// ---------------------------------------------------------------------------
void Com_Printf(const char *fmt, ...) {
	va_list ap; va_start(ap, fmt);
	vprintf(fmt, ap);
	va_end(ap);
}

// va — q_shared.c ring-buffer variadic formatter.
char *va(const char *fmt, ...) {
	static char buf[4][1024];
	static int idx = 0;
	char *out = buf[idx++ & 3];
	va_list ap; va_start(ap, fmt);
	vsnprintf(out, sizeof(buf[0]), fmt, ap);
	va_end(ap);
	return out;
}

// COM_StripExtension — faithful copy of q_shared.c.
void COM_StripExtension(const char *in, char *out) {
	while (*in && *in != '.') {
		*out++ = *in++;
	}
	*out = 0;
}

// AngleVectors — faithful copy of q_math.c:1315 (drives the translated golden).
void AngleVectors(const vec3_t angles, vec3_t forward, vec3_t right, vec3_t up) {
	float angle;
	static float sr, sp, sy, cr, cp, cy;

	// q_math.c uses double sin/cos (angle promoted); match for bit-exactness.
	angle = angles[YAW] * (M_PI * 2 / 360);
	sy = sin(angle);
	cy = cos(angle);
	angle = angles[PITCH] * (M_PI * 2 / 360);
	sp = sin(angle);
	cp = cos(angle);
	angle = angles[ROLL] * (M_PI * 2 / 360);
	sr = sin(angle);
	cr = cos(angle);

	if (forward) {
		forward[0] = cp * cy;
		forward[1] = cp * sy;
		forward[2] = -sp;
	}
	if (right) {
		right[0] = (-1 * sr * sp * cy + -1 * cr * -sy);
		right[1] = (-1 * sr * sp * sy + -1 * cr * cy);
		right[2] = -1 * sr * cp;
	}
	if (up) {
		up[0] = (cr * sp * cy + -sr * -sy);
		up[1] = (cr * sp * sy + -sr * cy);
		up[2] = cr * cp;
	}
}

} // extern "C"
