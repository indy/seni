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
import Interp from './Interp';

const publicBindings = [
  new PublicBinding(
    'identity',
    `returns value
      arguments: value`,
    {value: 42},
    self => params => {
      const {value} = self.mergeWithDefaults(params);
      return value;
    }
  ),

  new PublicBinding(
    'int',
    `returns an integer in the range min..max-1
      arguments: min max`,
    {min: 0, max: 100},
    // rng is a PseudoRandom returning values in the range 0..1
    (self, rng) => params => {
      const {min, max} = self.mergeWithDefaults(params);
      return Number.parseInt(Interp.interpolate(min, max, rng()), 10);
    }
  ),

  new PublicBinding(
    'scalar',
    `returns a number in the range 0..1
      arguments: -`,
    {min: 0.0, max: 1.0},
    // rng is a PseudoRandom returning values in the range 0..1
    (self, rng) => params => {
      const {min, max} = self.mergeWithDefaults(params);
      return Interp.interpolate(min, max, rng());
    }
  ),

  new PublicBinding(
    'vector',
    `returns a vector
      arguments: min max`,
    {min: 0.0, max: 1000.0},
    (self, rng) => params => {
      const {min, max} = self.mergeWithDefaults(params);
      const x = Interp.interpolate(min, max, rng());
      const y = Interp.interpolate(min, max, rng());
      return ['list', x, y];
    }
  ),

  new PublicBinding(
    'select',
    `returns a number in the range 0..1
      arguments: -`,
    {from: []},
    (self, rng) => params => {
      const {from} = self.mergeWithDefaults(params);
      if (from instanceof Array && from.length > 0) {
        const index = Number.parseInt(from.length * rng(), 10);
        return from[index];
      }
      console.log('select\'s from parameter should be a list');
      return undefined;
    }
  ),

  new PublicBinding(
    'col',
    `returns a random rgb colour it takes a single argument of alpha
since I think we'll often want to fix the alpha value. If you
need to fix the other values in a colour declaration then just
      use separate scalars for each of the components. e.g.

    (col/rgb r: 0.8
             g: [0.3 (scalar)]
             b: [0.2 (scalar)]
             alpha: 0.4)

random colour:
    [(col/rgb r: 0.8
             g: 0.3
             b: 0.2
             alpha: 0.4) (col)]

random colour, but keep alpha as 0.4:
    [(col/rgb r: 0.8
             g: 0.3
             b: 0.2
             alpha: 0.4) (col alpha: 0.4)]
`,
    {},
    (self, rng) => params => {
      const r = rng(), g = rng(), b = rng();
      let alpha = rng();
      if (params.alpha) {
        alpha = params.alpha;
      }
      return ['col/rgb', {r, g, b, alpha}];
    }
  ),

  new PublicBinding(
    'testPlus',
    `[FOR TESTING ONLY] returns + character
      arguments: -`,
    {},
    // rng is a PseudoRandom returning values in the range 0..1
    () => () => '+'
  )
];

export default {
  publicBindings
};
