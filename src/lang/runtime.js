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
    let ts = Lexer.tokenise(form);
    let astBox = Parser.parse(ts);
    if(astBox.error) {
      // some sort of error occurred
      console.log(astBox.error);
      return false;
    }

    let ast = astBox.nodes;
    let compiled = Compiler.compile(ast);
    return compiled.reduce((a, b) => Interpreter.evaluate(env, b), null);
  }
};

export default Runtime;
