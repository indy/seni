import bind from './bind';
import Interpreter from './interpreter';
import Env from './env';
import Parser from './parser';
import Lexer from './lexer';
import Compiler from './compiler';

export function createEnv() {
  return bind(new Env(), [Interpreter.specialForms,
                          Interpreter.classicFunctions]);
}

export function evalForm(env, form) {
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
