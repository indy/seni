const TRUE_STRING = '#t';
const FALSE_STRING = '#f';

export function evaluate(env, expr) {

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

  let fun = evaluate(env, listExpr[0]);

  // special forms that manipulate the listExpr
  if(isSpecialForm(listExpr)) {
    let specialFn = evaluate(env, listExpr[0]);
    return fun(env, listExpr);
  }

  // classic functions that don't require named arguments
  if(isClassicFunction(listExpr)) {
    let args = listExpr.slice(1).map(n => evaluate(env, n));
    return fun(args);
  }

  // normal functions that require named arguments
  let args = {};
  if(listExpr.length > 1) {
    let argObj = listExpr[1];
    for(let k in argObj) {
      args[k] = evaluate(env, argObj[k]);
    }
  }
  return fun(args);
}

function isSpecialForm(listExpr) {
  let node = listExpr[0];
  if(specialForms[node] !== undefined) {
    return true;
  }
  return false;
}

function isClassicFunction(listExpr) {
  let node = listExpr[0];
  if(classicFunctions[node] !== undefined) {
    return true;
  }
  return false;
}

function addBindings(env, exprs) {

  let addBinding = function(name, value) {
    let v = evaluate(env, value);
    if(name.constructor === Array) {
      // destructure
      // todo: error check if size of name array !== size of v
      name.forEach((n, i) => env.add(n, v[i]));
    } else {
      env.add(name, v);
    }
  }

  exprs.forEach(([name, value]) => addBinding(name, value));

  return env;
}

export var specialForms = {
  'if': (env, expr) => {
    let conditional = evaluate(env, expr[1]);
    // todo: only a value of #t is truthy, change this so that
    // any non-zero, non-falsy value is truthy
    if(conditional === TRUE_STRING) {
      return evaluate(env, expr[2]);
    } else {
      if(expr.length == 4) {
        return evaluate(env, expr[3]);
      }
    }
  },
  'quote': (env, expr) => {
    return expr[1];       
  },
  'define': (env, expr) => {
    // (define foo 12)
    let name = expr[1];
    let val = expr[2];
    env.add(name, evaluate(env, val));
  },
  'set!': (env, expr) => {
    // (set! foo 42)
    let name = expr[1];
    let val = expr[2];
    env.mutate(name, evaluate(env, val));
  },
  'begin': (env, expr) => {
    // (begin (f1 1) (f2 3) (f3 5))
    return expr.slice(1).reduce((a, b) => evaluate(env, b), null);
  },
  'let': (env, expr) => {
    // (let ((a 12) (b 24)) (+ a b foo))
    return evaluate(addBindings(env.newScope(), expr[1]), expr[2]);
  },
  'lambda': (env, expr) => {
    // (lambda (x: 0 y: 0) (+ x y))
    return function(args) {
      let newEnv = env.newScope();

      let defaultArgValues = expr[1];
      for(let k in defaultArgValues) {
        if(args[k] !== undefined) {
          newEnv.add(k, args[k]);
        } else {
          newEnv.add(k, defaultArgValues[k]);
        }
      }
      return evaluate(newEnv, expr[2]);
    };
  },
  'loop': (env, expr) => {
    // (loop (a from: 1 to: 30 step: 2)
    //   (+ a a))
    let [varName, varParameters] = expr[1];
    let newEnv = env.newScope();
    return loopingFn(newEnv, expr[2], varName, varParameters);
  }
}

function loopingFn(env, expr, varName, {from = 0,
                                        to = 10,
                                        step = 1}) {
  var res;
  for(let i=from;i<to;i+=step) {
    env.add(varName, i);
    res = evaluate(env, expr);
  }
  return res;
}

// todo: classic functions are here because it wouldn't make sense to use named parameters for these functions.
// perhaps there should by a syntax like prefixing with @ to indicate that the function takes a variable number of non-named paramters?
// could get rid of the concept of classic functions and allow the user to create @ functions at the expense of having code like: (@+ 4 3 7 4) rather than (+ 4 3 7 4)

export var classicFunctions = {
  '+': (args) => args.reduce((a, b) => a + b, 0),

  '*': (args) => args.reduce((a, b) => a * b, 1),
  
  '-': (args) => args.length === 1 ? -args[0] : args.reduce((a, b) => a - b),

  '/': (args) => args.reduce((a, b) => a / b),
  
  '=': (args) => {
    let first = args[0];
    let res = args.slice(1).every(a => a === first);
    return res ? TRUE_STRING : FALSE_STRING;
  },
  '<': (args) => {
    let prev = args[0];
    for(let i = 1; i < args.length; i++) {
      let current = args[i];
      if(!(current < prev)) {
        return FALSE_STRING
      }
      prev = current;
    }
    return TRUE_STRING;
  },
  '>': (args) => {
    let prev = args[0];
    for(let i = 1; i < args.length; i++) {
      let current = args[i];
      if(!(current > prev)) {
        return FALSE_STRING;
      }
      prev = current;
    }
    return TRUE_STRING;
  },
  'list' : (args) => args,
  'pair' : (args) => {
    let res = [];
    for(let i=0;i<args.length;i+=2) {
      res.push([args[i], args[i+1]]);
    }
    return res;
  }
}



