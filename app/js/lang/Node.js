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

export default class Node {
  /**
   * node constructor
   * @param type
   * @param value
   */
  constructor(type, value) {
    this.type = type;
    this.value = value;
    this.alterable = false;

    // node mutate specific
    this.parameterAST = [];

    // need a place for nodes that occur within curly brackets that should
    // be ignored, e.g. the whitespace before the 2 in: (+ 1 { 2} (int))
    this.parameterPrefix = [];
  }

  addParameterNode(parameter) {
    this.parameterAST.push(parameter);
  }

  addParameterNodePrefix(prefix) {
    this.parameterPrefix.push(prefix);
  }
}
