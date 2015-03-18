import Interpreter from './Interpreter';
import Parser from './Parser';
import Lexer from './Lexer';
import Compiler from './Compiler';
import Genetic from './Genetic';

let Runtime = {
  createEnv: function() {
    return Interpreter.getBasicEnv();
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

    const traits = Genetic.buildTraits(ast);
    const genotype = Genetic.createGenotypeFromInitialValues(traits);
    let compiled = Compiler.compile(ast, genotype);

    console.log(compiled);

    // add all of the define expressions to the env
    let [_env, _res] = compiled.forms.
          filter(Interpreter.isDefineExpression).
          reduce(([e, r], b) => Interpreter.evaluate(e, b), [env, false]);

    // now evaluate all of the non-define expressions
    return compiled.forms.
      filter((s) => !Interpreter.isDefineExpression(s)).
      reduce(([e, r], b) => Interpreter.evaluate(e, b), [_env, _res]);
  }
};

export default Runtime;
