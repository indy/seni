import { renderBezier, getBezierFn } from './Bezier';
import { Env } from 'lang/env';
import { Node, NodeType } from 'lang/node';
import { remapFn } from './MathUtil';

export function addBindings(env, renderer) {

  var glContainer = renderer.getGLContainer();
  var buffer = renderer.getBuffer();
  var bezier = getBezierFn(glContainer, buffer);
  
  env.addBinding('remapFn', function(args) {
    return remapFn(args);
  });

  env.addBinding('bezier', function(args) {
    bezier(args);
  });

  return env;
}
