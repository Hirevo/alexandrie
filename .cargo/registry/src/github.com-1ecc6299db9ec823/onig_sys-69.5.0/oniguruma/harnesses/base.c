/*
 * base.c  contributed by Mark Griffin
 * Copyright (c) 2019-2020  K.Kosako
 */
#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <time.h>

#include "oniguruma.h"

#define PARSE_DEPTH_LIMIT           8
#define RETRY_LIMIT              5000
#define CALL_MAX_NEST_LEVEL         8
//#define EXEC_PRINT_INTERVAL    500000
//#define DUMP_DATA_INTERVAL     100000
//#define STAT_PATH              "fuzzer.stat_log"

typedef unsigned char uint8_t;

#ifdef DUMP_INPUT
static void
dump_input(unsigned char* data, size_t len)
{
  static FILE* DumpFp;
  static char end[] = { 'E', 'N', 'D' };

  if (DumpFp == 0)
    DumpFp = fopen("dump-input", "w");

  fseek(DumpFp, 0, SEEK_SET);
  fwrite(data, sizeof(unsigned char), len, DumpFp);
  fwrite(end,  sizeof(char), sizeof(end), DumpFp);
  fflush(DumpFp);
}
#endif

#ifdef DUMP_DATA_INTERVAL
static void
dump_file(char* path, unsigned char* data, size_t len)
{
  FILE* fp;

  fp = fopen(path, "w");
  fwrite(data, sizeof(unsigned char), len, fp);
  fclose(fp);
}
#endif

#ifdef STANDALONE
#include <ctype.h>

static void
dump_data(FILE* fp, unsigned char* data, int len)
{
  int i;

  fprintf(fp, "{\n");
  for (i = 0; i < len; i++) {
    unsigned char c = data[i];

    if (isprint((int )c)) {
      if (c == '\\')
        fprintf(fp, " '\\\\'");
      else
        fprintf(fp, " '%c'", c);
    }
    else {
      fprintf(fp, "0x%02x", (int )c);
    }

    if (i == len - 1) {
      fprintf(fp, "\n");
    }
    else {
      if (i % 8 == 7)
        fprintf(fp, ",\n");
      else
        fprintf(fp, ", ");
    }
  }
  fprintf(fp, "};\n");
}

#else

static void
output_current_time(FILE* fp)
{
  char d[64];
  time_t t;

  t = time(NULL);
  strftime(d, sizeof(d), "%m/%d %H:%M:%S", localtime(&t));

  fprintf(fp, "%s", d);
}

#endif

static int
search(regex_t* reg, unsigned char* str, unsigned char* end, int backward)
{
  int r;
  unsigned char *start, *range;
  OnigRegion *region;

  region = onig_region_new();

  if (backward != 0) {
    start = end;
    range = str;
  }
  else {
    start = str;
    range = end;
  }

  r = onig_search(reg, str, end, start, range, region, ONIG_OPTION_NONE);
  if (r >= 0) {
#ifdef STANDALONE
    int i;

    fprintf(stdout, "match at %d  (%s)\n", r,
            ONIGENC_NAME(onig_get_encoding(reg)));
    for (i = 0; i < region->num_regs; i++) {
      fprintf(stdout, "%d: (%d-%d)\n", i, region->beg[i], region->end[i]);
    }
#endif
  }
  else if (r == ONIG_MISMATCH) {
#ifdef STANDALONE
    fprintf(stdout, "search fail (%s)\n",
            ONIGENC_NAME(onig_get_encoding(reg)));
#endif
  }
  else { /* error */
#ifdef STANDALONE
    char s[ONIG_MAX_ERROR_MESSAGE_LEN];

    onig_error_code_to_str((UChar* )s, r);
    fprintf(stdout, "ERROR: %s\n", s);
    fprintf(stdout, "  (%s)\n", ONIGENC_NAME(onig_get_encoding(reg)));
#endif
    onig_region_free(region, 1 /* 1:free self, 0:free contents only */);

    if (r == ONIGERR_STACK_BUG ||
        r == ONIGERR_UNDEFINED_BYTECODE ||
        r == ONIGERR_UNEXPECTED_BYTECODE)
      return -2;

    return -1;
  }

  onig_region_free(region, 1 /* 1:free self, 0:free contents only */);
  return 0;
}

static long INPUT_COUNT;
static long EXEC_COUNT;
static long EXEC_COUNT_INTERVAL;
static long REGEX_SUCCESS_COUNT;
static long VALID_STRING_COUNT;

static int
exec(OnigEncoding enc, OnigOptionType options, OnigSyntaxType* syntax,
     char* apattern, char* apattern_end, char* astr, UChar* end, int backward)
{
  int r;
  regex_t* reg;
  OnigErrorInfo einfo;
  UChar* pattern = (UChar* )apattern;
  UChar* str     = (UChar* )astr;
  UChar* pattern_end = (UChar* )apattern_end;

  EXEC_COUNT++;
  EXEC_COUNT_INTERVAL++;

  onig_initialize(&enc, 1);
  onig_set_retry_limit_in_search(RETRY_LIMIT);
#ifdef PARSE_DEPTH_LIMIT
  onig_set_parse_depth_limit(PARSE_DEPTH_LIMIT);
#endif
  onig_set_subexp_call_max_nest_level(CALL_MAX_NEST_LEVEL);

  r = onig_new(&reg, pattern, pattern_end,
               options, enc, syntax, &einfo);
  if (r != ONIG_NORMAL) {
    char s[ONIG_MAX_ERROR_MESSAGE_LEN];
    onig_error_code_to_str((UChar* )s, r, &einfo);
#ifdef STANDALONE
    fprintf(stdout, "ERROR: %s\n", s);
#endif
    onig_end();

    if (r == ONIGERR_PARSER_BUG ||
        r == ONIGERR_STACK_BUG  ||
        r == ONIGERR_UNDEFINED_BYTECODE ||
        r == ONIGERR_UNEXPECTED_BYTECODE) {
      return -2;
    }
    else
      return -1;
  }
  REGEX_SUCCESS_COUNT++;

  r = search(reg, pattern, pattern_end, backward);
  if (r == -2) return -2;

  if (onigenc_is_valid_mbc_string(enc, str, end) != 0) {
    VALID_STRING_COUNT++;
    r = search(reg, str, end, backward);
    if (r == -2) return -2;
  }

  onig_free(reg);
  onig_end();
  return 0;
}

static int
alloc_exec(OnigEncoding enc, OnigOptionType options, OnigSyntaxType* syntax,
           int backward, int pattern_size, size_t remaining_size, unsigned char *data)
{
  int r;
  unsigned char *pattern_end;
  unsigned char *str_null_end;

  // copy first PATTERN_SIZE bytes off to be the pattern
  unsigned char *pattern = (unsigned char *)malloc(pattern_size != 0 ? pattern_size : 1);
  memcpy(pattern, data, pattern_size);
  pattern_end = pattern + pattern_size;
  data += pattern_size;
  remaining_size -= pattern_size;

#if defined(UTF16_BE) || defined(UTF16_LE)
  if (remaining_size % 2 == 1) remaining_size--;
#endif

  unsigned char *str = (unsigned char*)malloc(remaining_size != 0 ? remaining_size : 1);
  memcpy(str, data, remaining_size);
  str_null_end = str + remaining_size;

  r = exec(enc, options, syntax,
           (char *)pattern, (char *)pattern_end,
           (char *)str, str_null_end, backward);

  free(pattern);
  free(str);
  return r;
}

#define OPTIONS_MASK  (ONIG_OPTION_IGNORECASE | ONIG_OPTION_EXTEND | ONIG_OPTION_MULTILINE | ONIG_OPTION_SINGLELINE | ONIG_OPTION_FIND_LONGEST | ONIG_OPTION_FIND_NOT_EMPTY | ONIG_OPTION_NEGATE_SINGLELINE | ONIG_OPTION_DONT_CAPTURE_GROUP | ONIG_OPTION_CAPTURE_GROUP)


#ifdef SYNTAX_TEST
#define NUM_CONTROL_BYTES      6
#else
#define NUM_CONTROL_BYTES      5
#endif

int LLVMFuzzerTestOneInput(const uint8_t * Data, size_t Size)
{
#if !defined(UTF16_BE) && !defined(UTF16_LE)
  static OnigEncoding encodings[] = {
    ONIG_ENCODING_UTF8,
    ONIG_ENCODING_UTF8,
    ONIG_ENCODING_UTF8,
    ONIG_ENCODING_UTF8,
    ONIG_ENCODING_UTF8,
    ONIG_ENCODING_UTF8,
    ONIG_ENCODING_UTF8,
    ONIG_ENCODING_UTF8,
    ONIG_ENCODING_ASCII,
    ONIG_ENCODING_EUC_JP,
    ONIG_ENCODING_EUC_TW,
    ONIG_ENCODING_EUC_KR,
    ONIG_ENCODING_EUC_CN,
    ONIG_ENCODING_SJIS,
    ONIG_ENCODING_KOI8_R,
    ONIG_ENCODING_CP1251,
    ONIG_ENCODING_BIG5,
    ONIG_ENCODING_GB18030,
    ONIG_ENCODING_UTF8,
    ONIG_ENCODING_UTF8,
    ONIG_ENCODING_UTF8,
    ONIG_ENCODING_UTF8,
    ONIG_ENCODING_UTF8,
    ONIG_ENCODING_UTF8,
    ONIG_ENCODING_UTF8,
    ONIG_ENCODING_UTF8,
    ONIG_ENCODING_ISO_8859_1,
    ONIG_ENCODING_ISO_8859_2,
    ONIG_ENCODING_ISO_8859_3,
    ONIG_ENCODING_ISO_8859_4,
    ONIG_ENCODING_ISO_8859_5,
    ONIG_ENCODING_ISO_8859_6,
    ONIG_ENCODING_ISO_8859_7,
    ONIG_ENCODING_ISO_8859_8,
    ONIG_ENCODING_ISO_8859_9,
    ONIG_ENCODING_ISO_8859_10,
    ONIG_ENCODING_ISO_8859_11,
    ONIG_ENCODING_ISO_8859_13,
    ONIG_ENCODING_ISO_8859_14,
    ONIG_ENCODING_ISO_8859_15,
    ONIG_ENCODING_ISO_8859_16
  };
  unsigned char encoding_choice;
#endif

#ifdef SYNTAX_TEST
  static OnigSyntaxType* syntaxes[] = {
    ONIG_SYNTAX_POSIX_EXTENDED,
    ONIG_SYNTAX_EMACS,
    ONIG_SYNTAX_GREP,
    ONIG_SYNTAX_GNU_REGEX,
    ONIG_SYNTAX_JAVA,
    ONIG_SYNTAX_PERL_NG,
    ONIG_SYNTAX_ONIGURUMA
  };

#ifdef STANDALONE
  static char* syntax_names[] = {
    "Posix Extended",
    "Emacs",
    "Grep",
    "GNU Regex",
    "Java",
    "Perl+NG",
    "Oniguruma"
  };
#endif

  unsigned char syntax_choice;
#endif

  int r;
  int backward;
  int pattern_size;
  size_t remaining_size;
  unsigned char *data;
  unsigned char pattern_size_choice;
  OnigOptionType  options;
  OnigEncoding    enc;
  OnigSyntaxType* syntax;

#ifndef STANDALONE
  static FILE* STAT_FP;
#endif

  INPUT_COUNT++;

#ifdef DUMP_DATA_INTERVAL
  if (INPUT_COUNT % DUMP_DATA_INTERVAL == 0) {
    char path[20];
    sprintf(path, "dump-%ld", INPUT_COUNT);
    dump_file(path, (unsigned char* )Data, Size);
  }
#endif

  if (Size < NUM_CONTROL_BYTES) return 0;

  remaining_size = Size;
  data = (unsigned char* )(Data);

#ifdef UTF16_BE
  enc = ONIG_ENCODING_UTF16_BE;
#else
#ifdef UTF16_LE
  enc = ONIG_ENCODING_UTF16_LE;
#else
  encoding_choice = data[0];
  data++;
  remaining_size--;

  int num_encodings = sizeof(encodings)/sizeof(encodings[0]);
  enc = encodings[encoding_choice % num_encodings];
#endif
#endif

#ifdef SYNTAX_TEST
  syntax_choice = data[0];
  data++;
  remaining_size--;

  int num_syntaxes = sizeof(syntaxes)/sizeof(syntaxes[0]);
  syntax = syntaxes[syntax_choice % num_syntaxes];
#else
  syntax = ONIG_SYNTAX_DEFAULT;
#endif

  if ((data[1] & 0xc0) == 0)
    options = (data[0] | (data[1] << 8)) & OPTIONS_MASK;
  else
    options = data[0] & ONIG_OPTION_IGNORECASE;

  data++;
  remaining_size--;
  data++;
  remaining_size--;

  pattern_size_choice = data[0];
  data++;
  remaining_size--;

  backward = (data[0] == 0xbb);
  data++;
  remaining_size--;

  if (remaining_size == 0)
    pattern_size = 0;
  else {
    pattern_size = (int )pattern_size_choice % remaining_size;
#if defined(UTF16_BE) || defined(UTF16_LE)
    if (pattern_size % 2 == 1) pattern_size--;
#endif
  }

#ifdef STANDALONE
  dump_data(stdout, data, pattern_size);
#ifdef SYNTAX_TEST
  fprintf(stdout,
          "enc: %s, syntax: %s, options: %u, pattern_size: %d, back:%d\n",
          ONIGENC_NAME(enc),
          syntax_names[syntax_choice % num_syntaxes],
          options,
          pattern_size, backward);
#else
  fprintf(stdout, "enc: %s, options: %u, pattern_size: %d, back:%d\n",
          ONIGENC_NAME(enc), options, pattern_size, backward);
#endif
#endif

#ifdef DUMP_INPUT
  dump_input((unsigned char* )Data, Size);
#endif

  r = alloc_exec(enc, options, syntax, backward, pattern_size,
                 remaining_size, data);
  if (r == -2) exit(-2);

#ifndef STANDALONE
#ifdef EXEC_PRINT_INTERVAL
  if (EXEC_COUNT_INTERVAL == EXEC_PRINT_INTERVAL) {
    float fexec, freg, fvalid;

    if (STAT_FP == 0) {
#ifdef STAT_PATH
      STAT_FP = fopen(STAT_PATH, "a");
#else
      STAT_FP = stdout;
#endif
    }

    output_current_time(STAT_FP);

    if (INPUT_COUNT != 0) { // overflow check
      fexec  = (float )EXEC_COUNT / INPUT_COUNT;
      freg   = (float )REGEX_SUCCESS_COUNT / INPUT_COUNT;
      fvalid = (float )VALID_STRING_COUNT / INPUT_COUNT;

      fprintf(STAT_FP, ": %ld: EXEC:%.2f, REG:%.2f, VALID:%.2f\n",
              EXEC_COUNT, fexec, freg, fvalid);
      fflush(STAT_FP);
    }
    else {
      fprintf(STAT_FP, ": ignore (input count overflow)\n");
    }

    EXEC_COUNT_INTERVAL = 0;
  }
  else if (EXEC_COUNT == 1) {
    output_current_time(stdout);
    fprintf(stdout, ": ------------ START ------------\n");
  }
#endif
#endif

  return r;
}

#ifdef STANDALONE

extern int main(int argc, char* argv[])
{
  size_t n;
  uint8_t Data[10000];

  n = read(0, Data, sizeof(Data));
  fprintf(stdout, "n: %ld\n", n);
  LLVMFuzzerTestOneInput(Data, n);

  return 0;
}
#endif /* STANDALONE */
