#include "seni_lang_bind.h"
#include "seni_lang_bind_core.h"


void interpreter_declare_keywords(word_lut *wlut)
{
  wlut->keywords_count = 0;

  bind_core_declarations(wlut);
}
