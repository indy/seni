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

/*
 A NodeVector is created with the [] syntax e.g. [1 2 3] it's equivalent to
 (list 1 2 3) with the exception of having different behaviour when defined
 as alterable.

 the alterable syntax for a vector specifies the values for each component
 of the vector
 e.g. {[1 2 3] (select from: [1 2 3 4 5 6])}
 -> each element in the vector [1 2 3] can have a value from 1..6
 [4 2 6], [1 2 2], [6 5 4] etc

 this differs from the alterable syntax for a list, since that specifies
 the values for the entire list
 e.g {(list 1 2 3) (select from: (list (list 1 2 3)
 (list 4 5 6)
 (list 7 8 9)))}
 -> (list 1 2 3) can be substituted for one of three lists
 (list 1 2 3), (list 4 5 6) or (list 7 8 9)
 */

export default class NodeVector extends Node {
  constructor() {
    super(NodeType.VECTOR);

    this.children = [];
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
