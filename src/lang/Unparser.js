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

const Unparser = {

  // converts a frontAST back into a string
  // ast is an array of nodes
  unparse: function(frontAst, genotype) {
    let genoIndex = 0;

    // warning: thie function mutates genoIndex
    function pullValueFromGenotype() {
      let value = genotype.get(genoIndex++).get('value');

      if(Array.isArray(value) &&
         value.length === 2 &&
         typeof(value[1]) === 'object') {
        // probably a form with named parameters
        // this looks like [fn name, fn args]
        // e.g. ['col/rgb', {r: 0 g: 0 b: 0 alpha: 1}]
        let args = value[1];
        let argsUnparse = '';
        for (let k in args) {
          argsUnparse = argsUnparse + k + ': ' + args[k] + ' ';
        }
        return '(' + value[0] + ' ' + argsUnparse.trim() + ')';
      }
      return value;
    }

    // does the node contain a 'map' name node
    function containsMapNode(ast) {
      return ast.some(n => n.type === NodeType.NAME && n.value === 'map');
    }

    // have a form like:
    // (define foo
    // [(list 11 12 13 14 15 16) map (select from: (list 1 2 3 4 5 6 7 8 9))])
    // and we're in the alterable list part
    function getMultipleValuesFromGenotype(node) {
      // go through the children: 'list 11 12 13 14 15 16'
      // ignoring the initial list name (is too specific a check?) and
      // any whitespace
      let res = '';
      node.children.forEach(n => {
        if(n.type === NodeType.NAME && n.value === 'list') {
          res += n.value;
        } else if(n.type === NodeType.COMMENT ||
                  n.type === NodeType.WHITESPACE) {
          res += n.value;
        } else {
          res += pullValueFromGenotype();
        }
      });

      return '(' + res + ')';
    }

    function add(term, str, node) {
      if(node.alterable) {
        // prefixes are any comments/whitespaces after the opening bracket
        let prefixes = node.parameterPrefix.reduce(unparseASTNode, '');
        let alterParams = node.parameterAST.reduce(unparseASTNode, '');
        let v;

        if (node.type === NodeType.LIST && containsMapNode(node.parameterAST)) {
          v = getMultipleValuesFromGenotype(node);
        } else {
          // don't use the term, replace with value from genotype
          //let value = genotype.get(genoIndex++).get('value');
          //v = formatValue(value);
          v = pullValueFromGenotype();
        }

        return str + '[' + prefixes + v + alterParams + ']';
      } else {
        return str + term;
      }
    }

    function unparseASTNode(str, node) {
      let res;
      if (node.type === NodeType.LIST) {
        // todo: mark the list node created by Parser.consumeQuotedForm
        // so that we can recreate the original 'FORM instead of the
        // current (quote FORM)

        if(node.alterable) {
          res = add('', str, node); // add will construct the correct term
        } else {
          let lst = node.children.reduce(unparseASTNode, '');
          res = add('(' + lst + ')', str, node);
        }
      } else if (node.type === NodeType.STRING) {
        res = add('"' + node.value + '"', str, node);
      } else if (node.type === NodeType.BOOLEAN) {
        res = add(node.value === '#t' ? 'true' : 'false', str, node);
      } else if (node.type === NodeType.LABEL) {
        res = add(node.value + ':', str, node);
      } else {
        res = add(node.value, str, node);
      }

      return res;
    }

    return frontAst.reduce(unparseASTNode, '');
  }
};

export default Unparser;
