import { renderBezier, getBezierFn } from './Bezier';
import { Env } from 'lang/env';
import { Node, NodeType } from 'lang/node';

export function addBindings(env, renderer) {

  var glContainer = renderer.getGLContainer();
  var buffer = renderer.getBuffer();
  var bezier = getBezierFn(glContainer, buffer);
  
  env.addBinding('bezier', function(args) {
    bezier(args);
  });

  return env;
}
