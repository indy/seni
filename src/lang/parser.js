import TokenType from './tokentype';
import Node from './node';
import NodeType from './nodetype';

// recursive code so switch off the jslint warnings
// about functions being used before they're defined
//
/*jslint latedef:false*/


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
  if(tokenType === TokenType.LIST_START) {
    return consumeList(tokens, alterable);
  } else if(tokenType === TokenType.LIST_END) {
    return {error: 'mismatched closing parens'};
  } else if(tokenType === TokenType.INT) {
    return boxNode(NodeType.INT, token.value, alterable);
  } else if(tokenType === TokenType.FLOAT) {
    return boxNode(NodeType.FLOAT, token.value, alterable);
  } else if(tokenType === TokenType.NAME) {
    let val = token.value;
    if(val === 'true') {
      return boxNode(NodeType.BOOLEAN, '#t', alterable);
    } else if (val === 'false') {
      return boxNode(NodeType.BOOLEAN, '#f', alterable);
    } else {
      return boxNode(NodeType.NAME, token.value, alterable);
    }
  } else if(tokenType === TokenType.LABEL) {
    return boxNode(NodeType.LABEL, token.value, alterable);
  } else if(tokenType === TokenType.STRING) {
    return boxNode(NodeType.STRING, token.value, alterable);
  } else if(tokenType === TokenType.QUOTE_ABBREVIATION) {
    return consumeQuotedForm(tokens);
  } else if(tokenType === TokenType.BRACKET_START) {
    return consumeBracketForm(tokens);
  } else if(tokenType === TokenType.BRACKET_END) {
    return {node: null};
  } else if(tokenType === TokenType.COMMENT) {
    return {node: null};
  }

  // e.g. TokenType.UNKNOWN
  return {error: 'unknown token type'};
}



function consumeBracketForm(tokens) {
  let nodeBox = consumeItem(tokens, true);
  if(nodeBox.error) {
    return nodeBox;
  }

  let node = nodeBox.node;
  let nodeType = node.type;

  if(nodeType !== NodeType.BOOLEAN &&
     nodeType !== NodeType.INT &&
     nodeType !== NodeType.FLOAT &&
     nodeType !== NodeType.NAME &&
     nodeType !== NodeType.STRING) {

    return {error: 'non-mutable node within square brackets'};
  }

  let parameterBox = true, parameter = true;
  while(parameter !== null) {

    parameterBox = consumeItem(tokens, false);
    if(parameterBox.error) {
      return parameterBox;
    }
    parameter = parameterBox.node;
    if(parameter !== null) {
      node.addParameterNode(parameter);
    }
  }

  return {node: node};
}


function consumeQuotedForm(tokens) {
  // '(2 3 4) -> (quote (2 3 4))

  let node = new Node(NodeType.LIST);

  node.addChild(new Node(NodeType.NAME, 'quote', false));
  let childBox = consumeItem(tokens, false);
  if(childBox.error) {
    return childBox;
  }
  node.addChild(childBox.node);

  return {node: node};
}

function consumeList(tokens, alterable) {
  let al = alterable;           // prevent jshint from complaining
  al = !al;
  let node = new Node(NodeType.LIST);

  while(true) {
    let token = tokens[0];
    if(token === undefined) {
      return {error: 'unexpected end of list'};
    }

    if(token.type === TokenType.LIST_END) {
      tokens.shift();
      return {node: node};
    }

    let nodeBox = consumeItem(tokens, false);
    if(nodeBox.error) {
      return nodeBox;
    }
    let n = nodeBox.node;
    if(n) {
      node.addChild(n);
    }

  }
}



/*
 returns an obj of the form:

 {
   nodes: array of nodes,
   error: possibly undefined
 }

*/

var Parser = {
  parse: function(tokens) {

    let nodes = [];
    let nodeBox;

    while(tokens.length !== 0) {
      nodeBox = consumeItem(tokens, false);

      if(nodeBox.error) {
        return nodeBox;
      }

      // n.node will be null on a comment
      if(nodeBox.node) {
        nodes.push(nodeBox.node);
      }
    }

    return {nodes: nodes};
  }
};


export default Parser;
