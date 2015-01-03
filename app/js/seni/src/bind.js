import { renderBezier, getBezierFn } from './Bezier';
import { MatrixStack } from './MatrixStack';
import { Env } from 'lang/env';
import { Node, NodeType } from 'lang/node';
import { remapFn } from './MathUtil';

export function addBindings(env, renderer) {

  env.addBinding('remapFn', function(args) {
    return remapFn(args);
  });

  bindMatrixStack(env, renderer.getMatrixStack());
  bindBezier(env, renderer);

  return env;
}

function bindMatrixStack(env, matrixStack) {
  env.addBinding('pushMatrix', function(args) {
    return matrixStack.pushMatrix();
  });

  env.addBinding('popMatrix', function(args) {
    return matrixStack.popMatrix();
  });

  env.addBinding('scale', function(args) {
    return matrixStack.scale(args.x, args.y);
  });

  env.addBinding('translate', function(args) {
    return matrixStack.translate(args.x, args.y);
  });

  env.addBinding('rotate', function(args) {
    return matrixStack.rotate(args.rad);
  });
}

function bindBezier(env, renderer) {
  let bezier = getBezierFn(renderer);
  env.addBinding('bezier', function(args) {
    bezier(args);
  });
}
