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

// import Util from '../seni/Util';
import Immutable from 'immutable';

const TRUE_STRING = '#t';
const FALSE_STRING = '#f';

const NO_ERROR = undefined;

// evaluate will return the new env, the resultant evaluated form
// and any potential error
//
function evaluate(env, expr) {

  if (expr === undefined) {
    // in case of non-existent else clause in if statement
    return [env, undefined, NO_ERROR];
  }

  // todo: may need something like:
  if (typeof expr === 'number') {
    return [env, expr, NO_ERROR];
  }
  if (typeof expr === 'string') {
    if (expr === TRUE_STRING || expr === FALSE_STRING) {
      return [env, expr, NO_ERROR];
    }
    const bind = env.get(expr);
    if (bind === undefined) {
      return [env, undefined, `${expr} is undefined`];
    }
    // use priority of special, classic and then normal binding
    return [env, bind.special || bind.classic || bind.binding, NO_ERROR];
  }
  return funApplication(env, expr);
}

function funApplication(env, listExpr) {
  const fnName = listExpr[0];

  const [e, fun, err] = evaluate(env, fnName);

  if (err) {
    return [e, fun, err];
  }

  const bind = env.get(fnName);
  if (bind && bind.special) {
    // special forms that manipulate the listExpr and can change the env
    return fun(e, listExpr);
  }

  if (isClassicFunction(fnName)) {
    // classic functions that don't require named arguments
    return funApplicationClassic(e, fun, listExpr);
  }

  // normal functions that require named arguments
  const args = {};
  if (listExpr.length > 1) {
    // the 2nd listExpr node will be an object containing the arguments
    const argObj = listExpr[1];
    for (const k in argObj) {
      const [env2, form2, err2] = evaluate(e, argObj[k]);
      if (err2) {
        return [env2, form2, err2];
      }
      args[k] = form2;
    }
  }

  return [e, fun(args), NO_ERROR];
}

// check if the first element in the list expression is a classic function
function isClassicFunction(fnName) {
  return classicFunctions[fnName] !== undefined;
}

function funApplicationClassic(env, fun, [fnName, ...fnArguments]) {

  let argumentError = undefined;
  const args = fnArguments.map(n => {
    const [_env1, form1, err1] = evaluate(env, n);
    if (err1) {
      argumentError = err1;
    }
    return form1;
  });

  if (argumentError) {
    return [env, fun, argumentError];
  }


  // the classic functions that require all of their arguments to be numbers
  const requiringNumbers = ['+', '*', '-', '/', 'sqrt', 'mod', '<', '>'];

  const requiresNumbers = name => requiringNumbers.some(f => name === f);
  const allNumbers = values => values.every(Number.isFinite);

  if (requiresNumbers(fnName) && !allNumbers(args)) {
    return [env, fun, `all arguments to ${fnName} should be numbers`];
  }

  return [env, fun(args), NO_ERROR];
}

function isDefineExpression(form) {
  return form.constructor === Array &&
    (form[0] === 'fn' || form[0] === 'define');
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
const essentialEnv = [
  classicFunctions
].reduce((env, bindings) => setupBinding(env, bindings), new Immutable.Map());


function getBasicEnv() {
  return essentialEnv;
}

const Interpreter = {
  evaluate,
  isDefineExpression,
  getBasicEnv,
  NO_ERROR,
  TRUE_STRING,
  FALSE_STRING
};

export default Interpreter;
