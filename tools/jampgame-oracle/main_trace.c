// main_trace.c -- raw-trace mini-dumper. Proves the pmworld.h axial-brush trace
// stub IN ISOLATION, before any pmove logic is layered on top. It reads a
// fixture describing a set of box brushes and a list of box sweeps, runs each
// sweep through pm_trace_impl(), and dumps every trace_t output field as
// IEEE-754 bit-hex. The committed golden (golden/pmove_trace.txt) is the
// contract the Rust TestTraps must reproduce for the trace layer alone, so a
// pmove mismatch can never be blamed on the collision stub.
//
// Fixture grammar (see fixtures/pmove/trace.txt):
//   # comment / blank lines ignored
//   brush  <x0> <y0> <z0> <x1> <y1> <z1> surf=<hex>
//   sweep  <sx> <sy> <sz> <ex> <ey> <ez> <mnx> <mny> <mnz> <mxx> <mxy> <mxz>
//   reset                         -- clear the brush set (start a new world)
// Every coordinate is either a plain (possibly negative) integer -- parsed
// exactly as (float)atol -- or an "0x????????" f32 bit pattern (used for
// fractional offsets like the +/-0.125 clip epsilon probes). No decimal-point
// tokens: those would double-round differently on the two sides.
#include "pmworld.h"
#include "dumpcommon.h"

#include <ctype.h>

static float parse_float(const char *tok) {
	if (tok[0] == '0' && (tok[1] == 'x' || tok[1] == 'X')) {
		union { float f; unsigned u; } u;
		u.u = (unsigned)strtoul(tok, NULL, 16);
		return u.f;
	}
	return (float)atol(tok);
}

// Split a line into whitespace tokens (mutates buf).
static int tokenize(char *buf, char *tok[], int maxtok) {
	int n = 0;
	char *p = buf;
	while (*p && n < maxtok) {
		while (*p && isspace((unsigned char)*p)) p++;
		if (!*p) break;
		tok[n++] = p;
		while (*p && !isspace((unsigned char)*p)) p++;
		if (*p) *p++ = 0;
	}
	return n;
}

static int parse_surf(const char *tok) {
	const char *eq = strchr(tok, '=');
	if (eq) return (int)strtol(eq + 1, NULL, 0);
	return 0;
}

int main(int argc, char **argv) {
	FILE *f;
	char line[1024];
	int sweepNo = 0;

	if (argc != 2) { fprintf(stderr, "usage: %s <fixture-file>\n", argv[0]); return 2; }
	f = fopen(argv[1], "rb");
	if (!f) { fprintf(stderr, "cannot open %s\n", argv[1]); return 2; }

	pmw_reset_world();
	printf("== pmove_trace ==\n");

	while (fgets(line, sizeof(line), f)) {
		char *tok[24];
		int n;
		char *hash = strchr(line, '#');
		if (hash) *hash = 0;
		n = tokenize(line, tok, 24);
		if (n == 0) continue;

		if (!strcmp(tok[0], "reset")) {
			pmw_reset_world();
			printf("reset\n");
		} else if (!strcmp(tok[0], "brush") && n >= 7) {
			int surf = (n >= 8) ? parse_surf(tok[7]) : 0;
			pmw_add_brush(parse_float(tok[1]), parse_float(tok[2]), parse_float(tok[3]),
			              parse_float(tok[4]), parse_float(tok[5]), parse_float(tok[6]), surf);
			printf("brush %d surf=%x\n", g_pmw_numBrushes - 1, surf);
		} else if (!strcmp(tok[0], "sweep") && n >= 13) {
			vec3_t start, end, mins, maxs;
			trace_t tr;
			start[0] = parse_float(tok[1]); start[1] = parse_float(tok[2]); start[2] = parse_float(tok[3]);
			end[0]   = parse_float(tok[4]); end[1]   = parse_float(tok[5]); end[2]   = parse_float(tok[6]);
			mins[0]  = parse_float(tok[7]); mins[1]  = parse_float(tok[8]); mins[2]  = parse_float(tok[9]);
			maxs[0]  = parse_float(tok[10]); maxs[1] = parse_float(tok[11]); maxs[2]  = parse_float(tok[12]);
			pm_trace_impl(&tr, start, mins, maxs, end);
			printf("sweep %d as=%d ss=%d frac=%08x end=%08x,%08x,%08x nrm=%08x,%08x,%08x "
			       "pd=%08x pt=%d psb=%d sf=%x ct=%x en=%d\n",
			       sweepNo++,
			       tr.allsolid, tr.startsolid, f2b(tr.fraction),
			       f2b(tr.endpos[0]), f2b(tr.endpos[1]), f2b(tr.endpos[2]),
			       f2b(tr.plane.normal[0]), f2b(tr.plane.normal[1]), f2b(tr.plane.normal[2]),
			       f2b(tr.plane.dist), tr.plane.type, tr.plane.signbits,
			       tr.surfaceFlags, tr.contents, tr.entityNum);
		} else {
			fprintf(stderr, "trace: bad line: %s\n", tok[0]);
			return 2;
		}
	}
	fclose(f);
	printf("== end ==\n");
	return 0;
}
