import * as Shapes from './shapes';
import * as MatrixStack from './MatrixStack';
import * as MathUtil from './MathUtil';
import * as ColourNS from './Colour';
import * as Perlin from './Perlin';

export function addBindings(env, renderer) {

  bindMath(env);
  bindMatrixStack(env, renderer.getMatrixStack());
  bindShapes(env, renderer);
  bindColour(env);
  bindPerlin(env);

  return env;
}

function bindMath(env) {
  let math = [MathUtil.remapFnBinding,
              MathUtil.PI,
              MathUtil.twoPI,
              MathUtil.PIbyTwo,
              MathUtil.buildPRNG];

  math.forEach((m) => env.add(m.name, m.create()));
}

function bindMatrixStack(env, matrixStack) {
  let mstack = [MatrixStack.pushMatrix, 
                MatrixStack.popMatrix, 
                MatrixStack.scale, 
                MatrixStack.translate, 
                MatrixStack.rotate];

  mstack.forEach((m) => env.add(m.name, m.create(matrixStack)));
}

function bindShapes(env, renderer) {

  let shapes = [Shapes.rect, 
                Shapes.bezier,
                Shapes.bezierTrailing,
                Shapes.bezierBulging];

  shapes.forEach((r) => env.add(r.name, r.create(renderer)));
}

function bindColour(env) {

  let colours = [ColourNS.rgbColour,
                 ColourNS.hslColour,
                 ColourNS.labColour,
                 ColourNS.hsvColour,
                 ColourNS.RGB,
                 ColourNS.HSL,
                 ColourNS.LAB,
                 ColourNS.HSV,
                 ColourNS.colourConvert,
                 ColourNS.complementary,
                 ColourNS.splitComplementary,
                 ColourNS.analagous,
                 ColourNS.triad,
                 ColourNS.setAlpha];
  
  colours.forEach((c) => env.add(c.name, c.create()));
}

function bindPerlin(env) {
  let pln = [Perlin.perlin, 
             Perlin.perlin2];

  pln.forEach((p) => env.add(p.name, p.create()));
}
