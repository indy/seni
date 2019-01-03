// Copyright (C) 2019 Inderjit Gill

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

use crate::compiler::Program;
use crate::error::{Error, Result};
use crate::vm::{Var, Vm};
use crate::keywords::Keyword;

pub fn bind_vector_length(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {

    let mut vector: Option<&Var> = None;

    let mut args_pointer = vm.sp - (num_args * 2);

    for _ in 0..num_args {
        let label = &vm.stack[args_pointer + 0];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            if *iname == Keyword::Vector as i32 {
                vector = Some(value);
                break;
            }
        }
    }

    if let Some(v) = vector {
        if let Var::Vector(vs) = v {
            let len = vs.len();
            return Ok(Var::Int(len as i32))
        }
    }

    return Err(Error::Bind("bind_vector_length requires vector argument".to_string()))
}

#[cfg(test)]
mod tests {
    use crate::vm::tests::*;

    #[test]
    fn test_bind_vector_length() {
        is_int("(define v []) (++ v 100) (vector/length vector: v)", 1);
        is_int("(define v [1]) (++ v 100) (vector/length vector: v)", 2);
        is_int("(define v [1 2]) (++ v 100) (vector/length vector: v)", 3);
        is_int("(define v [1 2 3]) (++ v 100) (vector/length vector: v)", 4);
        is_int("(define v [1 2 3 4]) (++ v 100) (vector/length vector: v)", 5);
        is_int("(define v []) (++ v 4) (++ v 3) (++ v 2) (++ v 1) (++ v 0) (vector/length vector: v)", 5);
        is_int("(define v [1 2]) (++ v 98) (++ v 99) (++ v 100) (vector/length vector: v)", 5);
    }
}
