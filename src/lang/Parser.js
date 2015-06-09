/*
 *  Seni
 *  Copyright (C) 2015  Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

import TokenType from './TokenType';
import Node from './Node';
import NodeType from './NodeType';

/*
 these functions will return {node: node, error: error}
 */

function boxNode(nodeType, value, alterable) {
  return {node: new Node(nodeType, value, alterable)};
}

function consumeItem(tokens, alterable) {

  const token = tokens[0];
  tokens.shift();            // remove the first token

  const tokenType = token.type;
  if (tokenType === TokenType.LIST_START) {
    return consumeList(tokens, alterable);
  } else if (tokenType === TokenType.LIST_END) {
    return {error: 'mismatched closing parens'};
  } else if (tokenType === TokenType.INT) {
    return boxNode(NodeType.INT, token.value, alterable);
  } else if (tokenType === TokenType.FLOAT) {
    return boxNode(NodeType.FLOAT, token.value, alterable);
  } else if (tokenType === TokenType.NAME) {
    const val = token.value;
    if (val === 'true') {
      return boxNode(NodeType.BOOLEAN, '#t', alterable);
    } else if (val === 'false') {
      return boxNode(NodeType.BOOLEAN, '#f', alterable);
    } else {
      return boxNode(NodeType.NAME, token.value, alterable);
    }
  } else if (tokenType === TokenType.LABEL) {
    return boxNode(NodeType.LABEL, token.value, alterable);
  } else if (tokenType === TokenType.STRING) {
    return boxNode(NodeType.STRING, token.value, alterable);
  } else if (tokenType === TokenType.QUOTE_ABBREVIATION) {
    return consumeQuotedForm(tokens);
  } else if (tokenType === TokenType.BRACKET_START) {
    return consumeBracketForm(tokens);
  } else if (tokenType === TokenType.BRACKET_END) {
    return {error: 'mismatched closing square brackets'};
  } else if (tokenType === TokenType.COMMENT) {
    return {node: null};
  }

  // e.g. TokenType.UNKNOWN
  return {error: 'unknown token type'};
}

function consumeBracketForm(tokens) {
  const nodeBox = consumeItem(tokens, true);
  if (nodeBox.error) {
    return nodeBox;
  }

  const node = nodeBox.node;
  const nodeType = node.type;

  if (nodeType !== NodeType.BOOLEAN &&
      nodeType !== NodeType.INT &&
      nodeType !== NodeType.FLOAT &&
      nodeType !== NodeType.NAME &&
      nodeType !== NodeType.STRING &&
      nodeType !== NodeType.LIST) {

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

    parameterBox = consumeItem(tokens, false);
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

  node.addChild(new Node(NodeType.NAME, 'quote', false));
  const childBox = consumeItem(tokens, false);
  if (childBox.error) {
    return childBox;
  }
  node.addChild(childBox.node);

  return { node };
}

function consumeList(tokens, alterable) {

  const boxedNode = boxNode(NodeType.LIST, undefined, alterable);

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

    const boxedItem = consumeItem(tokens, false);
    if (boxedItem.error) {
      return boxedItem;
    }
    const n = boxedItem.node;
    if (n) {
      boxedNode.node.addChild(n);
    }
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
      nodeBox = consumeItem(tokens, false);

      if (nodeBox.error) {
        return nodeBox;
      }

      // n.node will be null on a comment
      if (nodeBox.node) {
        nodes.push(nodeBox.node);
      }
    }

    return {nodes: nodes};
  }
};

export default Parser;
