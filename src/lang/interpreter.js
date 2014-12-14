import {
    Node,
    NodeType
} from '../../src/lang/node';

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

export function bindRequiredFunctions(env) {
    return binder(env, requiredFunctions);
}

export function bindSpecialForms(env) {
    return binder(env, specialForm);
}

function binder(env, obj) {
    for(let key in obj) {
        env.addBinding(key, obj[key])
    }
    return env;
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
    if(specialForm[node.getValue()] !== undefined) {
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

let specialForm = {
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
    }
}

let requiredFunctions = {
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
