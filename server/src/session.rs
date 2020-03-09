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

use crate::error::{Error, Result};
use std::str::FromStr;

pub const AUTH: &str = "auth";

pub fn user_id(session: &actix_session::Session) -> Result<i64> {
    if let Some(auth) = session.get::<String>(AUTH)? {
        let user_id: i64 = i64::from_str(&auth)?;
        Ok(user_id)
    } else {
        Err(Error::Authenticating)
    }
}
