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

const { evaluate, TRUE_STRING, NO_ERROR } = Interpreter;

// whoa bodyform, bodyform for you
function evalBodyForms(env, bodyForms) {
  return bodyForms.reduce(([e, _, error], b) => {
    const [env1, form1, error1] = evaluate(e, b);
    let retError = error;
    if (error1 && retError === NO_ERROR) {
      // store the first error and always return it
      retError = error1;
    }
    return [env1, form1, retError];
  }, [env, true, NO_ERROR]);
}

function defineFunction(env, defaultArgForms, body) {

  const defaultArgValues = {};
  for (const k in defaultArgForms) {
    const [_e, v, err] = evaluate(env, defaultArgForms[k]);
    if (err) {
      return [undefined, err];
    }
    defaultArgValues[k] = v;
  }

  return [function(args) {
    let newEnv = env;
    for (const k in defaultArgValues) {
      newEnv = newEnv.set(k, {
        binding: args[k] === undefined ? defaultArgValues[k] : args[k]
      });
    }
    return evalBodyForms(newEnv, body)[1];
  }, NO_ERROR];
}

function addBindings(env, exprs) {

  let lastValueBound = undefined;

  // adds a binding to the env, returning the ['new environment', error] pair
  const addBinding = function(e_, name, value) {
    let e = e_;
    const [env, v, error] = evaluate(e, value);
    if (error) {
      return [env, error];
    }

    if (name.constructor === Array && name[0] === `list`) {

      // for square bracket notation when declaring variables
      // e.g. (define [x y] [100 200])
      // the names compile to ['list', 'x', 'y']

      const values = v;
      const names = name.slice(1);

      if (names.length !== values.length) {
        return [e, `binding mismatch between ${names} and ${values}`];
      }

      names.forEach((n, i) => e = e.set(n, { binding: values[i] }));
      lastValueBound = values[names.length-1];
    } else {
      e = e.set(name, { binding: v });
      lastValueBound = v;
    }
    return [e, NO_ERROR];
  };

  let firstError = NO_ERROR;
  const newEnv = exprs.reduce((e, [name, value]) => {
    const [env2, error] = addBinding(e, name, value);
    if (error && firstError === NO_ERROR) {
      // store the first error to occur
      firstError = error;
    }
    return env2;
  }, env);

  return [newEnv, lastValueBound, firstError];
}

function loopingFn(env, expr, varName, params) {

  const { from, to, upto, steps, increment } = Object.assign({
    from: 0,
    to: 1,
    upto: undefined,
    steps: undefined,
    increment: 1}, params);

  let unit, i;
  let res = [env, undefined, NO_ERROR];

  if (steps !== undefined) {
    if (steps < 1) {
      return [env, undefined, `steps must be greater than 0`];
    }

    if (upto === undefined) {
      // from, to, steps
      unit = (to - from) / steps;
    } else {
      // from, upto, steps
      unit = (upto - from) / (steps - 1);
    }

    for (i = 0; i < steps; i++) {
      const val = from + (i * unit);
      res = evalBodyForms(env.set(varName, { binding: val }), expr);
    }
    return res;
  }

  if (increment === 0) {
    return [env, undefined, `increment of 0 given`];
  }

  let delta = increment;
  if (upto !== undefined) {
    if (from <= upto) {
      for (i = from; i <= upto; i += delta) {
        res = evalBodyForms(env.set(varName, { binding: i }), expr);
      }
    } else {
      delta = increment > 0 ? -increment : increment;
      for (i = from; i >= upto; i += delta) {
        res = evalBodyForms(env.set(varName, { binding: i }), expr);
      }
    }
  } else {
    if (from <= to) {
      for (i = from; i < to; i += delta) {
        res = evalBodyForms(env.set(varName, { binding: i }), expr);
      }
    } else {
      delta = increment > 0 ? -increment : increment;
      for (i = from; i > to; i += delta) {
        res = evalBodyForms(env.set(varName, { binding: i }), expr);
      }
    }
  }

  return res;
}

const publicBindings = [
  new PublicBinding(
    `if`,
    { description: `-`,
      args: [],
      returns: `-` },
    {},
    _self => (env, [_, cond, t, f]) => {
      const [e, conditionVal, err] = evaluate(env, cond);
      if (err) {
        return [e, conditionVal, err];
      }
      return evaluate(env, conditionVal === TRUE_STRING ? t : f);
    }
  ),

  /*
   todo: remove this code now that __string is used instead
   todo: the compiler will hack in a quote around strings, this
   needs to take that into account. e.g. given (quote "hi"), the ast
   built by the compiler will be: [`quote` [`quote` `hi`]] rather than
   the expected [`quote` `hi`]. So this is a hack to check inside the
   form, if it`s another list beginning with quote, just return that.

   the proper solution is not to pass in a simplified AST and to retain
   the nodeType information so that the interpreter can differentiate
   between names and strings, this would mean that the compiler
   wouldn`t have to wrap strings in quotes and the code for evaling
   quote becomes `quote`: (env, [_, form]) =>[env, form]

   the cost of this is a more complicated AST, but it seems like a
   price worth paying
   */
  new PublicBinding(
    `quote`,
    { description: `-`,
      args: [],
      returns: `-` },
    {},
    _self => (env, [_, form]) => {
      if (form.constructor === Array) {
        if (form[0] === `quote`) {
          return [env, form[1], NO_ERROR];
        }
      }
      return [env, form, NO_ERROR];
    }
  ),

  new PublicBinding(
    `__string`,
    { description: `-`,
      args: [],
      returns: `-` },
    {},
    _self => (env, [_, form]) => [env, form]
  ),

  new PublicBinding(
    `fn`,
    { description: `-`,
      args: [],
      returns: `-` },
    {},
    _self => (env, [_, nameForm, ...valueForms]) => {
      const [name, defaultArgForms] = nameForm;
      const [definedFunction, error] =
              defineFunction(env, defaultArgForms, valueForms);
      return [env.set(name, { binding: definedFunction }),
              definedFunction,
              error];
    }
  ),

  new PublicBinding(
    `define`,
    { description: `define creates bindings in it's parent scope.
      This may not be the expected behaviour`,
      args: [],
      returns: `-` },
    {},
    _self => (env, [_, ...args]) => {
      // wrap the args into pairs
      if (args.length % 2 === 1) {
        return [env, undefined, `define should have an even number of args`];
      }

      const argPairs = [];
      for (let i = 0; i < args.length; i += 2) {
        argPairs.push([args[i + 0], args[i + 1]]);
      }

      return addBindings(env, argPairs);
    }
  ),

  new PublicBinding(
    `begin`,
    { description: `(begin (f1 1) (f2 3) (f3 5))`,
      args: [],
      returns: `-` },
    {},
    _self => (env, [_, ...body]) => evalBodyForms(env, body)
  ),

  new PublicBinding(
    `loop`,
    { description: `(loop (a from: 1 to: 30 step: 2) (+ a a))`,
      args: [],
      returns: `-` },
    {},
    _self => (env, [_, [varName, varParameters], ...body]) => {
      const vp = {};
      for (const k in varParameters) {
        vp[k] = evaluate(env, varParameters[k])[1];
      }

      return loopingFn(env, body, varName, vp);
    }
  ),

  new PublicBinding(
    `on-matrix-stack`,
    { description: `(on-matrix-stack (f1 1) (f2 3) (f3 5))`,
      args: [],
      returns: `-` },
    {},
    _self => (env, [_, ...body]) => {
      env.get(`push-matrix`).binding();
      const res = evalBodyForms(env, body);
      env.get(`pop-matrix`).binding();
      return res;
    }
  )

];

export default {
  publicBindingType: `special`,
  publicBindings
};
