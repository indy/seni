#pragma once

#include "types.h"

void repeat_symmetry_vertical(seni_vm *vm, i32 fn);
void repeat_symmetry_horizontal(seni_vm *vm, i32 fn);
void repeat_symmetry_4(seni_vm *vm, i32 fn);
void repeat_symmetry_8(seni_vm *vm, i32 fn);
void repeat_rotate(seni_vm *vm, i32 fn, i32 copies);
void repeat_rotate_mirrored(seni_vm *vm, i32 fn, i32 copies);


