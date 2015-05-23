/*
 *  Seni
 *  Copyright (C) 2015  Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

const TokenType = {
  UNKNOWN: 0,
  LIST_START: 1,
  LIST_END: 2,
  BRACKET_START: 3,
  BRACKET_END: 4,
  INT: 5,
  FLOAT: 6,
  NAME: 7,
  STRING: 8,
  QUOTE_ABBREVIATION: 9,
  LABEL: 10,
  COMMENT: 11
};

export default TokenType;
