// ibi-gen — the ICARUS fixture compiler (docs/subsystems/icarus.md ruling 14 /
// § Verification strategy "Fixture provenance"). Compiles a hand-authored
// .icarus script into a precompiled .IBI block-instruction blob, using the
// oracle's own out-of-set Interpreter.cpp + Tokenizer.cpp front-end and the
// in-scope CBlockStream writer half. This tool is NOT part of the ported scope
// (Interpreter/Tokenizer are out of the WinDed link set, and the BlockStream
// writer methods are §20-dropped per ICARUS-D1) — it exists ONLY to produce the
// committed fixture corpus, exactly as the design doc specifies. The goldens
// themselves are produced by the in-scope reader/registers/sequencer TUs over
// the .IBI blobs this tool emits.
#include "exe_headers.h"   // q_shared.h + qcommon.h, exactly as every oracle icarus TU opens
#include "icarus.h"        // CTokenizer, CInterpreter, CBlockStream
#include "interpreter.h"

#include <cstdio>
#include <cstdlib>

int main(int argc, char **argv)
{
	if (argc != 3) { fprintf(stderr, "usage: %s <in.icarus> <out.IBI>\n", argv[0]); return 2; }

	FILE *f = fopen(argv[1], "rb");
	if (!f) { fprintf(stderr, "cannot open %s\n", argv[1]); return 2; }
	fseek(f, 0, SEEK_END);
	long len = ftell(f);
	fseek(f, 0, SEEK_SET);
	byte *src = (byte *)malloc(len + 1);
	if (fread(src, 1, len, f) != (size_t)len) { fprintf(stderr, "short read\n"); return 2; }
	src[len] = 0;
	fclose(f);

	CTokenizer *tok = CTokenizer::Create(0);
	CInterpreter interp;
	tok->SetSymbols(interp.GetSymbols());   // {, }, <, >, (, ), =, ! punctuation
	tok->AddParseStream(src, len);

	CBlockStream stream;
	if (stream.Create(argv[2]) == 0) { fprintf(stderr, "cannot create %s\n", argv[2]); return 2; }

	int r = interp.Interpret(tok, &stream, argv[1]);
	tok->Delete();

	// Interpret returns the count of parsed blocks on success (>=0); a negative
	// result is a syntax error (CInterpreter::Error negates m_iBadCBlockNumber).
	// The .IBI file handle is flushed/closed by normal process exit.
	if (r < 0) { fprintf(stderr, "%s: interpret error (block %d)\n", argv[1], -r); return 1; }
	return 0;
}
