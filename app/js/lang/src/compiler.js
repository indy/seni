import {
  Node,
  NodeType
} from './node';

// the world's simplest compiler, just converts Node objects into a json-like structure


export function compile(ast) {
  return ast.map((node) => _compile(node));
}

function _compile(node) {
  if(node.getType() === NodeType.LIST) {
    return _compileList(node);
  } else if(node.getType() === NodeType.STRING) {
    // without this the following forms will compile to the same thing:
    //     (foo "hello")
    //     (foo hello)
    //
    // we need to wrap the string form in a quote to prevent the interpreter
    // from trying to lookup the contents of the string
    return ["quote", node.getValue()];
  } else {
    return node.getValue();
  }
}

function _compileList(node) {
  let children = node.getChildren();

  if(isFunctionInvocation(children)) {
    return compileFunctionInvocation(children);
  } else if(isFunctionParameterDeclaration(children)) {
    return compileFunctionParameterDeclaration(children);
  } else {
    return children.map((child) => _compile(child));
  }
}

function isFunctionInvocation(children) {
  // note: this will return false for functions where 0 arguments are given
  return children.length > 1 && children[1].getType() === NodeType.LABEL;
}

function isFunctionParameterDeclaration(children) {
  return children[0].getType() === NodeType.LABEL;
}
  
function compileFunctionInvocation(children) {
  // can assume this is of the form (foo arg1: 3 arg2: 5)
  // combine the labels + arguments into an object

  if(!(children.length & 1)) {
    console.log("error: odd number of nodes expected: function name + pairs of label,arg");
  }

  let args = {};
  for(let i=1; i < children.length; i+=2) {
    let label = children[i];
    if(label.getType() != NodeType.LABEL) {
      console.log("error: expecting a label, actual: " + label.getValue());
    }
    let arg = _compile(children[i+1]);
    args[label.getValue()] = arg;
  }

  return [_compile(children[0]), args];
}

function compileFunctionParameterDeclaration(children) {
  // can assume this is of the form (arg1: 3 arg2: 5)
  // combine the labels + arguments into an object

  if(children.length & 1) {
    console.log("error: even number of nodes expected: pairs of label,arg");
  }

  let args = {};
  for(let i=0; i < children.length; i+=2) {
    let label = children[i];
    if(label.getType() != NodeType.LABEL) {
      console.log("error: expecting a label, actual: " + label.getValue());
    }
    let arg = _compile(children[i+1]);
    args[label.getValue()] = arg;
  }

  return args;
}
