import {
  Node,
  NodeType
} from './node';

export function evaluate(env, expr) {
/*
  // todo: may need something like:
  if (typeof expr === 'number') {
    return expr;
  }
  if (typeof expr === 'string') {
    return env.lookup(expr);
  }
*/
  
  switch(expr.getType()) {
  case NodeType.INT:
    return evalMutableNode(env, expr);
    break;
  case NodeType.FLOAT:
    return evalMutableNode(env, expr);
    break;
  case NodeType.BOOLEAN:
    return evalMutableNode(env, expr);
    break;
  case NodeType.STRING:
    return evalMutableNode(env, expr);
    break;
  case NodeType.COLOUR:
    return expr;
    break;
  case NodeType.NULL:
    return expr;
    break;
  case NodeType.NAME:
    let nodeName = evalMutableNode(env, expr);
    return env.lookup(nodeName);
    break;
  case NodeType.LIST:
    return funApplication(env, expr);
    break;
  }
  return null;
}

function funApplication(env, listExpr) {
  if(isSpecialForm(listExpr)) {
    let specialFn = evaluate(env, listExpr.getChild(0));
    return specialFn(env, listExpr);
  }

  return generalApplication(env, listExpr);
}

function isSpecialForm(listExpr) {
  let node = listExpr.getChild(0);
  if(node.getType === NodeType.LIST) {
    return false;
  }
  if(specialForms[node.getValue()] !== undefined) {
    return true;
  }
  return false;
}

function generalApplication(env, listExpr) {

  let children = listExpr.getChildren();

  let fun = evaluate(env, children[0]);
  let args = children.slice(1).map(n => evaluate(env, n));

  return fun(args);
}

function evalMutableNode(env, expr) {
  let node = expr;
  if(expr.isAlterable()) {
    // todo: assuming that the env.lookup will return a node?
    // might just be better to store the actual value
    node = env.lookup(expr.getGenSym());
  }
  return node.getValue();
}

function addBindings(env, exprs) {
  let children = exprs.getChildren();
  return children.reduce((a, b) => a.addBinding(b.getChild(0).getValue(),
                                                evaluate(a, b.getChild(1))),
                         env);
}

export var specialForms = {
  'if': (env, expr) => {
    let conditional = evaluate(env, expr.getChild(1));
    // todo: only a value of #t is truthy, change this so that
    // any non-zero, non-falsy value is truthy
    if(conditional === '#t') {
      return evaluate(env, expr.getChild(2));
    } else {
      if(expr.getChildren().length == 4) {
        return evaluate(env, expr.getChild(3));
      }
    }
  },
  'quote': (env, expr) => {
    return expr.getChild(1);       
  },
  'define': (env, expr) => {
    // (define foo 12)
    let name = expr.getChild(1);
    if(name.getType() !== NodeType.NAME) {
      // something weird has happened
    }
    let val = expr.getChild(2);
    env.addBinding(name.getValue(),evaluate(env, val));
  },
  'set!': (env, expr) => {
    // (set! foo 42)
    let name = expr.getChild(1);
    let val = expr.getChild(2);
    env.mutate(name.getValue(), evaluate(env, val));
  },
  'begin': (env, expr) => {
    // (begin (f1 1) (f2 3) (f3 5))
    let children = expr.getChildren();
    return children.slice(1).reduce((a, b) => evaluate(env, b), null);
  },
  'let': (env, expr) => {
    // (let ((a 12) (b 24)) (+ a b foo))
    return evaluate(addBindings(env.newScope(), expr.getChild(1)),
                    expr.getChild(2));
  },
  'lambda': (env, expr) => {
    // (lambda (x y z) (+ x y z))
    return function(args) {
      let newEnv = env.newScope();

      let binds = expr.getChild(1).getChildren();
      binds.forEach((b, i) => newEnv.addBinding(b.getValue(),
                                                args[i]));
      return evaluate(newEnv, expr.getChild(2));
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
