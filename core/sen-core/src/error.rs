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

pub type SenResult<T> = ::std::result::Result<T, SenError>;

#[derive(Debug)]
pub enum SenError {
    GeneralError,

    // colour
    //
    IncorrectColourFormat,
    InvalidColourHue,
    InvalidColourChannel,

    // parser
    ParserInvalidChar(char),
    ParserInvalidLiteral,
}
