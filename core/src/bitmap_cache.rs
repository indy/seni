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

use crate::error::Error;
use crate::result::Result;

use std::collections::HashMap;

use log::error;

pub struct BitmapCache {
    pub info: HashMap<String, BitmapInfo>,
}

impl Default for BitmapCache {
    fn default() -> BitmapCache {
        BitmapCache {
            info: HashMap::new(),
        }
    }
}

impl BitmapCache {
    pub fn insert(&mut self, name: &str, info: BitmapInfo) -> Result<()> {
        self.info.insert(name.to_string(), info);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&BitmapInfo> {
        match self.info.get(name) {
            Some(bitmap_info) => Ok(bitmap_info),
            None => {
                error!("can't find bitmap: {}", name);
                Err(Error::BitmapCache)
            }
        }
    }

    // returns the subset of bitmap_names which aren't in this cache
    pub fn uncached(&self, bitmap_names: Vec<String>) -> Vec<String> {
        let mut res = vec![];

        for bitmap_name in bitmap_names {
            if !self.info.contains_key(&bitmap_name) {
                res.push(bitmap_name)
            }
        }

        res
    }
}

#[derive(Default)]
pub struct BitmapInfo {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}
