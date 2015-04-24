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

import PublicBinding from './PublicBinding';
import MathUtil from './MathUtil';

const BracketBindings = {
  publicBindings: [
    new PublicBinding(
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

    new PublicBinding(
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

    new PublicBinding(
      'scalar',
      `returns a number in the range 0..1
      arguments: -`,
      {min: 0.0, max: 1.0},
      (self, rng) => {
        // rng is a SeedRandom returning values in the range 0..1
        return (params) => {
          const {min, max} = self.mergeWithDefaults(params);
          return MathUtil.interpolate(min, max, rng());
        };
      }
    ),

    new PublicBinding(
      'select',
      `returns a number in the range 0..1
      arguments: -`,
      {from: []},
      (self, rng) => {
        return (params) => {
          const {from} = self.mergeWithDefaults(params);
          if (from instanceof Array && from.length > 0) {
            const index = Number.parseInt(from.length * rng(), 10);
            return from[index];
          }
          console.log('select\'s from parameter should be a list');
          return undefined;
        };
      }
    ),

    new PublicBinding(
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
  ]
};

export default BracketBindings;
