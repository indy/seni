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
    for(let key in requiredFunctions) {
        env.addBinding(key, requiredFunctions[key]);
    }
    return env;
}

export function bindSpecialForms(env) {
    for(let key in specialForm) {
        env.addBinding(key, specialForm[key])
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
        let res = args.reduce(function(a, b) {return a + b.getValue();}, 0);
        return new Node(NodeType.FLOAT, res, false);        
    },
    '*': function(args) {
        let res = args.reduce(function(a, b) {return a * b.getValue();}, 1);
        return new Node(NodeType.FLOAT, res, false);
    }    
}
