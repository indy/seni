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
import NodeList from './NodeList';
import NodeVector from './NodeVector';

function compile(node, genotype) {
  // genotype !== null because we might call compileNodes with a
  // null argument for genotypes e.g. Genetic::buildTraitFromNode
  if (node.alterable && genotype !== null) {
    // todo: assert that there's another genotype value available
    return [genotype.first(), genotype.shift()];
  }

  if (node.type === NodeType.LIST) {
    if (usingNamedParameters(node.children)) {
      return compileFormUsingNamedParameters(node.children, genotype);
    } else {
      return compileNodes(node.children, genotype);
    }
  }

  if (node.type === NodeType.VECTOR) {
    let res;
    [res, genotype] = compileNodes(node.children, genotype);
    return [['quote', res], genotype];
  }

  if (node.type === NodeType.STRING) {
    // without this the following forms will compile to the same thing:
    //     (foo 'hello')
    //     (foo hello)
    //
    // we need to wrap the string form in a quote-like to prevent the
    // interpreter from trying to lookup the contents of the string
    return [['__string', node.value], genotype];
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

function suitableForBackAst(node) {
  return node.type !== NodeType.WHITESPACE && node.type !== NodeType.COMMENT;
}

// backAst
function compileForBackAst(nodes) {
  return nodes.reduce((nodeArray, n) => {
    if(suitableForBackAst(n)) {
      let newNode;
      if (n.type === NodeType.LIST) {
        newNode = new NodeList();
        newNode.usingAbbreviation = n.usingAbbreviation;
      } else if (n.type === NodeType.VECTOR) {
        newNode = new NodeVector();
      } else {
        newNode = new Node(n.type, n.value);
      };

      newNode.alterable = n.alterable;

      if(n.alterable) {
        newNode.parameterAST = compileForBackAst(n.parameterAST);
      }

      if(n.type === NodeType.LIST || n.type === NodeType.VECTOR) {
        newNode.children = compileForBackAst(n.children);
      };

      nodeArray.push(newNode);
    }
    return nodeArray;
  }, []);
}

function expandNodeForAlterableChildren(nodes) {
  return nodes.map(node => {
    if(node.type === NodeType.LIST) {
      if(node.alterable === true &&
         node.parameterAST.length > 1 &&
         node.parameterAST[0].value === 'map') {

        // make this node non-alterable and it's children alterable
        // e.g. {(list 1 2 3 4 5 6) map (select from: (list 1 2 3 4 5 6 7 8 9))}

        node.alterable = false;
        let parameterAst = node.parameterAST.slice(1); // remove the 'map''

        for(let i = 1; i < node.children.length; i++) {
          let n = node.children[i];
          n.alterable = true;
          n.parameterAST = parameterAst;
        }
      } else {
        node.children = expandNodeForAlterableChildren(node.children);
      }
    } else if(node.type === NodeType.VECTOR) {
      if(node.alterable === true) {
        // a big difference between lists and vectors is that the parameterAst
        // in an alterable statement for a vector applies to each element of
        // the vector, whereas for a list it applies to the list as a whole
        //
        // i.e. {[1 1 2 2 3 3] (select from: [1 2 3])}
        node.alterable = false;
        node.children.forEach(n => {
          n.alterable = true;
          n.parameterAST = node.parameterAST;
        });
      } else {
        node.children = expandNodeForAlterableChildren(node.children);
      }
    }

    return node;
  });
}

// frontAst -> backAst -> simplifiedAst

// frontAst: has whitespace, comment nodes
// backAst: removes whitespace and comment nodes, expands the map keyword
// in alterable nodes
// simplifiedAst: a json-like sexp used by the interpreter
//
const Compiler = {

  // transform a front end ast into a backAst
  // NOTE: we currently need to assume that the alterable nodes
  // stay in the same order
  compileBackAst: function(frontAst) {
    let backAst = compileForBackAst(frontAst);

    backAst = expandNodeForAlterableChildren(backAst);

    return backAst;
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
