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

import Shapes from './Shapes';
import Paths from './Paths';
import MatrixStackBindings from './MatrixStackBindings';
import MathUtil from './MathUtil';
import ColourBindings from './ColourBindings';
import Core from './Core';
import Bracket from './BracketBindings';
import PseudoRandom from './PseudoRandom';
import Focal from './Focal';
import Repeat from './Repeat';

function createBind(env, pb, restArgs) {

  // call the PublicBinding's create function passing in an explicit self
  // along with any additional arguments
  let value = pb.create.apply(null, [pb].concat(restArgs));

  // bind the value to the pb's name
  return env.set(pb.name, { binding : value, pb : pb });
}

// applies the publicBindings in namespace to env
function applyPublicBindings(env, namespace) {
  // grab any additional arguments that have been given to this function
  let restArgs = Array.prototype.slice.call(arguments, 2);
  let bindings = namespace.publicBindings;

  return bindings.reduce((e, pb) => createBind(e, pb, restArgs), env);
}

const Bind = {
  addBindings: function(env, renderer) {
    env = applyPublicBindings(env, Core);
    env = applyPublicBindings(env, MathUtil);
    env = applyPublicBindings(env, PseudoRandom);
    env = applyPublicBindings(env, MatrixStackBindings, renderer);
    env = applyPublicBindings(env, Shapes, renderer);
    env = applyPublicBindings(env, Paths);
    env = applyPublicBindings(env, ColourBindings);
    env = applyPublicBindings(env, Focal, renderer);
    env = applyPublicBindings(env, Repeat, renderer);
    return env;
  },

  addBracketBindings: function(env, rng) {
    env = applyPublicBindings(env, Core);
    env = applyPublicBindings(env, MathUtil);
    env = applyPublicBindings(env, PseudoRandom);
    env = applyPublicBindings(env, ColourBindings);
    env = applyPublicBindings(env, Bracket, rng);

    return env;
  }
};

export default Bind;
