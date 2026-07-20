// q_shared differential-oracle dumper. Compiled against the UNMODIFIED Raven
// codemp/game/q_shared.c (copied into build-qshared/ by run_qshared.sh) plus
// this TU's Com_Error/Com_Printf stubs (routed to stderr so parser diagnostics
// never enter the golden on stdout). Prints a canonical, byte-exact dump that
// the Rust parity test (crates/mp/game/tests/qshared_parity.rs) reproduces by
// calling the ported mp_game::q_shared functions.
//
// The whole dump is a SINGLE process, mirroring q_shared.c's file statics
// (com_lines, va's rotating buffer). The Rust test mirrors the section order.
//
// Scope note: Com_sprintf / va are exercised with LITERAL formats only (no `%`
// conversions). The mp_game port's Com_sprintf/va are documented variadic
// stubs that echo the format string without argument substitution, so `%s/%i/
// %d/%c/%x` cannot reach parity and are reported as a finding, not fixtured.
#include "q_shared.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>

// --- engine externs referenced by q_shared.c: routed to stderr (off-golden) ---
void QDECL Com_Printf(const char *fmt, ...) {
	va_list ap; va_start(ap, fmt); vfprintf(stderr, fmt, ap); va_end(ap);
}
// q_shared.h has a case typo (`Info_RemoveKey_big`); q_shared.c defines the
// capital-B `Info_RemoveKey_Big`. Forward-declare the real symbol.
void Info_RemoveKey_Big(char *s, const char *key);

void QDECL Com_Error(int level, const char *fmt, ...) {
	(void)level;
	va_list ap; va_start(ap, fmt); vfprintf(stderr, fmt, ap); va_end(ap);
	fputc('\n', stderr);
	exit(3); // mirrors the port's panic! on Com_Error(ERR_*) sites
}

// --- canonical emit helpers (mirrored byte-for-byte in the Rust test) ---
// A quoted, escaped string: printable ASCII except '"' and '\' verbatim,
// everything else (control, high-bit, quote, backslash) as \xHH (lowercase).
static void emit_qstr(const char *s) {
	putchar('"');
	for (const unsigned char *p = (const unsigned char *)s; *p; p++) {
		unsigned c = *p;
		if (c >= 0x20 && c <= 0x7e && c != '"' && c != '\\')
			putchar((int)c);
		else
			printf("\\x%02x", c);
	}
	putchar('"');
}

// A fixed-width byte window as space-separated %02x (captures NUL padding).
static void emit_hex(const unsigned char *b, int n) {
	for (int i = 0; i < n; i++) printf("%s%02x", i ? " " : "", b[i]);
}

static char *slurp(const char *path, long *len) {
	FILE *f = fopen(path, "rb");
	if (!f) { fprintf(stderr, "cannot open %s\n", path); exit(2); }
	fseek(f, 0, SEEK_END); long n = ftell(f); fseek(f, 0, SEEK_SET);
	char *buf = (char *)malloc(n + 1);
	fread(buf, 1, n, f); buf[n] = 0; fclose(f);
	if (len) *len = n;
	return buf;
}

// ===================== Com_Clamp / Com_Clampi =====================
static void dump_clamp(void) {
	printf("== clamp ==\n");
	static const int iv[] = { -100, -1, 0, 1, 5, 50, 100 };
	for (unsigned a = 0; a < sizeof(iv)/sizeof(iv[0]); a++)
		for (unsigned b = 0; b < sizeof(iv)/sizeof(iv[0]); b++)
			for (unsigned c = 0; c < sizeof(iv)/sizeof(iv[0]); c++)
				printf("ci %d\n", Com_Clampi(iv[a], iv[b], iv[c]));
	// floats as IEEE-754 bit-hex
	union { float f; unsigned u; } cv[] = {
		{-100.0f}, {-1.5f}, {0.0f}, {1.5f}, {5.25f}, {100.0f},
	};
	int nf = (int)(sizeof(cv)/sizeof(cv[0]));
	for (int a = 0; a < nf; a++)
		for (int b = 0; b < nf; b++)
			for (int c = 0; c < nf; c++) {
				union { float f; unsigned u; } r;
				r.f = Com_Clamp(cv[a].f, cv[b].f, cv[c].f);
				printf("cf %08x\n", r.u);
			}
}

// ===================== tokenizer (COM_ParseExt) =====================
static void dump_tokens(const char *dir) {
	printf("== tokens ==\n");
	char path[1024];
	snprintf(path, sizeof(path), "%s/tokens.txt", dir);
	long len; char *buf = slurp(path, &len);

	// pass 1: allowLineBreaks = qtrue
	COM_BeginParseSession("tokens");
	const char *p = buf;
	for (int i = 0; i < 200; i++) {
		char *tok = COM_ParseExt(&p, qtrue);
		int eof = (p == NULL);
		printf("qt "); emit_qstr(tok);
		printf(" line %d nul %d\n", COM_GetCurrentParseLine(), eof);
		if (tok[0] == 0 && eof) break;
	}

	// pass 2: allowLineBreaks = qfalse (empty token returned at line breaks)
	COM_BeginParseSession("tokens");
	p = buf;
	for (int i = 0; i < 200; i++) {
		char *tok = COM_ParseExt(&p, qfalse);
		int eof = (p == NULL);
		printf("qf "); emit_qstr(tok);
		printf(" line %d nul %d\n", COM_GetCurrentParseLine(), eof);
		if (eof) break;
	}
	free(buf);
}

// ===================== COM_Compress =====================
static void dump_compress(const char *dir) {
	printf("== compress ==\n");
	char path[1024];
	snprintf(path, sizeof(path), "%s/compress.txt", dir);
	long len; char *buf = slurp(path, &len);
	long off = 0; int idx = 0;
	while (off < len) {
		// COM_Compress mutates in place; copy the record into a scratch buffer.
		const char *rec = buf + off;
		size_t l = strlen(rec);
		char tmp[512];
		memcpy(tmp, rec, l + 1);
		int r = COM_Compress(tmp);
		printf("cz %d len %d ", idx, r); emit_qstr(tmp); printf("\n");
		off += l + 1; idx++;
	}
	free(buf);
}

// ===================== SkipBracedSection =====================
static void dump_braced(void) {
	printf("== braced ==\n");
	static const char *cases[] = {
		"{ a { b } c } after",
		"{ } trailing",
		"{nested{deeper{x}}} end",
		"{ unterminated { block ",
		"noBrace token here",
	};
	for (unsigned i = 0; i < sizeof(cases)/sizeof(cases[0]); i++) {
		char scratch[256];
		strcpy(scratch, cases[i]);
		COM_BeginParseSession("braced");
		const char *p = scratch;
		SkipBracedSection(&p);
		long off = (p == NULL) ? -1 : (long)(p - scratch);
		char *rest = (p == NULL) ? (char *)"" : COM_ParseExt((const char **)&p, qtrue);
		printf("br %u off %ld line %d rest ", i, off, COM_GetCurrentParseLine());
		emit_qstr(rest); printf("\n");
	}
}

// ===================== SkipRestOfLine =====================
static void dump_skipline(void) {
	printf("== skipline ==\n");
	static const char *cases[] = {
		"rest of line\nnext line",
		"no newline here",
		"\nimmediate",
		"a\nb\nc",
	};
	for (unsigned i = 0; i < sizeof(cases)/sizeof(cases[0]); i++) {
		char scratch[256];
		strcpy(scratch, cases[i]);
		COM_BeginParseSession("skipline");
		const char *p = scratch;
		SkipRestOfLine(&p);
		// Raven's SkipRestOfLine consumes the terminating NUL (`c = *p++`) when
		// there is no newline, leaving *data ONE PAST the NUL -- dereferencing
		// it is UB (indeterminate stack/heap bytes). The observable is the
		// cursor offset + line counter, so `rest` past-terminator is not dumped
		// (porting-rules §19).
		long off = (long)(p - scratch);
		printf("sl %u off %ld line %d\n", i, off, COM_GetCurrentParseLine());
	}
}

// ===================== string helpers =====================
static void dump_strhelpers(void) {
	printf("== strhelpers ==\n");

	// isXXX predicates over signed-char edges.
	static const int ic[] = { -1, 0, 0x1f, 0x20, 0x40, 0x5a, 0x61, 0x7a, 0x7e, 0x7f, 0x80, 0xff };
	for (unsigned i = 0; i < sizeof(ic)/sizeof(ic[0]); i++) {
		int c = ic[i];
		printf("is %d p %d l %d u %d a %d\n",
			c, Q_isprint(c), Q_islower(c), Q_isupper(c), Q_isalpha(c));
	}

	// COM_StripExtension.
	static const char *sx[] = { "", "file", "file.ext", "a.b.c", ".hidden", "no_dot", "path/to/file.tga" };
	for (unsigned i = 0; i < sizeof(sx)/sizeof(sx[0]); i++) {
		char out[64];
		COM_StripExtension(sx[i], out);
		printf("strip "); emit_qstr(sx[i]); printf(" -> "); emit_qstr(out); printf("\n");
	}

	// COM_DefaultExtension (non-empty paths; empty path reads path[-1] = UB).
	struct { const char *p; const char *e; } dx[] = {
		{"file", ".tga"}, {"file.bmp", ".tga"}, {"path/to/file", ".md3"},
		{"path/to/file.ext", ".md3"}, {"noext", ".cfg"}, {"dir.x/name", ".wav"},
	};
	for (unsigned i = 0; i < sizeof(dx)/sizeof(dx[0]); i++) {
		char path[128]; strcpy(path, dx[i].p);
		COM_DefaultExtension(path, (int)sizeof(path), dx[i].e);
		printf("defext "); emit_qstr(dx[i].p); printf(" "); emit_qstr(dx[i].e);
		printf(" -> "); emit_qstr(path); printf("\n");
	}

	// Q_strncpyz (24-byte window filled with 0xAA to observe zero-padding).
	static const char *ncz[] = { "", "hi", "exactfit", "longerthanthebuffer.....", "abc" };
	static const int nsz[] = { 1, 2, 4, 8, 16 };
	for (unsigned i = 0; i < sizeof(ncz)/sizeof(ncz[0]); i++)
		for (unsigned j = 0; j < sizeof(nsz)/sizeof(nsz[0]); j++) {
			unsigned char b[24]; memset(b, 0xAA, sizeof(b));
			Q_strncpyz((char *)b, ncz[i], nsz[j]);
			printf("ncpyz %u %d ", i, nsz[j]); emit_hex(b, 24); printf("\n");
		}

	// Q_strcat (skip combos the C would Com_Error-abort on: strlen(init) >= size).
	static const char *cat_i[] = { "", "foo", "12345" };
	static const char *cat_s[] = { "", "bar", "appendmelong" };
	static const int cat_sz[] = { 4, 8, 16 };
	for (unsigned a = 0; a < sizeof(cat_i)/sizeof(cat_i[0]); a++)
		for (unsigned b = 0; b < sizeof(cat_s)/sizeof(cat_s[0]); b++)
			for (unsigned z = 0; z < sizeof(cat_sz)/sizeof(cat_sz[0]); z++) {
				if ((int)strlen(cat_i[a]) >= cat_sz[z]) continue;
				unsigned char buf[24]; memset(buf, 0xAA, sizeof(buf));
				strcpy((char *)buf, cat_i[a]);
				Q_strcat((char *)buf, cat_sz[z], cat_s[b]);
				printf("cat %u %u %d ", a, b, cat_sz[z]); emit_hex(buf, 24); printf("\n");
			}

	// Q_stricmp / Q_stricmpn / Q_strncmp over case/prefix/high-bit pairs.
	static const char *cmp_a[] = { "", "a", "a", "abc", "abc", "Hello", "hello", "abc", "ab", "zoo", "Test123", "\x80x", "a\x80", "MixedCase" };
	static const char *cmp_b[] = { "", "a", "A", "abd", "abc", "hello", "HELLO", "ab", "abc", "zoon", "test123", "\x80x", "a\x7f", "mixedcase" };
	static const int cmp_n[] = { 0, 1, 2, 3, 5, 99999 };
	for (unsigned i = 0; i < sizeof(cmp_a)/sizeof(cmp_a[0]); i++) {
		printf("stricmp %u %d\n", i, Q_stricmp(cmp_a[i], cmp_b[i]));
		for (unsigned j = 0; j < sizeof(cmp_n)/sizeof(cmp_n[0]); j++)
			printf("stricmpn %u %d %d\n", i, cmp_n[j], Q_stricmpn(cmp_a[i], cmp_b[i], cmp_n[j]));
		for (unsigned j = 0; j < sizeof(cmp_n)/sizeof(cmp_n[0]); j++)
			printf("strncmp %u %d %d\n", i, cmp_n[j], Q_strncmp(cmp_a[i], cmp_b[i], cmp_n[j]));
	}
	// Q_stricmp / Q_stricmpn NULL handling.
	printf("stricmp_null %d %d %d\n",
		Q_stricmp(NULL, "x"), Q_stricmp("x", NULL), Q_stricmp(NULL, NULL));
	printf("stricmpn_null %d %d %d\n",
		Q_stricmpn(NULL, NULL, 5), Q_stricmpn(NULL, "x", 5), Q_stricmpn("x", NULL, 5));

	// Q_strlwr / Q_strupr (ASCII only; libc tolower/toupper on signed high-bit is UB).
	static const char *lu[] = { "", "Hello World", "ALLCAPS", "already lower", "MiXeD123!@#" };
	for (unsigned i = 0; i < sizeof(lu)/sizeof(lu[0]); i++) {
		char a[64], b[64]; strcpy(a, lu[i]); strcpy(b, lu[i]);
		Q_strlwr(a); Q_strupr(b);
		printf("lwr "); emit_qstr(a); printf(" upr "); emit_qstr(b); printf("\n");
	}

	// Q_PrintStrlen / Q_CleanStr over color/control-byte strings.
	static const char *col[] = {
		"", "hello", "^1red", "^1r^2g^3b", "^^literal", "^8notcolor",
		"trailing^", "^", "a\x01""b\x1f""c\x7f""d\x80""e", "^7white^0black^", "plain text 123",
	};
	for (unsigned i = 0; i < sizeof(col)/sizeof(col[0]); i++) {
		printf("pslen %u %d\n", i, Q_PrintStrlen(col[i]));
		char c[64]; strcpy(c, col[i]);
		Q_CleanStr(c);
		printf("clean %u ", i); emit_qstr(c); printf("\n");
	}
	printf("pslen_null %d\n", Q_PrintStrlen(NULL));

	// Q_strrchr: dump offset of the found char (or -1).
	static const char *rc[] = { "", "a", "hello", "abracadabra", "a/b/c/d", "trailing/", "^1color" };
	static const int rcc[] = { 'a', '/', 'z', 0, 'r' };
	for (unsigned i = 0; i < sizeof(rc)/sizeof(rc[0]); i++)
		for (unsigned j = 0; j < sizeof(rcc)/sizeof(rcc[0]); j++) {
			char *r = Q_strrchr(rc[i], rcc[j]);
			long off = r ? (long)(r - rc[i]) : -1;
			printf("rrchr %u %d %ld\n", i, rcc[j], off);
		}
}

// ===================== va (rotating buffer, literal formats only) =====================
static void dump_va(void) {
	printf("== va ==\n");
	char *p1 = va("first-literal");
	char *p2 = va("second-literal");
	printf("va1 "); emit_qstr(p1); printf("\n");
	printf("va2 "); emit_qstr(p2); printf("\n");
	char *p3 = va("third-literal");           // reuses p1's slot (2-slot rotation)
	printf("va1b "); emit_qstr(p1); printf("\n");
	printf("va2b "); emit_qstr(p2); printf("\n");
	printf("va3 "); emit_qstr(p3); printf("\n");
}

// ===================== Com_sprintf (literal formats only) =====================
static void dump_sprintf(void) {
	printf("== sprintf ==\n");
	{ unsigned char b[24]; memset(b, 0xAA, sizeof(b)); Com_sprintf((char *)b, 24, "hello world");
	  printf("sp1 "); emit_hex(b, 24); printf("\n"); }
	{ unsigned char b[24]; memset(b, 0xAA, sizeof(b)); Com_sprintf((char *)b, 8, "truncate me please");
	  printf("sp2 "); emit_hex(b, 24); printf("\n"); }
	{ unsigned char b[24]; memset(b, 0xAA, sizeof(b)); Com_sprintf((char *)b, 24, "");
	  printf("sp3 "); emit_hex(b, 24); printf("\n"); }
	{ unsigned char b[24]; memset(b, 0xAA, sizeof(b)); Com_sprintf((char *)b, 1, "anything");
	  printf("sp4 "); emit_hex(b, 24); printf("\n"); }
}

// ===================== info strings =====================
static const char *VKEYS[] = { "name", "team", "key2", "empty", "onlykey", "desc", "missing", "Name", "" };
static const char *RKEYS[] = { "name", "team", "key1", "desc", "onlykey", "missing", "quote", "semi" };
static const struct { const char *k; const char *v; } SKV[] = {
	{ "name", "alice" }, { "new", "val" }, { "team", "" }, { "x", "y" }, { "quote", "a\"b" },
};

static void dump_info_record(int idx, const char *s) {
	printf("rec %d ", idx); emit_qstr(s); printf("\n");
	printf("val %d\n", Info_Validate(s));

	for (unsigned i = 0; i < sizeof(VKEYS)/sizeof(VKEYS[0]); i++) {
		char *v = Info_ValueForKey(s, VKEYS[i]);
		printf("vfk "); emit_qstr(VKEYS[i]); printf(" "); emit_qstr(v); printf("\n");
	}

	for (unsigned i = 0; i < sizeof(RKEYS)/sizeof(RKEYS[0]); i++) {
		char tmp[1100]; memset(tmp, 0, sizeof(tmp)); strcpy(tmp, s);
		Info_RemoveKey(tmp, RKEYS[i]);
		printf("rk "); emit_qstr(RKEYS[i]); printf(" "); emit_qstr(tmp); printf("\n");
	}
	for (unsigned i = 0; i < sizeof(RKEYS)/sizeof(RKEYS[0]); i++) {
		char tmp[9000]; memset(tmp, 0, sizeof(tmp)); strcpy(tmp, s);
		Info_RemoveKey_Big(tmp, RKEYS[i]);
		printf("rkb "); emit_qstr(RKEYS[i]); printf(" "); emit_qstr(tmp); printf("\n");
	}
	for (unsigned i = 0; i < sizeof(SKV)/sizeof(SKV[0]); i++) {
		char tmp[1100]; memset(tmp, 0, sizeof(tmp)); strcpy(tmp, s);
		Info_SetValueForKey(tmp, SKV[i].k, SKV[i].v);
		printf("svk "); emit_qstr(SKV[i].k); printf(" "); emit_qstr(SKV[i].v);
		printf(" "); emit_qstr(tmp); printf("\n");
	}
	for (unsigned i = 0; i < sizeof(SKV)/sizeof(SKV[0]); i++) {
		char tmp[9000]; memset(tmp, 0, sizeof(tmp)); strcpy(tmp, s);
		Info_SetValueForKey_Big(tmp, SKV[i].k, SKV[i].v);
		printf("svkb "); emit_qstr(SKV[i].k); printf(" "); emit_qstr(SKV[i].v);
		printf(" "); emit_qstr(tmp); printf("\n");
	}
	printf("--\n");
}

static void dump_info(const char *dir) {
	printf("== info ==\n");
	char path[1024];
	snprintf(path, sizeof(path), "%s/infostrings.txt", dir);
	long len; char *buf = slurp(path, &len);
	long off = 0; int idx = 0;
	while (off < len) {
		const char *rec = buf + off;
		size_t l = strlen(rec);
		dump_info_record(idx, rec);
		off += l + 1; idx++;
	}
	free(buf);

	// Big infostring: length in (MAX_INFO_STRING, BIG_INFO_STRING) — exercises
	// Info_ValueForKey's BIG_INFO_STRING guard (only ValueForKey/Validate,
	// which lack the MAX_INFO_STRING guard, are safe here).
	char big[8192]; big[0] = 0; int i = 0;
	while (strlen(big) < 1100) {
		char pair[32]; snprintf(pair, sizeof(pair), "\\k%d\\v%d", i, i);
		strcat(big, pair); i++;
	}
	printf("big len %d\n", (int)strlen(big));
	printf("big val %d\n", Info_Validate(big));
	{ char *v = Info_ValueForKey(big, "k50"); printf("big vfk k50 "); emit_qstr(v); printf("\n"); }
	{ char *v = Info_ValueForKey(big, "missing"); printf("big vfk missing "); emit_qstr(v); printf("\n"); }
}

int main(int argc, char **argv) {
	if (argc != 2) { fprintf(stderr, "usage: %s <fixture-dir>\n", argv[0]); return 2; }
	char dir[1024];
	snprintf(dir, sizeof(dir), "%s/qshared", argv[1]);
	dump_clamp();
	dump_tokens(dir);
	dump_compress(dir);
	dump_braced();
	dump_skipline();
	dump_strhelpers();
	dump_va();
	dump_sprintf();
	dump_info(dir);
	printf("== end ==\n");
	return 0;
}
