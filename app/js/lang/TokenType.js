/*
 *  Seni
 *  Copyright (C) 2015 Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

const TokenType = {
  UNKNOWN: Symbol('UNKNOWN'),
  LIST_START: Symbol('LIST_START'),
  LIST_END: Symbol('LIST_END'),
  VECTOR_START: Symbol('VECTOR_START'),
  VECTOR_END: Symbol('VECTOR_END'),
  ALTERABLE_START: Symbol('ALTERABLE_START'),
  ALTERABLE_END: Symbol('ALTERABLE_END'),
  INT: Symbol('INT'),
  FLOAT: Symbol('FLOAT'),
  NAME: Symbol('NAME'),
  STRING: Symbol('STRING'),
  QUOTE_ABBREVIATION: Symbol('QUOTE_ABBREVIATION'),
  LABEL: Symbol('LABEL'),
  COMMENT: Symbol('COMMENT'),
  WHITESPACE: Symbol('WHITESPACE')
};

export default TokenType;
