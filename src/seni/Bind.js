import Shapes from './Shapes';
import MatrixStackBindings from './MatrixStackBindings';
import MathUtil from './MathUtil';
import ColourBindings from './ColourBindings';
import Perlin from './Perlin';
import Core from './Core';
import Bracket from './BracketBindings';

function bindCore(env) {
  let core = [Core.takeBinding];
  return core.reduce((a, b) => a.set(b.name, b.create(b)), env);
}

function bindMath(env) {
  let math = [MathUtil.remapFnBinding,
              MathUtil.PI,
              MathUtil.twoPI,
              MathUtil.PIbyTwo,
              MathUtil.rngSigned,
              MathUtil.rngUnsigned,
              MathUtil.distance2D,
              MathUtil.clamp];

  return math.reduce((a, b) => a.set(b.name, b.create(b)), env);
}

function bindMatrixStack(env, matrixStack) {
  let mstack = [MatrixStackBindings.pushMatrix,
                MatrixStackBindings.popMatrix,
                MatrixStackBindings.scale,
                MatrixStackBindings.translate,
                MatrixStackBindings.rotate];

  return mstack.reduce((a, b) => a.set(b.name, b.create(b, matrixStack)), env);
}

function bindShapes(env, renderer) {

  let shapes = [Shapes.rect,
                Shapes.bezier,
                Shapes.bezierTrailing,
                Shapes.bezierBulging];

  return shapes.reduce((a, b) => a.set(b.name, b.create(b, renderer)), env);
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
                 ColourBindings.colGetAlpha,
                 ColourBindings.colSetLightness,
                 ColourBindings.colGetLightness];

  return colours.reduce((a, b) => a.set(b.name, b.create(b)), env);
}

function bindPerlin(env) {
  let pln = [Perlin.perlin,
             Perlin.perlin2];

  return pln.reduce((a, b) => a.set(b.name, b.create(b)), env);
}

function bindBracket(env, rng) {
  let br = [Bracket.identity,
            Bracket.inRange,
            Bracket.scalar,
            Bracket.testPlus];

  return br.reduce((a, b) => a.set(b.name, b.create(b, rng)), env);
}


let Bind = {
  addBindings: function(env, renderer) {

    env = bindCore(env);
    env = bindMath(env);
    env = bindMatrixStack(env, renderer.getMatrixStack());
    env = bindShapes(env, renderer);
    env = bindColour(env);
    env = bindPerlin(env);

    return env;
  },

  addBracketBindings: function(env, rng) {

    env = bindCore(env);
    env = bindMath(env);
    env = bindColour(env);
    env = bindPerlin(env);
    env = bindBracket(env, rng);

    return env;
  }
};

export default Bind;
