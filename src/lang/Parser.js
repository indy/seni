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

import TokenType from './TokenType';
import Node from './Node';
import NodeType from './NodeType';

/*
 these functions will return {node: node, error: error}
 */

function boxNode(nodeType, value) {
  return {node: new Node(nodeType, value)};
}

function consumeItem(tokens) {
  let _ = _;
  const token = tokens[0];
  tokens.shift();            // remove the first token

  const tokenType = token.type;
  if (tokenType === TokenType.LIST_START) {
    return consumeList(tokens);
  } else if (tokenType === TokenType.LIST_END) {
    return {error: 'mismatched closing parens'};
  } else if (tokenType === TokenType.INT) {
    return boxNode(NodeType.INT, token.value);
  } else if (tokenType === TokenType.FLOAT) {
    return boxNode(NodeType.FLOAT, token.value);
  } else if (tokenType === TokenType.NAME) {
    const val = token.value;
    if (val === 'true') {
      return boxNode(NodeType.BOOLEAN, '#t');
    } else if (val === 'false') {
      return boxNode(NodeType.BOOLEAN, '#f');
    } else {
      return boxNode(NodeType.NAME, token.value);
    }
  } else if (tokenType === TokenType.LABEL) {
    return boxNode(NodeType.LABEL, token.value);
  } else if (tokenType === TokenType.STRING) {
    return boxNode(NodeType.STRING, token.value);
  } else if (tokenType === TokenType.QUOTE_ABBREVIATION) {
    return consumeQuotedForm(tokens);
  } else if (tokenType === TokenType.BRACKET_START) {
    return consumeBracketForm(tokens);
  } else if (tokenType === TokenType.BRACKET_END) {
    return {error: 'mismatched closing square brackets'};
  } else if (tokenType === TokenType.COMMENT) {
    return boxNode(NodeType.COMMENT, token.value + '\n');
  } else if (tokenType === TokenType.WHITESPACE) {
    return boxNode(NodeType.WHITESPACE, token.value);
  }

  // e.g. TokenType.UNKNOWN
  return {error: 'unknown token type'};
}

function consumeBracketForm(tokens) {
  let nodeBox, node, nodeType;
  let prefixParameters = [];

  while (true) {
    nodeBox = consumeItem(tokens);
    if (nodeBox.error) {
      return nodeBox;
    }
    node = nodeBox.node;
    nodeType = node.type;

    if(nodeType === NodeType.COMMENT || nodeType === NodeType.WHITESPACE) {
      prefixParameters.push(node);
    } else {
      // we've got the first node within the square brackets that's mutable
      node.alterable = true;
      break;
    }
  }

  prefixParameters.forEach(pp => node.addParameterNodePrefix(pp));

  if (nodeType !== NodeType.BOOLEAN &&
      nodeType !== NodeType.INT &&
      nodeType !== NodeType.FLOAT &&
      nodeType !== NodeType.NAME &&
      nodeType !== NodeType.STRING &&
      nodeType !== NodeType.LIST) {
    console.log('whooops', tokens, node);
    return {error: 'non-mutable node within square brackets ' + nodeType};
  }

  let token, parameterBox, parameter;

  /* eslint-disable no-constant-condition */
  while (true) {
    token = tokens[0];
    if (token === undefined) {
      return {error: 'unexpected end of list'};
    }
    if (token.type === TokenType.BRACKET_END) {
      tokens.shift();
      break;
    }

    parameterBox = consumeItem(tokens);
    if (parameterBox.error) {
      return parameterBox;
    }
    parameter = parameterBox.node;
    if (parameter !== null) {
      node.addParameterNode(parameter);
    }
  }
  /* eslint-enable no-constant-condition */

  return { node };

}

function consumeQuotedForm(tokens) {
  // '(2 3 4) -> (quote (2 3 4))

  const node = new Node(NodeType.LIST);

  node.addChild(new Node(NodeType.NAME, 'quote'));
  node.addChild(new Node(NodeType.WHITESPACE, ' '));
  const childBox = consumeItem(tokens);
  if (childBox.error) {
    return childBox;
  }
  node.addChild(childBox.node);

  return { node };
}

function consumeList(tokens) {

  const boxedNode = boxNode(NodeType.LIST, undefined);

  /* eslint-disable no-constant-condition */
  while (true) {
    const token = tokens[0];
    if (token === undefined) {
      return {error: 'unexpected end of list'};
    }

    if (token.type === TokenType.LIST_END) {
      tokens.shift();
      return boxedNode;
    }

    const boxedItem = consumeItem(tokens);
    if (boxedItem.error) {
      return boxedItem;
    }

    const n = boxedItem.node;
    boxedNode.node.addChild(n);
  }
  /* eslint-enable no-constant-condition */

}

/*
 returns an obj of the form:

 {
 nodes: array of nodes,
 error: possibly undefined
 }

 */

const Parser = {
  parse: function(tokens) {

    const nodes = [];
    let nodeBox;

    while (tokens.length !== 0) {
      nodeBox = consumeItem(tokens);

      if (nodeBox.error) {
        return nodeBox;
      }

      // n.node will be null on a comment
      if (nodeBox.node) {
        nodes.push(nodeBox.node);
      }
    }

    return {nodes: nodes};
  },

  // converts a frontAST back into a string
  // ast is an array of nodes
  unparse: function(frontAst, genotype) {
    let genoIndex = 0;

    function formatValue(value) {
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


    function add(term, str, node) {
      if(node.alterable) {
        // prefixes are any comments/whitespaces after the opening bracket
        let prefixes = node.parameterPrefix.reduce(unparseASTNode, '');
        let alterParams = node.parameterAST.reduce(unparseASTNode, '');
        // don't use the term, replace with value from genotype
        let value = genotype.get(genoIndex++).get('value');
        let v = formatValue(value);
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
        let lst = node.children.reduce(unparseASTNode, '');
        res = add('(' + lst + ')', str, node);
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

export default Parser;
