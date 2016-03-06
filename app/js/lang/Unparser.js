/*
 *  Seni
 *  Copyright (C) 2016 Inderjit Gill <email@indy.io>
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

import Immutable from 'immutable';

import NodeType from './NodeType';

const logToConsole = false;

// the values stored in genotypes will in simplifiedAst form
// in most cases these will be simple values, but there might
// occasionally be nested expressions and parameter objects
//
function unparseSimplifiedAst(value) {
  if (Array.isArray(value)) {
    if (value.length === 2 && value[0] === `__string`) {
      // the form "hello" is represented as (__string hello)
      // this is a hack used by the interpreter
      return `"${value[1]}"`;
    } else if (value.length > 0 && value[0] === `vector`) {
      // hack used to correctly unparse forms like '[1 2]', without this
      // the output would be '(list 1 2)' (see Genetic::buildTraitFromNode)
      const e = value.slice(1).map(unparseSimplifiedAst).join(` `).trim();
      return `[${e}]`;
    }
    const elements = value.map(unparseSimplifiedAst).join(` `).trim();
    return `(${elements})`;
  } else if (value instanceof Object) {
    let args = ``;
    for (const k in value) {
      args = `${args}${k}: ${unparseSimplifiedAst(value[k])} `;
    }
    return args.trim();
  } else if (!Number.isNaN(Number(value))) {
    // see if the number is a float, if so then format to 3dp
    const asString3dp = value.toFixed(3);
    return (asString3dp.match(/[.]000$/)) ? value : asString3dp;
  }
  return value;
}

function formatNodeValue(value, node) {
  let res;
  switch (node.type) {
  case NodeType.STRING:
    res = `"${value}"`;
    break;
  case NodeType.BOOLEAN:
    res = value === `#t` ? `true` : `false`;
    break;
  case NodeType.LABEL:
    res = `${value}:`;
    break;
  default:
    res = value;
  }
  return res;
}

function pullValueFromGenotype(genotype) {
  let geno = genotype.first();
  if (Immutable.Iterable.isIterable(geno)) {
    geno = geno.toJS();
  }
  return [unparseSimplifiedAst(geno), genotype.shift()];
}

function getMultipleValuesFromGenotype(nodes, genotype) {
  const listPrefix = `[`;
  const listPostfix = `]`;

  let v = undefined;
  let geno = genotype;

  const res = nodes.map(n => {
    if (n.type === NodeType.NAME && n.value === `vector`) {
      return formatNodeValue(n.value, n);
    } else if (n.type === NodeType.COMMENT ||
               n.type === NodeType.WHITESPACE) {
      return formatNodeValue(n.value, n);
    } else {
      [v, geno] = pullValueFromGenotype(geno);
      return formatNodeValue(v, n);
    }
  }).join(``);

  return [listPrefix + res + listPostfix, geno];
}

function unparseUnalterable(unalterableNode) {
  return unalterableNode.map(n => {
    const [v, _] = unparseASTNode(n, null);
    return v;
  }).join(``);
}

function unparseASTNode(node, genotype) {
  let term = ``;
  let v;
  let geno = genotype;
  let lst, listPrefix, listPostfix;

  if (node.alterable) {
    // prefixes are any comments/whitespaces after the opening bracket

    // Note: neither of these statements should consume any of the
    // genotype
    const prefixes = unparseUnalterable(node.parameterPrefix);
    const alterParams = unparseUnalterable(node.parameterAST);

    // use value from genotype
    if (node.type === NodeType.VECTOR) {
      // a vector requires multiple values from the genotype
      [v, geno] = getMultipleValuesFromGenotype(node.children, geno);
    } else {
      [v, geno] = pullValueFromGenotype(geno);
      v = formatNodeValue(v, node);
    }

    term = `{${prefixes}${v}${alterParams}}`;
  } else {
    let nval;
    if (node.type === NodeType.LIST) {
      if (node.usingAbbreviation) {
        listPrefix = `\'`;
        listPostfix = ``;
        // remove the 'quote' and whitespace nodes
        lst = node.children.slice(2);
      } else {
        listPrefix = `(`;
        listPostfix = `)`;
        lst = node.children;
      }

      v = listPrefix + lst.map(n => {
        [nval, geno] = unparseASTNode(n, geno);
        return nval;
      }).join(``) + listPostfix;
    } else if (node.type === NodeType.VECTOR) {
      listPrefix = `[`;
      listPostfix = `]`;
      lst = node.children;

      v = listPrefix + lst.map(n => {
        [nval, geno] = unparseASTNode(n, geno);
        return nval;
      }).join(``) + listPostfix;
    } else {
      v = node.value;
    }

    term = formatNodeValue(v, node);
  }

  return [term, geno];
}

const Unparser = {

  // converts a frontAST back into a string
  // ast is an array of nodes
  unparse: (frontAst, genotype) => {
    let term = undefined;
    let geno = genotype;
    const terms = frontAst.map(n => {
      [term, geno] = unparseASTNode(n, geno);
      return term;
    });
    const res = terms.join(``);

    if (logToConsole) {
      console.log(`Unparser::unparse`, frontAst, res);
    }

    return res;
  }
};

export default Unparser;
