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

import PublicBinding from './PublicBinding';


const publicBindings = [
  new PublicBinding(
    'canvas/width',
    'the width of the canvas',
    {},
    () => 1000
  ),

  new PublicBinding(
    'canvas/height',
    'the height of the canvas',
    {},
    () => 1000
  ),

  new PublicBinding(
    'canvas/centre',
    'the centre of the canvas',
    {},
    () => [500, 500]
  ),

  new PublicBinding(
    'list/length',

    '',

    {of: []},

    self => params => {
      const {of} = self.mergeWithDefaults(params);
      return of.length;
    }
  ),

  new PublicBinding(
    'list/get',

    '',

    {from: [], nth: 0},

    self => params => {
      const {from, nth} = self.mergeWithDefaults(params);
      return from[nth];
    }
  ),

  new PublicBinding(
    'take',

    `invokes the 'from' function 'num' times, returning a list`,

    {num: 1, from() { return 0; }},

    self => params => {
      const {num, from} = self.mergeWithDefaults(params);
      const res = [];
      for (let i = 0; i < num; i++) {
        res.push(from());
      }
      return res;
    }
  )];

export default {
  publicBindings
};
