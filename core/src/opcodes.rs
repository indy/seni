// Copyright (C) 2020 Inderjit Gill <email@indy.io>

// This file is part of Seni

// Seni is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Seni is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use strum_macros::{Display, EnumIter, EnumString};

use crate::error::Result;
use crate::packable::{Mule, Packable};

#[allow(non_camel_case_types)]
#[derive(Hash, Copy, Clone, Debug, Eq, PartialEq, Display, EnumString, EnumIter)]
pub enum Opcode {
    // load (push) a sen_var onto the stack
    LOAD,
    // store (pop) a sen_var from the stack
    STORE,
    // pop n vars from the stack, if they're 2 floats push a V2D, otherwise push a Vec
    SQUISH,
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    SQRT,
    EQ,
    GT,
    LT,
    AND,
    OR,
    NOT,
    // Jump the instruction pointer [arg] forward.
    JUMP,
    // Pop and if not truthy then jump the instruction pointer [arg] forward.
    JUMP_IF,
    // _0 == keep the existing frame, don't push/pop one
    //
    // reads the function offset and num args from the stack
    CALL,
    // reads the function's body offset from the stack (-1) and then push a return
    // value onto the stack (+1) => -1 + +1 == 0
    CALL_0,
    // RET will push the top value of the last frame onto the current frame
    RET,
    RET_0,
    // like CALL and CALL_0 except it reads an index from the stack
    // then it indexes into program->fn_info[index]
    CALL_F,
    // read index from stack (-1) then push a return value onto the stack (+1) => -1
    // + +1 == 0
    CALL_F_0,
    // calls a native function, leaving the result on the stack
    // offset is 0 as the vm->opcode_offset is modified by the native helper
    // function
    NATIVE,
    // appends item at top to vector at top-1, leaves vector on stack
    APPEND,
    // given a vector on the stack this unpacks it's contents onto the stack
    // offset is 0 as the vm->opcode_offset depends on the size of the stack
    // can only be used if each element on the lhs is a NODE_NAME
    // the first arg of the bytecode is the expected length of the vector
    // vm->opcode_offset is modified by the compiler
    PILE,
    // function look-up version of STORE
    // pop a value from the stack which is the index into program->fn_info
    // will then be used along with the argument's iname to find the index into the
    // MEM_SEG_ARGUMENT memory
    STORE_F,
    // temporary opcodes which are replaced by their non-placeholder versions during
    // a compilation pass
    PLACEHOLDER_STORE,

    // is the value at the top of the stack a non-empty vector?
    // pushes a boolean result onto the stack
    VEC_NON_EMPTY,
    // top of the stack has a vector
    // push the first element to the top
    VEC_LOAD_FIRST,
    // does the var at the top of the stack have a next value?
    // pushes a boolean result onto the stack
    VEC_HAS_NEXT,
    // (assumption the top of the stack contains a VAR_VECTOR)
    // replaces the top value on the stack with the next value
    // (+ hack to also treat a VAR_2D as a VAR_VECTOR)
    VEC_NEXT,

    STOP,
}

pub fn opcode_stack_offset(opcode: Opcode) -> i32 {
    match opcode {
        Opcode::LOAD => 1,
        Opcode::STORE => -1,
        Opcode::SQUISH => 0,
        Opcode::ADD => -1,
        Opcode::SUB => -1,
        Opcode::MUL => -1,
        Opcode::DIV => -1,
        Opcode::MOD => -1,
        Opcode::SQRT => 0,
        Opcode::EQ => -1,
        Opcode::GT => -1,
        Opcode::LT => -1,
        Opcode::AND => -1,
        Opcode::OR => -1,
        Opcode::NOT => 0,
        Opcode::JUMP => 0,
        Opcode::JUMP_IF => -1,
        Opcode::CALL => -2,
        Opcode::CALL_0 => 0,
        Opcode::RET => 0,
        Opcode::RET_0 => 0,
        Opcode::CALL_F => -1,
        Opcode::CALL_F_0 => 0,
        Opcode::NATIVE => 0,
        Opcode::APPEND => -1,
        Opcode::PILE => 0,
        Opcode::STORE_F => -2,
        Opcode::PLACEHOLDER_STORE => -1,
        Opcode::VEC_NON_EMPTY => 1,
        Opcode::VEC_LOAD_FIRST => 2,
        Opcode::VEC_HAS_NEXT => 1,
        Opcode::VEC_NEXT => 0,
        Opcode::STOP => 0,
    }
}

impl Packable for Opcode {
    fn pack(&self, cursor: &mut String) -> Result<()> {
        Mule::pack_label(cursor, &self.to_string());

        Ok(())
    }

    fn unpack(cursor: &str) -> Result<(Self, &str)> {
        let ns = Mule::next_space(cursor);
        let sub = &cursor[0..ns];
        let res = sub.parse::<Opcode>()?;

        Ok((res, &cursor[ns..]))
    }
}
