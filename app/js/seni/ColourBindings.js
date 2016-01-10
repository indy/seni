/*
 *  Seni
 *  Copyright (C) 2016 Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

import PublicBinding from './PublicBinding';
import MathUtil from './MathUtil';
import Colour from './Colour';

const Format = Colour.Format;

// sets a value for a colour component,
// converting the colour to the right format if needed
function colourSet(publicBinding, format, component) {
  return params => {
    const {colour, value} = publicBinding.mergeWithDefaults(params);
    return Colour.setComponent(Colour.cloneAs(colour, format),
                               component, value);
  };
}

// gets a value for a colour component,
// converting the colour to the right format if needed
function colourGet(publicBinding, format, component) {
  return params => {
    const {colour} = publicBinding.mergeWithDefaults(params);
    return Colour.getComponent(Colour.cloneAs(colour, format),
                               component);
  };
}

const publicBindings = [
  new PublicBinding(
    'col/set-rgb-r',
    `sets red component of the given colour`,
    {colour: Colour.defaultColour, value: 1.0},
    self => colourSet(self, Format.RGB, Colour.R)
  ),

  new PublicBinding(
    'col/get-rgb-r',
    `gets red component of the given colour`,
    {colour: Colour.defaultColour},
    self => colourGet(self, Format.RGB, Colour.R)
  ),

  new PublicBinding(
    'col/set-rgb-g',
    `sets red component of the given colour`,
    {colour: Colour.defaultColour, value: 1.0},
    self => colourSet(self, Format.RGB, Colour.G)
  ),

  new PublicBinding(
    'col/get-rgb-g',
    `gets red component of the given colour`,
    {colour: Colour.defaultColour},
    self => colourGet(self, Format.RGB, Colour.G)
  ),

  new PublicBinding(
    'col/set-rgb-b',
    `sets red component of the given colour`,
    {colour: Colour.defaultColour, value: 1.0},
    self => colourSet(self, Format.RGB, Colour.B)
  ),

  new PublicBinding(
    'col/get-rgb-b',
    `gets red component of the given colour`,
    {colour: Colour.defaultColour},
    self => colourGet(self, Format.RGB, Colour.B)
  ),

  new PublicBinding(
    'col/set-alpha',
    `sets red component of the given colour`,
    {colour: Colour.defaultColour, value: 1.0},
    self => params => {
      const {colour, value} = self.mergeWithDefaults(params);
      return Colour.setComponent(colour, Colour.ALPHA, value);
    }
  ),

  new PublicBinding(
    'col/get-alpha',
    `gets alpha component of the given colour`,
    {colour: Colour.defaultColour},
    self => params => {
      const {colour} = self.mergeWithDefaults(params);
      return Colour.getComponent(colour, Colour.ALPHA);
    }
  ),

  new PublicBinding(
    'col/set-lab-l',
    `sets l component of the given colour`,
    {colour: Colour.defaultColour, value: 1.0},
    self => colourSet(self, Format.LAB, Colour.L)
  ),

  new PublicBinding(
    'col/get-lab-l',
    `gets l component of the given colour`,
    {colour: Colour.defaultColour},
    self => colourGet(self, Format.LAB, Colour.L)
  ),

  new PublicBinding(
    'col/set-lab-a',
    `sets a component of the given colour`,
    {colour: Colour.defaultColour, value: 1.0},
    self => colourSet(self, Format.LAB, Colour.A)
  ),

  new PublicBinding(
    'col/get-lab-a',
    `gets a component of the given colour`,
    {colour: Colour.defaultColour},
    self => colourGet(self, Format.LAB, Colour.A)
  ),

  new PublicBinding(
    'col/set-lab-b',
    `sets b component of the given colour`,
    {colour: Colour.defaultColour, value: 1.0},
    self => colourSet(self, Format.LAB, Colour.B)
  ),

  new PublicBinding(
    'col/get-lab-b',
    `gets b component of the given colour`,
    {colour: Colour.defaultColour},
    self => colourGet(self, Format.LAB, Colour.B)
  ),

  new PublicBinding(
    'col/rgb',
    ``,
    {r: 1.0, g: 0.1, b: 0.2, alpha: 0.5},
    self => params => {
      const {r, g, b, alpha} = self.mergeWithDefaults(params);
      return Colour.construct(Format.RGB, [r, g, b, alpha]);
    }
  ),

  new PublicBinding(
    'col/hsl',
    `h: 0..360
      s: 0..1
      l: 0..1
      `,
    {h: 180.0, s: 0.1, l: 0.2, alpha: 0.5},
    self => params => {
      const {h, s, l, alpha} = self.mergeWithDefaults(params);
      const normalisedH = h % 360;
      return Colour.construct(Format.HSL, [normalisedH, s, l, alpha]);
    }
  ),

  new PublicBinding(
    'col/lab',
    ``,
    {l: 1.0, a: 0.1, b: 0.2, alpha: 0.5},
    self => params => {
      const {l, a, b, alpha} = self.mergeWithDefaults(params);
      return Colour.construct(Format.LAB, [l, a, b, alpha]);
    }
  ),

  new PublicBinding(
    'col/hsv',
    `h: 0..360
      s: 0..1
      v: 0..1
      `,
    {h: 180.0, s: 0.1, v: 0.2, alpha: 0.5},
    self => params => {
      const {h, s, v, alpha} = self.mergeWithDefaults(params);
      const normalisedH = h % 360;
      return Colour.construct(Format.HSV, [normalisedH, s, v, alpha]);
    }
  ),

  new PublicBinding('RGB', ``, {}, () => Format.RGB),
  new PublicBinding('HSL', ``, {}, () => Format.HSL),
  new PublicBinding('LAB', ``, {}, () => Format.LAB),
  new PublicBinding('HSV', ``, {}, () => Format.HSV),

  new PublicBinding('white', ``, {},
                    () => Colour.construct(Format.RGB, [1, 1, 1, 1])),
  new PublicBinding('black', ``, {},
                    () => Colour.construct(Format.RGB, [0, 0, 0, 1])),
  new PublicBinding('red', ``, {},
                    () => Colour.construct(Format.RGB, [1, 0, 0, 1])),
  new PublicBinding('green', ``, {},
                    () => Colour.construct(Format.RGB, [0, 1, 0, 1])),
  new PublicBinding('blue', ``, {},
                    () => Colour.construct(Format.RGB, [0, 0, 1, 1])),
  new PublicBinding('yellow', ``, {},
                    () => Colour.construct(Format.RGB, [1, 1, 0, 1])),
  new PublicBinding('magenta', ``, {},
                    () => Colour.construct(Format.RGB, [1, 0, 1, 1])),
  new PublicBinding('cyan', ``, {},
                    () => Colour.construct(Format.RGB, [0, 1, 1, 1])),


  new PublicBinding(
    'col/convert',
    ``,
    {format: Format.RGB, colour: Colour.defaultColour},
    self => params => {
      const {format, colour} = self.mergeWithDefaults(params);
      return Colour.cloneAs(colour, format);
    }
  ),

  new PublicBinding(
    'col/complementary',
    ``,
    {colour: Colour.defaultColour},
    self => params => {
      const {colour} = self.mergeWithDefaults(params);
      return Colour.complementary(colour);
    }
  ),

  new PublicBinding(
    'col/split-complementary',
    ``,
    {colour: Colour.defaultColour},
    self => params => {
      const {colour} = self.mergeWithDefaults(params);
      return Colour.splitComplementary(colour);
    }
  ),

  new PublicBinding(
    'col/analagous',
    ``,
    {colour: Colour.defaultColour},
    self => params => {
      const {colour} = self.mergeWithDefaults(params);
      return Colour.analagous(colour);
    }
  ),

  new PublicBinding(
    'col/triad',
    ``,
    {colour: Colour.defaultColour},
    self => params => {
      const {colour} = self.mergeWithDefaults(params);
      return Colour.triad(colour);
    }
  ),

  new PublicBinding(
    'col/procedural-fn',
    ``,
    {a: [0.5, 0.5, 0.5],
     b: [0.5, 0.5, 0.5],
     c: [1.0, 1.0, 1.0],
     d: [0.0, 0.33, 0.67],
     alpha: 1.0},
    self => params => {
      const {a, b, c, d, alpha} = self.mergeWithDefaults(params);
      return Colour.proceduralFn(a, b, c, d, alpha);
    }
  ),

  new PublicBinding(
    'col/bezier-fn',
    ``,
    {a: Colour.construct(Format.RGB, [1, 1, 1, 1]),
     b: Colour.construct(Format.RGB, [1, 1, 1, 1]),
     c: Colour.construct(Format.RGB, [1, 1, 1, 1]),
     d: Colour.construct(Format.RGB, [1, 1, 1, 1])},
    self => params => {
      const {a, b, c, d} = self.mergeWithDefaults(params);
      return Colour.bezierFn(a, b, c, d);
    }
  ),

  new PublicBinding(
    'col/quadratic-fn',
    ``,
    {a: Colour.construct(Format.RGB, [1, 1, 1, 1]),
     b: Colour.construct(Format.RGB, [1, 1, 1, 1]),
     c: Colour.construct(Format.RGB, [1, 1, 1, 1])},
    self => params => {
      const {a, b, c} = self.mergeWithDefaults(params);
      return Colour.quadraticFn(a, b, c);
    }
  ),

  new PublicBinding(
    'col/darken',
    `darkens the given colour`,
    {colour: Colour.defaultColour, value: 1.0},
    self => params => {
      const {colour, value} = self.mergeWithDefaults(params);
      return Colour.setComponent(Colour.cloneAs(colour, Format.LAB),
                                 Colour.L, value);
    }
  ),

  new PublicBinding(
    'col/lighten',
    `lightens the given colour by delta`,
    {colour: Colour.defaultColour, delta: 10.0},
    self => params => {
      const {colour, delta} = self.mergeWithDefaults(params);
      const lab = Colour.cloneAs(colour, Format.LAB);
      const currentL = Colour.getComponent(lab, Colour.L);
      const newL = MathUtil.clamp(currentL + delta, 0, 100);
      return Colour.setComponent(lab, Colour.L, newL);
    }
  )
];

export default {
  publicBindings
};
