// --------------------------------------------------------------------------------
// codemirrorSeni

function seniMode() {
  const BUILTIN = 'builtin';
  const COMMENT = 'comment';
  const STRING = 'string';
  const ATOM = 'atom';
  const NUMBER = 'number';
  const TILDE = 'tilde';     // ~
  const PAREN = 'paren';     // ()
  const BRACKET = 'bracket'; // []
  const SENICOMMON = 'seni-common';
  const PARAMETER = 'seni-parameter';


  const INDENT_WORD_SKIP = 2;

  function makeKeywords(str) {
    const obj = {}, words = str.split(/\s+/);
    for (let i = 0; i < words.length; ++i) obj[words[i]] = true;
    return obj;
  }

  // keywords are core to the seni language
  const keywords =
        makeKeywords('begin define fn if fence loop on-matrix-stack quote');
  const indentKeys = makeKeywords('define fence loop on-matrix-stack fn');

  // functions from the common seni library
  const seniCommon = makeKeywords(`* + - / < = > append begin bezier
bezier-bulging bezier-trailing box canvas/centre canvas/height canvas/width
circle circle-slice col/analagous col/bezier-fn col/complementary col/convert
col/darken col/alpha col/hsl col/hsluv col/hsv col/lab col/lighten
col/procedural-fn col/quadratic-fn col/rgb col/set-alpha col/e0 col/e1 col/e2 col/set-e0 col/set-e1 col/set-e2
col/split-complementary col/triad define image math/degrees->radians fence fn focal/hline
focal/point focal/vline if interp/bezier interp/bezier-fn interp/bezier-tangent
interp/bezier-tangent-fn interp/circle interp/fn line list list/get list/length
log loop math/PI math/TAU math/atan2 math/clamp math/cos math/distance-2d
math/sin mod on-matrix-stack path/bezier path/circle path/linear path/spline
poly pop-matrix print prng/perlin-signed prng/perlin-unsigned prng/range
push-matrix quote radians->degrees rect repeat/rotate repeat/rotate-mirrored
repeat/symmetry-4 repeat/symmetry-8 repeat/symmetry-horizontal
repeat/symmetry-vertical rotate scale spline sqrt stroked-bezier
take translate`);

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
    return state.afterTilde === true ? 'geno-' + token : token;
  }


  function setAfterTilde(value, state) {
    if (value === true) {
      // switch off afterTilde when we get to a closing paren of parenDepth + 1
      state.afterTildeParenDepth = state.parenDepth + 1;
    }
    state.afterTilde = value;
  }

  return {
    startState: () => {
      const state = {
        indentStack: null,
        indentation: 0,
        mode: false,
        sExprComment: false,

        parenDepth: 0,

        afterTilde: false,
        afterTildeParenDepth: 0,
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
        } else if (ch === '~') {
          setAfterTilde(true, state);
          returnType = tokenType(TILDE, state);
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
        } else if (ch === '(') {
          let keyWord = '', letter;
          const indentTemp = stream.column();

          state.parenDepth++;

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

          returnType = tokenType(PAREN, state, ch);
        } else if (ch === ')') {
          returnType = tokenType(PAREN, state, ch);
          if (state.indentStack != null && state.indentStack.type === '(') {
            popStack(state);

            if (typeof state.sExprComment === 'number') {
              if (--state.sExprComment === 0) {
                returnType = tokenType(COMMENT, state);
                state.sExprComment = false; // turn off s-expr commenting mode
              }
            }
          }

          if (state.afterTilde === true && state.parenDepth === state.afterTildeParenDepth) {
            setAfterTilde(false, state);
          }

          state.parenDepth--;
        } else {
          stream.eatWhile(/[\w\$_\-!$%&*+\.\/:<=>?@\^]/);

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

    indent: state => {
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
