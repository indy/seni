/*
    Seni
    Copyright (C) 2015  Inderjit Gill <email@indy.io>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

/* eslint-disable no-use-before-define */

import NodeType from './NodeType';

function compile(node, genotype) {

  if (node.alterable) {
    // todo: assert that there's another genotype value available
    return [genotype.first().get('value'), genotype.shift()];
  }

  if (node.type === NodeType.LIST) {
    return compileList(node, genotype);
  }

  if (node.type === NodeType.STRING) {
    // without this the following forms will compile to the same thing:
    //     (foo 'hello')
    //     (foo hello)
    //
    // we need to wrap the string form in a quote to prevent the interpreter
    // from trying to lookup the contents of the string
    return [['quote', node.value], genotype];
  }

  return [node.value, genotype];
}

function compileNodes(nodes, genotype) {

  let n, res = nodes.map((node) => {
    [n, genotype] = compile(node, genotype);
    return n;
  });

  return [res, genotype];
}

function compileList(node, genotype) {
  const children = node.children;

  if (usingNamedParameters(children)) {
    return compileFormUsingNamedParameters(children, genotype);
  } else {
    return compileNodes(children, genotype);
  }
}

function compileFormUsingNamedParameters(children, genotype) {
  // this is a form that has the pattern (foo arg1: 3 arg2: 5)
  // combine the labels + arguments into an object

  if (!(children.length & 1)) {
    let msg = 'error: odd number of nodes expected: ';
    msg += ' function name + pairs of label,arg';
    console.log(msg);
  }

  const args = {};

  for (let i = 1; i < children.length; i += 2) {
    const label = children[i];
    if (label.type !== NodeType.LABEL) {
      console.log('error: expecting a label, actual: ' + label.value);
    }
    let [arg, g] = compile(children[i + 1], genotype);
    args[label.value] = arg;
    genotype = g;
  }

  let [n, g] = compile(children[0], genotype);

  return [[n, args], g];
}

function usingNamedParameters(children) {
  // note: this will return false for functions where 0 arguments are given
  if (children.length > 1) {
    return children[1].type === NodeType.LABEL;
  }
  return false;
}

const Compiler = {

  compile: function(ast, genotype) {

    let [forms, _] = compileNodes(ast, genotype);
    _ = _;

    return {
      forms: forms
    };
  }
};

export default Compiler;
