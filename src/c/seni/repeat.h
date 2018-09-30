#pragma once

#include "types.h"

void repeat_symmetry_vertical(sen_vm* vm, i32 fn, i32* copy);
void repeat_symmetry_horizontal(sen_vm* vm, i32 fn, i32* copy);
void repeat_symmetry_4(sen_vm* vm, i32 fn, i32* copy);
void repeat_symmetry_8(sen_vm* vm, i32 fn, i32* copy);
void repeat_rotate(sen_vm* vm, i32 fn, i32 copies);
void repeat_rotate_mirrored(sen_vm* vm, i32 fn, i32 copies);
