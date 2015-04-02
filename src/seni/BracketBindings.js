/*
    Seni
    Copyright (C) 2015  Inderjit Gill <email@indy.io>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

import PublicBinding from '../lang/PublicBinding';
import MathUtil from './MathUtil';

const BracketBindings = {
  identity: new PublicBinding(
    'identity',
    `returns value
    arguments: value`,
    {value: 42},
    (self) => {
      return (params) => {
        const {value} = self.mergeWithDefaults(params);
        return value;
      };
    }
  ),

  int: new PublicBinding(
    'int',
    `returns an integer in the range min..max-1
    arguments: min max`,
    {min: 0, max: 100},
    (self, rng) => {
      // rng is a SeedRandom returning values in the range 0..1
      return (params) => {
        const {min, max} = self.mergeWithDefaults(params);
        return Number.parseInt(MathUtil.interpolate(min, max, rng()));
      };
    }
  ),

  scalar: new PublicBinding(
    'scalar',
    `returns a number in the range 0..1
    arguments: -`,
    {min: 0.0, max: 1.0},
    (self, rng) => {
      self = self;
      // rng is a SeedRandom returning values in the range 0..1
      return (params) => {
        const {min, max} = self.mergeWithDefaults(params);
        return MathUtil.interpolate(min, max, rng());
      };
    }
  ),

  testPlus: new PublicBinding(
    'testPlus',
    `[FOR TESTING ONLY] returns + character
    arguments: -`,
    {},
    () => {
      // rng is a SeedRandom returning values in the range 0..1
      return () => {
        return '+';
      };
    }
  )
};

export default BracketBindings;
