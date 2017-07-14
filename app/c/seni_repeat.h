#ifndef SENI_REPEAT_H
#define SENI_REPEAT_H

#include "seni_types.h"
#include "seni_lang.h"

void repeat_symmetry_vertical(seni_vm *vm, i32 draw);
void repeat_symmetry_horizontal(seni_vm *vm, i32 draw);
void repeat_symmetry_4(seni_vm *vm, i32 draw);
void repeat_symmetry_8(seni_vm *vm, i32 draw);
void repeat_rotate(seni_vm *vm, i32 draw, i32 copies);
void repeat_rotate_mirrored(seni_vm *vm, i32 draw, i32 copies);


#endif
