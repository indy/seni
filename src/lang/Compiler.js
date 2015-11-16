/*
 *  Seni
 *  Copyright (C) 2015 Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

/* eslint-disable no-use-before-define */

import NodeType from './NodeType';
import Node from './Node';

function compile(node, genotype) {

  if (node.alterable) {
    // todo: assert that there's another genotype value available
    return [genotype.first().get('value'), genotype.shift()];
  }

  if (node.type === NodeType.LIST) {
    return compileList(node.children, genotype);
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

  let n;
  let res = nodes.map(node => {
    [n, genotype] = compile(node, genotype);
    return n;
  });

  return [res, genotype];
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

  let fnName;
  [fnName, genotype] = compile(children[0], genotype);

  for (let i = 1; i < children.length; i += 2) {
    const label = children[i];
    if (label.type !== NodeType.LABEL) {
      console.log('error: expecting a label, actual: ' + label.value);
    }
    let arg;
    [arg, genotype] = compile(children[i + 1], genotype);
    args[label.value] = arg;
  }

  return [[fnName, args], genotype];
}

function usingNamedParameters(children) {
  // note: this will return false for functions where 0 arguments are given
  if (children.length > 1) {
    return children[1].type === NodeType.LABEL;
  }
  return false;
}

function compileList(children, genotype) {
  if (usingNamedParameters(children)) {
    return compileFormUsingNamedParameters(children, genotype);
  } else {
    return compileNodes(children, genotype);
  }
}

function suitableForBackendAst(node) {
  return node.type !== NodeType.WHITESPACE && node.type !== NodeType.COMMENT;
}

// backendAst
function compileForBackendAst(nodes) {
  return nodes.reduce((nodeArray, n) => {
    if(suitableForBackendAst(n)) {
      let newNode = new Node(n.type, n.value);
      newNode.alterable = n.alterable;

      if(n.alterable) {
        newNode.parameterAST = compileForBackendAst(n.parameterAST);
      }

      if(n.type === NodeType.LIST) {
        newNode.children = compileForBackendAst(n.children);
      };

      nodeArray.push(newNode);
    }
    return nodeArray;
  }, []);
}

const Compiler = {

  // used by genetic when an alterable node contains a list
  //
  compileListInAlterable: function(children) {
    // don't pass a genotype since we're already going to be inside an
    // alterable node and nested alterables aren't supported
    //
    let nullGenotype = null;
    let [simplifiedAst, _] = compileList(children, nullGenotype);
    _ = _;

    return simplifiedAst;
  },

  compileInAlterable: function(nodes) {
    // don't pass a genotype since we're already going to be inside an
    // alterable node and nested alterables aren't supported
    //
    let nullGenotype = null;
    let [simplifiedAsts, _] = compileNodes(nodes, nullGenotype);
    _ = _;

    // return an array of simplified ast objects
    return simplifiedAsts;
  },

  // transform a front end ast into a backend
  // NOTE: we currently need to assume that the alterable nodes
  // stay in the same order
  compileBackEndAst: function(frontendAst) {
    return compileForBackendAst(frontendAst);
  },

  // the nodes should be from the back-end ast
  compileWithGenotype: function(nodes, genotype) {
    let [simplifiedAsts, _] = compileNodes(nodes, genotype);
    _ = _;

    // return an array of simplified ast objects
    return simplifiedAsts;
  }
};

export default Compiler;
