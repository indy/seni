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

import NodeType from './NodeType';
import Node from './Node';

export default class NodeList extends Node {
  constructor() {
    super(NodeType.LIST);

    this.children = [];
    // true if the list was using the '(something) list form
    // as opposed to (quote (something))
    this.usingAbbreviation = false;
  }

  addChild(child) {
    this.children.push(child);
  }

  getChild(nth) {
    return this.children[nth];
  }

  size() {
    return this.children.length;
  }
}
