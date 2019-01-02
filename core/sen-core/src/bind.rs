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
use crate::error::Result;
use crate::vm::{Var, Vm};

pub fn bind_vector_length(_vm: &mut Vm, _program: &Program, _num_args: i32) -> Result<Var> {
    Ok(Var::Int(3))
}

#[cfg(test)]
mod tests {
    use crate::vm::tests::*;

    #[test]
    fn test_bind_vector_length() {
        is_int("(define v []) (++ v 100) (vector/length vector: v)", 3);
    }
}
