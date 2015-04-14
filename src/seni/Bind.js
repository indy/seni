/*
    Seni
    Copyright (C) 2015  Inderjit Gill <email@indy.io>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

import Shapes from './Shapes';
import MatrixStackBindings from './MatrixStackBindings';
import MathUtil from './MathUtil';
import ColourBindings from './ColourBindings';
import Perlin from './Perlin';
import Core from './Core';
import Bracket from './BracketBindings';
import SeedRandom from './SeedRandom';

function bindCore(env) {
  const core = [Core.takeBinding];
  return core.reduce((a, b) => a.set(b.name, b.create(b)), env);
}

function bindMath(env) {
  const math = [MathUtil.remapFnBinding,
                MathUtil.PI,
                MathUtil.twoPI,
                MathUtil.PIbyTwo,
                MathUtil.sin,
                MathUtil.cos,
                MathUtil.distance2D,
                MathUtil.clamp];

  return math.reduce((a, b) => a.set(b.name, b.create(b)), env);
}

function bindSeedRandom(env) {
  const random = [SeedRandom.rngSigned,
                SeedRandom.rngUnsigned];

  return random.reduce((a, b) => a.set(b.name, b.create(b)), env);
}

function bindMatrixStack(env, matrixStack) {
  const mstack = [MatrixStackBindings.pushMatrix,
                  MatrixStackBindings.popMatrix,
                  MatrixStackBindings.scale,
                  MatrixStackBindings.translate,
                  MatrixStackBindings.rotate];

  return mstack.reduce((a, b) => a.set(b.name, b.create(b, matrixStack)), env);
}

function bindShapes(env, renderer) {

  const shapes = [Shapes.rect,
                  Shapes.poly,
                  Shapes.spline,
                  Shapes.bezier,
                  Shapes.bezierTrailing,
                  Shapes.bezierBulging,
                  Shapes.strokedBezier,
                  Shapes.strokedBezierRect];

  return shapes.reduce((a, b) => a.set(b.name, b.create(b, renderer)), env);
}

function bindColour(env) {

  const colours = [ColourBindings.colRGB,
                   ColourBindings.colHSL,
                   ColourBindings.colLAB,
                   ColourBindings.colHSV,

                   ColourBindings.colSetRGBRed,
                   ColourBindings.colGetRGBRed,
                   ColourBindings.colSetRGBGreen,
                   ColourBindings.colGetRGBGreen,
                   ColourBindings.colSetRGBBlue,
                   ColourBindings.colGetRGBBlue,
                   ColourBindings.colSetAlpha,
                   ColourBindings.colGetAlpha,
                   ColourBindings.colSetLABL,
                   ColourBindings.colGetLABL,
                   ColourBindings.colSetLABA,
                   ColourBindings.colGetLABA,
                   ColourBindings.colSetLABB,
                   ColourBindings.colGetLABB,

                   ColourBindings.RGB,
                   ColourBindings.HSL,
                   ColourBindings.LAB,
                   ColourBindings.HSV,

                   ColourBindings.colConvert,
                   ColourBindings.colComplementary,
                   ColourBindings.colSplitComplementary,
                   ColourBindings.colAnalagous,
                   ColourBindings.colTriad];

  return colours.reduce((a, b) => a.set(b.name, b.create(b)), env);
}

function bindPerlin(env) {
  const pln = [Perlin.perlin,
               Perlin.perlin2];

  return pln.reduce((a, b) => a.set(b.name, b.create(b)), env);
}

function bindBracket(env, rng) {
  const br = [Bracket.identity,
              Bracket.int,
              Bracket.scalar,
              Bracket.select,
              Bracket.testPlus];

  return br.reduce((a, b) => a.set(b.name, b.create(b, rng)), env);
}

const Bind = {
  addBindings: function(env, renderer) {

    env = bindCore(env);
    env = bindMath(env);
    env = bindSeedRandom(env);
    env = bindMatrixStack(env, renderer.getMatrixStack());
    env = bindShapes(env, renderer);
    env = bindColour(env);
    env = bindPerlin(env);

    return env;
  },

  addBracketBindings: function(env, rng) {

    env = bindCore(env);
    env = bindMath(env);
    env = bindSeedRandom(env);
    env = bindColour(env);
    env = bindPerlin(env);
    env = bindBracket(env, rng);

    return env;
  }
};

export default Bind;
