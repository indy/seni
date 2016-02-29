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

const { TRUE_STRING, FALSE_STRING } = Interpreter;

const publicBindings = [
  new PublicBinding(
    `+`,
    { description: `-`,
      args: [],
      returns: `-` },
    {},
    _self => args => args.reduce((a, b) => a + b, 0)
  ),

  new PublicBinding(
    `*`,
    { description: `-`,
      args: [],
      returns: `-` },
    {},
    _self => args => args.reduce((a, b) => a * b, 1)
  ),

  new PublicBinding(
    `-`,
    { description: `-`,
      args: [],
      returns: `-` },
    {},
    _self => args => args.length === 1 ? -args[0] : args.reduce((a, b) => a - b)
  ),

  new PublicBinding(
    `/`,
    { description: `-`,
      args: [],
      returns: `-` },
    {},
    _self => args => args.reduce((a, b) => a / b)
  ),

  new PublicBinding(
    `sqrt`,
    { description: `-`,
      args: [],
      returns: `-` },
    {},
    _self => args => Math.sqrt(args[0])
  ),

  new PublicBinding(
    `mod`,
    { description: `-`,
      args: [],
      returns: `-` },
    {},
    _self => args => args[0] % args[1]
  ),

  new PublicBinding(
    `=`,
    { description: `-`,
      args: [],
      returns: `-` },
    {},
    _self => ([first, ...rest]) =>
      rest.every(a => a === first) ? TRUE_STRING : FALSE_STRING
  ),

  new PublicBinding(
    `<`,
    { description: `-`,
      args: [],
      returns: `-` },
    {},
    _self => args => {
      let prev = args[0];
      for (let i = 1; i < args.length; i++) {
        const current = args[i];
        if (prev >= current) {
          return FALSE_STRING;
        }
        prev = current;
      }
      return TRUE_STRING;
    }
  ),

  new PublicBinding(
    `>`,
    { description: `-`,
      args: [],
      returns: `-` },
    {},
    _self => args => {
      let prev = args[0];
      for (let i = 1; i < args.length; i++) {
        const current = args[i];
        if (prev <= current) {
          return FALSE_STRING;
        }
        prev = current;
      }
      return TRUE_STRING;
    }
  ),

  new PublicBinding(
    `vector`,
    { description: `-`,
      args: [],
      returns: `-` },
    {},
    _self => args => args
  ),

  new PublicBinding(
    `vector/append`,
    { description: `-`,
      args: [],
      returns: `-` },
    {},
    _self => ([list, ...items]) => {
      items.forEach(i => list.push(i));
      return list;
    }
  )];

export default {
  publicBindingType: `classic`,
  publicBindings
};
