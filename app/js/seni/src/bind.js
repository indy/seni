import { renderBezier, getBezierFn } from './Bezier';
import { Env } from 'lang/env';
import { Node, NodeType } from 'lang/node';
import { remapFn } from './MathUtil';

export function addBindings(env, renderer) {

  env.addBinding('remapFn', function(args) {
    return remapFn(args);
  });

  let bezier = getBezierFn(renderer);
  env.addBinding('bezier', function(args) {
    bezier(args);
  });

  return env;
}
