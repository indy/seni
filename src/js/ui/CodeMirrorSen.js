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

/* eslint-disable eqeqeq */
/* eslint-disable no-cond-assign */

function senMode() {
  const BUILTIN = 'builtin';
  const COMMENT = 'comment';
  const STRING = 'string';
  const ATOM = 'atom';
  const NUMBER = 'number';
  const PAREN = 'paren';      // ()
  const CURLY = 'curly';    // {}
  const BRACKET = 'bracket';    // []
  const SENCOMMON = 'sen-common';
  const PARAMETER = 'sen-parameter';

  const INDENT_WORD_SKIP = 2;

  function makeKeywords(str) {
    const obj = {}, words = str.split(/\s+/);
    for (let i = 0; i < words.length; ++i) obj[words[i]] = true;
    return obj;
  }

  // keywords are core to the sen language
  const keywords =
        makeKeywords('begin define fn if fence loop on-matrix-stack quote');
  const indentKeys = makeKeywords('define fence loop on-matrix-stack fn');

  // functions from the common sen library
  const senCommon = makeKeywords(`* + - / < = > append begin bezier
bezier-bulging bezier-trailing box canvas/centre canvas/height canvas/width
circle circle-slice col/analagous col/bezier-fn col/complementary col/convert
col/darken col/get-alpha col/get-lab-a col/get-lab-b col/get-lab-l
col/get-rgb-b col/get-rgb-g col/get-rgb-r col/hsl col/hsv col/lab col/lighten
col/procedural-fn col/quadratic-fn col/rgb col/set-alpha col/set-lab-a
col/set-lab-b col/set-lab-l col/set-rgb-b col/set-rgb-g col/set-rgb-r
col/split-complementary col/triad define degrees->radians fence fn focal/hline
focal/point focal/vline if interp/bezier interp/bezier-fn interp/bezier-tangent
interp/bezier-tangent-fn interp/circle interp/fn line list list/get list/length
log loop math/PI math/TAU math/atan2 math/clamp math/cos math/distance-2d
math/sin mod on-matrix-stack path/bezier path/circle path/linear path/spline
poly pop-matrix print prng/perlin-signed prng/perlin-unsigned prng/range
push-matrix quote radians->degrees rect repeat/rotate repeat/rotate-mirrored
repeat/symmetry-4 repeat/symmetry-8 repeat/symmetry-horizontal
repeat/symmetry-vertical rotate scale spline sqrt stroked-bezier
stroked-bezier-rect take translate`);

  function StateStack(indent, type, prev) { // represents a state stack object
    this.indent = indent;
    this.type = type;
    this.prev = prev;
  }

  function pushStack(state, indent, type) {
    state.indentStack = new StateStack(indent, type, state.indentStack);
  }

  function popStack(state) {
    state.indentStack = state.indentStack.prev;
  }

  const decimalMatcher = new RegExp(/^([-+]?\d*\.?\d*)/);

  function isDecimalNumber(stream, backup) {
    if (backup === true) {
      stream.backUp(1);
    }
    return stream.match(decimalMatcher);
  }

  function isParameter(word) {
    return word.slice(-1) === ':';
  }

  function tokenType(token, state, ch) {
    const prefix = 'geno-';
    let usePrefix = false;

    if (state.insideCurly) {
      // leave the first element inside curlys as is.

      if (state.curlyCounter === 1) {
        usePrefix = false;
        // this is the first element in the curlys
        state.curlyedFirstChildIsParen = (token === PAREN);
        if (state.curlyedFirstChildIsParen) {
          // special case of the first child in curlys being a s-exp.
          // we'll need to keep count of parenDepth
          state.firstParenCurlyDepth = state.parenDepth;
        }
      } else {
        // normally grey out, except if we're curlyedFirstChildIsParen
        if (state.curlyedFirstChildIsParen &&
            state.firstParenCurlyDepth <= state.parenDepth) {
          // keep on colouring as normal
          usePrefix = false;

          // if this is a closing parens then we've processed the first s-exp
          // and can start using prefix
          // (i.e. start greying out the remainder of the curly contents)
          if (state.firstParenCurlyDepth === state.parenDepth && ch === ')') {
            state.curlyedFirstChildIsParen = false;
          }
        } else {
          usePrefix = true;
        }
      }

      state.curlyCounter++;
    }

    return usePrefix ? prefix + token : token;
  }

  function setInsideCurly(value, state) {
    if (value === true) {
      state.curlyCounter = 0;
    }
    state.insideCurly = value;
  }

  return {
    startState: () => {
      const state = {
        indentStack: null,
        indentation: 0,
        mode: false,
        sExprComment: false,

        parenDepth: 0,

        insideCurly: false,
        curlyCounter: 0,
        firstParenCurlyDepth: 0,
        curlyedFirstChildIsParen: false
      };
      return state;
    },

    token: (stream, state) => {
      if (state.indentStack === null && stream.sol()) {
        // update indentation, but only if indentStack is empty
        state.indentation = stream.indentation();
      }

      // skip spaces
      if (stream.eatSpace()) {
        return null;
      }
      let returnType = null;
      let next;

      switch (state.mode) {
      case 'string': // multi-line string parsing mode
        let escaped = false;
        while ((next = stream.next()) != null) {
          if (next === '\"' && !escaped) {
            state.mode = false;
            break;
          }
          escaped = !escaped && next === '\\';
        }
        // continue on in scheme-string mode
        returnType = tokenType(STRING, state);
        break;
      case 'comment': // comment parsing mode
        let maybeEnd = false;
        while ((next = stream.next()) != null) {
          if (next === '#' && maybeEnd) {
            state.mode = false;
            break;
          }
          maybeEnd = (next === '|');
        }
        returnType = tokenType(COMMENT, state);
        break;
      default: // default parsing mode
        const ch = stream.next();

        if (ch === '\"') {
          state.mode = 'string';
          returnType = tokenType(STRING, state);
        } else if (ch === '\'') {
          returnType = tokenType(ATOM, state);
        } else if (/^[-+0-9.]/.test(ch) && isDecimalNumber(stream, true)) {
          // match non-prefixed number, must be decimal
          returnType = tokenType(NUMBER, state);
        } else if (ch === ';') { // comment
          stream.skipToEnd(); // rest of the line is a comment
          returnType = tokenType(COMMENT, state);
        } else if (ch === '[') { // bracket
          pushStack(state, stream.column() + 1, ch);
          returnType = tokenType(BRACKET, state);
        } else if (ch === ']') { // bracket
          popStack(state);
          returnType = tokenType(BRACKET, state);
        } else if (ch === '(' || ch === '{') {
          let keyWord = '', letter;
          const indentTemp = stream.column();

          if (ch === '{') {
            setInsideCurly(true, state);
          } else {
            state.parenDepth++;
          }

          while ((letter = stream.eat(/[^\s\(\)\[\]\{\}\;]/)) != null) {
            keyWord += letter;
          }

          if (keyWord.length > 0 && indentKeys.propertyIsEnumerable(keyWord)) {
            // indent-word
            pushStack(state, indentTemp + INDENT_WORD_SKIP, ch);
          } else { // non-indent word
            // we continue eating the spaces
            stream.eatSpace();
            if (stream.eol() || stream.peek() === ';') {
              // nothing significant after
              // we restart indentation 1 space after
              pushStack(state, indentTemp + 1, ch);
            } else {
              pushStack(state, indentTemp + stream.current().length, ch);
              // else we match
            }
          }
          stream.backUp(stream.current().length - 1); // undo all the eating

          if (typeof state.sExprComment === 'number') state.sExprComment++;

          returnType = tokenType(ch === '{' ? CURLY : PAREN, state, ch);
        } else if (ch === ')' || ch === '}') {
          returnType = tokenType(ch === '}' ? CURLY : PAREN, state, ch);
          if (state.indentStack != null &&
              state.indentStack.type === (ch === ')' ? '(' : '{')) {
            popStack(state);

            if (typeof state.sExprComment === 'number') {
              if (--state.sExprComment === 0) {
                returnType = tokenType(COMMENT, state); // final closing curly
                state.sExprComment = false; // turn off s-expr commenting mode
              }
            }
          }
          if (ch === '}') {
            setInsideCurly(false, state);
          } else {
            state.parenDepth--;
          }
        } else {
          stream.eatWhile(/[\w\$_\-!$%&*+\.\/:<=>?@\^~]/);

          if (keywords.propertyIsEnumerable(stream.current())) {
            returnType = tokenType(BUILTIN, state);
          } else if (senCommon.propertyIsEnumerable(stream.current())) {
            returnType = tokenType(SENCOMMON, state);
          } else if (isParameter(stream.current())) {
            returnType = tokenType(PARAMETER, state);
          } else returnType = tokenType('variable', state);
        }
      }
      return (typeof state.sExprComment === 'number') ? COMMENT : returnType;
    },

    indent: state => {
      if (state.indentStack === null) return state.indentation;
      return state.indentStack.indent;
    },

    closeBrackets: {pairs: '()[]{}\"\"'},
    lineComment: ';;'
  };
}

const CodeMirrorSen = {
  senMode
};

export default CodeMirrorSen;
