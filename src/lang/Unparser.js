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

  } else if(value instanceof Object) {

    let args = '';
    for (let k in value) {
      args = args + k + ': ' + unparseSimplifiedAst(value[k]) + ' ';
    }
    return args.trim();

  } else if(!Number.isNaN(Number(value))) {
    // see if the number is a float, if so then format to 3dp
    let asString3dp = value.toFixed(3);
    return (asString3dp.match(/[.]000$/)) ? value : asString3dp;
  }
  return value;
}

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

// warning: thie function mutates genoIndex
function pullValueFromGenotype(genotype) {
  // let value = genotype.get(genoIndex++).get('value');
  let value = genotype.first().get('value');
  return [unparseSimplifiedAst(value), genotype.shift()];
}

// have a form like:
// (define foo
// [(list 11 12 13 14 15 16) map (select from: (list 1 2 3 4 5 6 7 8 9))])
// and we're in the alterable list part
function getMultipleValuesFromGenotype(listNode, genotype) {
  // go through the children: 'list 11 12 13 14 15 16'
  // ignoring the initial list name (is too specific a check?) and
  // any whitespace
  let v;
  let lst, listPrefix, listPostfix;

  if(listNode.usingAbbreviation) {
    listPrefix = '\'';
    listPostfix = '';
    // remove the 'quote' and whitespace nodes
    lst = listNode.children.slice(2);
  } else {
    listPrefix = '(';
    listPostfix = ')';
    lst = listNode.children;
  }

  let res = lst.map(n => {
    if(n.type === NodeType.NAME && n.value === 'list') {
      return formatNodeValue(n.value, n);
    } else if(n.type === NodeType.COMMENT ||
              n.type === NodeType.WHITESPACE) {
      return formatNodeValue(n.value, n);
    } else {
      [v, genotype] = pullValueFromGenotype(genotype);
      return formatNodeValue(v, n);
    }
  }).join('');

  return [listPrefix + res + listPostfix, genotype];
}

function unparseUnalterable(unalterableNode) {
  let v, _;
  return unalterableNode.map(n => {
    [v, _] = unparseASTNode(n, null);
    _ = _;
    return v;
  }).join('');
}

function unparseASTNode(node, genotype) {
  let term = '';
  let v;

  if(node.alterable) {
    // prefixes are any comments/whitespaces after the opening bracket

    // Note: neither of these statements should consume any of the
    // genotype
    let prefixes = unparseUnalterable(node.parameterPrefix);
    let alterParams = unparseUnalterable(node.parameterAST);

    // use value from genotype
    if (node.type === NodeType.LIST && containsMapNode(node.parameterAST)) {
      [v, genotype] = getMultipleValuesFromGenotype(node, genotype);
    } else {
      [v, genotype] = pullValueFromGenotype(genotype);
      v = formatNodeValue(v, node);
    }

    term = '[' + prefixes + v + alterParams + ']';

  } else {
    let nval;
    if(node.type === NodeType.LIST) {

      let lst, listPrefix, listPostfix;
      if(node.usingAbbreviation) {
        listPrefix = '\'';
        listPostfix = '';
        // remove the 'quote' and whitespace nodes
        lst = node.children.slice(2);
      } else {
        listPrefix = '(';
        listPostfix = ')';
        lst = node.children;
      }

      v = listPrefix + lst.map(n => {
        [nval, genotype] = unparseASTNode(n, genotype);
        return nval;
      }).join('') + listPostfix;
    } else {
      v = node.value;
    }

    term = formatNodeValue(v, node);
  }

  return [term, genotype];
}

const Unparser = {

  // converts a frontAST back into a string
  // ast is an array of nodes
  unparse: function(frontAst, genotype) {

    let term;
    let terms = frontAst.map(n => {
      [term, genotype] = unparseASTNode(n, genotype);
      return term;
    });

    return terms.join('');
  }
};

export default Unparser;
