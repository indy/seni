import {
  Token,
  TokenType
} from './token';

export class Lexer {

  get MINUS()  { return '-'; }
  get PERIOD() { return '.'; }

  tokenise(input) {
    let q = [],                 // queue of tokens to return
        p = [];                 // [token, remaining] pair

    let s = this.skipWhitespace(input);

    while(s.length > 0) {
      switch(this.nextTokenType(s)) {
      case TokenType.LIST_START :
        p = this.consumeListStart(s);
        break;
      case TokenType.LIST_END :
        p = this.consumeListEnd(s);
        break;
      case TokenType.BRACKET_START :
        p = this.consumeBracketStart(s);
        break;
      case TokenType.BRACKET_END :
        p = this.consumeBracketEnd(s);
        break;
      case TokenType.STRING :
        p = this.consumeString(s);
        break;
      case TokenType.NAME :
        p = this.consumeName(s);
        break;
      case TokenType.INT :
        p = this.consumeInt(s);
        break;
      case TokenType.FLOAT :
        p = this.consumeFloat(s);
        break;
      case TokenType.QUOTE_ABBREVIATION :
        p = this.consumeQuoteAbbreviation(s);
        break;
        // todo: add a default that throws an exception?
        // todo: or just add an UNKNOWN token to q
      };

      let [token, remaining] = p;
      
      q.push(token);
      s = this.skipWhitespace(remaining);
    }

    return q;
  }

  skipWhitespace(s) {
    for(let i=0;i<s.length;i++) {
      if(!this.isWhitespace(s[i])) {
        return s.substring(i);
      }
    }
    return "";
  }

  consumeInt(s) {
    let i = 0;
    for(i=0;i<s.length;i++) {
      let c = s[i];
      if(!this.isDigit(c) && c != this.MINUS) {
        break;
      }

      if(!this.isDigit(c) && !(i === 0 && c === this.MINUS)) {
        break;
      }
    }
    
    let token = new Token(TokenType.INT, parseInt(s.substring(0, i)));
    return [token, s.substring(i, s.length)];
  }

  consumeFloat(s) {
    let i = 0;
    for(i=0;i<s.length;i++) {
      let c = s[i];
      if(!this.isDigit(c) && !(i===0 && c === this.MINUS) && c != this.PERIOD) {
        break;
      }
    }
    
    let token = new Token(TokenType.FLOAT, parseFloat(s.substring(0, i)));
    return [token, s.substring(i, s.length)];
  }

  consumeListStart(s) {
    return [new Token(TokenType.LIST_START), s.substring(1)];
  }

  consumeListEnd(s) {
    return [new Token(TokenType.LIST_END), s.substring(1)];
  }

  consumeBracketStart(s) {
    return [new Token(TokenType.BRACKET_START), s.substring(1)];
  }

  consumeBracketEnd(s) {
    return [new Token(TokenType.BRACKET_END), s.substring(1)];
  }

  consumeString(s) {
    let val = s.substring(1); // skip first \"
    let nextQuote = val.indexOf('\"');
    val = val.substring(0, nextQuote);

    let token = new Token(TokenType.STRING, val);
    return [token, s.substring(nextQuote + 2)];
  }
  
  consumeName(s) {
    let i=0;
    for(i=0;i<s.length;i++) {
      let c = s[i];
      if(!this.isAlpha(c) && !this.isDigit(c) && !this.isSymbol(c)) {
        break;
      }
    }
    let token = new Token(TokenType.NAME, s.substring(0, i));
    return [token, s.substring(i, s.length)];
  }
  
  consumeQuoteAbbreviation(s) {
    return [new Token(TokenType.QUOTE_ABBREVIATION), s.substring(1)];
  }

  nextTokenType(s) {
    let c = s[0];

    if(this.isQuoteAbbreviation(c)) {
      return TokenType.QUOTE_ABBREVIATION;
    }

    if(this.isListStart(c)) {
      return TokenType.LIST_START;
    }

    if(this.isListEnd(c)) {
      return TokenType.LIST_END;
    }

    if(this.isBracketStart(c)) {
      return TokenType.BRACKET_START;
    }

    if(this.isBracketEnd(c)) {
      return TokenType.BRACKET_END;
    }

    if(this.isQuotedString(c)) {
      return TokenType.STRING;
    }

    if(this.isName(c)) {
      if(c === this.MINUS && s.length > 1 && this.isDigit(s[1])) {
        // don't treat negative numbers as NAMEs
      } else {
        return TokenType.NAME;
      }
    }

    if(this.isDigit(c) || c === this.MINUS || c === this.PERIOD) {
      return this.hasPeriod(s) ? TokenType.FLOAT : TokenType.INT;
    }

    return TokenType.UNKNOWN;
  }

  // is there a period in the stream of characters before we get to whitespace
  hasPeriod(s) {
    for(let i=0; i<s.length; i++) {
      if(s[i] === this.PERIOD) {
        return true;
      }
      if(this.isWhitespace(s[i])) {
        return false;
      }
    }
    return false;
  }

  isWhitespace(character) {
    return sWhitespaceSet.has(character);
  }

  isDigit(character) {
    return sDigitSet.has(character);
  }

  isAlpha(character) {
    return sAlphaSet.has(character);
  }

  isSymbol(character) {
    return sSymbolSet.has(character);
  }

  isListStart(character) {
    return character === '(';
  }

  isListEnd(character) {
    return character === ')';
  }

  isBracketStart(character) {
    return character === '[';
  }

  isBracketEnd(character) {
    return character === ']';
  }

  isQuotedString(character) {
    return character === '"';
  }
  
  isQuoteAbbreviation(character) {
    return character === '\'';
  }

  isName(character) {
    return this.isAlpha(character);
  }
};

var sWhitespaceSet = characterSet(" \t\n");
var sDigitSet = characterSet("0123456789");
var sAlphaSet = characterSet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ+-*/<>=");
var sSymbolSet = characterSet("-!@#$%^&*<>?");

function characterSet(characters) {
  let s = new Set();

  // todo: is there a better way than iterating over the string?
  for(let i=0;i<characters.length;i++) {
    s.add(characters[i]);
  }
  return s;
}


