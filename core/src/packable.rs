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

use crate::error::{Error, Result};

/// convert Rust structures into a compact text format suitable for transferring over to the JS side
pub trait Packable {
    fn pack(&self, cursor: &mut String) -> Result<()>;
    fn unpack(cursor: &str) -> Result<(Self, &str)>
    where
        Self: std::marker::Sized;
}

pub struct Mule {}

impl Mule {
    pub fn pack_bool_sp(cursor: &mut String, val: bool) {
        Mule::pack_bool(cursor, val);
        Mule::pack_space(cursor);
    }

    pub fn pack_bool(cursor: &mut String, val: bool) {
        if val {
            cursor.push_str(&"1".to_string());
        } else {
            cursor.push_str(&"0".to_string());
        }
    }

    pub fn pack_label_bool(cursor: &mut String, label: &str, val: bool) {
        if val {
            cursor.push_str(&format!("{} 1", label));
        } else {
            cursor.push_str(&format!("{} 0", label));
        }
    }

    pub fn pack_usize_sp(cursor: &mut String, val: usize) {
        Mule::pack_usize(cursor, val);
        Mule::pack_space(cursor);
    }

    pub fn pack_usize(cursor: &mut String, val: usize) {
        cursor.push_str(&format!("{}", val));
    }

    pub fn pack_i32_sp(cursor: &mut String, val: i32) {
        Mule::pack_i32(cursor, val);
        Mule::pack_space(cursor);
    }

    pub fn pack_i32(cursor: &mut String, val: i32) {
        cursor.push_str(&format!("{}", val));
    }

    pub fn pack_f32_sp(cursor: &mut String, val: f32) {
        Mule::pack_f32(cursor, val);
        Mule::pack_space(cursor);
    }

    pub fn pack_f32(cursor: &mut String, val: f32) {
        cursor.push_str(&format!("{}", val));
    }

    pub fn pack_label_sp(cursor: &mut String, val: &str) {
        Mule::pack_label(cursor, val);
        Mule::pack_space(cursor);
    }

    pub fn pack_label(cursor: &mut String, val: &str) {
        cursor.push_str(&val.to_string());
    }

    pub fn pack_space(cursor: &mut String) {
        cursor.push_str(" ");
    }

    // --------------------------------------------------------------------------------

    pub fn skip_forward(cursor: &str, skip_by: usize) -> &str {
        &cursor[skip_by..]
    }

    pub fn skip_space(cursor: &str) -> &str {
        Mule::skip_forward(cursor, 1) // todo: add more logic to make sure cursor is on a space
    }

    pub fn next_space(cursor: &str) -> usize {
        for (ind, ch) in cursor.char_indices() {
            if ch == ' ' {
                return ind;
            }
        }
        cursor.len()
    }

    pub fn unpack_f32(cursor: &str) -> Result<(f32, &str)> {
        let ns = Mule::next_space(cursor);
        let sub = &cursor[0..ns];
        let res = sub.parse::<f32>()?;

        Ok((res, &cursor[ns..]))
    }

    pub fn unpack_i32(cursor: &str) -> Result<(i32, &str)> {
        let ns = Mule::next_space(cursor);
        let sub = &cursor[0..ns];
        let res = sub.parse::<i32>()?;

        Ok((res, &cursor[ns..]))
    }

    pub fn unpack_usize(cursor: &str) -> Result<(usize, &str)> {
        let ns = Mule::next_space(cursor);
        let sub = &cursor[0..ns];
        let res = sub.parse::<usize>()?;

        Ok((res, &cursor[ns..]))
    }

    pub fn unpack_u64(cursor: &str) -> Result<(u64, &str)> {
        let ns = Mule::next_space(cursor);
        let sub = &cursor[0..ns];
        let res = sub.parse::<u64>()?;

        Ok((res, &cursor[ns..]))
    }

    pub fn unpack_bool(cursor: &str) -> Result<(bool, &str)> {
        let ns = Mule::next_space(cursor);
        let sub = &cursor[0..ns];
        let res = sub.parse::<i32>()?;

        if res == 0 {
            Ok((false, &cursor[ns..]))
        } else if res == 1 {
            Ok((true, &cursor[ns..]))
        } else {
            Err(Error::Packable(format!("unpack_bool given {}", res)))
        }
    }

    pub fn unpack_f32_sp(cursor: &str) -> Result<(f32, &str)> {
        let (res, rem) = Mule::unpack_f32(cursor)?;
        let rem = Mule::skip_space(rem);
        Ok((res, rem))
    }

    pub fn unpack_i32_sp(cursor: &str) -> Result<(i32, &str)> {
        let (res, rem) = Mule::unpack_i32(cursor)?;
        let rem = Mule::skip_space(rem);
        Ok((res, rem))
    }

    pub fn unpack_usize_sp(cursor: &str) -> Result<(usize, &str)> {
        let (res, rem) = Mule::unpack_usize(cursor)?;
        let rem = Mule::skip_space(rem);
        Ok((res, rem))
    }

    pub fn unpack_u64_sp(cursor: &str) -> Result<(u64, &str)> {
        let (res, rem) = Mule::unpack_u64(cursor)?;
        let rem = Mule::skip_space(rem);
        Ok((res, rem))
    }

    pub fn unpack_bool_sp(cursor: &str) -> Result<(bool, &str)> {
        let (res, rem) = Mule::unpack_bool(cursor)?;
        let rem = Mule::skip_space(rem);
        Ok((res, rem))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_var_unpack_i32() {
        let (res, rem) = Mule::unpack_i32(&"67").unwrap();
        assert_eq!(res, 67);
        assert_eq!(rem, "");

        let (res, rem) = Mule::unpack_i32(&"-273").unwrap();
        assert_eq!(res, -273);
        assert_eq!(rem, "");
    }

    #[test]
    fn test_var_unpack_i32_2() {
        let (res, rem) = Mule::unpack_i32(&"1234 shabba").unwrap();
        assert_eq!(res, 1234);
        assert_eq!(rem, " shabba");
    }
}
