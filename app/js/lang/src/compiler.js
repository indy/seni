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
    return node.getChildren().map((child) => _compile(child));
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
