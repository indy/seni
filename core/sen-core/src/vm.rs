// Copyright (C) 2018 Inderjit Gill

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::compiler::{Bytecode, FnInfo, Program};
use crate::error::{Error, Result};

const FP_OFFSET_TO_LOCALS: i32 = 4;
const FP_OFFSET_TO_HOP_BACK: i32 = 3;
const FP_OFFSET_TO_NUM_ARGS: i32 = 2;
const FP_OFFSET_TO_IP: i32 = 1;

// void gc_mark_vector(sen_var* vector)
// void gc_mark(sen_vm* vm)
// void gc_sweep(sen_vm* vm)

// **************************************************
// VM bytecode interpreter
// **************************************************

// sen_var* arg_memory_from_iname(sen_fn_info* fn_info, i32 iname, sen_var* args)
// void vm_function_set_argument_to_var(sen_vm* vm, sen_fn_info* fn_info, i32 iname, sen_var* src)
// void vm_function_set_argument_to_f32(sen_vm* vm, sen_fn_info* fn_info, i32 iname, f32 f)
// void vm_function_set_argument_to_2d(sen_vm* vm, sen_fn_info* fn_info, i32 iname, f32 x, f32 y)
// void vm_function_call_default_arguments(sen_vm* vm, sen_fn_info* fn_info)
// void vm_function_call_body(sen_vm* vm, sen_fn_info* fn_info)
// bool vm_run(sen_vm* vm, sen_env* env, sen_program* program)
// bool vm_interpret(sen_vm* vm, sen_env* env, sen_program* program)

// vm, env, sen_var
