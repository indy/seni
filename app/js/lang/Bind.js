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

import Special from './Special';
import Classic from './Classic';
import HigherOrder from './HigherOrder';

import SpecialDebug from '../seni/SpecialDebug';
import Shapes from '../seni/Shapes';
import Paths from '../seni/Paths';
import MatrixStackBindings from '../seni/MatrixStackBindings';
import MathUtil from '../seni/MathUtil';
import ColourBindings from '../seni/ColourBindings';
import Core from '../seni/Core';
import BracketBindings from '../seni/BracketBindings';
import PseudoRandom from '../seni/PseudoRandom';
import Focal from '../seni/Focal';
import Repeat from '../seni/Repeat';
import Interp from '../seni/Interp';

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

function createBind(env, bindType, pb, restArgs) {
  // call the PublicBinding's create function passing in an explicit self
  // along with any additional arguments
  const binding = pb.create.apply(null, [pb].concat(restArgs));

  // bind the value to the pb's name
  const obj = {};
  obj[bindType] = binding;
  obj.pb = pb;

  return env.set(pb.name, obj);
}

// take the publicBindings in the namespace and add them into env
// store them under the given publicBindingType
//
// e.g. applyBindings(env, {publicBindings: ['rect':...],
//                          publicBindingType: 'binding'})
// results in: env['rect'] = {binding: ...}
function applyBindings(env, namespace, ...args) {
  // grab any additional arguments that have been given to this function
  const bindings = namespace.publicBindings;
  const bindType = namespace.publicBindingType;

  return bindings.reduce((e, pb) => createBind(e, bindType, pb, args), env);
}

const Bind = {
  addBindings: (env_, renderer) => {
    let env = env_;
    env = applyBindings(env, Core);
    env = applyBindings(env, MathUtil);
    env = applyBindings(env, PseudoRandom);
    env = applyBindings(env, Paths);
    env = applyBindings(env, ColourBindings);
    env = applyBindings(env, Interp);

    env = applyBindings(env, MatrixStackBindings, renderer);
    env = applyBindings(env, Shapes, renderer);
    env = applyBindings(env, Focal, renderer);
    env = applyBindings(env, Repeat, renderer);

    return env;
  },

  addBracketBindings: (env_, rng) => {
    let env = env_;
    env = applyBindings(env, Core);
    env = applyBindings(env, MathUtil);
    env = applyBindings(env, PseudoRandom);
    env = applyBindings(env, ColourBindings);
    env = applyBindings(env, BracketBindings, rng);

    return env;
  },

  addSpecialBindings: env_ => {
    let env = env_;
    env = applyBindings(env, Special);

    return env;
  },

  addSpecialDebugBindings: (env_, _konsole) => {
    let env = env_;
    env = applyBindings(env, SpecialDebug);

    return env;
  },

  addClassicBindings: env_ => {
    let env = env_;
    env = applyBindings(env, Classic);
    env = applyBindings(env, HigherOrder);

    return env;
  }
};

export default Bind;
