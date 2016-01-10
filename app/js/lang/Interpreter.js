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

/* eslint-disable no-use-before-define */
/* eslint-disable no-redeclare */

import Util from '../seni/Util';
import Immutable from 'immutable';

const TRUE_STRING = '#t';
const FALSE_STRING = '#f';

function evaluate(env, expr) {

  if (expr === undefined) {
    // in case of non-existent else clause in if statement
    return [env, undefined];
  }

  // todo: may need something like:
  if (typeof expr === 'number') {
    return [env, expr];
  }
  if (typeof expr === 'string') {
    if (expr === TRUE_STRING || expr === FALSE_STRING) {
      return [env, expr];
    }
    if (env.get(expr) === undefined) {
      console.log(expr, 'is undefined');
      return undefined;
    }
    return [env, env.get(expr).binding];
  }
  return funApplication(env, expr);
}

function funApplication(env, listExpr) {

  const [e, fun] = evaluate(env, listExpr[0]);

  if (fun === undefined) {
    // todo: use something better than console.log
    console.log(listExpr.toJS(), `${listExpr[0]} is undefined`);
    return [e, undefined];
  }

  // special forms that manipulate the listExpr and can change the env
  if (isSpecialForm(listExpr)) {
    return fun(e, listExpr);
  }

  // classic functions that don't require named arguments
  if (isClassicFunction(listExpr)) {
    const argu = listExpr.slice(1).map(n => evaluate(e, n)[1]);
    return [e, fun(argu)];
  }

  // normal functions that require named arguments
  const args = {};
  if (listExpr.length > 1) {
    const argObj = listExpr[1];
    for (const k in argObj) {
      args[k] = evaluate(e, argObj[k])[1];
    }
  }
  return [e, fun(args)];
}

function isSpecialForm(listExpr) {
  const node = listExpr[0];
  return specialForms[node] !== undefined;
}

function isClassicFunction(listExpr) {
  const node = listExpr[0];
  return classicFunctions[node] !== undefined;
}

function addBindings(env, exprs) {

  let lastValueBound = undefined;

  const addBinding = function(e, name, value) {
    const v = evaluate(e, value)[1];
    if (name.constructor === Array && name[0] === 'list') {

      // for square bracket notation when declaring variables
      // e.g. (define [x y] [100 200])
      // the names compile to ['list', 'x', 'y']

      const values = v;
      const names = name.slice(1);

      if (names.length !== values.length) {
        console.error('binding mismatch between', names, values);
      }

      names.forEach((n, i) => e = e.set(n, { binding: values[i] }));
      lastValueBound = values[names.length-1];
    } else {
      e = e.set(name, { binding: v });
      lastValueBound = v;
    }
    return e;
  };

  return [exprs.reduce((a, [name, value]) => addBinding(a, name, value), env),
          lastValueBound];
}

function isDefineExpression(form) {
  return form.constructor === Array &&
    (form[0] === 'fn' || form[0] === 'define');
}

function defineFunction(env, defaultArgForms, body) {

  const defaultArgValues = {};
  for (const k in defaultArgForms) {
    defaultArgValues[k] = evaluate(env, defaultArgForms[k])[1];
  }

  return function(args) {
    let newEnv = env;
    for (const k in defaultArgValues) {
      newEnv = newEnv.set(k, {
        binding: args[k] === undefined ? defaultArgValues[k] : args[k]
      });
    }
    return evalBodyForms(newEnv, body)[1];
  };
}

/* eslint-disable no-unused-vars */
const specialForms = {
  // (if something truthy falsey) || (if something truthy)
  'if': (env, [_, cond, t, f]) =>
    evaluate(env, evaluate(env, cond)[1] === TRUE_STRING ? t : f),

  // (quote (age 99))
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
  'quote': (env, [_, form]) => {
    if (form.constructor === Array) {
      if (form[0] === 'quote') {
        return [env, form[1]];
      }
    }
    return [env, form];
  },

  '__string': (env, [_, form]) => [env, form],

  'fn': (env, [_, nameForm, ...valueForms]) => {
    const [name, defaultArgForms] = nameForm;
    const definedFunction = defineFunction(env, defaultArgForms, valueForms);
    return [env.set(name, { binding: definedFunction }), definedFunction];
  },

  // NOTE: in the documentation state that define creates bindings in
  // it's parent scope. This may not be the expected behaviour
  'define': (env, [_, ...args]) => {
    // wrap the args into pairs
    if (args.length % 2 === 1) {
      console.error('define should have an even number of args', args);
    }

    const argPairs = [];
    for (let i = 0; i < args.length; i += 2) {
      argPairs.push([args[i + 0], args[i + 1]]);
    }

    return addBindings(env, argPairs);
  },

  // (begin (f1 1) (f2 3) (f3 5))
  'begin': (env, [_, ...body]) => evalBodyForms(env, body),

  // (print 'hi' foo) => hi 42
  'print': (env, [_, ...msgs]) => {
    const printMsg = msgs.reduce((a, b) => `${a} ${evaluate(env, b)[1]}`, '');
    console.log(printMsg.trim());
    return [env, true];
  },

  // (log 'hi' foo) => hi <foo:42>
  'log': (env, [_, ...msgs]) => {
    const message = msgs.reduce((a, b) => {
      const r = evaluate(env, b);
      const res = r[1];
      if (typeof b === 'string' && b !== TRUE_STRING && b !== FALSE_STRING) {
        return `${a} < ${b}:${res}>`;
      }
      return `${a} ${res}`;
    }, '');
    console.log(message);
    return [env, true];
  },

  // (loop (a from: 1 to: 30 step: 2) (+ a a))
  'loop': (env, [_, [varName, varParameters], ...body]) => {
    const vp = {};
    for (const k in varParameters) {
      vp[k] = evaluate(env, varParameters[k])[1];
    }

    return loopingFn(env, body, varName, vp);
  },

  // (on-matrix-stack (f1 1) (f2 3) (f3 5))
  'on-matrix-stack': (env, [_, ...body]) => {
    env.get('push-matrix').binding();
    const res =  evalBodyForms(env, body);
    env.get('pop-matrix').binding();
    return res;
  }
};
/* eslint-enable no-unused-vars */

// whoa bodyform, bodyform for you
function evalBodyForms(env, bodyForms) {
  return bodyForms.reduce((a, b) => evaluate(a[0], b), [env, true]);
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


  if (stepsUpto !== undefined || steps !== undefined) {
    const s = stepsUpto || steps;
    if (s < 1) {
      console.log('steps-upto | steps  must be greater than 0');
      return undefined;
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
    return undefined;
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

// todo: classic functions are here because it wouldn't make sense to
// use named parameters for these functions. perhaps there should be a
// syntax like prefixing with @ to indicate that the function takes a
// variable number of non-named paramters?

// could get rid of the concept of classic functions and allow the
// user to create @ functions at the expense of having code like: (@+
// 4 3 7 4) rather than (+ 4 3 7 4)

const classicFunctions = {
  '+': args => args.reduce((a, b) => a + b, 0),

  '*': args => args.reduce((a, b) => a * b, 1),

  '-': args => args.length === 1 ? -args[0] : args.reduce((a, b) => a - b),

  '/': args => args.reduce((a, b) => a / b),

  'sqrt': args => Math.sqrt(args[0]),

  'mod': args => args[0] % args[1],

  '=': ([first, ...rest]) =>
    rest.every(a => a === first) ? TRUE_STRING : FALSE_STRING,

  '<': args => {
    let prev = args[0];
    for (let i = 1; i < args.length; i++) {
      const current = args[i];
      if (prev >= current) {
        return FALSE_STRING;
      }
      prev = current;
    }
    return TRUE_STRING;
  },

  '>': args => {
    let prev = args[0];
    for (let i = 1; i < args.length; i++) {
      const current = args[i];
      if (prev <= current) {
        return FALSE_STRING;
      }
      prev = current;
    }
    return TRUE_STRING;
  },

  'list': args => args,

  'append': ([list, ...items]) => {
    items.forEach(i => list.push(i));
    return list;
  }
};

function setupBinding(env, rawBindings) {

  for (const prop in rawBindings) {
    env = env.set(prop, { binding: rawBindings[prop] });
  }
  return env;
}

// specialForms and classicFunctions are defined in name:value pairs
// so transform them to name:{binding: value} pairs
const basicEnv = [
  specialForms,
  classicFunctions
].reduce((env, bindings) => setupBinding(env, bindings), new Immutable.Map());


function getBasicEnv() {
  return basicEnv;
}

const Interpreter = {
  evaluate,
  isDefineExpression,
  getBasicEnv
};

export default Interpreter;
