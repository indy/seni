// Copyright (C) 2019 Inderjit Gill

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

use crate::error::Result;
use crate::keywords::Keyword;
use crate::native::Native;
use crate::packable::{Mule, Packable};
use std::fmt;

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Iname(i32);

impl Iname {
    pub fn new(i: i32) -> Self {
        Iname(i)
    }

    pub fn enclosed_by(self, a: Iname, b: Iname) -> bool {
        self.0 > a.0 && self.0 < b.0
    }
}

impl fmt::Display for Iname {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Packable for Iname {
    fn pack(&self, cursor: &mut String) -> Result<()> {
        cursor.push_str(&format!("{}", self.0));
        Ok(())
    }

    fn unpack(cursor: &str) -> Result<(Self, &str)> {
        let (val, rem) = Mule::unpack_i32(cursor)?;
        Ok((Iname(val), rem))
    }
}

impl From<Keyword> for Iname {
    fn from(kw: Keyword) -> Iname {
        Iname(kw as i32)
    }
}

impl From<Native> for Iname {
    fn from(n: Native) -> Iname {
        Iname(n as i32)
    }
}
