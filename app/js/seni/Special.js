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

import PublicBinding from './PublicBinding';
import Util from './Util';
import Interpreter from '../lang/Interpreter';

const { evaluate, TRUE_STRING, FALSE_STRING, NO_ERROR } = Interpreter;

// whoa bodyform, bodyform for you
function evalBodyForms(env, bodyForms) {
  return bodyForms.reduce(([e, _, error], b) => {
    const [env1, form1, error1] = evaluate(e, b);
    if (error1 && error === NO_ERROR) {
      // store the first error and always return it
      error = error1;
    }
    return [env1, form1, error];
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
  const addBinding = function(e, name, value) {
    const [env, v, error] = evaluate(e, value);
    if (error) {
      return [env, error];
    }

    if (name.constructor === Array && name[0] === 'list') {

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
  // todo: 'to' should be <=, and 'upto' should be '<'

  // todo: upto isn't going to work with steps, perhaps remove it?
  const merged = Util.merge(params, {from: 0,
                                     to: 1,
                                     upto: undefined,
                                     steps: undefined,
                                     'steps-upto': undefined,
                                     increment: 1});
  let res, limit, unit, val;

  const {from,
         to,
         upto,
         steps,
         increment} = merged;

  const stepsUpto = merged['steps-upto'];

  // initialise res in case we don't assign anything to it again.
  // (could happen in cases such as the 'to' is less than the 'from')
  res = [env, undefined];

  if (stepsUpto !== undefined || steps !== undefined) {
    const s = stepsUpto || steps;
    if (s < 1) {
      console.log('steps-upto | steps  must be greater than 0');
      return res;
    }

    limit = upto !== undefined ? upto : to;
    if (stepsUpto !== undefined) {
      unit = (limit - from) / s;
    } else {
      unit = (limit - from) / (s - 1);
    }

    for (let i = 0; i < s; i++) {
      val = from + (i * unit);
      res = evalBodyForms(env.set(varName, { binding: val }), expr);
    }
    return res;
  }

  if (increment === 0) {
    console.log('increment of 0 given');
    return res;
  }

  if (upto !== undefined) {
    for (let i = from; i <= upto; i += increment) {
      res = evalBodyForms(env.set(varName, { binding: i }), expr);
    }
  } else {
    for (let i = from; i < to; i += increment) {
      res = evalBodyForms(env.set(varName, { binding: i }), expr);
    }
  }

  return res;
}

const publicBindings = [
  new PublicBinding(
    'if',
    {
      description: '-',
      args: [],
      returns: `-`
    },
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
    built by the compiler will be: ['quote' ['quote' 'hi']] rather than
    the expected ['quote' 'hi']. So this is a hack to check inside the
    form, if it's another list beginning with quote, just return that.

    the proper solution is not to pass in a simplified AST and to retain
    the nodeType information so that the interpreter can differentiate
    between names and strings, this would mean that the compiler
    wouldn't have to wrap strings in quotes and the code for evaling
    quote becomes 'quote': (env, [_, form]) =>[env, form]

    the cost of this is a more complicated AST, but it seems like a
    price worth paying
  */
  new PublicBinding(
    'quote',
    {
      description: '-',
      args: [],
      returns: `-`
    },
    {},
    _self => (env, [_, form]) => {
      if (form.constructor === Array) {
        if (form[0] === 'quote') {
          return [env, form[1], NO_ERROR];
        }
      }
      return [env, form, NO_ERROR];
    }
  ),

  new PublicBinding(
    '__string',
    {
      description: '-',
      args: [],
      returns: `-`
    },
    {},
    _self => (env, [_, form]) => [env, form]
  ),

  new PublicBinding(
    'fn',
    {
      description: '-',
      args: [],
      returns: `-`
    },
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
    'define',
    {
      description: `define creates bindings in it's parent scope.
This may not be the expected behaviour`,
      args: [],
      returns: `-`
    },
    {},
    _self => (env, [_, ...args]) => {
      // wrap the args into pairs
      if (args.length % 2 === 1) {
        return [env, undefined, 'define should have an even number of args'];
      }

      const argPairs = [];
      for (let i = 0; i < args.length; i += 2) {
        argPairs.push([args[i + 0], args[i + 1]]);
      }

      return addBindings(env, argPairs);
    }
  ),

  new PublicBinding(
    'begin',
    {
      description: '(begin (f1 1) (f2 3) (f3 5))',
      args: [],
      returns: `-`
    },
    {},
    _self => (env, [_, ...body]) => evalBodyForms(env, body)
  ),

  new PublicBinding(
    'print',
    {
      description: `(print 'hi' foo) => hi 42`,
      args: [],
      returns: `-`
    },
    {},
    _self => (env, [_, ...msgs]) => {
      const printMsg = msgs.reduce((a, b) => `${a} ${evaluate(env, b)[1]}`, '');
      console.log(printMsg.trim());
      return [env, true, NO_ERROR];
    }
  ),

  new PublicBinding(
    'log',
    {
      description: `(log 'hi' foo) => hi <foo:42>`,
      args: [],
      returns: `-`
    },
    {},
    _self => (env, [_, ...msgs]) => {
      let firstError = NO_ERROR;
      const message = msgs.reduce((a, b) => {
        const [_e, res, err] = evaluate(env, b);
        if (err && firstError === NO_ERROR) {
          firstError = err;
        }
        if (typeof b === 'string' && b !== TRUE_STRING && b !== FALSE_STRING) {
          return `${a} < ${b}:${res}>`;
        }
        return `${a} ${res}`;
      }, '');
      console.log(message);
      return [env, true, firstError];
    }
  ),

  new PublicBinding(
    'loop',
    {
      description: `(loop (a from: 1 to: 30 step: 2) (+ a a))`,
      args: [],
      returns: `-`
    },
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
    'on-matrix-stack',
    {
      description: `(on-matrix-stack (f1 1) (f2 3) (f3 5))`,
      args: [],
      returns: `-`
    },
    {},
    _self => (env, [_, ...body]) => {
      env.get('push-matrix').binding();
      const res = evalBodyForms(env, body);
      env.get('pop-matrix').binding();
      return res;
    }
  )

];

export default {
  publicBindingType: 'special',
  publicBindings
};
