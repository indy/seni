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

import TokenType from './TokenType';
import Node from './Node';
import NodeList from './NodeList';
import NodeVector from './NodeVector';
import NodeType from './NodeType';

/*
 these functions will return {node: node, error: error}
 */

function boxNode(nodeType, value) {
  return {node: new Node(nodeType, value)};
}

function consumeItem(tokens) {
  const token = tokens[0];
  tokens.shift();            // remove the first token

  const tokenType = token.type;
  if (tokenType === TokenType.LIST_START) {
    return consumeList(tokens);
  } else if (tokenType === TokenType.LIST_END) {
    return {error: `mismatched closing parens`};
  } else if (tokenType === TokenType.VECTOR_START) {
    return consumeVector(tokens);
  } else if (tokenType === TokenType.VECTOR_END) {
    return {error: `mismatched closing square brackets`};
  } else if (tokenType === TokenType.INT) {
    return boxNode(NodeType.INT, token.value);
  } else if (tokenType === TokenType.FLOAT) {
    return boxNode(NodeType.FLOAT, token.value);
  } else if (tokenType === TokenType.NAME) {
    const val = token.value;
    if (val === `true`) {
      return boxNode(NodeType.BOOLEAN, `#t`);
    } else if (val === `false`) {
      return boxNode(NodeType.BOOLEAN, `#f`);
    } else {
      return boxNode(NodeType.NAME, token.value);
    }
  } else if (tokenType === TokenType.LABEL) {
    return boxNode(NodeType.LABEL, token.value);
  } else if (tokenType === TokenType.STRING) {
    return boxNode(NodeType.STRING, token.value);
  } else if (tokenType === TokenType.QUOTE_ABBREVIATION) {
    return consumeQuotedForm(tokens);
  } else if (tokenType === TokenType.ALTERABLE_START) {
    return consumeBracketForm(tokens);
  } else if (tokenType === TokenType.ALTERABLE_END) {
    return {error: `mismatched closing alterable brackets`};
  } else if (tokenType === TokenType.COMMENT) {
    return boxNode(NodeType.COMMENT, `${token.value}\n`);
  } else if (tokenType === TokenType.WHITESPACE) {
    return boxNode(NodeType.WHITESPACE, token.value);
  }

  // e.g. TokenType.UNKNOWN
  return {error: `unknown token type`};
}


function consumeBracketForm(tokens) {
  let node, nodeType;
  const prefixParameters = [];

  /*eslint-disable no-constant-condition */
  while (true) {
    const nodeBox = consumeItem(tokens);
    if (nodeBox.error) {
      return nodeBox;
    }
    node = nodeBox.node;
    nodeType = node.type;

    if (nodeType === NodeType.COMMENT || nodeType === NodeType.WHITESPACE) {
      prefixParameters.push(node);
    } else {
      // we've got the first node within the curly brackets that's mutable
      node.alterable = true;
      break;
    }
  }
  /*eslint-enable no-constant-condition */

  prefixParameters.forEach(pp => node.addParameterNodePrefix(pp));

  if (nodeType !== NodeType.BOOLEAN &&
      nodeType !== NodeType.INT &&
      nodeType !== NodeType.FLOAT &&
      nodeType !== NodeType.NAME &&
      nodeType !== NodeType.STRING &&
      nodeType !== NodeType.LIST &&
      nodeType !== NodeType.VECTOR) {
    console.log(`whooops`, tokens, node);
    return {error: `non-mutable node within curly brackets ${nodeType}`};
  }

  /* eslint-disable no-constant-condition */
  while (true) {
    const token = tokens[0];
    if (token === undefined) {
      return {error: `unexpected end of list`};
    }
    if (token.type === TokenType.ALTERABLE_END) {
      tokens.shift();
      break;
    }

    const parameterBox = consumeItem(tokens);
    if (parameterBox.error) {
      return parameterBox;
    }
    const parameter = parameterBox.node;
    if (parameter !== null) {
      node.addParameterNode(parameter);
    }
  }
  /* eslint-enable no-constant-condition */

  return { node };

}

function consumeQuotedForm(tokens) {
  // '(2 3 4) -> (quote (2 3 4))

  const node = new NodeList();

  node.usingAbbreviation = true;
  node.addChild(new Node(NodeType.NAME, `quote`));
  node.addChild(new Node(NodeType.WHITESPACE, ` `));
  const childBox = consumeItem(tokens);
  if (childBox.error) {
    return childBox;
  }
  node.addChild(childBox.node);

  return { node };
}

function consumeList(tokens) {

  const node = new NodeList();

  /* eslint-disable no-constant-condition */
  while (true) {
    const token = tokens[0];
    if (token === undefined) {
      return {error: `unexpected end of list`};
    }

    if (token.type === TokenType.LIST_END) {
      tokens.shift();
      return {node};
    }

    const boxedItem = consumeItem(tokens);
    if (boxedItem.error) {
      return boxedItem;
    }

    const n = boxedItem.node;
    node.addChild(n);
  }
  /* eslint-enable no-constant-condition */

}

function consumeVector(tokens) {

  const node = new NodeVector();

  /* eslint-disable no-constant-condition */
  while (true) {
    const token = tokens[0];
    if (token === undefined) {
      return {error: `unexpected end of vector`};
    }

    if (token.type === TokenType.VECTOR_END) {
      tokens.shift();
      return {node};
    }

    const boxedItem = consumeItem(tokens);
    if (boxedItem.error) {
      return boxedItem;
    }

    const n = boxedItem.node;
    node.addChild(n);
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

  /**
   * parse some stuff
   * @param tokens the tokens
   */
  parse: tokens => {

    const nodes = [];
    while (tokens.length !== 0) {
      const nodeBox = consumeItem(tokens);

      if (nodeBox.error) {
        return nodeBox;
      }

      // n.node will be null on a comment
      if (nodeBox.node) {
        nodes.push(nodeBox.node);
      }
    }

    return {nodes};
  }
};

export default Parser;
