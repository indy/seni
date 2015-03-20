import Token from './Token';
import TokenType from './TokenType';

function characterSet(characters) {
  let s = new Set();

  // todo: is there a better way than iterating over the string?
  for(let i=0;i<characters.length;i++) {
    s.add(characters[i]);
  }
  return s;
}

let sWhitespaceSet = characterSet(' \t\n,');
let sDigitSet = characterSet('0123456789');
let sAlphaSet = characterSet('abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ+-*/<>=');
let sSymbolSet = characterSet('-!@#$%^&*<>?');

let MINUS = '-';
let PERIOD = '.';

function isWhitespace(character) {
  return sWhitespaceSet.has(character);
}

function isDigit(character) {
  return sDigitSet.has(character);
}

function isAlpha(character) {
  return sAlphaSet.has(character);
}

function isSymbol(character) {
  return sSymbolSet.has(character);
}

function isListStart(character) {
  return character === '(';
}

function isListEnd(character) {
  return character === ')';
}

function isBracketStart(character) {
  return character === '[';
}

function isBracketEnd(character) {
  return character === ']';
}

function isQuotedString(character) {
  return character === '"';
}

function isQuoteAbbreviation(character) {
  return character === '\'';
}

function isComment(character) {
  return character === ';';
}

function isNewline(character) {
  return character === '\n';
}

function isLabel(s) {
  let i = 0;
  for(i=0;i<s.length;i++) {
    let c = s[i];
    if(!isAlpha(c) && !isDigit(c) && !isSymbol(c)) {
      break;
    }
  }
  return i < s.length && s[i] === ':';
}

// is there a period in the stream of characters before we get to whitespace
function hasPeriod(s) {
  for(let i=0; i<s.length; i++) {
    if(s[i] === PERIOD) {
      return true;
    }
    if(isWhitespace(s[i])) {
      return false;
    }
  }
  return false;
}


function skipWhitespace(s) {
  for(let i=0;i<s.length;i++) {
    if(!isWhitespace(s[i])) {
      return s.substring(i);
    }
  }
  return '';
}

function consumeInt(s) {
  let i = 0;
  for(i=0;i<s.length;i++) {
    let c = s[i];
    if(!isDigit(c) && c !== MINUS) {
      break;
    }

    if(!isDigit(c) && !(i === 0 && c === MINUS)) {
      break;
    }
  }

  let token = new Token(TokenType.INT, parseInt(s.substring(0, i)));
  return [token, s.substring(i, s.length)];
}

function consumeFloat(s) {
  let i = 0;
  for(i=0;i<s.length;i++) {
    let c = s[i];
    if(!isDigit(c) && !(i===0 && c === MINUS) && c !== PERIOD) {
      break;
    }
  }

  let token = new Token(TokenType.FLOAT, parseFloat(s.substring(0, i)));
  return [token, s.substring(i, s.length)];
}

function consumeUnknown(s) {
  return [new Token(TokenType.UNKNOWN, s[0]), s.substring(1)];
}

function consumeListStart(s) {
  return [new Token(TokenType.LIST_START), s.substring(1)];
}

function consumeListEnd(s) {
  return [new Token(TokenType.LIST_END), s.substring(1)];
}

function consumeBracketStart(s) {
  return [new Token(TokenType.BRACKET_START), s.substring(1)];
}

function consumeBracketEnd(s) {
  return [new Token(TokenType.BRACKET_END), s.substring(1)];
}

function consumeString(s) {
  let val = s.substring(1); // skip first \"
  const nextQuote = val.indexOf('\"');
  val = val.substring(0, nextQuote);

  const token = new Token(TokenType.STRING, val);
  return [token, s.substring(nextQuote + 2)];
}

function consumeName(s) {
  let i=0;
  for(i=0;i<s.length;i++) {
    let c = s[i];
    if(!isAlpha(c) && !isDigit(c) && !isSymbol(c)) {
      break;
    }
  }
  let token = new Token(TokenType.NAME, s.substring(0, i));
  return [token, s.substring(i, s.length)];
}

function consumeComment(s) {
  let i=0;
  for(i=0;i<s.length;i++) {
    if(isNewline(s[i])) {
      break;
    }
  }
  let token = new Token(TokenType.COMMENT, s.substring(0, i));
  // skip past the newline
  return [token, s.substring(i+1, s.length)];
}

function consumeLabel(s) {
  let i=0;
  for(i=0;i<s.length;i++) {
    let c = s[i];
    if(!isAlpha(c) && !isDigit(c) && !isSymbol(c)) {
      break;
    }
  }
  // read the label name - the ':' character
  let token = new Token(TokenType.LABEL, s.substring(0, i));
  // the remainder should skip past the ':'
  return [token, s.substring(i+1, s.length)];
}

function consumeQuoteAbbreviation(s) {
  return [new Token(TokenType.QUOTE_ABBREVIATION), s.substring(1)];
}

function nextTokenType(s) {
  let c = s[0];

  if(isQuoteAbbreviation(c)) {
    return TokenType.QUOTE_ABBREVIATION;
  }

  if(isListStart(c)) {
    return TokenType.LIST_START;
  }

  if(isListEnd(c)) {
    return TokenType.LIST_END;
  }

  if(isBracketStart(c)) {
    return TokenType.BRACKET_START;
  }

  if(isBracketEnd(c)) {
    return TokenType.BRACKET_END;
  }

  if(isQuotedString(c)) {
    return TokenType.STRING;
  }

  if(isAlpha(c)) {
    if(!(c === MINUS && s.length > 1 && isDigit(s[1]))) {
      return isLabel(s) ? TokenType.LABEL : TokenType.NAME;
    }
  }

  if(isDigit(c) || c === MINUS || c === PERIOD) {
    return hasPeriod(s) ? TokenType.FLOAT : TokenType.INT;
  }

  if(isComment(c)) {
    return TokenType.COMMENT;
  }

  return TokenType.UNKNOWN;
}


const Lexer = {
  tokenise: function(input) {
    let q = [],   // queue of tokens to return
        p = [];       // [token, remaining] pair

    let s = skipWhitespace(input);

    while(s.length > 0) {
      switch(nextTokenType(s)) {
      case TokenType.LIST_START :
        p = consumeListStart(s);
        break;
      case TokenType.LIST_END :
        p = consumeListEnd(s);
        break;
      case TokenType.BRACKET_START :
        p = consumeBracketStart(s);
        break;
      case TokenType.BRACKET_END :
        p = consumeBracketEnd(s);
        break;
      case TokenType.STRING :
        p = consumeString(s);
        break;
      case TokenType.NAME :
        p = consumeName(s);
        break;
      case TokenType.LABEL :
        p = consumeLabel(s);
        break;
      case TokenType.INT :
        p = consumeInt(s);
        break;
      case TokenType.FLOAT :
        p = consumeFloat(s);
        break;
      case TokenType.QUOTE_ABBREVIATION :
        p = consumeQuoteAbbreviation(s);
        break;
      case TokenType.COMMENT :
        p = consumeComment(s);
        break;
      default:
        // read the unknown token and return it
        let tok = consumeUnknown(s)[0];
        return {error: 'unknown token: ' + tok.value,
                tokens: [tok]};
      }

      let [token, remaining] = p;

      q.push(token);
      s = skipWhitespace(remaining);
    }

    return {tokens: q};
  }
};

export default Lexer;
