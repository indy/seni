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

import Immutable from 'immutable';

const TRUE_STRING = `#t`;
const FALSE_STRING = `#f`;

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
  if (typeof expr === `number`) {
    return [env, expr, NO_ERROR];
  }
  if (typeof expr === `string`) {
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

  let fun = undefined;
  let error = undefined;
  const fnName = listExpr[0];
  [env, fun, error] = evaluate(env, fnName);

  if (error) {
    return [env, fun, error];
  }

  const bind = env.get(fnName);

  if (bind && bind.special) {
    // special forms that manipulate the listExpr and can change the env
    return fun(env, listExpr);
  }

  if (bind && bind.classic) {
    // classic functions that don't require named arguments
    return funApplicationClassic(env, fun, listExpr);
  }

  // normal functions that require named arguments
  const [args, argsError] = buildArgs(env, listExpr);
  if (argsError) {
    return [env, undefined, argsError];
  }

  return [env, fun(args), NO_ERROR];
}

function buildArgs(env, listExpr) {
  const args = {};
  if (listExpr.length > 1) {
    // the 2nd listExpr node will be an object containing the arguments
    const argObj = listExpr[1];
    for (const k in argObj) {
      const [_, res, error] = evaluate(env, argObj[k]);
      if (error) {
        return [undefined, error];
      }
      args[k] = res;
    }
  }
  return [args, NO_ERROR];
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
  const requiringNumbers = [`+`, `*`, `-`, `/`, `sqrt`, `mod`, `<`, `>`];

  const requiresNumbers = name => requiringNumbers.some(f => name === f);
  const allNumbers = values => values.every(Number.isFinite);

  if (requiresNumbers(fnName) && !allNumbers(args)) {
    return [env, fun, `all arguments to ${fnName} should be numbers`];
  }

  return [env, fun(args), NO_ERROR];
}

function isDefineExpression(form) {
  return form.constructor === Array &&
    (form[0] === `fn` || form[0] === `define`);
}

function getBasicEnv() {
  return new Immutable.Map();
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
