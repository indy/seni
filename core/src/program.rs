// Copyright (C) 2020 Inderjit Gill <email@indy.io>

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::BTreeMap;
use std::fmt;

use crate::colour::Colour;
use crate::error::{Error, Result};
use crate::iname::Iname;
use crate::keywords::Keyword;
use crate::native::Native;
use crate::opcodes::Opcode;
use crate::packable::{Mule, Packable};

use log::error;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mem {
    Argument = 0, // store the function's arguments
    Local = 1,    // store the function's local arguments
    Global = 2,   // global variables shared by all functions
    Constant = 3, // pseudo-segment holds constants in range 0..32767
    Void = 4,     // nothing
}

#[derive(Debug, Default)]
pub struct Program {
    pub data: Data,
    pub code: Vec<Bytecode>,
    pub fn_info: Vec<FnInfo>,
}

#[derive(Debug)]
pub struct Data {
    // the sub-section of WordLut::iname_to_word that stores Node::String
    pub strings: BTreeMap<Iname, String>,
}

#[derive(Debug)]
pub struct FnInfo {
    pub fn_name: String,
    pub arg_address: usize,
    pub body_address: usize,
    pub num_args: i32,
    pub argument_offsets: Vec<Iname>,
}

#[derive(Debug, PartialEq)]
pub struct Bytecode {
    pub op: Opcode,
    pub arg0: BytecodeArg,
    pub arg1: BytecodeArg,
}

impl fmt::Display for Mem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mem::Argument => write!(f, "ARG"),
            Mem::Local => write!(f, "LOCAL"),
            Mem::Global => write!(f, "GLOBAL"),
            Mem::Constant => write!(f, "CONST"),
            Mem::Void => write!(f, "VOID"),
        }
    }
}

impl Packable for Mem {
    fn pack(&self, cursor: &mut String) -> Result<()> {
        Mule::pack_i32(cursor, *self as i32);
        Ok(())
    }

    fn unpack(cursor: &str) -> Result<(Self, &str)> {
        let (res_i32, rem) = Mule::unpack_i32(cursor)?;

        let res = match res_i32 {
            0 => Mem::Argument,
            1 => Mem::Local,
            2 => Mem::Global,
            3 => Mem::Constant,
            4 => Mem::Void,
            _ => {
                error!("Mem::unpack invalid value: {}", res_i32);
                return Err(Error::Packable);
            }
        };

        Ok((res, rem))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BytecodeArg {
    Int(i32),
    Float(f32),
    Name(Iname),
    String(Iname),
    Native(Native),
    Mem(Mem),
    Keyword(Keyword),
    Colour(Colour),
}

impl BytecodeArg {
    pub fn get_int(&self) -> Result<i32> {
        match self {
            BytecodeArg::Int(i) => Ok(*i),
            _ => {
                error!("BytecodeArg expected to be int");
                Err(Error::Program)
            }
        }
    }

    pub fn is_int(&self, val: i32) -> bool {
        match self {
            BytecodeArg::Int(i) => *i == val,
            _ => false,
        }
    }
}

impl fmt::Display for BytecodeArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BytecodeArg::Int(i) => write!(f, "{}", i),
            BytecodeArg::Float(s) => write!(f, "{:.2}", s),
            BytecodeArg::Name(n) => write!(f, "Name({})", n),
            BytecodeArg::String(n) => write!(f, "String({})", n),
            BytecodeArg::Native(n) => write!(f, "{:?}", n),
            BytecodeArg::Mem(m) => write!(f, "{}", m),
            BytecodeArg::Keyword(kw) => write!(f, "{}", kw),
            BytecodeArg::Colour(c) => {
                write!(f, "{}({} {} {} {})", c.format, c.e0, c.e1, c.e2, c.e3)
            }
        }
    }
}

impl Packable for BytecodeArg {
    fn pack(&self, cursor: &mut String) -> Result<()> {
        match self {
            BytecodeArg::Int(i) => cursor.push_str(&format!("INT {}", i)),
            BytecodeArg::Float(f) => cursor.push_str(&format!("FLOAT {}", f)),
            BytecodeArg::Name(i) => cursor.push_str(&format!("NAME {}", i)),
            BytecodeArg::String(i) => cursor.push_str(&format!("STRING {}", i)),
            BytecodeArg::Native(native) => {
                cursor.push_str("NATIVE ");
                native.pack(cursor)?;
            }
            BytecodeArg::Mem(mem) => {
                cursor.push_str("MEM ");
                mem.pack(cursor)?;
            }
            BytecodeArg::Keyword(kw) => {
                cursor.push_str("KW ");
                kw.pack(cursor)?;
            }
            BytecodeArg::Colour(col) => {
                cursor.push_str("COLOUR ");
                col.pack(cursor)?;
            }
        }
        Ok(())
    }

    fn unpack(cursor: &str) -> Result<(Self, &str)> {
        if cursor.starts_with("INT ") {
            let rem = Mule::skip_forward(cursor, "INT ".len());
            let (val, rem) = Mule::unpack_i32(rem)?;
            Ok((BytecodeArg::Int(val), rem))
        } else if cursor.starts_with("FLOAT ") {
            let rem = Mule::skip_forward(cursor, "FLOAT ".len());
            let (val, rem) = Mule::unpack_f32(rem)?;
            Ok((BytecodeArg::Float(val), rem))
        } else if cursor.starts_with("NAME ") {
            let rem = Mule::skip_forward(cursor, "NAME ".len());
            let (val, rem) = Iname::unpack(rem)?;
            Ok((BytecodeArg::Name(val), rem))
        } else if cursor.starts_with("STRING ") {
            let rem = Mule::skip_forward(cursor, "STRING ".len());
            let (val, rem) = Iname::unpack(rem)?;
            Ok((BytecodeArg::String(val), rem))
        } else if cursor.starts_with("NATIVE ") {
            let rem = Mule::skip_forward(cursor, "NATIVE ".len());
            let (val, rem) = Native::unpack(rem)?;
            Ok((BytecodeArg::Native(val), rem))
        } else if cursor.starts_with("MEM ") {
            let rem = Mule::skip_forward(cursor, "MEM ".len());
            let (val, rem) = Mem::unpack(rem)?;
            Ok((BytecodeArg::Mem(val), rem))
        } else if cursor.starts_with("KW ") {
            let rem = Mule::skip_forward(cursor, "KW ".len());
            let (val, rem) = Keyword::unpack(rem)?;
            Ok((BytecodeArg::Keyword(val), rem))
        } else if cursor.starts_with("COLOUR ") {
            let rem = Mule::skip_forward(cursor, "COLOUR ".len());
            let (val, rem) = Colour::unpack(rem)?;
            Ok((BytecodeArg::Colour(val), rem))
        } else {
            error!("BytecodeArg::unpack");
            Err(Error::Packable)
        }
    }
}

impl fmt::Display for Bytecode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.op {
            Opcode::LOAD | Opcode::STORE | Opcode::STORE_F => {
                write!(f, "{}\t{}\t{}", self.op, self.arg0, self.arg1)?;
            }
            Opcode::JUMP | Opcode::JUMP_IF => {
                if let BytecodeArg::Int(i) = self.arg0 {
                    if i > 0 {
                        write!(f, "{}\t+{}", self.op, self.arg0)?
                    } else {
                        write!(f, "{}\t{}", self.op, self.arg0)?
                    }
                }
            }
            Opcode::NATIVE => write!(f, "{}\t{}\t{}", self.op, self.arg0, self.arg1)?,
            Opcode::PILE => write!(f, "{}\t{}\t{}", self.op, self.arg0, self.arg1)?,
            _ => write!(f, "{}", self.op)?,
        };
        Ok(())
    }
}

impl Packable for Bytecode {
    fn pack(&self, cursor: &mut String) -> Result<()> {
        self.op.pack(cursor)?;
        Mule::pack_space(cursor);
        self.arg0.pack(cursor)?;
        Mule::pack_space(cursor);
        self.arg1.pack(cursor)?;

        Ok(())
    }

    fn unpack(cursor: &str) -> Result<(Self, &str)> {
        let (op, rem) = Opcode::unpack(cursor)?;
        let rem = Mule::skip_space(rem);

        let (arg0, rem) = BytecodeArg::unpack(rem)?;
        let rem = Mule::skip_space(rem);

        let (arg1, rem) = BytecodeArg::unpack(rem)?;

        Ok((Bytecode { op, arg0, arg1 }, rem))
    }
}

impl Default for FnInfo {
    fn default() -> FnInfo {
        FnInfo {
            fn_name: "".into(),
            arg_address: 0,
            body_address: 0,
            num_args: 0,
            argument_offsets: Vec::new(),
        }
    }
}

impl FnInfo {
    pub fn get_argument_mapping(&self, argument_iname: Iname) -> Option<usize> {
        for (i, arg) in self.argument_offsets.iter().enumerate() {
            if *arg == argument_iname {
                return Some((i * 2) + 1);
            }
        }
        None
    }
}

impl Default for Data {
    fn default() -> Data {
        Data {
            strings: BTreeMap::new(),
        }
    }
}

impl Packable for Data {
    fn pack(&self, cursor: &mut String) -> Result<()> {
        Mule::pack_usize(cursor, self.strings.len());
        for (iname, s) in &self.strings {
            Mule::pack_space(cursor);
            iname.pack(cursor)?;
            Mule::pack_space(cursor);
            Mule::pack_string(cursor, s);
        }

        Ok(())
    }

    fn unpack(cursor: &str) -> Result<(Self, &str)> {
        let (strings_size, rem) = Mule::unpack_usize(cursor)?;

        let mut strings: BTreeMap<Iname, String> = BTreeMap::new();

        let mut r = rem;
        for _ in 0..strings_size {
            r = Mule::skip_space(r);

            let (iname, rem) = Iname::unpack(r)?;
            let rem = Mule::skip_space(rem);
            let (string, rem) = Mule::unpack_string(rem)?;
            r = rem;

            strings.insert(iname, string);
        }

        let data = Data { strings };
        Ok((data, r))
    }
}

impl Data {
    pub fn bitmap_strings(&self) -> Vec<String> {
        self.strings
            .values()
            .cloned()
            .filter(|s| !s.starts_with("mask/") && s.ends_with(".png")) // hack
            .collect()
    }

    pub fn string_from_iname(&self, iname: Iname) -> Result<String> {
        match self.strings.get(&iname) {
            Some(s) => Ok(s.into()),
            None => {
                error!("Data::string_from_iname {}", iname);
                Err(Error::Program)
            }
        }
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, b) in self.code.iter().enumerate() {
            writeln!(f, "{}\t{}", i, b)?;
        }
        Ok(())
    }
}

impl Packable for Program {
    fn pack(&self, cursor: &mut String) -> Result<()> {
        self.data.pack(cursor)?;
        Mule::pack_space(cursor);

        Mule::pack_usize(cursor, self.code.len());
        for b in &self.code {
            Mule::pack_space(cursor);
            b.pack(cursor)?;
        }

        Ok(())
    }

    fn unpack(cursor: &str) -> Result<(Self, &str)> {
        let (data, rem) = Data::unpack(cursor)?;
        let rem = Mule::skip_space(rem);

        let (codesize, rem) = Mule::unpack_usize(rem)?;

        // note: current assumption is that
        // fn_info isn't used after a program has been unpacked
        let fn_info: Vec<FnInfo> = Vec::new();

        let mut code: Vec<Bytecode> = Vec::new();

        let mut r = rem;
        for _ in 0..codesize {
            r = Mule::skip_space(r);
            let (bc, rem) = Bytecode::unpack(r)?;
            r = rem;
            code.push(bc);
        }

        let program = Program {
            data,
            code,
            fn_info,
        };

        Ok((program, r))
    }
}

impl Program {
    pub fn stop_location(&self) -> usize {
        // the final opcode in the program will always be a STOP
        self.code.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_bitmap_strings() {
        let mut d: Data = Default::default();

        d.strings.insert(Iname::new(3), "hello".into());
        d.strings.insert(Iname::new(4), "image.png".into());
        d.strings.insert(Iname::new(5), "bitmap.png".into());
        let res = d.bitmap_strings();

        assert_eq!(res.len(), 2);
        assert_eq!(res[0], "image.png");
        assert_eq!(res[1], "bitmap.png");
    }

    #[test]
    fn test_mem_pack() {
        let mut res: String = "".into();
        Mem::Constant.pack(&mut res).unwrap();
        assert_eq!("3", res);
    }

    #[test]
    fn test_mem_unpack() {
        let (res, _rem) = Mem::unpack("3").unwrap();
        assert_eq!(res, Mem::Constant);
    }

    #[test]
    fn test_bytecode_arg_pack() {
        let mut res: String = "".into();
        BytecodeArg::Native(Native::Circle).pack(&mut res).unwrap();
        assert_eq!("NATIVE circle", res);
    }

    #[test]
    fn test_bytecode_arg_unpack() {
        let (res, _rem) = BytecodeArg::unpack("NATIVE circle").unwrap();
        assert_eq!(res, BytecodeArg::Native(Native::Circle));

        let (res, rem) = BytecodeArg::unpack("NATIVE col/triad otherstuff here").unwrap();
        assert_eq!(res, BytecodeArg::Native(Native::ColTriad));
        assert_eq!(rem, " otherstuff here");
    }

    #[test]
    fn test_bytecode_pack() {
        let mut res: String = "".into();

        // a nonsense bytecode
        let bc = Bytecode {
            op: Opcode::APPEND,
            arg0: BytecodeArg::Int(42),
            arg1: BytecodeArg::Mem(Mem::Global),
        };

        bc.pack(&mut res).unwrap();
        assert_eq!("APPEND INT 42 MEM 2", res);
    }

    #[test]
    fn test_bytecode_unpack() {
        let (res, _rem) = Bytecode::unpack("APPEND INT 42 MEM 2").unwrap();

        assert_eq!(res.op, Opcode::APPEND);
        assert_eq!(res.arg0, BytecodeArg::Int(42));
        assert_eq!(res.arg1, BytecodeArg::Mem(Mem::Global));
    }
}
