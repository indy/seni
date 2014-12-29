export function evaluate(env, expr) {

  // todo: may need something like:
  if (typeof expr === 'number') {
    return expr;
  }
  if (typeof expr === 'string') {
    if(expr === '#t' || expr === '#f') {
      return expr;
    }
    return env.lookup(expr);
  }
  return funApplication(env, expr);
}

function funApplication(env, listExpr) {
  if(isSpecialForm(listExpr)) {
    let specialFn = evaluate(env, listExpr[0]);
    return specialFn(env, listExpr);
  }

  return generalApplication(env, listExpr);
}

function isSpecialForm(listExpr) {
  let node = listExpr[0];
  if(specialForms[node] !== undefined) {
    return true;
  }
  return false;
}

function generalApplication(env, listExpr) {

  let fun = evaluate(env, listExpr[0]);
  let args = listExpr.slice(1).map(n => evaluate(env, n));

  return fun(args);
}

function addBindings(env, exprs) {
  return exprs.reduce((a, b) => a.addBinding(b[0], evaluate(a, b[1])),
                      env);
}

export var specialForms = {
  'if': (env, expr) => {
    let conditional = evaluate(env, expr[1]);
    // todo: only a value of #t is truthy, change this so that
    // any non-zero, non-falsy value is truthy
    if(conditional === '#t') {
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
    env.addBinding(name, evaluate(env, val));
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
    // (lambda (x y z) (+ x y z))
    return function(args) {
      let newEnv = env.newScope();

      let binds = expr[1];
      binds.forEach((b, i) => newEnv.addBinding(b, args[i]));
      return evaluate(newEnv, expr[2]);
    };
  }
}

export var requiredFunctions = {
  '+': (args) => {
    return args.reduce((a, b) => a + b, 0);
  },
  '*': (args) => {
    return args.reduce((a, b) => a * b, 1);
  },
  '-': (args) => {
    if(args.length === 1) {
      return -args[0];
    }
    return args.slice(1).reduce((a, b) => a - b, args[0]);
  },
  '/': (args) => {
    return args.slice(1).reduce((a, b) => a / b, args[0]);
  },
  '=': (args) => {
    let first = args[0];
    let res = args.slice(1).every(a => a === first);
    return res ? '#t' : '#f';
  },
  '<': (args) => {
    let prev = args[0];
    for(let i = 1;i<args.length;i++) {
      let current = args[i];
      if(!(current < prev)) {
        return '#f'
      }
      prev = current;
    }
    return '#t';
  },
  '>': (args) => {
    let prev = args[0];
    for(let i = 1;i<args.length;i++) {
      let current = args[i];
      if(!(current > prev)) {
        return '#f';
      }
      prev = current;
    }
    return '#t';
  }
}
