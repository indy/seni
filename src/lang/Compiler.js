import NodeType from './NodeType';

// recursive code so switch off the jslint warnings
// about functions being used before they're defined
// also allow bitwise operations
//
/*jslint latedef:false, bitwise:true*/

function _compile(node, genes) {
  if(node.type === NodeType.LIST) {
    return _compileList(node, genes);
  }

  if(node.alterable === true) {
    // expect a form in the parameterAST
    let ast;
    if(node.parameterAST.length) {
      // todo: currently assuming that there's only one form
      // inside square brackets, will probably have to
      // change this assumption in the future
      ast = _compile(node.parameterAST[0]);
    } else {
      // this is to allow code like (+ 2 [2])
      // which should behave as if there were no square brackets
      // todo: implement identity in this context
      ast = ['identity', {value: node.value}];
    }

    let gene = {initialValue: node.value,
                ast: ast,
                gensym: '__GENSYM__' + genes.length + '__'};
    genes.push(gene);  // mutate the genes
    return gene.gensym;
  }

  if(node.type === NodeType.STRING) {
    // without this the following forms will compile to the same thing:
    //     (foo 'hello')
    //     (foo hello)
    //
    // we need to wrap the string form in a quote to prevent the interpreter
    // from trying to lookup the contents of the string
    return ['quote', node.value];
  }

  return node.value;
}

function _compileList(node, genes) {
  const children = node.children;

  if(usingNamedParameters(children)) {
    return compileFormUsingNamedParameters(children, genes);
  } else if(allNamedParameters(children)) {
    return compileAllNamedParameters(children, genes);
  } else {
    return children.map((child) => _compile(child, genes));
  }
}

function usingNamedParameters(children) {
  // note: this will return false for functions where 0 arguments are given
  if(children.length > 1) {
    return children[1].type === NodeType.LABEL;
  }
  return false;
}

function allNamedParameters(children) {
  // a basic test, but if it passes this it should be all <label,value> pairs
  if(children.length > 0) {
    return children[0].type === NodeType.LABEL;
  }
  return false;
}

function compileFormUsingNamedParameters(children, genes) {
  // this is a form that has the pattern (foo arg1: 3 arg2: 5)
  // combine the labels + arguments into an object

  if(!(children.length & 1)) {
    console.log('error: odd number of nodes expected: function name + pairs of label,arg');
  }

  let args = {};
  for(let i=1; i < children.length; i+=2) {
    const label = children[i];
    if(label.type !== NodeType.LABEL) {
      console.log('error: expecting a label, actual: ' + label.value);
    }
    let arg = _compile(children[i+1], genes);
    args[label.value] = arg;
  }

  return [_compile(children[0], genes), args];
}

// todo: is this ever used anymore?
function compileAllNamedParameters(children, genes) {
  // can assume this is of the form (arg1: 3 arg2: 5)
  // combine the labels + arguments into an object

  if(children.length & 1) {
    console.log('error: even number of nodes expected: pairs of label,arg');
  }

  let args = {};
  for(let i=0; i < children.length; i+=2) {
    const label = children[i];
    if(label.type !== NodeType.LABEL) {
      console.log('error: expecting a label, actual: ' + label.value);
    }
    let arg = _compile(children[i+1], genes);
    args[label.value] = arg;
  }

  return args;
}

var Compiler = {
  compile: function(ast) {

    // genes will be mutated
    let genes = [];
    let forms = ast.map((node) => _compile(node, genes));

    return {forms: forms,
            genes: genes};
  }
};

export default Compiler;
