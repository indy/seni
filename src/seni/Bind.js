import Shapes from './Shapes';
import MatrixStackBindings from './MatrixStackBindings';
import MathUtil from './MathUtil';
import ColourBindings from './ColourBindings';
import Perlin from './Perlin';
import Core from './Core';

function bindCore(env) {
  let core = [Core.takeBinding];

  core.forEach((m) => env.add(m.name, m.create(m)));
}

function bindMath(env) {
  let math = [MathUtil.remapFnBinding,
              MathUtil.PI,
              MathUtil.twoPI,
              MathUtil.PIbyTwo,
              MathUtil.buildSigned,
              MathUtil.buildUnsigned];

  math.forEach((m) => env.add(m.name, m.create(m)));
}

function bindMatrixStack(env, matrixStack) {
  let mstack = [MatrixStackBindings.pushMatrix,
                MatrixStackBindings.popMatrix,
                MatrixStackBindings.scale,
                MatrixStackBindings.translate,
                MatrixStackBindings.rotate];

  mstack.forEach((m) => env.add(m.name, m.create(m, matrixStack)));
}

function bindShapes(env, renderer) {

  let shapes = [Shapes.rect,
                Shapes.bezier,
                Shapes.bezierTrailing,
                Shapes.bezierBulging];

  shapes.forEach((r) => env.add(r.name, r.create(r, renderer)));
}

function bindColour(env) {

  let colours = [ColourBindings.colRGB,
                 ColourBindings.colHSL,
                 ColourBindings.colLAB,
                 ColourBindings.colHSV,
                 ColourBindings.RGB,
                 ColourBindings.HSL,
                 ColourBindings.LAB,
                 ColourBindings.HSV,
                 ColourBindings.colConvert,
                 ColourBindings.colComplementary,
                 ColourBindings.colSplitComplementary,
                 ColourBindings.colAnalagous,
                 ColourBindings.colTriad,
                 ColourBindings.colSetAlpha,
                 ColourBindings.colGetAlpha];

  colours.forEach((c) => env.add(c.name, c.create(c)));
}

function bindPerlin(env) {
  let pln = [Perlin.perlin,
             Perlin.perlin2];

  pln.forEach((p) => env.add(p.name, p.create(p)));
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
