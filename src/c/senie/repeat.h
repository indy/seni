#pragma once

#include "types.h"

void repeat_symmetry_vertical(senie_vm* vm, i32 fn, i32* copy);
void repeat_symmetry_horizontal(senie_vm* vm, i32 fn, i32* copy);
void repeat_symmetry_4(senie_vm* vm, i32 fn, i32* copy);
void repeat_symmetry_8(senie_vm* vm, i32 fn, i32* copy);
void repeat_rotate(senie_vm* vm, i32 fn, i32 copies);
void repeat_rotate_mirrored(senie_vm* vm, i32 fn, i32 copies);
