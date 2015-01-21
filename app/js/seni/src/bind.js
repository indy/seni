import * as Shapes from './shapes';
import { MatrixStack } from './MatrixStack';
import { Env } from 'lang/env';
import { Node, NodeType } from 'lang/node';
import { remapFn } from './MathUtil';
import * as Colour from './Colour';

export function addBindings(env, renderer) {

  bindMath(env);
  bindMatrixStack(env, renderer.getMatrixStack());
  bindShapes(env, renderer);
  bindColour(env);

  return env;
}

function bindMath(env) {
  env.add('remapFn', (args) => remapFn(args));
}

function bindMatrixStack(env, matrixStack) {
  env.add('pushMatrix', () => matrixStack.pushMatrix());

  env.add('popMatrix', () => matrixStack.popMatrix());

  env.add('scale', ({x = 1.0, y = 1.0}) =>  matrixStack.scale(x, y));

  env.add('translate', ({x = 0.0, y = 0.0}) => matrixStack.translate(x, y));

  env.add('rotate', ({angle = 0.0}) => matrixStack.rotate(angle));
}

function bindShapes(env, renderer) {
  env.add('bezier',
          (args) => Shapes.renderBezier(renderer, args));
  env.add('bezierTrailing',
          (args) => Shapes.renderBezierTrailing(renderer, args));
  env.add('bezierBulging',
          (args) => Shapes.renderBezierBulging(renderer, args));
}

function bindColour(env) {
  env.add('rgbColour', (args) => Colour.rgbColour(args));

  env.add('hslColour', (args) => Colour.hslColour(args));

  env.add('labColour', (args) => Colour.labColour(args));

  env.add('hsvColour', (args) => Colour.hsvColour(args));

  env.add('RGB', Colour.Format.RGB);
  
  env.add('HSL', Colour.Format.HSL);
  
  env.add('LAB', Colour.Format.LAB);
  
  env.add('HSV', Colour.Format.HSV);

  env.add('colourConvert', (args) => Colour.colourConvert(args));
}
