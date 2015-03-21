import Interpreter from './Interpreter';
import Parser from './Parser';
import Lexer from './Lexer';
import Compiler from './Compiler';

const Runtime = {
  createEnv: function() {
    return Interpreter.getBasicEnv();
  },

  buildAst: function(env, form) {
    const tokensBox = Lexer.tokenise(form);
    if(tokensBox.error) {
      console.log(tokensBox.error);
      return false;
    }
    const astBox = Parser.parse(tokensBox.tokens);
    if(astBox.error) {
      // some sort of error occurred
      console.log(astBox.error);
      return false;
    }

    return astBox.nodes;
  },

  evalAst: function(env, ast, genotype) {
    const compiled = Compiler.compile(ast, genotype);

    // add all of the define expressions to the env
    const [_env, _res] = compiled.forms.
            filter(Interpreter.isDefineExpression).
            reduce(([e, r], b) => Interpreter.evaluate(e, b), [env, false]);

    // now evaluate all of the non-define expressions
    return compiled.forms.
      filter((s) => !Interpreter.isDefineExpression(s)).
      reduce(([e, r], b) => Interpreter.evaluate(e, b), [_env, _res]);
  }
};

export default Runtime;
