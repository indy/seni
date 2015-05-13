
/* eslint-disable no-unused-vars */
/* eslint-disable curly */
/* eslint-disable max-len */
/* eslint-disable new-cap */
/* eslint-disable no-redeclare */
/* eslint-disable no-fallthrough */

function seniMode() {
  let BUILTIN = 'builtin', COMMENT = 'comment', STRING = 'string',
      ATOM = 'atom', NUMBER = 'number', BRACKET = 'bracket',
      SENICOMMON = 'seni-common', PARAMETER = 'seni-parameter';
  let INDENT_WORD_SKIP = 2;

  function makeKeywords(str) {
    let obj = {}, words = str.split(' ');
    for (let i = 0; i < words.length; ++i) obj[words[i]] = true;
    return obj;
  }

  // keywords are core to the seni language
  let keywords = makeKeywords('begin define if let lambda loop on-matrix-stack quote');
  let indentKeys = makeKeywords(`define let lambda loop on-matrix-stack`);

  // functions from the common seni library
  let seniCommon = makeKeywords('* + - / < = > bezier bezier-bulging bezier-trailing col/analagous col/complementary col/convert col/get-alpha col/get-lab-a col/get-lab-b col/get-lab-l col/get-rgb-b col/get-rgb-g col/get-rgb-r col/hsl col/hsv col/lab col/rgb col/set-alpha col/set-lab-a col/set-lab-b col/set-lab-l col/set-rgb-b col/set-rgb-g col/set-rgb-r col/split-complementary col/triad cos fn focal/hline focal/point focal/vline gradient-quad gradient-triangle list log math/clamp math/distance-2d on-matrix-stack path/bezier path/circle path/linear path/spline perlin/signed perlin/unsigned poly pop-matrix print push-matrix quote rect red remap-fn rng/signed rng/unsigned rotate scale sin spline sqrt stroked-bezier stroked-bezier-rect take translate v2 v2/* v2/+ v2/- v2// v2/= v2/x v2/y');

  function stateStack(indent, type, prev) { // represents a state stack object
    this.indent = indent;
    this.type = type;
    this.prev = prev;
  }

  function pushStack(state, indent, type) {
    state.indentStack = new stateStack(indent, type, state.indentStack);
  }

  function popStack(state) {
    state.indentStack = state.indentStack.prev;
  }

  let decimalMatcher = new RegExp(/^([-+]?\d*\.?\d*)/);

  function isDecimalNumber (stream, backup) {
    if (backup === true) {
      stream.backUp(1);
    }
    return stream.match(decimalMatcher);
  }

  function isParameter(word) {
    return word.slice(-1) === ':';
  }

  function tokenType(token, state) {
    let prefix = 'geno-';
    let usePrefix = false;

    if(state.insideSquareBracket) {
      // leave the first element inside square brackets as is.
      usePrefix = state.squareBracketCounter !== 1;
      state.squareBracketCounter++;
    }

    return usePrefix ? prefix + token : token;
  }

  function setInsideSquareBracket(value, state) {
    if(value === true) {
      state.squareBracketCounter = 0;
    }
    state.insideSquareBracket = value;
  }

  return {
    startState: function () {
      return {
        indentStack: null,
        indentation: 0,
        mode: false,
        sExprComment: false,

        insideSquareBracket: false,
        squareBracketCounter: 0
      };
    },

    token: function (stream, state) {
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

      switch(state.mode){
      case 'string': // multi-line string parsing mode
        let escaped = false;
        while ((next = stream.next()) != null) {
          if (next === '\"' && !escaped) {

            state.mode = false;
            break;
          }
          escaped = !escaped && next === '\\';
        }
        returnType = tokenType(STRING, state); // continue on in scheme-string mode
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
        let ch = stream.next();

        if (ch === '\"') {
          state.mode = 'string';
          returnType = tokenType(STRING, state);

        } else if (ch === '\'') {
          returnType = tokenType(ATOM, state);
        } else if (/^[-+0-9.]/.test(ch) && isDecimalNumber(stream, true)) { // match non-prefixed number, must be decimal
          returnType = tokenType(NUMBER, state);
        } else if (ch === ';') { // comment
          stream.skipToEnd(); // rest of the line is a comment
          returnType = tokenType(COMMENT, state);
        } else if (ch === '(' || ch === '[') {
          let keyWord = ''; let indentTemp = stream.column(), letter;

          if (ch === '[') {
            setInsideSquareBracket(true, state);
          }

          while ((letter = stream.eat(/[^\s\(\[\;\)\]]/)) != null) {
            keyWord += letter;
          }

          if (keyWord.length > 0 && indentKeys.propertyIsEnumerable(keyWord)) { // indent-word

            pushStack(state, indentTemp + INDENT_WORD_SKIP, ch);
          } else { // non-indent word
            // we continue eating the spaces
            stream.eatSpace();
            if (stream.eol() || stream.peek() === ';') {
              // nothing significant after
              // we restart indentation 1 space after
              pushStack(state, indentTemp + 1, ch);
            } else {
              pushStack(state, indentTemp + stream.current().length, ch); // else we match
            }
          }
          stream.backUp(stream.current().length - 1); // undo all the eating

          if(typeof state.sExprComment === 'number') state.sExprComment++;

          returnType = tokenType(BRACKET, state);
        } else if (ch === ')' || ch === ']') {
          returnType = tokenType(BRACKET, state);
          if (state.indentStack != null && state.indentStack.type === (ch === ')' ? '(' : '[')) {
            popStack(state);

            if(typeof state.sExprComment === 'number'){
              if(--state.sExprComment === 0){
                returnType = tokenType(COMMENT, state); // final closing bracket
                state.sExprComment = false; // turn off s-expr commenting mode
              }
            }
          }
          if(ch === ']') {
            setInsideSquareBracket(false, state);
          }
        } else {
          stream.eatWhile(/[\w\$_\-!$%&*+\.\/:<=>?@\^~]/);

          if (keywords.propertyIsEnumerable(stream.current())) {
            returnType = tokenType(BUILTIN, state);
          } else if (seniCommon.propertyIsEnumerable(stream.current())) {
            returnType = tokenType(SENICOMMON, state);
          } else if (isParameter(stream.current())) {
            returnType = tokenType(PARAMETER, state);
          } else returnType = tokenType('variable', state);
        }
      }
      return (typeof state.sExprComment === 'number') ? COMMENT : returnType;
    },

    indent: function (state) {
      if (state.indentStack === null) return state.indentation;
      return state.indentStack.indent;
    },

    closeBrackets: {pairs: '()[]{}\"\"'},
    lineComment: ';;'
  };
}

const CodeMirrorSeni = {
  seniMode
};

export default CodeMirrorSeni;
