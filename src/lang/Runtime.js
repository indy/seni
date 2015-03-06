import Interpreter from './Interpreter';
import Env from './Env';
import Parser from './Parser';
import Lexer from './Lexer';
import Compiler from './Compiler';
import bind from './bind';

let Runtime = {
  createEnv: function() {
    return bind(new Env(), [Interpreter.specialForms,
                            Interpreter.classicFunctions]);
  },

  evalForm: function(env, form) {
    let tokensBox = Lexer.tokenise(form);
    if(tokensBox.error) {
      console.log(tokensBox.error);
      return false;
    }
    let astBox = Parser.parse(tokensBox.tokens);
    if(astBox.error) {
      // some sort of error occurred
      console.log(astBox.error);
      return false;
    }

    let ast = astBox.nodes;
    let compiled = Compiler.compile(ast);

    // add all of the define expressions to the env
    compiled.
      filter(Interpreter.isDefineExpression).
      reduce((a, b) => Interpreter.evaluate(env, b), null);

    // now evaluate all of the non-define expressions
    return compiled.
      filter((s) => !Interpreter.isDefineExpression(s)).
      reduce((a, b) => Interpreter.evaluate(env, b), null);
  }
};

export default Runtime;
