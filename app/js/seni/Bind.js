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
import Interp from './Interp';
import Special from './Special';
import Classic from './Classic';

/*
  Env is an immutable map

  Each entry in the map is another map that contains:
  - pb: the Public Binding structure, can be used for in-app documentation

  and one of the following:
  - binding: a normal function/variable binding
  - special: a special binding for macro like functionality
  - classic: a binding for functions with classic lisp calling conventions

  see Interpreter::funApplication for how they're evaluated differently
*/

function createBind(env, key, pb, restArgs) {
  // call the PublicBinding's create function passing in an explicit self
  // along with any additional arguments
  const binding = pb.create.apply(null, [pb].concat(restArgs));

  // bind the value to the pb's name
  const obj = {};
  obj[key] = binding;
  obj.pb = pb;

  return env.set(pb.name, obj);
}

// take the publicBindings in the namespace and add them into env
// store them under the given key
//
// e.g. applyBindings(env, 'binding', {publicBindings: ['rect':...]})
// results in: env['rect'] = {binding: ...}
function applyBindings(env, key, namespace) {

  // grab any additional arguments that have been given to this function
  const restArgs = Array.prototype.slice.call(arguments, 3);
  const bindings = namespace.publicBindings;

  return bindings.reduce((e, pb) => createBind(e, key, pb, restArgs), env);
}

const Bind = {
  addBindings: (env, renderer) => {
    env = applyBindings(env, 'binding', Core);
    env = applyBindings(env, 'binding', MathUtil);
    env = applyBindings(env, 'binding', PseudoRandom);
    env = applyBindings(env, 'binding', Paths);
    env = applyBindings(env, 'binding', ColourBindings);
    env = applyBindings(env, 'binding', Interp);

    env = applyBindings(env, 'binding', MatrixStackBindings, renderer);
    env = applyBindings(env, 'binding', Shapes, renderer);
    env = applyBindings(env, 'binding', Focal, renderer);
    env = applyBindings(env, 'binding', Repeat, renderer);

    return env;
  },

  addBracketBindings: (env, rng) => {
    env = applyBindings(env, 'binding', Core);
    env = applyBindings(env, 'binding', MathUtil);
    env = applyBindings(env, 'binding', PseudoRandom);
    env = applyBindings(env, 'binding', ColourBindings);
    env = applyBindings(env, 'binding', Bracket, rng);

    return env;
  },

  addSpecialBindings: env => {
    env = applyBindings(env, 'special', Special);

    return env;
  },

  addClassicBindings: env => {
    env = applyBindings(env, 'classic', Classic);

    return env;
  }
};

export default Bind;
