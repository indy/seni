import {
    Node,
    NodeType
} from './node';

export function evaluate(env, expr) {
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
        return env.lookup(nodeName.getValue());
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
    if(expr.isAlterable()) {
        return env.lookup(expr.getGenSym());
    } else {
        return expr;
    }
}

function addBindings(env, exprs) {
    let children = exprs.getChildren();
    return children.reduce((a, b) => a.addBinding(b.getChild(0).getValue(),
                                                  evaluate(a, b.getChild(1))),
                           env);
}

export var specialForms = {
    'if': function(env, expr) {
        let conditional = evaluate(env, expr.getChild(1));
        if(conditional.getType() !== NodeType.BOOLEAN) {
            // something weird has happened
        }
        if(conditional.getValue()) {
            return evaluate(env, expr.getChild(2));
        } else {
            if(expr.getChildren().length == 4) {
                return evaluate(env, expr.getChild(3));
            }
        }
    },
    'quote': function(env, expr) {
        return expr.getChild(1);       
    },
    'define': function(env, expr) {
        // (define foo 12)
        let name = expr.getChild(1);
        if(name.getType() !== NodeType.NAME) {
            // something weird has happened
        }
        let val = expr.getChild(2);
        env.addBinding(name.getValue(),evaluate(env, val));
    },
    'set!': function(env, expr) {
        // (set! foo 42)
        let name = expr.getChild(1);
        let val = expr.getChild(2);
        env.mutate(name.getValue(), evaluate(env, val));
    },
    'begin': function(env, expr) {
        // (begin (f1 1) (f2 3) (f3 5))
        let children = expr.getChildren();
        return children.slice(1).reduce((a, b) => evaluate(env, b), null);
    },
    'let': function(env, expr) {
        // (let ((a 12) (b 24)) (+ a b foo))
        return evaluate(addBindings(env.newScope(), expr.getChild(1)),
                        expr.getChild(2));
    },
    'lambda': function(env, expr) {
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
    '+': function(args) {
        let res = args.reduce((a, b) => a + b.getValue(), 0);
        return new Node(NodeType.FLOAT, res, false);        
    },
    '*': function(args) {
        let res = args.reduce((a, b) => a * b.getValue(), 1);
        return new Node(NodeType.FLOAT, res, false);
    },
    '-': function(args) {
        if(args.length === 1) {
            return new Node(NodeType.FLOAT, -args[0].getValue(), false);
        }
        let first = args[0].getValue();
        let res = args.slice(1).reduce((a, b) => a - b.getValue(), first);
        return new Node(NodeType.FLOAT, res, false);
    },
    '/': function(args) {
        let first = args[0].getValue();
        let res = args.slice(1).reduce((a, b) => a / b.getValue(), first);
        return new Node(NodeType.FLOAT, res, false);
    },
    '=': function(args) {
        let first = args[0].getValue();
        let res = args.slice(1).every(a => a.getValue() === first);
        return new Node(NodeType.BOOLEAN, res, false);
    },
    '<': function(args) {
        let prev = args[0].getValue();
        for(let i = 1;i<args.length;i++) {
            let current = args[i].getValue();
            if(!(current < prev)) {
                return new Node(NodeType.BOOLEAN, false, false);
            }
            prev = current;
        }
        return new Node(NodeType.BOOLEAN, true, false);
    },
    '>': function(args) {
        let prev = args[0].getValue();
        for(let i = 1;i<args.length;i++) {
            let current = args[i].getValue();
            if(!(current > prev)) {
                return new Node(NodeType.BOOLEAN, false, false);
            }
            prev = current;
        }
        return new Node(NodeType.BOOLEAN, true, false);
    }
}