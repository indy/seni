import {evaluate, specialForms, classicFunctions} from './interpreter';
import {Env, bind} from './env';
import {parse} from './parser';
import {tokenise} from './lexer';
import {compile} from './compiler';

export function createEnv() {
  return bind(new Env(), [specialForms, classicFunctions]);
}

export function evalForm(env, form) {
  let ts = tokenise(form);
  let ast = parse(ts);
  let compiled = compile(ast);

  return compiled.reduce((a, b) => evaluate(env, b), null);
}
