// recursive code so switch off the jslint warnings
// about functions being used before they're defined
//
/*jslint ignore:start*/
/*jslint latedef:false, maxparams:6*/

import Util from '../seni/Util';

const TRUE_STRING = '#t';
const FALSE_STRING = '#f';

function _evaluate(env, expr) {

  if(expr === undefined) {
    // in case of non-existent else clause in if statement
    return undefined;
  }

  // todo: may need something like:
  if (typeof expr === 'number') {
    return expr;
  }
  if (typeof expr === 'string') {
    if(expr === TRUE_STRING || expr === FALSE_STRING) {
      return expr;
    }
    return env.lookup(expr);
  }
  return funApplication(env, expr);
}

function funApplication(env, listExpr) {

  const fun = _evaluate(env, listExpr[0]);

  if(fun === undefined) {
    // todo: use something better than console.log
    console.log(listExpr[0] + ' is undefined');
    return undefined;
  }

  // special forms that manipulate the listExpr
  if(isSpecialForm(listExpr)) {
    return fun(env, listExpr);
  }

  // classic functions that don't require named arguments
  if(isClassicFunction(listExpr)) {
    let args = listExpr.slice(1).map(n => _evaluate(env, n));
    return fun(args);
  }

  // normal functions that require named arguments
  let args = {};
  if(listExpr.length > 1) {
    const argObj = listExpr[1];
    for(let k in argObj) {
      args[k] = _evaluate(env, argObj[k]);
    }
  }
  return fun(args);
}

function isSpecialForm(listExpr) {
  const node = listExpr[0];
  return _specialForms[node] !== undefined;
}

function isClassicFunction(listExpr) {
  const node = listExpr[0];
  return _classicFunctions[node] !== undefined;
}

function addBindings(env, exprs) {

  const addBinding = function(name, value) {
    const v = _evaluate(env, value);
    if(name.constructor === Array) {
      // destructure
      // todo: error check if size of name array !== size of v
      name.forEach((n, i) => env.add(n, v[i]));
    } else {
      env.add(name, v);
    }
  };

  exprs.forEach(([name, value]) => addBinding(name, value));

  return env;
}

function assert(assertion) {
  if(!assertion) {
    console.log('ASSERT FAILED');
  }
}

function isDefiningFunction(nameForm) {

  let isFunction = false;
  if(nameForm.constructor === Array) {
    // it will either have one element in the array
    // e.g. (define (shout) (log "WOOHOOO")) => ['shout']
    // or there will be an argument map
    // e.g. (defin (doubler x: 3) (+ x x)) => ['doubler', {x: 3}]
    if(nameForm.length === 1) {
      isFunction = true;
    } else if (nameForm.length === 2) {
      // todo: also check for Map when we have immutable data structures here
      isFunction = true;
    } else {
      // invalid define statement
      assert(false);
      isFunction = false;
    }
  }
  return isFunction;
}

function defineFunction(env, defaultArgForms, body) {

  let defaultArgValues = {};
  for(let k in defaultArgForms) {
    defaultArgValues[k] = _evaluate(env, defaultArgForms[k]);
  }

  return function(args) {
    const newEnv = env.newScope();
    for(let k in defaultArgValues) {
      newEnv.add(k, args[k] === undefined ? defaultArgValues[k] : args[k]);
    }
    return evalBodyForms(newEnv, body);
  };
}

var _specialForms = {

  // (if something truthy falsey) || (if something truthy)
  'if': (env, [_, cond, t, f]) =>
    _evaluate(env, _evaluate(env, cond) === TRUE_STRING ? t : f),

  // (quote (age 99))
  'quote': (env, [_, form]) =>
    form,

  'define': (env, [_, nameForm, ...valueForms]) => {
    if(isDefiningFunction(nameForm)) {
      // e.g. (define (add x: 1 y: 2) (log x) (+ x y))
      let [name, defaultArgForms] = nameForm;
      let definedFunction = defineFunction(env, defaultArgForms, valueForms);
      return env.add(name, definedFunction);
    } else {
      // e.g. (define foo 12)
      assert(valueForms.length === 1);
      let val = valueForms[0];
      return env.add(nameForm, _evaluate(env, val));
    }
  },

  // (set! foo 42)
  'set!': (env, [_, name, val]) =>
    env.mutate(name, _evaluate(env, val)),

  // (begin (f1 1) (f2 3) (f3 5))
  'begin': (env, [_, ...body]) =>
    evalBodyForms(env, body),

  // (let ((a 12) (b 24)) (+ a b foo))
  'let': (env, [_, args, ...body]) => {
    return evalBodyForms(addBindings(env.newScope(), args), body);
  },

  // (fn (x: 0 y: 0) (+ x y))
  'fn': (env, [_, defaultArgForms, ...body]) => {
    return defineFunction(env, defaultArgForms, body);
  },

  // (print 'hi' foo) => hi 42
  'print': (env, [_, ...msgs]) => {
    console.log(msgs.reduce((a, b) => a + ' ' + _evaluate(env, b), ''));
  },

  // (log 'hi' foo) => hi <foo:42>
  'log': (env, [_, ...msgs]) => {
    let message = msgs.reduce((a, b) => {
      if(typeof b === 'string' && b !== TRUE_STRING && b !== FALSE_STRING) {
        return a + ' <' + b + ':' + _evaluate(env, b) + '>';
      }
      return a + ' ' + _evaluate(env, b);
    }, '');
    console.log(message);
  },


  // (loop (a from: 1 to: 30 step: 2) (+ a a))
  'loop': (env, [_, [varName, varParameters], ...body]) => {

    let vp = {};
    for(let k in varParameters) {
      vp[k] = _evaluate(env, varParameters[k]);
    }

    return loopingFn(env.newScope(), body, varName, vp);
  },

  // (onMatrixStack (f1 1) (f2 3) (f3 5))
  'onMatrixStack': (env, [_, ...body]) => {
    env.lookup('pushMatrix')();
    let res =  evalBodyForms(env, body);
    env.lookup('popMatrix')();
    return res;
  },
};

// whoa bodyform, bodyform for you
function evalBodyForms(env, bodyForms) {
  return bodyForms.reduce((a, b) => _evaluate(env, b), null);
}

function loopingFn(env, expr, varName, params) {
  // todo: 'to' should be <=, and 'until' should be '<'

  let {from, to, until, step} = Util.merge(params, {from: 0,
                                                    to: 10,
                                                    until: undefined,
                                                    step: 1});
  if(step === 0) {
    console.log('step size of 0 given');
    return undefined;
  }

  var res;
  if(until !== undefined) {
    for(let i=from;i<=until;i+=step) {
      env.add(varName, i);
      res = expr.reduce((a, b) => _evaluate(env, b), null);
    }
  } else {
    for(let i=from;i<to;i+=step) {
      env.add(varName, i);
      res = expr.reduce((a, b) => _evaluate(env, b), null);
    }
  }

  return res;
}

// todo: classic functions are here because it wouldn't make sense to
// use named parameters for these functions. perhaps there should by a
// syntax like prefixing with @ to indicate that the function takes a
// variable number of non-named paramters?

// could get rid of the concept of classic functions and allow the
// user to create @ functions at the expense of having code like: (@+
// 4 3 7 4) rather than (+ 4 3 7 4)


var _classicFunctions = {
  '+': (args) =>
    args.reduce((a, b) => a + b, 0),

  '*': (args) =>
    args.reduce((a, b) => a * b, 1),

  '-': (args) =>
    args.length === 1 ? -args[0] : args.reduce((a, b) => a - b),

  '/': (args) =>
    args.reduce((a, b) => a / b),

  '=': ([first, ...rest]) =>
    rest.every(a => a === first) ? TRUE_STRING : FALSE_STRING,

  '<': (args) => {
    let prev = args[0];
    for(let i = 1; i < args.length; i++) {
      const current = args[i];
      if(current >= prev) {
        return FALSE_STRING;
      }
      prev = current;
    }
    return TRUE_STRING;
  },

  '>': (args) => {
    let prev = args[0];
    for(let i = 1; i < args.length; i++) {
      const current = args[i];
      if(current <= prev) {
        return FALSE_STRING;
      }
      prev = current;
    }
    return TRUE_STRING;
  },

  'list' : (args) =>
    args,

  'pair' : (args) => {
    let res = [];
    for(let i=0;i<args.length;i+=2) {
      res.push([args[i], args[i+1]]);
    }
    return res;
  }
};


var Interpreter = {
  evaluate: function(env, expr) {
    return _evaluate(env, expr);
  },
  specialForms: _specialForms,
  classicFunctions: _classicFunctions
};

export default Interpreter;

