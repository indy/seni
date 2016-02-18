/*
 *  Seni
 *  Copyright (C) 2016 Inderjit Gill <email@indy.io>
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

const NodeType = {
  LIST: Symbol(`LIST`),
  VECTOR: Symbol(`VECTOR`),
  INT: Symbol(`INT`),
  FLOAT: Symbol(`FLOAT`),
  NAME: Symbol(`NAME`),
  LABEL: Symbol(`LABEL`),
  STRING: Symbol(`STRING`),
  BOOLEAN: Symbol(`BOOLEAN`),
  LAMBDA: Symbol(`LAMBDA`),         // todo: remove this??
  SPECIAL: Symbol(`SPECIAL`),       // todo: remove this??
  COLOUR: Symbol(`COLOUR`),         // todo: remove this??
  WHITESPACE: Symbol(`WHITESPACE`), // only used by front-end ast
  COMMENT: Symbol(`COMMENT`),       // only used by front-end ast
  NULL: Symbol(`NULL`)
};

export default NodeType;
