import {
    Token,
    TokenType
} from './token';

import {
    Node,
    NodeType
} from './node';

export function parse(tokens) {

    let nodes = [];
    
    while(tokens.length != 0) {
        nodes.push(consumeItem(tokens, false));
    }

    return nodes;
}

function consumeItem(tokens, alterable) {

    let token = tokens[0];
    tokens.shift();            // remove the first token

    let tokenType = token.getType();
    if(tokenType === TokenType.LIST_START) {
        return consumeList(tokens, alterable);
    } else if(tokenType === TokenType.INT) {
        return new Node(NodeType.INT, token.getValue(), alterable)
    } else if(tokenType === TokenType.FLOAT) {
        return new Node(NodeType.FLOAT, token.getValue(), alterable)
    } else if(tokenType === TokenType.NAME) {
        let val = token.getValue();
        if(val === "true") {
            return new Node(NodeType.BOOLEAN, '#t', alterable)
        } else if (val === "false") {
            return new Node(NodeType.BOOLEAN, '#f', alterable)
        } else {
            return new Node(NodeType.NAME, token.getValue(), alterable)
        }
    } else if(tokenType === TokenType.STRING) {
        return new Node(NodeType.STRING, token.getValue(), alterable)
    } else if(tokenType === TokenType.QUOTE_ABBREVIATION) {
        return consumeQuotedForm(tokens);
    } else if(tokenType === TokenType.BRACKET_START) {
        return consumeBracketForm(tokens);
    } else if(tokenType === TokenType.BRACKET_END) {
        return null;
    }
    // todo: throw an error?

    return null;
}


function consumeBracketForm(tokens) {
    let node = consumeItem(tokens, true),
    nodeType = node.getType();

    if(nodeType != NodeType.BOOLEAN &&
       nodeType != NodeType.INT &&
       nodeType != NodeType.FLOAT &&
       nodeType != NodeType.NAME &&
       nodeType != NodeType.STRING) {
        // throw an error - non-mutable node within square brackets
    }

    let parameter = consumeItem(tokens, false);
    while(parameter !== null) {
        node.addParameterNode(parameter);
        parameter = consumeItem(tokens, false);
    }

    return node;
}


function consumeQuotedForm(tokens) {
    // '(2 3 4) -> (quote (2 3 4))

    let node = new Node(NodeType.LIST);

    node.addChild(new Node(NodeType.NAME, "quote", false));
    node.addChild(consumeItem(tokens, false));

    return node;
}

function consumeList(tokens, alterable) {
    let node = new Node(NodeType.LIST);

    while(true) {
        let token = tokens[0];
        if(token.getType() === TokenType.LIST_END) {
            tokens.shift();
            return node;
        } else {
            node.addChild(consumeItem(tokens, false));
        }
    }
}
