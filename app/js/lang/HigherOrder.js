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

import PublicBinding from '../lang/PublicBinding';
import Interpreter from './Interpreter';

const { TRUE_STRING } = Interpreter;

function identity({item}) {
  return item;
}

const publicBindings = [
  new PublicBinding(
    `map`,
    { description: `-`,
      args: [],
      returns: `-` },
    { fn: identity,
      bind: `item`,
      vector: []},
    self => params => {
      const {fn, bind, vector} = self.mergeWithDefaults(params);
      const args = {};
      return vector.map(v => {
        args[bind] = v;
        return fn(args);
      });
    }
  ),
  new PublicBinding(
    `filter`,
    { description: `-`,
      args: [],
      returns: `-` },
    { fn: identity,
      bind: `item`,
      vector: []},
    self => params => {
      const {fn, bind, vector} = self.mergeWithDefaults(params);
      const args = {};
      return vector.filter(v => {
        args[bind] = v;
        return fn(args) === TRUE_STRING;
      });
    }
  )];

export default {
  publicBindingType: `binding`,
  publicBindings
};
