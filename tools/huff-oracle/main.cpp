// huff-oracle — differential golden dumper for the adaptive-Huffman port
// (crates/mp/engine/qcommon/src/qcommon/huff.rs). Drives the UNMODIFIED oracle
// codemp/qcommon/huffman.cpp standalone: seeds the msgHuff tree exactly as
// MSG_initHuffman does (Huff_Init then, per symbol i, Huff_addRef i for
// msg_hData[i] repetitions on BOTH compressor and decompressor), then emits the
// per-symbol prefix codes and two concatenated bitstreams. The Rust twin
// (tests/huff_golden.rs) must reproduce every byte.
//
// Emission of a symbol's code goes through the real send() via
// Huff_offsetTransmit, so the goldens exercise the exact bit order the wire
// sees. oracle/ is never edited.

#include "codemp/qcommon/exe_headers.h"
#include "msg_hdata.h"

#include <cstdio>
#include <cstring>

// huffman.cpp declares `extern int oldsize;` (line 356); define it here.
int oldsize = 0;

// Oracle entry points (codemp/qcommon/qcommon.h:1078-1083).
extern void Huff_Init(huffman_t *huff);
extern void Huff_addRef(huff_t *huff, byte ch);
extern void Huff_offsetTransmit(huff_t *huff, int ch, byte *fout, int *offset);

static huffman_t msgHuff;

// Replica of MSG_initHuffman (msg.cpp:3219-3234) minus the file-static flag.
static void seed(void) {
	Huff_Init(&msgHuff);
	for (int i = 0; i < 256; i++) {
		for (int j = 0; j < msg_hData[i]; j++) {
			Huff_addRef(&msgHuff.compressor, (byte)i);
			Huff_addRef(&msgHuff.decompressor, (byte)i);
		}
	}
}

// Print bits [0, nbits) of buf, LSB-first within each byte — exactly the order
// add_bit() lays them down and get_bit() reads them back.
static void print_bits(FILE *fp, const byte *buf, int nbits) {
	for (int b = 0; b < nbits; b++) {
		int bit = (buf[b >> 3] >> (b & 7)) & 1;
		fputc('0' + bit, fp);
	}
}

// Golden A: the frozen prefix code the compressor assigns to each symbol,
// emitted through send() (via Huff_offsetTransmit into a fresh buffer at
// offset 0). The tree is not mutated (offsetTransmit never calls addRef), so
// codes are stable and order-independent.
static void dump_codes(FILE *fp) {
	for (int sym = 0; sym < 256; sym++) {
		byte buf[64];
		memset(buf, 0, sizeof(buf));
		int bloc = 0;
		Huff_offsetTransmit(&msgHuff.compressor, sym, buf, &bloc);
		fprintf(fp, "%3d: ", sym);
		print_bits(fp, buf, bloc);
		fprintf(fp, "  (%d bits)\n", bloc);
	}
}

// Golden B/C: concatenate the codes of a byte sequence into one bitstream from
// offset 0 (the frozen compressor, as the live wire path uses it), then dump
// bloc + the raw output bytes as hex, 32 bytes per line.
static void dump_stream(FILE *fp, const char *name, const byte *seq, int n) {
	byte buf[8192];
	memset(buf, 0, sizeof(buf));
	int bloc = 0;
	for (int i = 0; i < n; i++) {
		Huff_offsetTransmit(&msgHuff.compressor, seq[i], buf, &bloc);
	}
	int nbytes = (bloc + 7) >> 3;
	fprintf(fp, "%s bloc=%d bytes=%d\n", name, bloc, nbytes);
	for (int i = 0; i < nbytes; i++) {
		fprintf(fp, "%02x", buf[i]);
		if ((i + 1) % 32 == 0) {
			fputc('\n', fp);
		}
	}
	if (nbytes % 32 != 0) {
		fputc('\n', fp);
	}
}

int main(int argc, char **argv) {
	if (argc < 3) {
		fprintf(stderr, "usage: %s <codes|seq|chat> <outfile>\n", argv[0]);
		return 2;
	}
	seed();

	FILE *fp = fopen(argv[2], "w");
	if (!fp) {
		fprintf(stderr, "cannot open %s\n", argv[2]);
		return 2;
	}

	const char *what = argv[1];
	if (strcmp(what, "codes") == 0) {
		dump_codes(fp);
	} else if (strcmp(what, "seq") == 0) {
		byte seq[256];
		for (int i = 0; i < 256; i++) {
			seq[i] = (byte)i;
		}
		dump_stream(fp, "seq", seq, 256);
	} else if (strcmp(what, "chat") == 0) {
		// chat "P^7\x19: yo \xb0/.s"  — the symptomatic 0xB0 byte is at index 15.
		static const byte chat[] = {
			'c', 'h', 'a', 't', ' ', '"', 'P', '^', '7', 0x19,
			':', ' ', 'y', 'o', ' ', 0xb0, '/', '.', 's', '"',
		};
		dump_stream(fp, "chat", chat, (int)sizeof(chat));
	} else {
		fprintf(stderr, "unknown dump %s\n", what);
		fclose(fp);
		return 2;
	}

	fclose(fp);
	return 0;
}
