/*
 *  Seni
 *  Copyright (C) 2015  Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

import PublicBinding from './PublicBinding';
import seedrandom from 'seedrandom';

function buildUnsigned(seedVal) {
  //const seedrandom = Math.seedrandom;
  const saveable = seedrandom(seedVal, {state: true});
  return () => saveable();
}

function buildSigned(seedVal) {
  //const seedrandom = Math.seedrandom;
  const saveable = seedrandom(seedVal, {state: true});
  return () => (saveable() * 2.0) - 1.0;
}

const SeedRandom = {
  buildUnsigned,
  buildSigned,

  publicBindings: [
    new PublicBinding(
      'rng/unsigned',
      `returns a function that generates a random number in the range 0..1`,
      {seed: 'shabba'},
      (self) => function(params) {
        const {seed} = self.mergeWithDefaults(params);
        return buildUnsigned(seed);
      }
    ),

    new PublicBinding(
      'rng/signed',
      `returns a function that generates a random number in the range -1..1`,
      {seed: 'shabba'},
      (self) => function(params) {
        const {seed} = self.mergeWithDefaults(params);
        return buildSigned(seed);
      }
    )
  ]
};

export default SeedRandom;
