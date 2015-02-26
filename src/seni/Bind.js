import Shapes from './Shapes';
import MatrixStack from './MatrixStack';
import MathUtil from './MathUtil';
import ColourConstants from './ColourConstants';
import ColourFns from './ColourFns';
import Perlin from './Perlin';
import Core from './Core';

function bindCore(env) {
  let core = [Core.takeBinding];

  core.forEach((m) => env.add(m.name, m.create()));
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

  let colours = [ColourFns.rgbColour,
                 ColourFns.hslColour,
                 ColourFns.labColour,
                 ColourFns.hsvColour,
                 ColourConstants.RGB,
                 ColourConstants.HSL,
                 ColourConstants.LAB,
                 ColourConstants.HSV,
                 ColourFns.colourConvert,
                 ColourFns.complementary,
                 ColourFns.splitComplementary,
                 ColourFns.analagous,
                 ColourFns.triad,
                 ColourFns.setAlpha];

  colours.forEach((c) => env.add(c.name, c.create()));
}

function bindPerlin(env) {
  let pln = [Perlin.perlin,
             Perlin.perlin2];

  pln.forEach((p) => env.add(p.name, p.create()));
}


var Bind = {
  addBindings: function(env, renderer) {

    bindCore(env);
    bindMath(env);
    bindMatrixStack(env, renderer.getMatrixStack());
    bindShapes(env, renderer);
    bindColour(env);
    bindPerlin(env);

    return env;
  }
};

export default Bind;
