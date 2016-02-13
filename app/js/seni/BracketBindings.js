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
import Interp from './Interp';

const publicBindings = [
  new PublicBinding(
    'identity',
    {
      description: 'basic identity function',
      args: [['value', 'the value to return']],
      returns: 'the given value'
    },
    {value: 42},
    self => params => {
      const {value} = self.mergeWithDefaults(params);
      return value;
    }
  ),

  new PublicBinding(
    'int',
    {
      description: 'generate an integer',
      args: [['min', ''],
             ['max', '']],
      returns: 'an integer in the range min..max-1'
    },
    {min: 0, max: 100},
    // rng is a PseudoRandom returning values in the range 0..1
    (self, rng) => params => {
      const {min, max} = self.mergeWithDefaults(params);
      return Number.parseInt(Interp.interpolate(min, max, rng()), 10);
    }
  ),

  new PublicBinding(
    'scalar',
    {
      description: 'generate an scalar',
      args: [['min', ''],
             ['max', '']],
      returns: 'an scalar in the range min..max'
    },
    {min: 0.0, max: 1.0},
    // rng is a PseudoRandom returning values in the range 0..1
    (self, rng) => params => {
      const {min, max} = self.mergeWithDefaults(params);
      return Interp.interpolate(min, max, rng());
    }
  ),

  new PublicBinding(
    'vector',
    {
      description: 'generate a 2d vector',
      args: [['min', ''],
             ['max', '']],
      returns: 'a 2d vector with each element in the range min..max'
    },
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
    {
      description: 'selects a value from a vector',
      args: [['from', 'a vector of values']],
      returns: 'one of the values in the from vector'
    },
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
    {
      description: 'generates a colour',
      args: [['alpha', 'the alpha value to use']],
      returns: `a colour`
    }
    ,
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
    {
      description: '[FOR TESTING ONLY] returns + character'
    },
    {},
    // rng is a PseudoRandom returning values in the range 0..1
    () => () => '+'
  )
];

export default {
  publicBindingType: 'binding',
  publicBindings
};
