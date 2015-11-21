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

import NodeType from './NodeType';

// the values stored in genotypes will in simplifiedAst form
// in most cases these will be simple values, but there might
// occasionally be nested expressions and parameter objects
//
function unparseSimplifiedAst(value) {
  if(Array.isArray(value)) {
    if(value.length === 2 &&
       value[0] === 'quote' &&
       !Array.isArray(value[1])) {
      // a string disguised as a quoteed expression
      // e.g. the form "hello" is represented as (quote hello)
      // this is a hack used by the interpreter
      return '"' + value[1] + '"';
    }

    let elements = value.map(unparseSimplifiedAst).join(' ').trim();
    return '(' + elements + ')';

  } else if(typeof(value) === 'object') {

    let args = '';
    for (let k in value) {
      args = args + k + ': ' + unparseSimplifiedAst(value[k]) + ' ';
    }
    return args.trim();

  }
  return value;
};

// does the node contain a 'map' name node
function containsMapNode(ast) {
  return ast.some(n => n.type === NodeType.NAME && n.value === 'map');
}

function formatNodeValue(value, node) {
  let res;
  switch(node.type) {
  case NodeType.STRING:
    res = '"' + value + '"';
    break;
  case NodeType.BOOLEAN:
    res = value === '#t' ? 'true' : 'false';
    break;
  case NodeType.LABEL:
    res = value + ':';
    break;
  default:
    res = value;
  };
  return res;
}

const Unparser = {

  // converts a frontAST back into a string
  // ast is an array of nodes
  unparse: function(frontAst, genotype) {
    let genoIndex = 0;

    // warning: thie function mutates genoIndex
    function pullValueFromGenotype() {
      let value = genotype.get(genoIndex++).get('value');

      return unparseSimplifiedAst(value);
    }

    // have a form like:
    // (define foo
    // [(list 11 12 13 14 15 16) map (select from: (list 1 2 3 4 5 6 7 8 9))])
    // and we're in the alterable list part
    function getMultipleValuesFromGenotype(node) {
      // go through the children: 'list 11 12 13 14 15 16'
      // ignoring the initial list name (is too specific a check?) and
      // any whitespace
      let res = node.children.map(n => {
        if(n.type === NodeType.NAME && n.value === 'list') {
          return formatNodeValue(n.value, n);
        } else if(n.type === NodeType.COMMENT ||
                  n.type === NodeType.WHITESPACE) {
          return formatNodeValue(n.value, n);
        } else {
          return formatNodeValue(pullValueFromGenotype(), n);
        }
      }).join('');

      return '(' + res + ')';
    }

    function unparseASTNode(node) {
      let term = '';

      if(node.alterable) {
        // use value from genotype
        let v;

        if (node.type === NodeType.LIST && containsMapNode(node.parameterAST)) {
          v = getMultipleValuesFromGenotype(node);
        } else {
          v = formatNodeValue(pullValueFromGenotype(), node);
        }
        // prefixes are any comments/whitespaces after the opening bracket
        let prefixes = node.parameterPrefix.map(unparseASTNode).join('');
        let alterParams = node.parameterAST.map(unparseASTNode).join('');

        term = '[' + prefixes + v + alterParams + ']';

      } else {
        let nval;
        if(node.type === NodeType.LIST) {
          nval = '(' + node.children.map(unparseASTNode).join('') + ')';
        } else {
          nval = node.value;
        }
        term = formatNodeValue(nval, node);
      }

      return term;
    }

    return frontAst.map(unparseASTNode).join('');
  }
};

export default Unparser;
