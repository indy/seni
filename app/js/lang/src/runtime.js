import {evaluate, specialForms, requiredFunctions} from './interpreter';
import {Env, bind} from './env';
import {parse} from './parser';
import {tokenise} from './lexer';


export function createEnv() {
  return bind(new Env(), [specialForms, requiredFunctions]);
}

export function evalForm(env, form) {
  let ts = tokenise(form);
  let ast = parse(ts);

  // todo: replace nodes with simpler json like structure
  // currently not sure how to deal with special forms like quote
  
  return ast.reduce((a, b) => evaluate(env, b), null);
}
