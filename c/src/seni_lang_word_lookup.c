#include "seni_lang_word_lookup.h"

#include <string.h>
#include <stdlib.h>

/*
#if defined(_WIN32)
#else
#include "strings.h"
#endif

bool string_compare(char* a, char *b)
{
#if defined(_WIN32)
  return _stricmp(a, b) == 0;
#else
  return strcasecmp(a, b) == 0;
#endif
}
*/

void string_copy_len(char **dst, char *src, size_t len)
{
  char *c = (char *)malloc(sizeof(char) * (len + 1));
  strncpy(c, src, len);
  c[len] = '\0';

  *dst = c;
}


/* returns 0 if not found */
i32 lookup_reserved_name(word_lut *wlut, char *string, size_t len)
{
  i32 i = 0;
  for (i = 0; i < wlut->keywords_count; i++) {
    char *name = wlut->keywords[i];
    bool found = true;
    /* can't use string_compare since 'string' could be a substring */
    size_t j = 0;
    for (j = 0; j < len; j++) {
      if (name[j] == '\0' || (name[j] != string[j])) {
        found = false;
        break;
      }
    }
    /* searched all of 'string' and the early exit wasn't triggered */
    if (name[j] == '\0' && found == true) {
      return i + KEYWORD_START; // add offset
    }
  }

  return 0;
}

i32 word_lookup_or_add(word_lut *wlut, char *string, size_t len)
{
  i32 reserved = lookup_reserved_name(wlut, string, len);
  if (reserved != 0) {
    return reserved;
  }
  
  i32 i = 0;
  for (i = 0; i < wlut->words_count; i++) {
    char *name = wlut->words[i];
    bool found = true;
    /* can't use string_compare since 'string' could be a substring */
    size_t j = 0;
    for (j = 0; j < len; j++) {
      if (name[j] == '\0' || (name[j] != string[j])) {
        found = false;
        break;
      }
    }
    /* searched all of 'string' and the early exit wasn't triggered */
    if (name[j] == '\0' && found == true) {
      return i;
    }
  }
  
  /* string is not in the table and there's no room for another entry */
  if (i >= MAX_WORD_LOOKUPS) {
    return -1;
  }

  // the string is not in the lookup table, so add it
  string_copy_len(&(wlut->words[i]), string, len);
  wlut->words_count++;

  return i;
}

char *word_lookup_i32(word_lut *wlut, i32 index)
{
  if (index >= KEYWORD_START) {
    return wlut->keywords[index - KEYWORD_START];
  }
  return wlut->words[index];
}

void word_lookup_free_keywords(word_lut *wlut)
{
  for( int i = 0; i < MAX_KEYWORD_LOOKUPS; i++) {
    if (wlut->keywords[i]) {
      free(wlut->keywords[i]);
    }
    wlut->keywords[i] = 0;      
  }
  wlut->keywords_count = 0;
}

void word_lookup_free_words(word_lut *wlut)
{
  for( int i = 0; i < MAX_WORD_LOOKUPS; i++) {
    if (wlut->words[i]) {
      free(wlut->words[i]);
    }
    wlut->words[i] = 0;      
  }
  wlut->words_count = 0;
}

word_lut *word_lookup_allocate()
{
  word_lut *wl = (word_lut *)calloc(1, sizeof(word_lut));
  return wl;
}

void word_lookup_free(word_lut *wlut)
{
  word_lookup_free_words(wlut);
  word_lookup_free_keywords(wlut);
  free(wlut);
}
