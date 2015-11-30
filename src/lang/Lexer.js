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

import Token from './Token';
import TokenType from './TokenType';

function characterSet(characters) {
  const s = new Set();

  // todo: is there a better way than iterating over the string?
  for (let i = 0; i < characters.length; i++) {
    s.add(characters[i]);
  }
  return s;
}

const sWhitespaceSet = characterSet(' \t\n,');
const sDigitSet = characterSet('0123456789');
const sAlphaSet =
        characterSet(
          'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ+-*/<>=');
const sSymbolSet = characterSet('-!@#$%^&*<>?');

const MINUS = '-';
const PERIOD = '.';

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

function isVectorStart(character) {
  return character === '[';
}

function isVectorEnd(character) {
  return character === ']';
}

function isAlterableStart(character) {
  return character === '{';
}

function isAlterableEnd(character) {
  return character === '}';
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
  for (i = 0; i < s.length; i++) {
    const c = s[i];
    if (!isAlpha(c) && !isDigit(c) && !isSymbol(c)) {
      break;
    }
  }
  return i < s.length && s[i] === ':';
}

// is there a period in the stream of characters before we get to whitespace
function hasPeriod(s) {
  for (let i = 0; i < s.length; i++) {
    if (s[i] === PERIOD) {
      return true;
    }
    if (isWhitespace(s[i])) {
      return false;
    }
  }
  return false;
}

function consumeInt(s) {
  let i = 0;
  for (i = 0; i < s.length; i++) {
    const c = s[i];
    if (!isDigit(c) && c !== MINUS) {
      break;
    }

    if (!isDigit(c) && !(i === 0 && c === MINUS)) {
      break;
    }
  }

  const token = new Token(TokenType.INT, parseInt(s.substring(0, i)));
  return [token, s.substring(i, s.length)];
}

function consumeFloat(s) {
  let i = 0;
  for (i = 0; i < s.length; i++) {
    const c = s[i];
    if (!isDigit(c) && !(i === 0 && c === MINUS) && c !== PERIOD) {
      break;
    }
  }

  const token = new Token(TokenType.FLOAT, parseFloat(s.substring(0, i)));
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

function consumeVectorStart(s) {
  return [new Token(TokenType.VECTOR_START), s.substring(1)];
}

function consumeVectorEnd(s) {
  return [new Token(TokenType.VECTOR_END), s.substring(1)];
}

function consumeAlterableStart(s) {
  return [new Token(TokenType.ALTERABLE_START), s.substring(1)];
}

function consumeAlterableEnd(s) {
  return [new Token(TokenType.ALTERABLE_END), s.substring(1)];
}

function consumeString(s) {
  let val = s.substring(1); // skip first \"
  const nextQuote = val.indexOf('\"');
  val = val.substring(0, nextQuote);

  const token = new Token(TokenType.STRING, val);
  return [token, s.substring(nextQuote + 2)];
}

function consumeName(s) {
  let i = 0;
  for (i = 0; i < s.length; i++) {
    const c = s[i];
    if (!isAlpha(c) && !isDigit(c) && !isSymbol(c)) {
      break;
    }
  }
  const token = new Token(TokenType.NAME, s.substring(0, i));
  return [token, s.substring(i, s.length)];
}


function consumeWhitespace(s) {
  let i = 0;
  for (i = 0; i < s.length; i++) {
    if (!isWhitespace(s[i])) {
      break;
    }
  }
  const token = new Token(TokenType.WHITESPACE, s.substring(0, i));
  return [token, s.substring(i, s.length)];
}

function consumeComment(s) {
  let i = 0;
  for (i = 0; i < s.length; i++) {
    if (isNewline(s[i])) {
      break;
    }
  }
  const token = new Token(TokenType.COMMENT, s.substring(0, i));
  // skip past the newline
  return [token, s.substring(i + 1, s.length)];
}

function consumeLabel(s) {
  let i = 0;
  for (i = 0; i < s.length; i++) {
    const c = s[i];
    if (!isAlpha(c) && !isDigit(c) && !isSymbol(c)) {
      break;
    }
  }
  // read the label name - the ':' character
  const token = new Token(TokenType.LABEL, s.substring(0, i));
  // the remainder should skip past the ':'
  return [token, s.substring(i + 1, s.length)];
}

function consumeQuoteAbbreviation(s) {
  return [new Token(TokenType.QUOTE_ABBREVIATION), s.substring(1)];
}

function nextTokenType(s) {
  const c = s[0];

  if (isWhitespace(c)) {
    return TokenType.WHITESPACE;
  }

  if (isQuoteAbbreviation(c)) {
    return TokenType.QUOTE_ABBREVIATION;
  }

  if (isListStart(c)) {
    return TokenType.LIST_START;
  }

  if (isListEnd(c)) {
    return TokenType.LIST_END;
  }

  if (isVectorStart(c)) {
    return TokenType.VECTOR_START;
  }

  if (isVectorEnd(c)) {
    return TokenType.VECTOR_END;
  }

  if (isAlterableStart(c)) {
    return TokenType.ALTERABLE_START;
  }

  if (isAlterableEnd(c)) {
    return TokenType.ALTERABLE_END;
  }

  if (isQuotedString(c)) {
    return TokenType.STRING;
  }

  if (isAlpha(c)) {
    if (!(c === MINUS && s.length > 1 && isDigit(s[1]))) {
      return isLabel(s) ? TokenType.LABEL : TokenType.NAME;
    }
  }

  if (isDigit(c) || c === MINUS || c === PERIOD) {
    return hasPeriod(s) ? TokenType.FLOAT : TokenType.INT;
  }

  if (isComment(c)) {
    return TokenType.COMMENT;
  }

  return TokenType.UNKNOWN;
}

const Lexer = {
  tokenise: input => {
    const q = [];   // queue of tokens to return
    let p = [];   // [token, remaining] pair

    let s = input;

    while (s.length > 0) {
      switch (nextTokenType(s)) {
      case TokenType.WHITESPACE :
        p = consumeWhitespace(s);
        break;
      case TokenType.LIST_START :
        p = consumeListStart(s);
        break;
      case TokenType.LIST_END :
        p = consumeListEnd(s);
        break;
      case TokenType.VECTOR_START :
        p = consumeVectorStart(s);
        break;
      case TokenType.VECTOR_END :
        p = consumeVectorEnd(s);
        break;
      case TokenType.ALTERABLE_START :
        p = consumeAlterableStart(s);
        break;
      case TokenType.ALTERABLE_END :
        p = consumeAlterableEnd(s);
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
        const tok = consumeUnknown(s)[0];
        return {error: `unknown token: ${tok.value}`,
                tokens: [tok]};
      }

      const [token, remaining] = p;

      q.push(token);
      s = remaining;
    }

    return {tokens: q};
  }
};

export default Lexer;
