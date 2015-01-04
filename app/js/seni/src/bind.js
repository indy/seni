import { renderBezier, getBezierFn } from './shapes';
import { MatrixStack } from './MatrixStack';
import { Env } from 'lang/env';
import { Node, NodeType } from 'lang/node';
import { remapFn } from './MathUtil';

export function addBindings(env, renderer) {

  env.add('remapFn', function(args) {
    return remapFn(args);
  });

  bindMatrixStack(env, renderer.getMatrixStack());
  bindBezier(env, renderer);

  return env;
}

function bindMatrixStack(env, matrixStack) {
  env.add('pushMatrix', () => matrixStack.pushMatrix());

  env.add('popMatrix', () => matrixStack.popMatrix());

  env.add('scale', ({x = 1.0, y = 1.0}) =>  matrixStack.scale(x, y));

  env.add('translate', ({x = 0.0, y = 0.0}) => matrixStack.translate(x, y));

  env.add('rotate', ({angle = 0.0}) => matrixStack.rotate(angle));
}

function bindBezier(env, renderer) {
  let bezier = getBezierFn(renderer);

  env.add('bezier', (args) => bezier(args));
}
