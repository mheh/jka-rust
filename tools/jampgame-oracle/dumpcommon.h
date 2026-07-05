// Shared bit-exact dump helpers for the jampgame-oracle dumpers.
// Every float is printed as its IEEE-754 bit pattern (%08x for f32,
// %016llx for f64) so the Rust parity test can reproduce it exactly with
// f32::to_bits()/f64::to_bits() — no textual float rounding is ever involved.
#ifndef JAMPGAME_ORACLE_DUMPCOMMON_H
#define JAMPGAME_ORACLE_DUMPCOMMON_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static unsigned f2b(float f) {
	union { float f; unsigned u; } u;
	u.f = f;
	return u.u;
}
static unsigned long long d2b(double d) {
	union { double d; unsigned long long u; } u;
	u.d = d;
	return u.u;
}

#define PF(f)     printf("%08x\n", f2b(f))
#define PD(d)     printf("%016llx\n", d2b(d))
#define PV3(v)    printf("%08x %08x %08x\n", f2b((v)[0]), f2b((v)[1]), f2b((v)[2]))
#define PV4(v)    printf("%08x %08x %08x %08x\n", f2b((v)[0]), f2b((v)[1]), f2b((v)[2]), f2b((v)[3]))

// Read whole file into a malloc'd buffer; sets *len. Caller frees.
static char *slurp(const char *path, long *len) {
	FILE *f = fopen(path, "rb");
	if (!f) { fprintf(stderr, "cannot open %s\n", path); exit(2); }
	fseek(f, 0, SEEK_END);
	long n = ftell(f);
	fseek(f, 0, SEEK_SET);
	char *buf = (char *)malloc(n + 1);
	fread(buf, 1, n, f);
	buf[n] = 0;
	fclose(f);
	if (len) *len = n;
	return buf;
}

#endif
